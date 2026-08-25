use adoc_application::{
    document::{Draft, ValidatedContent, canonical_hash},
    governance::{Command, GovernanceError, PublishMode},
    identity::TokenHash,
    operations::{
        AuditAction, AuditEventInput, AuditTarget, AuditTargetKind, EventAudience, StreamAccess,
    },
    permission::{Access, PublishPolicy},
    publishing::{
        CreatePublicLinkCommand, DocumentDiff, PublicDocument, PublicLink, PublishCommand,
        PublishedVersion, PublishingRepository, RestoreVersionCommand, RevokePublicLinkCommand,
        VersionPage, structural_diff,
    },
};
use adoc_ports::BoxFuture;
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::{DateTime, Utc};
use serde_json::{Value, json};
use sqlx::{PgPool, Postgres, Row, Transaction, postgres::PgRow};
use subtle::ConstantTimeEq;
use uuid::Uuid;

use super::{
    PostgresStore, append_audit_event,
    document::{require_access, require_effective_active},
    file::sync_file_references,
    governance::{
        OutboxEvent, append_event, begin_workspace, check_revision, complete_workspace, map_store,
    },
    permission::load_effective_policy,
};

#[derive(Clone)]
pub struct PostgresPublishingRepository {
    pool: PgPool,
}

impl PostgresPublishingRepository {
    pub fn new(store: &PostgresStore) -> Self {
        Self {
            pool: store.pool().clone(),
        }
    }
}

impl PublishingRepository for PostgresPublishingRepository {
    fn list_versions<'a>(
        &'a self,
        actor: Uuid,
        workspace: Uuid,
        document: Uuid,
        cursor: Option<String>,
    ) -> BoxFuture<'a, Result<VersionPage, GovernanceError>> {
        Box::pin(async move {
            let cursor = cursor.map(|value| parse_cursor(&value)).transpose()?;
            let mut tx = self.pool.begin().await.map_err(map_store)?;
            require_access(&mut tx, actor, workspace, document, Access::Viewer, false).await?;
            require_effective_active(&mut tx, workspace, document).await?;
            let (number, id) = cursor.unzip();
            let statement = VERSION_SELECT.to_owned()
                + " WHERE pv.workspace_id=$1 AND pv.document_id=$2 AND ($3::bigint IS NULL OR (pv.number,pv.id)<($3,$4)) ORDER BY pv.number DESC,pv.id DESC LIMIT 51";
            let rows = sqlx::query(&statement)
                .bind(workspace)
                .bind(document)
                .bind(number)
                .bind(id)
                .fetch_all(&mut *tx)
                .await
                .map_err(map_store)?;
            let items = rows
                .iter()
                .take(50)
                .map(version)
                .collect::<Result<Vec<_>, _>>()?;
            let next_cursor =
                (rows.len() > 50).then(|| format_cursor(items.last().expect("page is nonempty")));
            tx.commit().await.map_err(map_store)?;
            Ok(VersionPage { items, next_cursor })
        })
    }
    fn get_version<'a>(
        &'a self,
        actor: Uuid,
        workspace: Uuid,
        document: Uuid,
        version_id: Uuid,
    ) -> BoxFuture<'a, Result<PublishedVersion, GovernanceError>> {
        Box::pin(async move {
            let mut tx = self.pool.begin().await.map_err(map_store)?;
            require_access(&mut tx, actor, workspace, document, Access::Viewer, false).await?;
            require_effective_active(&mut tx, workspace, document).await?;
            let result = load_version(&mut tx, workspace, document, version_id).await?;
            tx.commit().await.map_err(map_store)?;
            Ok(result)
        })
    }
    fn compare_versions<'a>(
        &'a self,
        actor: Uuid,
        workspace: Uuid,
        document: Uuid,
        from: Uuid,
        to: Uuid,
    ) -> BoxFuture<'a, Result<DocumentDiff, GovernanceError>> {
        Box::pin(async move {
            let mut tx = self.pool.begin().await.map_err(map_store)?;
            require_access(&mut tx, actor, workspace, document, Access::Viewer, false).await?;
            require_effective_active(&mut tx, workspace, document).await?;
            let from = load_version(&mut tx, workspace, document, from).await?;
            let to = load_version(&mut tx, workspace, document, to).await?;
            tx.commit().await.map_err(map_store)?;
            structural_diff(&from, &to).map_err(|_| GovernanceError::Internal)
        })
    }
    fn publish<'a>(
        &'a self,
        input: PublishCommand,
    ) -> BoxFuture<'a, Result<PublishedVersion, GovernanceError>> {
        Box::pin(async move {
            let mut tx = self.pool.begin().await.map_err(map_store)?;
            require_access(
                &mut tx,
                input.command.actor_id,
                input.workspace_id,
                input.document_id,
                Access::Editor,
                false,
            )
            .await?;
            if let Some(replay) =
                begin_workspace::<PublishedVersion>(&mut tx, input.workspace_id, &input.command)
                    .await?
            {
                tx.commit().await.map_err(map_store)?;
                return Ok(replay);
            }
            require_effective_active(&mut tx, input.workspace_id, input.document_id).await?;
            let document_row=sqlx::query("SELECT current_version_id,revision,status::text FROM documents WHERE workspace_id=$1 AND id=$2 FOR UPDATE")
                .bind(input.workspace_id).bind(input.document_id).fetch_optional(&mut *tx).await.map_err(map_store)?.ok_or(GovernanceError::DocumentNotFound)?;
            if document_row.get::<String, _>("status") != "ACTIVE" {
                return Err(GovernanceError::DocumentStateInvalid);
            }
            let current_version_id: Option<Uuid> = document_row.get("current_version_id");
            let draft_row=sqlx::query("SELECT id,base_version_id,content_json,schema_version,revision FROM drafts WHERE workspace_id=$1 AND document_id=$2 FOR UPDATE")
                .bind(input.workspace_id).bind(input.document_id).fetch_optional(&mut *tx).await.map_err(map_store)?.ok_or(GovernanceError::DraftNotFound)?;
            let draft_id: Uuid = draft_row.get("id");
            let draft_revision: i64 = draft_row.get("revision");
            check_revision(draft_revision, input.expected_draft_revision)?;
            let base_version_id: Option<Uuid> = draft_row.get("base_version_id");
            if base_version_id != current_version_id {
                return Err(GovernanceError::PublishBaseStale {
                    base_version_id,
                    current_version_id,
                    draft_id,
                });
            }
            validate_publish_lease(&mut tx, &input).await?;
            let policy =
                load_effective_policy(&mut *tx, input.workspace_id, input.document_id).await?;
            let review_snapshot = match policy.mode {
                PublishMode::Direct => json!({}),
                PublishMode::ReviewRequired => {
                    approved_review_snapshot(
                        &mut tx,
                        input.workspace_id,
                        input.document_id,
                        draft_id,
                        draft_revision,
                        &policy,
                    )
                    .await?
                }
            };
            let content = ValidatedContent::parse(draft_row.get("content_json"))
                .map_err(|_| GovernanceError::OperationPreconditionFailed)?
                .into_value();
            let fingerprint = canonical_hash(&content);
            let number: i64 = sqlx::query_scalar(
                "SELECT COALESCE(MAX(number),0)+1 FROM published_versions WHERE document_id=$1",
            )
            .bind(input.document_id)
            .fetch_one(&mut *tx)
            .await
            .map_err(map_store)?;
            sqlx::query("INSERT INTO published_versions(id,workspace_id,document_id,number,content_json,schema_version,content_fingerprint,based_on_version_id,source_draft_revision,publisher_id,summary,published_at) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12)")
                .bind(input.version_id).bind(input.workspace_id).bind(input.document_id).bind(number).bind(&content).bind(draft_row.get::<i32,_>("schema_version")).bind(&fingerprint).bind(base_version_id).bind(draft_revision).bind(input.command.actor_id).bind(&input.summary).bind(input.command.now).execute(&mut *tx).await.map_err(map_store)?;
            sync_file_references(
                &mut tx,
                input.workspace_id,
                "PUBLISHED_VERSION",
                input.version_id,
                &content,
            )
            .await?;
            sqlx::query("DELETE FROM file_references WHERE workspace_id=$1 AND owner_kind='DRAFT' AND owner_id=$2")
                .bind(input.workspace_id).bind(draft_id).execute(&mut *tx).await.map_err(map_store)?;
            sqlx::query("INSERT INTO version_context(version_id,review_snapshot_json,discussion_ids,source_revision) VALUES($1,$2,'{}'::uuid[],$3)")
                .bind(input.version_id).bind(&review_snapshot).bind(draft_revision).execute(&mut *tx).await.map_err(map_store)?;
            let document_revision:i64=sqlx::query_scalar("UPDATE documents SET current_version_id=$3,revision=revision+1,updated_at=$4 WHERE workspace_id=$1 AND id=$2 RETURNING revision")
                .bind(input.workspace_id).bind(input.document_id).bind(input.version_id).bind(input.command.now).fetch_one(&mut *tx).await.map_err(map_store)?;
            sqlx::query("DELETE FROM drafts WHERE workspace_id=$1 AND document_id=$2")
                .bind(input.workspace_id)
                .bind(input.document_id)
                .execute(&mut *tx)
                .await
                .map_err(map_store)?;
            sqlx::query("UPDATE edit_leases SET released_at=COALESCE(released_at,$3),revision=revision+1 WHERE workspace_id=$1 AND document_id=$2 AND released_at IS NULL")
                .bind(input.workspace_id).bind(input.document_id).bind(input.command.now).execute(&mut *tx).await.map_err(map_store)?;
            let result = load_version(
                &mut tx,
                input.workspace_id,
                input.document_id,
                input.version_id,
            )
            .await?;
            let tree_revision: i64 = sqlx::query_scalar(
                "SELECT tree_revision FROM workspace_document_revisions WHERE workspace_id=$1",
            )
            .bind(input.workspace_id)
            .fetch_one(&mut *tx)
            .await
            .map_err(map_store)?;
            append_event(&mut tx,OutboxEvent{workspace_id:input.workspace_id,aggregate_kind:"Version",aggregate_id:input.version_id,sequence:1,event_type:"VersionPublished.v1",payload:json!({"documentId":input.document_id,"versionId":input.version_id,"number":number,"sourceDraftRevision":draft_revision}),audience:EventAudience::document(input.document_id,StreamAccess::Viewer),occurred_at:input.command.now}).await?;
            append_event(&mut tx,OutboxEvent{workspace_id:input.workspace_id,aggregate_kind:"Document",aggregate_id:input.document_id,sequence:document_revision+1,event_type:"DocumentChanged.v1",payload:json!({"documentId":input.document_id,"action":"PUBLISHED","revision":document_revision,"treeRevision":tree_revision}),audience:EventAudience::document(input.document_id,StreamAccess::Viewer),occurred_at:input.command.now}).await?;
            audit_publish(
                &mut tx,
                &input.command,
                input.workspace_id,
                AuditAction::VersionPublished,
                AuditTargetKind::Version,
                input.version_id,
            )
            .await?;
            complete_workspace(&mut tx, input.workspace_id, &input.command, 201, &result).await?;
            tx.commit().await.map_err(map_store)?;
            Ok(result)
        })
    }
    fn restore_version<'a>(
        &'a self,
        input: RestoreVersionCommand,
    ) -> BoxFuture<'a, Result<Draft, GovernanceError>> {
        Box::pin(async move {
            let mut tx = self.pool.begin().await.map_err(map_store)?;
            require_access(
                &mut tx,
                input.command.actor_id,
                input.workspace_id,
                input.document_id,
                Access::Editor,
                false,
            )
            .await?;
            if let Some(replay) =
                begin_workspace::<Draft>(&mut tx, input.workspace_id, &input.command).await?
            {
                tx.commit().await.map_err(map_store)?;
                return Ok(replay);
            }
            require_effective_active(&mut tx, input.workspace_id, input.document_id).await?;
            let revision:i64=sqlx::query_scalar("SELECT revision FROM documents WHERE workspace_id=$1 AND id=$2 AND status='ACTIVE' FOR UPDATE")
                .bind(input.workspace_id).bind(input.document_id).fetch_optional(&mut *tx).await.map_err(map_store)?.ok_or(GovernanceError::DocumentNotFound)?;
            check_revision(revision, input.expected_document_revision)?;
            let version = load_version(
                &mut tx,
                input.workspace_id,
                input.document_id,
                input.version_id,
            )
            .await?;
            ValidatedContent::parse(version.content.clone())
                .map_err(|_| GovernanceError::OperationPreconditionFailed)?;
            let inserted=sqlx::query("INSERT INTO drafts(id,workspace_id,document_id,base_version_id,content_json,schema_version,revision,updated_by,created_at,updated_at) VALUES($1,$2,$3,$4,$5,$6,0,$7,$8,$8) ON CONFLICT(document_id) DO NOTHING RETURNING id,document_id,base_version_id,content_json,schema_version,revision")
                .bind(input.draft_id).bind(input.workspace_id).bind(input.document_id).bind(input.version_id).bind(&version.content).bind(version.schema_version).bind(input.command.actor_id).bind(input.command.now).fetch_optional(&mut *tx).await.map_err(map_store)?.ok_or(GovernanceError::DraftExists)?;
            sync_file_references(
                &mut tx,
                input.workspace_id,
                "DRAFT",
                input.draft_id,
                &version.content,
            )
            .await?;
            let result = draft(&inserted)?;
            append_event(&mut tx,OutboxEvent{workspace_id:input.workspace_id,aggregate_kind:"Draft",aggregate_id:input.draft_id,sequence:1,event_type:"DraftChanged.v1",payload:json!({"documentId":input.document_id,"draftId":input.draft_id,"revision":0,"operationIds":[]}),audience:EventAudience::document(input.document_id,StreamAccess::Contributor),occurred_at:input.command.now}).await?;
            audit_publish(
                &mut tx,
                &input.command,
                input.workspace_id,
                AuditAction::DraftCreated,
                AuditTargetKind::Draft,
                input.draft_id,
            )
            .await?;
            complete_workspace(&mut tx, input.workspace_id, &input.command, 201, &result).await?;
            tx.commit().await.map_err(map_store)?;
            Ok(result)
        })
    }
    fn list_public_links<'a>(
        &'a self,
        actor: Uuid,
        workspace: Uuid,
        document: Uuid,
    ) -> BoxFuture<'a, Result<Vec<PublicLink>, GovernanceError>> {
        Box::pin(async move {
            let mut tx = self.pool.begin().await.map_err(map_store)?;
            require_access(&mut tx, actor, workspace, document, Access::Editor, true).await?;
            require_effective_active(&mut tx, workspace, document).await?;
            let rows=sqlx::query("SELECT id,expires_at,revoked_at,created_at,revision FROM public_links WHERE workspace_id=$1 AND document_id=$2 ORDER BY created_at DESC,id DESC").bind(workspace).bind(document).fetch_all(&mut *tx).await.map_err(map_store)?;
            let result = rows.iter().map(public_link).collect();
            tx.commit().await.map_err(map_store)?;
            result
        })
    }
    fn create_public_link<'a>(
        &'a self,
        input: CreatePublicLinkCommand,
    ) -> BoxFuture<'a, Result<Uuid, GovernanceError>> {
        Box::pin(async move {
            let mut tx = self.pool.begin().await.map_err(map_store)?;
            require_access(
                &mut tx,
                input.command.actor_id,
                input.workspace_id,
                input.document_id,
                Access::Editor,
                true,
            )
            .await?;
            if begin_workspace::<Uuid>(&mut tx, input.workspace_id, &input.command)
                .await?
                .is_some()
            {
                return Err(GovernanceError::PublicLinkTokenAlreadyIssued);
            }
            require_effective_active(&mut tx, input.workspace_id, input.document_id).await?;
            let row=sqlx::query("SELECT revision,current_version_id FROM documents WHERE workspace_id=$1 AND id=$2 FOR UPDATE").bind(input.workspace_id).bind(input.document_id).fetch_optional(&mut *tx).await.map_err(map_store)?.ok_or(GovernanceError::DocumentNotFound)?;
            check_revision(row.get("revision"), input.expected_document_revision)?;
            if row.get::<Option<Uuid>, _>("current_version_id").is_none() {
                return Err(GovernanceError::DocumentUnpublished);
            }
            sqlx::query("INSERT INTO public_links(id,workspace_id,document_id,token_hash,expires_at,created_by,created_at) VALUES($1,$2,$3,$4,$5,$6,$7)").bind(input.link_id).bind(input.workspace_id).bind(input.document_id).bind(input.token_hash.0.as_slice()).bind(input.expires_at).bind(input.command.actor_id).bind(input.command.now).execute(&mut *tx).await.map_err(map_store)?;
            append_event(
                &mut tx,
                OutboxEvent {
                    workspace_id: input.workspace_id,
                    aggregate_kind: "PublicLink",
                    aggregate_id: input.link_id,
                    sequence: 1,
                    event_type: "PublicLinkChanged.v1",
                    payload: json!({"entityId":input.link_id,"revision":0,"action":"CREATED"}),
                    audience: EventAudience::document(input.document_id, StreamAccess::Editor),
                    occurred_at: input.command.now,
                },
            )
            .await?;
            audit_publish(
                &mut tx,
                &input.command,
                input.workspace_id,
                AuditAction::PublicLinkCreated,
                AuditTargetKind::PublicLink,
                input.link_id,
            )
            .await?;
            complete_workspace(
                &mut tx,
                input.workspace_id,
                &input.command,
                201,
                &input.link_id,
            )
            .await?;
            tx.commit().await.map_err(map_store)?;
            Ok(input.link_id)
        })
    }
    fn revoke_public_link<'a>(
        &'a self,
        input: RevokePublicLinkCommand,
    ) -> BoxFuture<'a, Result<(), GovernanceError>> {
        Box::pin(async move {
            let mut tx = self.pool.begin().await.map_err(map_store)?;
            require_access(
                &mut tx,
                input.command.actor_id,
                input.workspace_id,
                input.document_id,
                Access::Editor,
                true,
            )
            .await?;
            if begin_workspace::<Value>(&mut tx, input.workspace_id, &input.command)
                .await?
                .is_some()
            {
                tx.commit().await.map_err(map_store)?;
                return Ok(());
            }
            let row=sqlx::query("SELECT revision,revoked_at FROM public_links WHERE workspace_id=$1 AND document_id=$2 AND id=$3 FOR UPDATE").bind(input.workspace_id).bind(input.document_id).bind(input.link_id).fetch_optional(&mut *tx).await.map_err(map_store)?.ok_or(GovernanceError::PublicLinkStateInvalid)?;
            check_revision(row.get("revision"), input.expected_link_revision)?;
            if row.get::<Option<DateTime<Utc>>, _>("revoked_at").is_some() {
                return Err(GovernanceError::PublicLinkStateInvalid);
            }
            let revision:i64=sqlx::query_scalar("UPDATE public_links SET revoked_at=$4,revision=revision+1 WHERE workspace_id=$1 AND document_id=$2 AND id=$3 RETURNING revision").bind(input.workspace_id).bind(input.document_id).bind(input.link_id).bind(input.command.now).fetch_one(&mut *tx).await.map_err(map_store)?;
            append_event(&mut tx,OutboxEvent{workspace_id:input.workspace_id,aggregate_kind:"PublicLink",aggregate_id:input.link_id,sequence:revision+1,event_type:"PublicLinkChanged.v1",payload:json!({"entityId":input.link_id,"revision":revision,"action":"INVALIDATED"}),audience:EventAudience::document(input.document_id,StreamAccess::Editor),occurred_at:input.command.now}).await?;
            audit_publish(
                &mut tx,
                &input.command,
                input.workspace_id,
                AuditAction::PublicLinkRevoked,
                AuditTargetKind::PublicLink,
                input.link_id,
            )
            .await?;
            complete_workspace(&mut tx, input.workspace_id, &input.command, 204, &json!({}))
                .await?;
            tx.commit().await.map_err(map_store)?;
            Ok(())
        })
    }
    fn public_document<'a>(
        &'a self,
        token_hash: TokenHash,
        now: DateTime<Utc>,
    ) -> BoxFuture<'a, Result<PublicDocument, GovernanceError>> {
        Box::pin(async move {
            let row=sqlx::query("SELECT d.title,pv.number,pv.published_at,pv.schema_version,pv.content_json FROM public_links pl JOIN documents d ON d.workspace_id=pl.workspace_id AND d.id=pl.document_id JOIN published_versions pv ON pv.workspace_id=d.workspace_id AND pv.id=d.current_version_id WHERE pl.token_hash=$1 AND pl.revoked_at IS NULL AND (pl.expires_at IS NULL OR pl.expires_at>$2) AND d.status='ACTIVE'").bind(token_hash.0.as_slice()).bind(now).fetch_optional(&self.pool).await.map_err(map_store)?.ok_or(GovernanceError::PublicLinkInvalid)?;
            Ok(PublicDocument {
                title: row.get("title"),
                version_number: row.get("number"),
                published_at: row.get("published_at"),
                schema_version: row.get("schema_version"),
                content: row.get("content_json"),
            })
        })
    }
}

async fn audit_publish(
    tx: &mut Transaction<'_, Postgres>,
    command: &Command,
    workspace: Uuid,
    action: AuditAction,
    kind: AuditTargetKind,
    id: Uuid,
) -> Result<(), GovernanceError> {
    append_audit_event(
        tx,
        AuditEventInput::user(
            workspace,
            command.actor_id,
            action,
            AuditTarget { kind, id },
            command.now,
            &command.idempotency_key,
        ),
    )
    .await?;
    Ok(())
}

async fn approved_review_snapshot(
    tx: &mut Transaction<'_, Postgres>,
    workspace: Uuid,
    document: Uuid,
    draft: Uuid,
    draft_revision: i64,
    policy: &PublishPolicy,
) -> Result<Value, GovernanceError> {
    let row=sqlx::query("SELECT id,policy_snapshot_json,revision FROM reviews WHERE workspace_id=$1 AND document_id=$2 AND draft_id=$3 AND draft_revision=$4 AND status='APPROVED' FOR UPDATE").bind(workspace).bind(document).bind(draft).bind(draft_revision).fetch_optional(&mut **tx).await.map_err(map_store)?.ok_or(GovernanceError::PublishReviewRequired)?;
    let snapshot: Value = row.get("policy_snapshot_json");
    let source = serde_json::to_value(policy.inherited_from_document_id)
        .map_err(|_| GovernanceError::Internal)?;
    if snapshot.get("policyRevision").and_then(Value::as_i64) != Some(policy.revision)
        || snapshot.get("sourceDocumentId") != Some(&source)
    {
        return Err(GovernanceError::PublishReviewRequired);
    }
    let required = snapshot
        .get("requiredApprovals")
        .and_then(Value::as_u64)
        .ok_or(GovernanceError::Internal)? as usize;
    let approvals=sqlx::query("SELECT reviewer_id,revision,decided_at FROM review_assignments WHERE workspace_id=$1 AND review_id=$2 AND decision='APPROVED' ORDER BY reviewer_id FOR UPDATE").bind(workspace).bind(row.get::<Uuid,_>("id")).fetch_all(&mut **tx).await.map_err(map_store)?;
    let mut approved = Vec::new();
    for assignment in approvals {
        let reviewer: Uuid = assignment.get("reviewer_id");
        if require_access(tx, reviewer, workspace, document, Access::Viewer, false)
            .await
            .is_err()
        {
            return Err(GovernanceError::PublishReviewRequired);
        }
        approved.push(json!({"reviewerId":reviewer,"assignmentRevision":assignment.get::<i64,_>("revision"),"decidedAt":assignment.get::<Option<DateTime<Utc>>,_>("decided_at")}));
    }
    if approved.len() < required {
        return Err(GovernanceError::PublishReviewRequired);
    }
    Ok(
        json!({"reviewId":row.get::<Uuid,_>("id"),"reviewRevision":row.get::<i64,_>("revision"),"policy":snapshot,"approvals":approved}),
    )
}

const VERSION_SELECT: &str = "SELECT pv.id,pv.document_id,pv.number,pv.content_json,pv.schema_version,pv.content_fingerprint,pv.based_on_version_id,pv.source_draft_revision,pv.publisher_id,pv.summary,pv.published_at,vc.review_snapshot_json,vc.discussion_ids FROM published_versions pv JOIN version_context vc ON vc.version_id=pv.id";
async fn load_version(
    tx: &mut Transaction<'_, Postgres>,
    workspace: Uuid,
    document: Uuid,
    id: Uuid,
) -> Result<PublishedVersion, GovernanceError> {
    let row = sqlx::query(
        &(VERSION_SELECT.to_owned()
            + " WHERE pv.workspace_id=$1 AND pv.document_id=$2 AND pv.id=$3"),
    )
    .bind(workspace)
    .bind(document)
    .bind(id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(map_store)?
    .ok_or(GovernanceError::VersionNotFound)?;
    version(&row)
}
fn version(row: &PgRow) -> Result<PublishedVersion, GovernanceError> {
    let result = PublishedVersion {
        id: row.try_get("id").map_err(|_| GovernanceError::Internal)?,
        document_id: row
            .try_get("document_id")
            .map_err(|_| GovernanceError::Internal)?,
        number: row
            .try_get("number")
            .map_err(|_| GovernanceError::Internal)?,
        content: row
            .try_get("content_json")
            .map_err(|_| GovernanceError::Internal)?,
        schema_version: row
            .try_get("schema_version")
            .map_err(|_| GovernanceError::Internal)?,
        content_fingerprint: row
            .try_get::<String, _>("content_fingerprint")
            .map_err(|_| GovernanceError::Internal)?
            .trim_end()
            .to_owned(),
        based_on_version_id: row
            .try_get("based_on_version_id")
            .map_err(|_| GovernanceError::Internal)?,
        source_draft_revision: row
            .try_get("source_draft_revision")
            .map_err(|_| GovernanceError::Internal)?,
        publisher_id: row
            .try_get("publisher_id")
            .map_err(|_| GovernanceError::Internal)?,
        summary: row
            .try_get("summary")
            .map_err(|_| GovernanceError::Internal)?,
        published_at: row
            .try_get("published_at")
            .map_err(|_| GovernanceError::Internal)?,
        review_snapshot: row
            .try_get("review_snapshot_json")
            .map_err(|_| GovernanceError::Internal)?,
        discussion_ids: row
            .try_get("discussion_ids")
            .map_err(|_| GovernanceError::Internal)?,
    };
    if result.validate_snapshot() {
        Ok(result)
    } else {
        Err(GovernanceError::Internal)
    }
}
fn draft(row: &PgRow) -> Result<Draft, GovernanceError> {
    let content: Value = row
        .try_get("content_json")
        .map_err(|_| GovernanceError::Internal)?;
    Ok(Draft {
        id: row.try_get("id").map_err(|_| GovernanceError::Internal)?,
        document_id: row
            .try_get("document_id")
            .map_err(|_| GovernanceError::Internal)?,
        base_version_id: row
            .try_get("base_version_id")
            .map_err(|_| GovernanceError::Internal)?,
        revision: row
            .try_get("revision")
            .map_err(|_| GovernanceError::Internal)?,
        schema_version: row
            .try_get("schema_version")
            .map_err(|_| GovernanceError::Internal)?,
        content_fingerprint: canonical_hash(&content),
        content,
    })
}
fn public_link(row: &PgRow) -> Result<PublicLink, GovernanceError> {
    Ok(PublicLink {
        id: row.try_get("id").map_err(|_| GovernanceError::Internal)?,
        expires_at: row
            .try_get("expires_at")
            .map_err(|_| GovernanceError::Internal)?,
        revoked_at: row
            .try_get("revoked_at")
            .map_err(|_| GovernanceError::Internal)?,
        created_at: row
            .try_get("created_at")
            .map_err(|_| GovernanceError::Internal)?,
        revision: row
            .try_get("revision")
            .map_err(|_| GovernanceError::Internal)?,
    })
}
async fn validate_publish_lease(
    tx: &mut Transaction<'_, Postgres>,
    input: &PublishCommand,
) -> Result<(), GovernanceError> {
    let row=sqlx::query("SELECT holder_user_id,client_instance_id,token_hash,expires_at,released_at FROM edit_leases WHERE workspace_id=$1 AND document_id=$2 FOR UPDATE").bind(input.workspace_id).bind(input.document_id).fetch_optional(&mut **tx).await.map_err(map_store)?;
    let Some(row) = row else { return Ok(()) };
    let active = row.get::<Option<DateTime<Utc>>, _>("released_at").is_none()
        && row.get::<DateTime<Utc>, _>("expires_at") > input.command.now;
    if !active {
        return Ok(());
    }
    let hash: Vec<u8> = row.get("token_hash");
    let supplied = input
        .lease_token_hash
        .as_ref()
        .is_some_and(|value| bool::from(hash.as_slice().ct_eq(value.0.as_slice())));
    if row.get::<Uuid, _>("holder_user_id") == input.command.actor_id
        && Some(row.get::<Uuid, _>("client_instance_id")) == input.client_instance_id
        && supplied
    {
        Ok(())
    } else {
        Err(GovernanceError::PublishLeaseConflict)
    }
}
fn format_cursor(value: &PublishedVersion) -> String {
    URL_SAFE_NO_PAD.encode(format!("{}:{}", value.number, value.id))
}
fn parse_cursor(value: &str) -> Result<(i64, Uuid), GovernanceError> {
    let decoded = URL_SAFE_NO_PAD
        .decode(value)
        .map_err(|_| GovernanceError::Validation)?;
    let text = String::from_utf8(decoded).map_err(|_| GovernanceError::Validation)?;
    let (number, id) = text.split_once(':').ok_or(GovernanceError::Validation)?;
    let number = number.parse().map_err(|_| GovernanceError::Validation)?;
    let id = Uuid::parse_str(id).map_err(|_| GovernanceError::Validation)?;
    Ok((number, id))
}
