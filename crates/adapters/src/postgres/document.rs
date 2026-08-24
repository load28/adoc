use std::collections::{BTreeMap, BTreeSet};

use adoc_application::{
    document::{
        Document, DocumentChange, DocumentDetail, DocumentPage, DocumentRepository, DocumentStatus,
        DocumentTree, DocumentTreeNode, Draft, DraftCreate, DraftMutation, EditLeaseView,
        LeaseAcquire, LeaseMutation, MoveCommit, MovePreviewRequest, MutationResult, NewDocument,
        OperationError, OperationErrorCode, ReducerInput, ReferenceEffect, ReferenceSnapshot,
        ReferenceTarget, RegionResolutionStatus, StoredImpactPreview, TreeRank, ValidatedContent,
        apply_operations, canonical_hash, reanchor_region,
    },
    governance::GovernanceError,
    identity::TokenHash,
    permission::{Access, compile_permission_scope, resolve_permission_path},
};
use adoc_ports::BoxFuture;
use chrono::{DateTime, Utc};
use serde_json::{Value, json};
use sqlx::{PgPool, Postgres, Row, Transaction, postgres::PgRow};
use subtle::ConstantTimeEq;
use uuid::Uuid;

use super::{
    PostgresStore,
    collaboration::invalidate_reviews,
    governance::{
        OutboxEvent, append_event, begin_workspace, check_revision, complete_workspace, map_store,
    },
    permission::{point_snapshot_tx, scope_snapshot_tx},
};

#[derive(Clone)]
pub struct PostgresDocumentRepository {
    pool: PgPool,
}

impl PostgresDocumentRepository {
    pub fn new(store: &PostgresStore) -> Self {
        Self {
            pool: store.pool().clone(),
        }
    }
}

impl DocumentRepository for PostgresDocumentRepository {
    fn tree<'a>(
        &'a self,
        actor: Uuid,
        workspace: Uuid,
    ) -> BoxFuture<'a, Result<DocumentTree, GovernanceError>> {
        Box::pin(async move {
            let mut tx = self.pool.begin().await.map_err(map_store)?;
            let snapshot = scope_snapshot_tx(&mut tx, actor, workspace).await?;
            let resolved =
                compile_permission_scope(&snapshot.nodes).map_err(|_| GovernanceError::Internal)?;
            let accessible = resolved
                .into_iter()
                .filter_map(|(id, permission)| permission.access.can_view().then_some(id))
                .collect::<BTreeSet<_>>();
            let rows = sqlx::query("SELECT id,parent_id,title,status::text,current_version_id,revision FROM documents WHERE workspace_id=$1 AND status='ACTIVE' ORDER BY rank COLLATE \"C\",id")
                .bind(workspace).fetch_all(&mut *tx).await.map_err(map_store)?;
            let watermark: i64 = sqlx::query_scalar(
                "SELECT tree_revision FROM workspace_document_revisions WHERE workspace_id=$1",
            )
            .bind(workspace)
            .fetch_optional(&mut *tx)
            .await
            .map_err(map_store)?
            .ok_or(GovernanceError::WorkspaceNotFound)?;
            let nodes = build_tree(&rows, &accessible)?;
            tx.commit().await.map_err(map_store)?;
            Ok(DocumentTree { nodes, watermark })
        })
    }

    fn trash<'a>(
        &'a self,
        actor: Uuid,
        workspace: Uuid,
        cursor: Option<String>,
    ) -> BoxFuture<'a, Result<DocumentPage, GovernanceError>> {
        Box::pin(async move {
            let cursor = cursor.map(|value| parse_trash_cursor(&value)).transpose()?;
            require_workspace_role(&self.pool, actor, workspace, true).await?;
            let (cursor_time, cursor_id) = cursor.unzip();
            let rows = sqlx::query("SELECT id,parent_id,title,status::text,current_version_id,revision,trashed_at FROM documents WHERE workspace_id=$1 AND status='TRASHED' AND ($2::timestamptz IS NULL OR (trashed_at,id)<($2,$3)) ORDER BY trashed_at DESC,id DESC LIMIT 51")
                .bind(workspace).bind(cursor_time).bind(cursor_id).fetch_all(&self.pool).await.map_err(map_store)?;
            let mut items = rows
                .iter()
                .take(50)
                .map(document)
                .collect::<Result<Vec<_>, _>>()?;
            let next_cursor = (rows.len() > 50).then(|| {
                let row = &rows[49];
                format_trash_cursor(row.get("trashed_at"), row.get("id"))
            });
            items.shrink_to_fit();
            Ok(DocumentPage { items, next_cursor })
        })
    }

    fn detail<'a>(
        &'a self,
        actor: Uuid,
        workspace: Uuid,
        document_id: Uuid,
    ) -> BoxFuture<'a, Result<DocumentDetail, GovernanceError>> {
        Box::pin(async move {
            let mut tx = self.pool.begin().await.map_err(map_store)?;
            require_access(
                &mut tx,
                actor,
                workspace,
                document_id,
                Access::Viewer,
                false,
            )
            .await?;
            require_effective_active(&mut tx, workspace, document_id).await?;
            let row = sqlx::query("SELECT id,parent_id,title,status::text,current_version_id,revision FROM documents WHERE workspace_id=$1 AND id=$2")
                .bind(workspace).bind(document_id).fetch_one(&mut *tx).await.map_err(map_store)?;
            let draft = load_draft(&mut tx, workspace, document_id).await?;
            tx.commit().await.map_err(map_store)?;
            Ok(DocumentDetail {
                document: document(&row)?,
                draft,
            })
        })
    }

    fn create<'a>(
        &'a self,
        input: NewDocument,
    ) -> BoxFuture<'a, Result<Document, GovernanceError>> {
        Box::pin(async move {
            retry_rank_conflict(3, || async {
                let mut tx = self.pool.begin().await.map_err(map_store)?;
                require_active_member(&mut tx, input.command.actor_id, input.workspace_id).await?;
                if let Some(parent) = input.parent_id {
                    require_access(&mut tx, input.command.actor_id, input.workspace_id, parent, Access::Contributor, false).await?;
                    require_effective_active(&mut tx, input.workspace_id, parent).await?;
                }
                if let Some(replay) = begin_workspace::<Document>(&mut tx, input.workspace_id, &input.command).await? {
                    tx.commit().await.map_err(map_store)?;
                    return Ok(replay);
                }
                lock_tree_revision(&mut tx, input.workspace_id).await?;
                let rank = rank_for(&mut tx, input.workspace_id, input.parent_id, input.after_document_id, None).await?;
                let row = sqlx::query("INSERT INTO documents(id,workspace_id,parent_id,rank,title,created_by,created_at,updated_at) VALUES($1,$2,$3,$4,$5,$6,$7,$7) RETURNING id,parent_id,title,status::text,current_version_id,revision")
                    .bind(input.id).bind(input.workspace_id).bind(input.parent_id).bind(rank.to_string()).bind(&input.title).bind(input.command.actor_id).bind(input.command.now)
                    .fetch_one(&mut *tx).await.map_err(map_document_store)?;
                sqlx::query("INSERT INTO permission_grants(id,workspace_id,document_id,subject_kind,subject_id,access,can_manage,granted_by,created_at,updated_at) VALUES($1,$2,$1,'USER',$3,'EDITOR',true,$3,$4,$4)")
                    .bind(input.id).bind(input.workspace_id).bind(input.command.actor_id).bind(input.command.now).execute(&mut *tx).await.map_err(map_store)?;
                let result = document(&row)?;
                let tree_revision = bump_tree_revision(&mut tx, input.workspace_id, input.command.now).await?;
                append_document_changed(&mut tx, input.workspace_id, &result, tree_revision, "CREATED", input.command.now).await?;
                complete_workspace(&mut tx, input.workspace_id, &input.command, 201, &result).await?;
                tx.commit().await.map_err(map_store)?;
                Ok(result)
            }).await
        })
    }

    fn change<'a>(
        &'a self,
        input: DocumentChange,
    ) -> BoxFuture<'a, Result<Document, GovernanceError>> {
        Box::pin(async move {
            let mut tx = self.pool.begin().await.map_err(map_store)?;
            let required = if input.title.is_some() {
                Access::Contributor
            } else {
                Access::Editor
            };
            require_access(
                &mut tx,
                input.command.actor_id,
                input.workspace_id,
                input.document_id,
                required,
                false,
            )
            .await?;
            if let Some(replay) =
                begin_workspace::<Document>(&mut tx, input.workspace_id, &input.command).await?
            {
                tx.commit().await.map_err(map_store)?;
                return Ok(replay);
            }
            lock_tree_revision(&mut tx, input.workspace_id).await?;
            let row = lock_document(&mut tx, input.workspace_id, input.document_id).await?;
            check_revision(row.revision, input.expected_revision)?;
            let (action, result_row) = if let Some(title) = input.title {
                require_effective_active(&mut tx, input.workspace_id, input.document_id).await?;
                if row.title == title {
                    return Err(GovernanceError::NoEffect);
                }
                ("RENAMED", sqlx::query("UPDATE documents SET title=$3,revision=revision+1,updated_at=$4 WHERE workspace_id=$1 AND id=$2 RETURNING id,parent_id,title,status::text,current_version_id,revision")
                    .bind(input.workspace_id).bind(input.document_id).bind(title).bind(input.command.now).fetch_one(&mut *tx).await.map_err(map_store)?)
            } else if let Some(restore) = input.restore {
                if row.status != DocumentStatus::Trashed
                    || row
                        .purge_after
                        .is_some_and(|value| value <= input.command.now)
                {
                    return Err(GovernanceError::DocumentStateInvalid);
                }
                validate_destination(
                    &mut tx,
                    input.command.actor_id,
                    input.workspace_id,
                    restore.parent_id,
                )
                .await?;
                let rank = rank_for(
                    &mut tx,
                    input.workspace_id,
                    restore.parent_id,
                    restore.after_document_id,
                    Some(input.document_id),
                )
                .await?;
                ("RESTORED", sqlx::query("UPDATE documents SET status='ACTIVE',parent_id=$3,rank=$4,trashed_at=NULL,purge_after=NULL,revision=revision+1,updated_at=$5 WHERE workspace_id=$1 AND id=$2 RETURNING id,parent_id,title,status::text,current_version_id,revision")
                    .bind(input.workspace_id).bind(input.document_id).bind(restore.parent_id).bind(rank.to_string()).bind(input.command.now).fetch_one(&mut *tx).await.map_err(map_store)?)
            } else {
                require_effective_active(&mut tx, input.workspace_id, input.document_id).await?;
                if row.status != DocumentStatus::Active {
                    return Err(GovernanceError::DocumentStateInvalid);
                }
                close_subtree_leases_and_reviews(
                    &mut tx,
                    input.workspace_id,
                    input.document_id,
                    input.command.now,
                )
                .await?;
                ("TRASHED", sqlx::query("UPDATE documents SET status='TRASHED',trashed_at=$3,purge_after=$3+interval '30 days',revision=revision+1,updated_at=$3 WHERE workspace_id=$1 AND id=$2 RETURNING id,parent_id,title,status::text,current_version_id,revision")
                    .bind(input.workspace_id).bind(input.document_id).bind(input.command.now).fetch_one(&mut *tx).await.map_err(map_store)?)
            };
            let result = document(&result_row)?;
            let tree_revision =
                bump_tree_revision(&mut tx, input.workspace_id, input.command.now).await?;
            append_document_changed(
                &mut tx,
                input.workspace_id,
                &result,
                tree_revision,
                action,
                input.command.now,
            )
            .await?;
            complete_workspace(&mut tx, input.workspace_id, &input.command, 200, &result).await?;
            tx.commit().await.map_err(map_store)?;
            Ok(result)
        })
    }

    fn preview_move<'a>(
        &'a self,
        input: MovePreviewRequest,
    ) -> BoxFuture<'a, Result<StoredImpactPreview, GovernanceError>> {
        Box::pin(async move {
            let mut tx = self.pool.begin().await.map_err(map_store)?;
            require_access(
                &mut tx,
                input.actor_id,
                input.workspace_id,
                input.document_id,
                Access::Editor,
                false,
            )
            .await?;
            validate_destination(
                &mut tx,
                input.actor_id,
                input.workspace_id,
                input.input.new_parent_id,
            )
            .await?;
            require_effective_active(&mut tx, input.workspace_id, input.document_id).await?;
            ensure_no_cycle(
                &mut tx,
                input.workspace_id,
                input.document_id,
                input.input.new_parent_id,
            )
            .await?;
            let row = lock_document(&mut tx, input.workspace_id, input.document_id).await?;
            check_revision(row.revision, input.expected_revision)?;
            let (_, before_id, after_rank, before_rank) = rank_anchors(
                &mut tx,
                input.workspace_id,
                input.input.new_parent_id,
                input.input.after_document_id,
                Some(input.document_id),
            )
            .await?;
            let (permission_revision, policy_revision): (i64, i64) = sqlx::query_as("SELECT permission_revision,policy_revision FROM workspace_access_revisions WHERE workspace_id=$1")
                .bind(input.workspace_id).fetch_one(&mut *tx).await.map_err(map_store)?;
            let affected: i64 =
                subtree_count(&mut tx, input.workspace_id, input.document_id).await?;
            let changed = row.parent_id != input.input.new_parent_id;
            let claims = json!({
                "documentRevision":row.revision,"oldParentId":row.parent_id,"newParentId":input.input.new_parent_id,
                "afterDocumentId":input.input.after_document_id,"beforeDocumentId":before_id,
                "afterRank":after_rank,"beforeRank":before_rank,"permissionRevision":permission_revision,"policyRevision":policy_revision
            });
            sqlx::query("INSERT INTO document_move_previews(token_hash,workspace_id,actor_user_id,document_id,claims_json,expires_at,created_at) VALUES($1,$2,$3,$4,$5,$6,clock_timestamp())")
                .bind(input.token_hash.0.as_slice()).bind(input.workspace_id).bind(input.actor_id).bind(input.document_id).bind(claims).bind(input.expires_at)
                .execute(&mut *tx).await.map_err(map_store)?;
            tx.commit().await.map_err(map_store)?;
            Ok(StoredImpactPreview {
                permission_changes: if changed { affected } else { 0 },
                policy_changes: if changed { affected } else { 0 },
                expires_at: input.expires_at,
            })
        })
    }

    fn move_document<'a>(
        &'a self,
        input: MoveCommit,
    ) -> BoxFuture<'a, Result<Document, GovernanceError>> {
        Box::pin(async move {
            retry_rank_conflict(3, || async {
                let mut tx = self.pool.begin().await.map_err(map_store)?;
                require_access(&mut tx, input.command.actor_id, input.workspace_id, input.document_id, Access::Editor, false).await?;
                require_effective_active(&mut tx, input.workspace_id, input.document_id).await?;
                validate_destination(&mut tx, input.command.actor_id, input.workspace_id, input.input.new_parent_id).await?;
                if let Some(replay) = begin_workspace::<Document>(&mut tx, input.workspace_id, &input.command).await? { tx.commit().await.map_err(map_store)?; return Ok(replay); }
                lock_tree_revision(&mut tx, input.workspace_id).await?;
                let preview = sqlx::query("SELECT claims_json,expires_at FROM document_move_previews WHERE token_hash=$1 AND workspace_id=$2 AND actor_user_id=$3 AND document_id=$4 FOR UPDATE")
                    .bind(input.preview_token_hash.0.as_slice()).bind(input.workspace_id).bind(input.command.actor_id).bind(input.document_id)
                    .fetch_optional(&mut *tx).await.map_err(map_store)?.ok_or(GovernanceError::MovePreviewStale)?;
                if preview.get::<DateTime<Utc>, _>("expires_at") <= input.command.now { return Err(GovernanceError::MovePreviewStale); }
                let claims: Value = preview.get("claims_json");
                let row = lock_document(&mut tx, input.workspace_id, input.document_id).await?;
                check_revision(row.revision, input.expected_revision)?;
                ensure_no_cycle(&mut tx, input.workspace_id, input.document_id, input.input.new_parent_id).await?;
                let (_, before_id, after_rank, before_rank) = rank_anchors(&mut tx, input.workspace_id, input.input.new_parent_id, input.input.after_document_id, Some(input.document_id)).await?;
                let (permission_revision, policy_revision): (i64, i64) = sqlx::query_as("SELECT permission_revision,policy_revision FROM workspace_access_revisions WHERE workspace_id=$1")
                    .bind(input.workspace_id).fetch_one(&mut *tx).await.map_err(map_store)?;
                let current_claims = json!({"documentRevision":row.revision,"oldParentId":row.parent_id,"newParentId":input.input.new_parent_id,"afterDocumentId":input.input.after_document_id,"beforeDocumentId":before_id,"afterRank":after_rank,"beforeRank":before_rank,"permissionRevision":permission_revision,"policyRevision":policy_revision});
                if claims != current_claims { return Err(GovernanceError::MovePreviewStale); }
                let rank = rank_for(&mut tx, input.workspace_id, input.input.new_parent_id, input.input.after_document_id, Some(input.document_id)).await?;
                if row.parent_id == input.input.new_parent_id && row.rank == rank.to_string() { return Err(GovernanceError::NoEffect); }
                let result_row = sqlx::query("UPDATE documents SET parent_id=$3,rank=$4,revision=revision+1,updated_at=$5 WHERE workspace_id=$1 AND id=$2 RETURNING id,parent_id,title,status::text,current_version_id,revision")
                    .bind(input.workspace_id).bind(input.document_id).bind(input.input.new_parent_id).bind(rank.to_string()).bind(input.command.now).fetch_one(&mut *tx).await.map_err(map_document_store)?;
                let result = document(&result_row)?;
                let tree_revision = bump_tree_revision(&mut tx, input.workspace_id, input.command.now).await?;
                append_event(&mut tx, OutboxEvent { workspace_id:input.workspace_id, aggregate_kind:"Document", aggregate_id:input.document_id, sequence:result.revision+1, event_type:"DocumentMoved.v1", payload:json!({"documentId":input.document_id,"beforeParentId":row.parent_id,"afterParentId":input.input.new_parent_id,"revision":result.revision,"treeRevision":tree_revision}), occurred_at:input.command.now }).await?;
                sqlx::query("DELETE FROM document_move_previews WHERE token_hash=$1").bind(input.preview_token_hash.0.as_slice()).execute(&mut *tx).await.map_err(map_store)?;
                complete_workspace(&mut tx, input.workspace_id, &input.command, 200, &result).await?;
                tx.commit().await.map_err(map_store)?;
                Ok(result)
            }).await
        })
    }

    fn draft<'a>(
        &'a self,
        actor: Uuid,
        workspace: Uuid,
        document_id: Uuid,
    ) -> BoxFuture<'a, Result<Draft, GovernanceError>> {
        Box::pin(async move {
            let mut tx = self.pool.begin().await.map_err(map_store)?;
            require_access(
                &mut tx,
                actor,
                workspace,
                document_id,
                Access::Contributor,
                false,
            )
            .await?;
            require_effective_active(&mut tx, workspace, document_id).await?;
            let result = load_draft(&mut tx, workspace, document_id)
                .await?
                .ok_or(GovernanceError::DraftNotFound)?;
            tx.commit().await.map_err(map_store)?;
            Ok(result)
        })
    }

    fn create_draft<'a>(
        &'a self,
        input: DraftCreate,
    ) -> BoxFuture<'a, Result<Draft, GovernanceError>> {
        Box::pin(async move {
            let mut tx = self.pool.begin().await.map_err(map_store)?;
            require_access(
                &mut tx,
                input.command.actor_id,
                input.workspace_id,
                input.document_id,
                Access::Contributor,
                false,
            )
            .await?;
            require_effective_active(&mut tx, input.workspace_id, input.document_id).await?;
            if let Some(replay) =
                begin_workspace::<Draft>(&mut tx, input.workspace_id, &input.command).await?
            {
                tx.commit().await.map_err(map_store)?;
                return Ok(replay);
            }
            let version_row = sqlx::query("SELECT d.current_version_id,v.content_json,v.schema_version FROM documents d LEFT JOIN published_versions v ON v.workspace_id=d.workspace_id AND v.id=d.current_version_id WHERE d.workspace_id=$1 AND d.id=$2 FOR UPDATE OF d")
                .bind(input.workspace_id).bind(input.document_id).fetch_optional(&mut *tx).await.map_err(map_store)?.ok_or(GovernanceError::DocumentNotFound)?;
            let current_version = version_row
                .get::<Option<Uuid>, _>("current_version_id")
                .map(|id| {
                    Ok::<_, GovernanceError>((
                        id,
                        version_row
                            .get::<Option<Value>, _>("content_json")
                            .ok_or(GovernanceError::Internal)?,
                        version_row
                            .get::<Option<i32>, _>("schema_version")
                            .ok_or(GovernanceError::Internal)?,
                    ))
                })
                .transpose()?;
            if let Some(existing) =
                load_draft(&mut tx, input.workspace_id, input.document_id).await?
            {
                complete_workspace(&mut tx, input.workspace_id, &input.command, 200, &existing)
                    .await?;
                tx.commit().await.map_err(map_store)?;
                return Ok(existing);
            }
            let (base_version, content, schema_version) = current_version
                .map_or((None, empty_content(), 1), |(id, content, version)| {
                    (Some(id), content, version)
                });
            let content =
                ValidatedContent::parse(content).map_err(|_| GovernanceError::Internal)?;
            sqlx::query("INSERT INTO drafts(id,workspace_id,document_id,base_version_id,content_json,schema_version,updated_by,created_at,updated_at) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$8)")
                .bind(input.id).bind(input.workspace_id).bind(input.document_id).bind(base_version).bind(content.as_value()).bind(schema_version).bind(input.command.actor_id).bind(input.command.now)
                .execute(&mut *tx).await.map_err(map_document_store)?;
            let result = Draft {
                id: input.id,
                document_id: input.document_id,
                base_version_id: base_version,
                revision: 0,
                schema_version,
                content_fingerprint: canonical_hash(content.as_value()),
                content: content.into_value(),
            };
            complete_workspace(&mut tx, input.workspace_id, &input.command, 200, &result).await?;
            tx.commit().await.map_err(map_store)?;
            Ok(result)
        })
    }

    fn acquire_lease<'a>(
        &'a self,
        input: LeaseAcquire,
    ) -> BoxFuture<'a, Result<EditLeaseView, GovernanceError>> {
        Box::pin(async move {
            let mut tx = self.pool.begin().await.map_err(map_store)?;
            let permission = require_access(
                &mut tx,
                input.actor_id,
                input.workspace_id,
                input.document_id,
                Access::Contributor,
                false,
            )
            .await?;
            require_effective_active(&mut tx, input.workspace_id, input.document_id).await?;
            if let Some(replay) =
                begin_workspace::<EditLeaseView>(&mut tx, input.workspace_id, &input.command)
                    .await?
            {
                tx.commit().await.map_err(map_store)?;
                return Ok(replay);
            }
            let document = lock_document(&mut tx, input.workspace_id, input.document_id).await?;
            check_revision(document.revision, input.expected_document_revision)?;
            let current = sqlx::query("SELECT holder_user_id,client_instance_id,expires_at,released_at,revision FROM edit_leases WHERE workspace_id=$1 AND document_id=$2 FOR UPDATE")
                .bind(input.workspace_id).bind(input.document_id).fetch_optional(&mut *tx).await.map_err(map_store)?;
            let now: DateTime<Utc> = sqlx::query_scalar("SELECT clock_timestamp()")
                .fetch_one(&mut *tx)
                .await
                .map_err(map_store)?;
            let available = current.as_ref().is_none_or(|row| {
                row.get::<Option<DateTime<Utc>>, _>("released_at").is_some()
                    || row.get::<DateTime<Utc>, _>("expires_at") <= now
            });
            if !(available
                || input.input.force
                    && permission.manage
                    && input
                        .input
                        .reason
                        .as_ref()
                        .is_some_and(|reason| !reason.trim().is_empty()))
            {
                return Err(GovernanceError::EditLeaseHeld {
                    expires_at: current
                        .as_ref()
                        .map(|row| row.get("expires_at"))
                        .unwrap_or(now),
                });
            }
            let revision = current
                .as_ref()
                .map_or(0, |row| row.get::<i64, _>("revision") + 1);
            let row = sqlx::query("INSERT INTO edit_leases(document_id,workspace_id,holder_user_id,client_instance_id,token_hash,expires_at,released_at,revision,acquired_at) VALUES($1,$2,$3,$4,$5,clock_timestamp()+interval '90 seconds',NULL,$6,clock_timestamp()) ON CONFLICT(document_id) DO UPDATE SET workspace_id=EXCLUDED.workspace_id,holder_user_id=EXCLUDED.holder_user_id,client_instance_id=EXCLUDED.client_instance_id,token_hash=EXCLUDED.token_hash,expires_at=EXCLUDED.expires_at,released_at=NULL,revision=EXCLUDED.revision,acquired_at=EXCLUDED.acquired_at RETURNING holder_user_id,client_instance_id,expires_at,revision")
                .bind(input.document_id).bind(input.workspace_id).bind(input.actor_id).bind(input.input.client_instance_id).bind(input.token_hash.0.as_slice()).bind(revision)
                .fetch_one(&mut *tx).await.map_err(map_store)?;
            let result = lease_view(&row);
            append_lease_event(
                &mut tx,
                input.workspace_id,
                input.document_id,
                &result,
                input.command.now,
            )
            .await?;
            complete_workspace(&mut tx, input.workspace_id, &input.command, 200, &result).await?;
            tx.commit().await.map_err(map_store)?;
            Ok(result)
        })
    }

    fn mutate_lease<'a>(
        &'a self,
        input: LeaseMutation,
    ) -> BoxFuture<'a, Result<Option<EditLeaseView>, GovernanceError>> {
        Box::pin(async move {
            let mut tx = self.pool.begin().await.map_err(map_store)?;
            if let Some(replay) = begin_workspace::<Option<EditLeaseView>>(
                &mut tx,
                input.workspace_id,
                &input.command,
            )
            .await?
            {
                tx.commit().await.map_err(map_store)?;
                return Ok(replay);
            }
            let row = sqlx::query("SELECT holder_user_id,client_instance_id,token_hash,expires_at,released_at,revision FROM edit_leases WHERE workspace_id=$1 AND document_id=$2 FOR UPDATE")
                .bind(input.workspace_id).bind(input.document_id).fetch_optional(&mut *tx).await.map_err(map_store)?.ok_or(GovernanceError::EditLeaseInvalid)?;
            validate_lease_row(
                &row,
                &input.token_hash,
                input.actor_id,
                input.client_instance_id,
                input.expected_lease_revision,
                &mut tx,
            )
            .await?;
            let revision = input.expected_lease_revision + 1;
            let result = if input.release {
                sqlx::query("UPDATE edit_leases SET released_at=clock_timestamp(),expires_at=clock_timestamp(),revision=$3 WHERE workspace_id=$1 AND document_id=$2")
                    .bind(input.workspace_id).bind(input.document_id).bind(revision).execute(&mut *tx).await.map_err(map_store)?;
                append_event(&mut tx, OutboxEvent { workspace_id:input.workspace_id, aggregate_kind:"EditLease", aggregate_id:input.document_id, sequence:revision+1, event_type:"LeaseChanged.v1", payload:json!({"documentId":input.document_id,"holderUserId":Value::Null,"expiresAt":Value::Null,"revision":revision}), occurred_at:input.command.now }).await?;
                None
            } else {
                let renewed = sqlx::query("UPDATE edit_leases SET expires_at=clock_timestamp()+interval '90 seconds',revision=$3 WHERE workspace_id=$1 AND document_id=$2 RETURNING holder_user_id,client_instance_id,expires_at,revision")
                    .bind(input.workspace_id).bind(input.document_id).bind(revision).fetch_one(&mut *tx).await.map_err(map_store)?;
                let lease = lease_view(&renewed);
                append_lease_event(
                    &mut tx,
                    input.workspace_id,
                    input.document_id,
                    &lease,
                    input.command.now,
                )
                .await?;
                Some(lease)
            };
            complete_workspace(
                &mut tx,
                input.workspace_id,
                &input.command,
                if input.release { 204 } else { 200 },
                &result,
            )
            .await?;
            tx.commit().await.map_err(map_store)?;
            Ok(result)
        })
    }

    fn apply_operations<'a>(
        &'a self,
        input: DraftMutation,
    ) -> BoxFuture<'a, Result<MutationResult, GovernanceError>> {
        Box::pin(async move {
            let mut tx = self.pool.begin().await.map_err(map_store)?;
            require_access(
                &mut tx,
                input.actor_id,
                input.workspace_id,
                input.document_id,
                Access::Contributor,
                false,
            )
            .await?;
            require_effective_active(&mut tx, input.workspace_id, input.document_id).await?;
            if let Some(replay) =
                begin_workspace::<MutationResult>(&mut tx, input.workspace_id, &input.command)
                    .await?
            {
                tx.commit().await.map_err(map_store)?;
                return Ok(replay);
            }
            let _ = lock_document(&mut tx, input.workspace_id, input.document_id).await?;
            let draft_row = sqlx::query("SELECT id,content_json,revision FROM drafts WHERE workspace_id=$1 AND document_id=$2 FOR UPDATE")
                .bind(input.workspace_id).bind(input.document_id).fetch_optional(&mut *tx).await.map_err(map_store)?.ok_or(GovernanceError::DraftNotFound)?;
            let lease = sqlx::query("SELECT holder_user_id,client_instance_id,token_hash,expires_at,released_at,revision FROM edit_leases WHERE workspace_id=$1 AND document_id=$2 FOR UPDATE")
                .bind(input.workspace_id).bind(input.document_id).fetch_optional(&mut *tx).await.map_err(map_store)?.ok_or(GovernanceError::EditLeaseInvalid)?;
            validate_lease_row_without_revision(
                &lease,
                &input.token_hash,
                input.actor_id,
                input.client_instance_id,
                &mut tx,
            )
            .await?;
            let current_revision: i64 = draft_row.get("revision");
            check_revision(current_revision, input.expected_draft_revision)?;
            let references =
                load_references(&mut tx, input.workspace_id, input.document_id).await?;
            let reduced = apply_operations(ReducerInput {
                content: draft_row.get("content_json"),
                base_revision: current_revision,
                operations: input.operations,
                references,
            })
            .map_err(map_reducer)?;
            apply_reference_effects(
                &mut tx,
                input.actor_id,
                input.workspace_id,
                input.document_id,
                &reduced.reference_effects,
                input.command.now,
            )
            .await?;
            let result = MutationResult {
                revision: current_revision + 1,
                content_fingerprint: reduced.content_fingerprint,
                applied_operation_ids: reduced.applied_operation_ids,
                inverse_operations: reduced.inverse_operations,
            };
            sqlx::query("UPDATE drafts SET content_json=$3,schema_version=1,revision=$4,updated_by=$5,updated_at=$6 WHERE workspace_id=$1 AND document_id=$2")
                .bind(input.workspace_id).bind(input.document_id).bind(reduced.content).bind(result.revision).bind(input.actor_id).bind(input.command.now).execute(&mut *tx).await.map_err(map_store)?;
            invalidate_reviews(
                &mut tx,
                input.workspace_id,
                &[input.document_id],
                input.command.now,
            )
            .await?;
            append_event(&mut tx, OutboxEvent { workspace_id:input.workspace_id, aggregate_kind:"Draft", aggregate_id:draft_row.get("id"), sequence:result.revision+1, event_type:"DraftChanged.v1", payload:json!({"documentId":input.document_id,"draftId":draft_row.get::<Uuid,_>("id"),"revision":result.revision,"operationIds":result.applied_operation_ids}), occurred_at:input.command.now }).await?;
            complete_workspace(&mut tx, input.workspace_id, &input.command, 200, &result).await?;
            tx.commit().await.map_err(map_store)?;
            Ok(result)
        })
    }
}

#[derive(Clone)]
struct LockedDocument {
    parent_id: Option<Uuid>,
    rank: String,
    title: String,
    status: DocumentStatus,
    revision: i64,
    purge_after: Option<DateTime<Utc>>,
}

async fn require_active_member(
    tx: &mut Transaction<'_, Postgres>,
    actor: Uuid,
    workspace: Uuid,
) -> Result<(), GovernanceError> {
    let exists:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM memberships m JOIN workspaces w ON w.id=m.workspace_id WHERE m.workspace_id=$1 AND m.user_id=$2 AND m.status='ACTIVE' AND w.status='ACTIVE')").bind(workspace).bind(actor).fetch_one(&mut **tx).await.map_err(map_store)?;
    if exists {
        Ok(())
    } else {
        Err(GovernanceError::WorkspaceNotFound)
    }
}
async fn require_workspace_role(
    pool: &PgPool,
    actor: Uuid,
    workspace: Uuid,
    admin: bool,
) -> Result<(), GovernanceError> {
    let role:Option<String>=sqlx::query_scalar("SELECT role::text FROM memberships WHERE workspace_id=$1 AND user_id=$2 AND status='ACTIVE'").bind(workspace).bind(actor).fetch_optional(pool).await.map_err(map_store)?;
    if role
        .as_deref()
        .is_some_and(|value| !admin || matches!(value, "ADMIN" | "OWNER"))
    {
        Ok(())
    } else {
        Err(GovernanceError::WorkspaceNotFound)
    }
}
pub(super) async fn require_access(
    tx: &mut Transaction<'_, Postgres>,
    actor: Uuid,
    workspace: Uuid,
    document: Uuid,
    minimum: Access,
    manage: bool,
) -> Result<adoc_application::permission::EffectivePermission, GovernanceError> {
    let snapshot = point_snapshot_tx(tx, actor, workspace, document).await?;
    let effective = resolve_permission_path(&snapshot.nodes)
        .map_err(|_| GovernanceError::Internal)?
        .0;
    if effective.access >= minimum && (!manage || effective.manage) {
        Ok(effective)
    } else {
        Err(GovernanceError::DocumentNotFound)
    }
}
async fn validate_destination(
    tx: &mut Transaction<'_, Postgres>,
    actor: Uuid,
    workspace: Uuid,
    parent: Option<Uuid>,
) -> Result<(), GovernanceError> {
    if let Some(parent) = parent {
        require_access(tx, actor, workspace, parent, Access::Contributor, false).await?;
        require_effective_active(tx, workspace, parent).await?;
    } else {
        require_active_member(tx, actor, workspace).await?;
    }
    Ok(())
}
pub(super) async fn require_effective_active(
    tx: &mut Transaction<'_, Postgres>,
    workspace: Uuid,
    document: Uuid,
) -> Result<(), GovernanceError> {
    let invalid:bool=sqlx::query_scalar("WITH RECURSIVE ancestry AS (SELECT id,parent_id,status FROM documents WHERE workspace_id=$1 AND id=$2 UNION ALL SELECT d.id,d.parent_id,d.status FROM documents d JOIN ancestry a ON a.parent_id=d.id WHERE d.workspace_id=$1) SELECT NOT EXISTS(SELECT 1 FROM ancestry WHERE status<>'ACTIVE')").bind(workspace).bind(document).fetch_optional(&mut **tx).await.map_err(map_store)?.ok_or(GovernanceError::DocumentNotFound)?;
    if invalid {
        Ok(())
    } else {
        Err(GovernanceError::DocumentEffectivelyTrashed)
    }
}
async fn ensure_no_cycle(
    tx: &mut Transaction<'_, Postgres>,
    workspace: Uuid,
    document: Uuid,
    parent: Option<Uuid>,
) -> Result<(), GovernanceError> {
    if let Some(parent) = parent {
        let cycle:bool=sqlx::query_scalar("WITH RECURSIVE ancestry AS (SELECT id,parent_id FROM documents WHERE workspace_id=$1 AND id=$2 UNION ALL SELECT d.id,d.parent_id FROM documents d JOIN ancestry a ON a.parent_id=d.id WHERE d.workspace_id=$1) SELECT EXISTS(SELECT 1 FROM ancestry WHERE id=$3)").bind(workspace).bind(parent).bind(document).fetch_one(&mut **tx).await.map_err(map_store)?;
        if cycle {
            return Err(GovernanceError::DocumentTreeCycle);
        }
    }
    Ok(())
}
async fn lock_document(
    tx: &mut Transaction<'_, Postgres>,
    workspace: Uuid,
    id: Uuid,
) -> Result<LockedDocument, GovernanceError> {
    let row=sqlx::query("SELECT parent_id,rank,title,status::text,revision,purge_after FROM documents WHERE workspace_id=$1 AND id=$2 FOR UPDATE").bind(workspace).bind(id).fetch_optional(&mut **tx).await.map_err(map_store)?.ok_or(GovernanceError::DocumentNotFound)?;
    Ok(LockedDocument {
        parent_id: row.get("parent_id"),
        rank: row.get("rank"),
        title: row.get("title"),
        status: status(row.get("status"))?,
        revision: row.get("revision"),
        purge_after: row.get("purge_after"),
    })
}
async fn lock_tree_revision(
    tx: &mut Transaction<'_, Postgres>,
    workspace: Uuid,
) -> Result<(), GovernanceError> {
    sqlx::query(
        "SELECT tree_revision FROM workspace_document_revisions WHERE workspace_id=$1 FOR UPDATE",
    )
    .bind(workspace)
    .fetch_optional(&mut **tx)
    .await
    .map_err(map_store)?
    .ok_or(GovernanceError::WorkspaceNotFound)?;
    Ok(())
}
async fn bump_tree_revision(
    tx: &mut Transaction<'_, Postgres>,
    workspace: Uuid,
    now: DateTime<Utc>,
) -> Result<i64, GovernanceError> {
    sqlx::query_scalar("UPDATE workspace_document_revisions SET tree_revision=tree_revision+1,updated_at=$2 WHERE workspace_id=$1 RETURNING tree_revision").bind(workspace).bind(now).fetch_one(&mut **tx).await.map_err(map_store)
}

async fn rank_anchors(
    tx: &mut Transaction<'_, Postgres>,
    workspace: Uuid,
    parent: Option<Uuid>,
    after: Option<Uuid>,
    exclude: Option<Uuid>,
) -> Result<(Option<Uuid>, Option<Uuid>, Option<String>, Option<String>), GovernanceError> {
    let rows=sqlx::query("SELECT id,rank FROM documents WHERE workspace_id=$1 AND parent_id IS NOT DISTINCT FROM $2 AND status='ACTIVE' AND ($3::uuid IS NULL OR id<>$3) ORDER BY rank COLLATE \"C\",id FOR UPDATE").bind(workspace).bind(parent).bind(exclude).fetch_all(&mut **tx).await.map_err(map_store)?;
    let index = if let Some(after) = after {
        rows.iter()
            .position(|row| row.get::<Uuid, _>("id") == after)
            .ok_or(GovernanceError::DocumentParentInvalid)?
            + 1
    } else {
        0
    };
    let after_row = index.checked_sub(1).and_then(|value| rows.get(value));
    let before_row = rows.get(index);
    Ok((
        after_row.map(|row| row.get("id")),
        before_row.map(|row| row.get("id")),
        after_row.map(|row| row.get("rank")),
        before_row.map(|row| row.get("rank")),
    ))
}
async fn rank_for(
    tx: &mut Transaction<'_, Postgres>,
    workspace: Uuid,
    parent: Option<Uuid>,
    after: Option<Uuid>,
    exclude: Option<Uuid>,
) -> Result<TreeRank, GovernanceError> {
    let (_, _, lower, upper) = rank_anchors(tx, workspace, parent, after, exclude).await?;
    let lower = lower
        .as_deref()
        .map(TreeRank::parse)
        .transpose()
        .map_err(|_| GovernanceError::Internal)?;
    let upper = upper
        .as_deref()
        .map(TreeRank::parse)
        .transpose()
        .map_err(|_| GovernanceError::Internal)?;
    if let Some(rank) = TreeRank::between(lower.as_ref(), upper.as_ref()) {
        return Ok(rank);
    }
    let rows=sqlx::query("SELECT id FROM documents WHERE workspace_id=$1 AND parent_id IS NOT DISTINCT FROM $2 AND status='ACTIVE' AND ($3::uuid IS NULL OR id<>$3) ORDER BY rank COLLATE \"C\",id FOR UPDATE").bind(workspace).bind(parent).bind(exclude).fetch_all(&mut **tx).await.map_err(map_store)?;
    let ranks =
        TreeRank::rebalance(rows.len()).map_err(|_| GovernanceError::DocumentRankConflict)?;
    for (row, rank) in rows.iter().zip(ranks) {
        sqlx::query("UPDATE documents SET rank=$2 WHERE id=$1")
            .bind(row.get::<Uuid, _>("id"))
            .bind(rank.to_string())
            .execute(&mut **tx)
            .await
            .map_err(map_store)?;
    }
    let (_, _, lower, upper) = rank_anchors(tx, workspace, parent, after, exclude).await?;
    TreeRank::between(
        lower
            .as_deref()
            .map(TreeRank::parse)
            .transpose()
            .map_err(|_| GovernanceError::Internal)?
            .as_ref(),
        upper
            .as_deref()
            .map(TreeRank::parse)
            .transpose()
            .map_err(|_| GovernanceError::Internal)?
            .as_ref(),
    )
    .ok_or(GovernanceError::DocumentRankConflict)
}
async fn subtree_count(
    tx: &mut Transaction<'_, Postgres>,
    workspace: Uuid,
    root: Uuid,
) -> Result<i64, GovernanceError> {
    sqlx::query_scalar("WITH RECURSIVE subtree AS (SELECT id FROM documents WHERE workspace_id=$1 AND id=$2 UNION ALL SELECT d.id FROM documents d JOIN subtree s ON d.parent_id=s.id WHERE d.workspace_id=$1) SELECT count(*) FROM subtree").bind(workspace).bind(root).fetch_one(&mut **tx).await.map_err(map_store)
}
async fn close_subtree_leases_and_reviews(
    tx: &mut Transaction<'_, Postgres>,
    workspace: Uuid,
    root: Uuid,
    now: DateTime<Utc>,
) -> Result<(), GovernanceError> {
    let ids=sqlx::query_scalar::<_,Uuid>("WITH RECURSIVE subtree AS (SELECT id FROM documents WHERE workspace_id=$1 AND id=$2 UNION ALL SELECT d.id FROM documents d JOIN subtree s ON d.parent_id=s.id WHERE d.workspace_id=$1) SELECT id FROM subtree ORDER BY id FOR UPDATE").bind(workspace).bind(root).fetch_all(&mut **tx).await.map_err(map_store)?;
    sqlx::query("UPDATE edit_leases SET released_at=$2,expires_at=$2,revision=revision+1 WHERE document_id=ANY($1) AND released_at IS NULL").bind(&ids).bind(now).execute(&mut **tx).await.map_err(map_store)?;
    invalidate_reviews(tx, workspace, &ids, now).await?;
    Ok(())
}

async fn load_references(
    tx: &mut Transaction<'_, Postgres>,
    workspace: Uuid,
    document: Uuid,
) -> Result<Vec<ReferenceSnapshot>, GovernanceError> {
    let rows = sqlx::query("SELECT id,source_region_json,target_kind,target_id,target_region_json FROM references_graph WHERE workspace_id=$1 AND source_kind='DOCUMENT' AND source_id=$2 AND deleted_at IS NULL ORDER BY id FOR UPDATE")
        .bind(workspace).bind(document).fetch_all(&mut **tx).await.map_err(map_store)?;
    rows.into_iter()
        .map(|row| {
            Ok(ReferenceSnapshot {
                reference_id: row.get("id"),
                source_region: serde_json::from_value(row.get("source_region_json"))
                    .map_err(|_| GovernanceError::Internal)?,
                target: ReferenceTarget {
                    kind: row.get("target_kind"),
                    id: row.get("target_id"),
                    region: row
                        .get::<Option<Value>, _>("target_region_json")
                        .map(|value| {
                            serde_json::from_value(value).map_err(|_| GovernanceError::Internal)
                        })
                        .transpose()?,
                },
            })
        })
        .collect()
}

async fn apply_reference_effects(
    tx: &mut Transaction<'_, Postgres>,
    actor: Uuid,
    workspace: Uuid,
    document: Uuid,
    effects: &[ReferenceEffect],
    now: DateTime<Utc>,
) -> Result<(), GovernanceError> {
    for effect in effects {
        match effect {
            ReferenceEffect::Add { reference } => {
                let title =
                    validate_reference_target(tx, actor, workspace, &reference.target).await?;
                let snapshot_hash =
                    canonical_hash(&json!({"title":title,"target":reference.target}));
                sqlx::query("INSERT INTO references_graph(id,workspace_id,source_kind,source_id,target_kind,target_id,target_region_json,source_region_json,snapshot_json,created_by,created_at) VALUES($1,$2,'DOCUMENT',$3,$4,$5,$6,$7,$8,$9,$10)")
                    .bind(reference.reference_id).bind(workspace).bind(document)
                    .bind(&reference.target.kind).bind(&reference.target.id)
                    .bind(reference.target.region.as_ref().map(|value| serde_json::to_value(value).map_err(|_| GovernanceError::Internal)).transpose()?)
                    .bind(serde_json::to_value(&reference.source_region).map_err(|_| GovernanceError::Internal)?)
                    .bind(json!({"title":title,"snapshotHash":snapshot_hash})).bind(actor).bind(now)
                    .execute(&mut **tx).await.map_err(map_reference_store)?;
                append_event(tx, OutboxEvent { workspace_id:workspace, aggregate_kind:"Reference", aggregate_id:reference.reference_id, sequence:1, event_type:"ReferenceChanged.v1", payload:json!({"referenceId":reference.reference_id,"sourceDocumentId":document,"action":"ADDED"}), occurred_at:now }).await?;
            }
            ReferenceEffect::Remove { reference } => {
                let affected = sqlx::query("UPDATE references_graph SET deleted_at=$4 WHERE workspace_id=$1 AND source_kind='DOCUMENT' AND source_id=$2 AND id=$3 AND deleted_at IS NULL")
                    .bind(workspace).bind(document).bind(reference.reference_id).bind(now).execute(&mut **tx).await.map_err(map_store)?.rows_affected();
                if affected == 0 {
                    return Err(GovernanceError::ReferenceNotFound);
                }
                append_event(tx, OutboxEvent { workspace_id:workspace, aggregate_kind:"Reference", aggregate_id:reference.reference_id, sequence:2, event_type:"ReferenceChanged.v1", payload:json!({"referenceId":reference.reference_id,"sourceDocumentId":document,"action":"REMOVED"}), occurred_at:now }).await?;
            }
        }
    }
    Ok(())
}

async fn validate_reference_target(
    tx: &mut Transaction<'_, Postgres>,
    actor: Uuid,
    workspace: Uuid,
    target: &ReferenceTarget,
) -> Result<String, GovernanceError> {
    match target.kind.as_str() {
        "DOCUMENT" | "REGION" => {
            let target_id =
                Uuid::parse_str(&target.id).map_err(|_| GovernanceError::ReferenceTargetInvalid)?;
            require_access(tx, actor, workspace, target_id, Access::Viewer, false)
                .await
                .map_err(|_| GovernanceError::ReferenceTargetInvalid)?;
            require_effective_active(tx, workspace, target_id)
                .await
                .map_err(|_| GovernanceError::ReferenceTargetInvalid)?;
            let row=sqlx::query("SELECT d.title,COALESCE(dr.content_json,pv.content_json) AS content_json FROM documents d LEFT JOIN drafts dr ON dr.workspace_id=d.workspace_id AND dr.document_id=d.id LEFT JOIN published_versions pv ON pv.workspace_id=d.workspace_id AND pv.id=d.current_version_id WHERE d.workspace_id=$1 AND d.id=$2 FOR SHARE OF d")
                .bind(workspace).bind(target_id).fetch_one(&mut **tx).await.map_err(map_store)?;
            if target.kind == "REGION" {
                let region = target
                    .region
                    .as_ref()
                    .ok_or(GovernanceError::ReferenceTargetInvalid)?;
                let content: Option<Value> = row.get("content_json");
                let resolution = content
                    .as_ref()
                    .and_then(|value| reanchor_region(value.clone(), region).ok())
                    .ok_or(GovernanceError::ReferenceTargetInvalid)?;
                if !matches!(
                    resolution.status,
                    RegionResolutionStatus::Resolved | RegionResolutionStatus::Moved
                ) {
                    return Err(GovernanceError::ReferenceTargetInvalid);
                }
            } else if target.region.is_some() {
                return Err(GovernanceError::ReferenceTargetInvalid);
            }
            Ok(row.get("title"))
        }
        "DISCUSSION" => {
            let target_id =
                Uuid::parse_str(&target.id).map_err(|_| GovernanceError::ReferenceTargetInvalid)?;
            let row=sqlx::query("SELECT document_id,title FROM discussions WHERE workspace_id=$1 AND id=$2 FOR SHARE")
                .bind(workspace).bind(target_id).fetch_optional(&mut **tx).await.map_err(map_store)?.ok_or(GovernanceError::ReferenceTargetInvalid)?;
            require_access(
                tx,
                actor,
                workspace,
                row.get("document_id"),
                Access::Viewer,
                false,
            )
            .await
            .map_err(|_| GovernanceError::ReferenceTargetInvalid)?;
            Ok(row.get("title"))
        }
        "VOCABULARY" => {
            let target_id =
                Uuid::parse_str(&target.id).map_err(|_| GovernanceError::ReferenceTargetInvalid)?;
            let title=sqlx::query_scalar("SELECT canonical_term FROM vocabulary_concepts WHERE workspace_id=$1 AND id=$2 FOR SHARE")
                .bind(workspace).bind(target_id).fetch_optional(&mut **tx).await.map_err(map_store)?.ok_or(GovernanceError::ReferenceTargetInvalid)?;
            Ok(title)
        }
        "EXTERNAL" => {
            let url = reqwest::Url::parse(&target.id)
                .map_err(|_| GovernanceError::ReferenceTargetInvalid)?;
            if url.scheme() != "https"
                || !url.username().is_empty()
                || url.password().is_some()
                || url.fragment().is_some()
                || target.region.is_some()
            {
                return Err(GovernanceError::ReferenceTargetInvalid);
            }
            Ok(url.host_str().unwrap_or("External link").to_owned())
        }
        _ => Err(GovernanceError::ReferenceTargetInvalid),
    }
}

fn map_reference_store(error: sqlx::Error) -> GovernanceError {
    if error
        .as_database_error()
        .is_some_and(|value| value.is_unique_violation())
    {
        GovernanceError::Validation
    } else {
        map_store(error)
    }
}

async fn load_draft(
    tx: &mut Transaction<'_, Postgres>,
    workspace: Uuid,
    document: Uuid,
) -> Result<Option<Draft>, GovernanceError> {
    let row=sqlx::query("SELECT id,document_id,base_version_id,content_json,schema_version,revision FROM drafts WHERE workspace_id=$1 AND document_id=$2").bind(workspace).bind(document).fetch_optional(&mut **tx).await.map_err(map_store)?;
    row.map(|row| {
        let content = ValidatedContent::parse(row.get("content_json"))
            .map_err(|_| GovernanceError::Internal)?;
        Ok(Draft {
            id: row.get("id"),
            document_id: row.get("document_id"),
            base_version_id: row.get("base_version_id"),
            revision: row.get("revision"),
            schema_version: row.get("schema_version"),
            content_fingerprint: canonical_hash(content.as_value()),
            content: content.into_value(),
        })
    })
    .transpose()
}
fn empty_content() -> Value {
    json!({"schemaVersion":1,"root":{"type":"doc","children":[]}})
}

async fn validate_lease_row(
    row: &PgRow,
    token: &TokenHash,
    actor: Uuid,
    client: Uuid,
    revision: i64,
    tx: &mut Transaction<'_, Postgres>,
) -> Result<(), GovernanceError> {
    if row.get::<i64, _>("revision") != revision {
        return Err(GovernanceError::RevisionConflict {
            current_revision: row.get("revision"),
        });
    }
    validate_lease_row_without_revision(row, token, actor, client, tx).await
}
async fn validate_lease_row_without_revision(
    row: &PgRow,
    token: &TokenHash,
    actor: Uuid,
    client: Uuid,
    tx: &mut Transaction<'_, Postgres>,
) -> Result<(), GovernanceError> {
    let now: DateTime<Utc> = sqlx::query_scalar("SELECT clock_timestamp()")
        .fetch_one(&mut **tx)
        .await
        .map_err(map_store)?;
    if row.get::<Option<DateTime<Utc>>, _>("released_at").is_some() {
        return Err(GovernanceError::EditLeaseInvalid);
    }
    if row.get::<DateTime<Utc>, _>("expires_at") <= now {
        return Err(GovernanceError::EditLeaseExpired);
    }
    let stored: Vec<u8> = row.get("token_hash");
    if row.get::<Uuid, _>("holder_user_id") != actor
        || row.get::<Uuid, _>("client_instance_id") != client
        || stored.as_slice().ct_eq(token.0.as_slice()).unwrap_u8() != 1
    {
        return Err(GovernanceError::EditLeaseInvalid);
    }
    Ok(())
}
fn lease_view(row: &PgRow) -> EditLeaseView {
    EditLeaseView {
        holder_user_id: row.get("holder_user_id"),
        client_instance_id: row.get("client_instance_id"),
        token: None,
        expires_at: row.get("expires_at"),
        revision: row.get("revision"),
    }
}
async fn append_lease_event(
    tx: &mut Transaction<'_, Postgres>,
    workspace: Uuid,
    document: Uuid,
    lease: &EditLeaseView,
    now: DateTime<Utc>,
) -> Result<(), GovernanceError> {
    append_event(tx,OutboxEvent{workspace_id:workspace,aggregate_kind:"EditLease",aggregate_id:document,sequence:lease.revision+1,event_type:"LeaseChanged.v1",payload:json!({"documentId":document,"holderUserId":lease.holder_user_id,"expiresAt":lease.expires_at,"revision":lease.revision}),occurred_at:now}).await
}
async fn append_document_changed(
    tx: &mut Transaction<'_, Postgres>,
    workspace: Uuid,
    document: &Document,
    tree_revision: i64,
    action: &str,
    now: DateTime<Utc>,
) -> Result<(), GovernanceError> {
    append_event(tx,OutboxEvent{workspace_id:workspace,aggregate_kind:"Document",aggregate_id:document.id,sequence:document.revision+1,event_type:"DocumentChanged.v1",payload:json!({"documentId":document.id,"revision":document.revision,"treeRevision":tree_revision,"action":action}),occurred_at:now}).await
}

fn build_tree(
    rows: &[PgRow],
    accessible: &BTreeSet<Uuid>,
) -> Result<Vec<DocumentTreeNode>, GovernanceError> {
    let mut docs = BTreeMap::new();
    let mut children: BTreeMap<Option<Uuid>, Vec<Uuid>> = BTreeMap::new();
    for row in rows {
        let item = document(row)?;
        if accessible.contains(&item.id) {
            children.entry(item.parent_id).or_default().push(item.id);
            docs.insert(item.id, item);
        }
    }
    fn build(
        parent: Option<Uuid>,
        docs: &BTreeMap<Uuid, Document>,
        children: &BTreeMap<Option<Uuid>, Vec<Uuid>>,
    ) -> Vec<DocumentTreeNode> {
        children
            .get(&parent)
            .into_iter()
            .flatten()
            .filter_map(|id| {
                docs.get(id).cloned().map(|document| DocumentTreeNode {
                    document,
                    children: build(Some(*id), docs, children),
                })
            })
            .collect()
    }
    Ok(build(None, &docs, &children))
}
fn document(row: &PgRow) -> Result<Document, GovernanceError> {
    Ok(Document {
        id: row.get("id"),
        title: row.get("title"),
        parent_id: row.get("parent_id"),
        status: status(row.get("status"))?,
        current_version_id: row.get("current_version_id"),
        revision: row.get("revision"),
    })
}
fn status(value: String) -> Result<DocumentStatus, GovernanceError> {
    match value.as_str() {
        "ACTIVE" => Ok(DocumentStatus::Active),
        "TRASHED" => Ok(DocumentStatus::Trashed),
        "PURGING" => Ok(DocumentStatus::Purging),
        _ => Err(GovernanceError::Internal),
    }
}
fn parse_trash_cursor(value: &str) -> Result<(DateTime<Utc>, Uuid), GovernanceError> {
    let (micros, id) = value.split_once('.').ok_or(GovernanceError::Validation)?;
    let micros = micros
        .parse::<i64>()
        .map_err(|_| GovernanceError::Validation)?;
    let timestamp = DateTime::from_timestamp_micros(micros).ok_or(GovernanceError::Validation)?;
    let id = Uuid::parse_str(id).map_err(|_| GovernanceError::Validation)?;
    Ok((timestamp, id))
}
fn format_trash_cursor(timestamp: DateTime<Utc>, id: Uuid) -> String {
    format!("{}.{}", timestamp.timestamp_micros(), id)
}
fn map_reducer(error: OperationError) -> GovernanceError {
    match error.code {
        OperationErrorCode::PreconditionFailed
        | OperationErrorCode::RegionNotFound
        | OperationErrorCode::RegionAmbiguous
        | OperationErrorCode::TargetConflict => GovernanceError::OperationPreconditionFailed,
        OperationErrorCode::NoEffect => GovernanceError::NoEffect,
        OperationErrorCode::SchemaInvalid
        | OperationErrorCode::ContentInvalid
        | OperationErrorCode::BatchInvalid
        | OperationErrorCode::DependencyInvalid
        | OperationErrorCode::LimitExceeded => GovernanceError::Validation,
    }
}
fn map_document_store(error: sqlx::Error) -> GovernanceError {
    if let sqlx::Error::Database(database) = &error {
        match database.constraint() {
            Some("documents_sibling_rank_idx") => return GovernanceError::DocumentRankConflict,
            Some("documents_workspace_id_parent_id_fkey") => {
                return GovernanceError::DocumentParentInvalid;
            }
            Some("drafts_document_id_key") => return GovernanceError::DraftExists,
            _ => {}
        }
    }
    map_store(error)
}
async fn retry_rank_conflict<F, Fut, T>(
    attempts: usize,
    mut operation: F,
) -> Result<T, GovernanceError>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, GovernanceError>>,
{
    let mut remaining = attempts;
    loop {
        match operation().await {
            Err(GovernanceError::DocumentRankConflict) if remaining > 1 => remaining -= 1,
            result => return result,
        }
    }
}
