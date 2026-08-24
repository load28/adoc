use std::collections::{BTreeMap, BTreeSet};

use adoc_application::{
    governance::{Command, GovernanceError},
    operations::{AuditAction, AuditEventInput, AuditTarget, AuditTargetKind},
    permission::{
        Access, AccessStamp, PermissionGrant, PermissionMutation, PermissionNode,
        PermissionRepository, PointSnapshot, PolicyMutation, PublishMode, PublishPolicy,
        ReviewerRule, ScopeSnapshot, SubjectKind, compile_permission_scope,
        resolve_permission_path,
    },
};
use adoc_ports::BoxFuture;
use serde_json::{Value, json};
use sqlx::{PgPool, Postgres, Row, Transaction, postgres::PgRow};
use uuid::Uuid;

use super::{
    PostgresStore, append_audit_event,
    governance::{
        OutboxEvent, append_event, begin_workspace, check_revision, complete_workspace, map_store,
    },
};

#[derive(Clone)]
pub struct PostgresPermissionRepository {
    pool: PgPool,
}

impl PostgresPermissionRepository {
    pub fn new(store: &PostgresStore) -> Self {
        Self {
            pool: store.pool().clone(),
        }
    }
}

impl PermissionRepository for PostgresPermissionRepository {
    fn access_stamp<'a>(
        &'a self,
        user_id: Uuid,
        workspace_id: Uuid,
        document_id: Uuid,
    ) -> BoxFuture<'a, Result<AccessStamp, GovernanceError>> {
        Box::pin(async move {
            let mut tx = self.pool.begin().await.map_err(map_store)?;
            let stamp = access_stamp_tx(&mut tx, user_id, workspace_id, document_id).await?;
            tx.commit().await.map_err(map_store)?;
            Ok(stamp)
        })
    }

    fn point_snapshot<'a>(
        &'a self,
        user_id: Uuid,
        workspace_id: Uuid,
        document_id: Uuid,
    ) -> BoxFuture<'a, Result<PointSnapshot, GovernanceError>> {
        Box::pin(async move {
            let mut tx = self.pool.begin().await.map_err(map_store)?;
            let snapshot = point_snapshot_tx(&mut tx, user_id, workspace_id, document_id).await?;
            tx.commit().await.map_err(map_store)?;
            Ok(snapshot)
        })
    }

    fn scope_snapshot<'a>(
        &'a self,
        user_id: Uuid,
        workspace_id: Uuid,
    ) -> BoxFuture<'a, Result<ScopeSnapshot, GovernanceError>> {
        Box::pin(async move {
            let mut tx = self.pool.begin().await.map_err(map_store)?;
            let snapshot = scope_snapshot_tx(&mut tx, user_id, workspace_id).await?;
            tx.commit().await.map_err(map_store)?;
            Ok(snapshot)
        })
    }

    fn permission_metadata<'a>(
        &'a self,
        workspace_id: Uuid,
        document_id: Uuid,
    ) -> BoxFuture<'a, Result<(Vec<PermissionGrant>, i64), GovernanceError>> {
        Box::pin(async move {
            let revision: i64 = sqlx::query_scalar("SELECT permission_revision FROM documents WHERE workspace_id=$1 AND id=$2 AND status<>'PURGING'")
                .bind(workspace_id).bind(document_id).fetch_optional(&self.pool).await.map_err(map_store)?.ok_or(GovernanceError::DocumentNotFound)?;
            let rows = sqlx::query("SELECT id,subject_kind::text,subject_id,access::text,can_manage,revision FROM permission_grants WHERE workspace_id=$1 AND document_id=$2 ORDER BY subject_kind,subject_id,id")
                .bind(workspace_id).bind(document_id).fetch_all(&self.pool).await.map_err(map_store)?;
            let grants = rows
                .iter()
                .map(permission_grant)
                .collect::<Result<_, _>>()?;
            Ok((grants, revision))
        })
    }

    fn subject_snapshot<'a>(
        &'a self,
        workspace_id: Uuid,
        document_id: Uuid,
        kind: SubjectKind,
        subject_id: Uuid,
    ) -> BoxFuture<'a, Result<PointSnapshot, GovernanceError>> {
        Box::pin(async move {
            let mut tx = self.pool.begin().await.map_err(map_store)?;
            let snapshot = match kind {
                SubjectKind::User => {
                    point_snapshot_tx(&mut tx, subject_id, workspace_id, document_id).await?
                }
                SubjectKind::Group => {
                    group_snapshot_tx(&mut tx, subject_id, workspace_id, document_id).await?
                }
            };
            tx.commit().await.map_err(map_store)?;
            Ok(snapshot)
        })
    }

    fn set_permission<'a>(
        &'a self,
        input: PermissionMutation,
    ) -> BoxFuture<'a, Result<PermissionGrant, GovernanceError>> {
        Box::pin(async move {
            let mut tx = self.pool.begin().await.map_err(map_store)?;
            lock_access_stamp(&mut tx, input.workspace_id).await?;
            require_manager(
                &mut tx,
                input.command.actor_id,
                input.workspace_id,
                input.document_id,
            )
            .await?;
            if let Some(replay) =
                begin_workspace::<PermissionGrant>(&mut tx, input.workspace_id, &input.command)
                    .await?
            {
                tx.commit().await.map_err(map_store)?;
                return Ok(replay);
            }
            let current_revision =
                lock_permission_revision(&mut tx, input.workspace_id, input.document_id).await?;
            check_revision(current_revision, input.expected_revision)?;
            let requested = input.input.ok_or(GovernanceError::Validation)?;
            validate_subject(
                &mut tx,
                input.workspace_id,
                requested.subject_kind,
                requested.subject_id,
            )
            .await?;
            let before = lock_grant_identity(
                &mut tx,
                input.workspace_id,
                input.document_id,
                input.grant_id,
                requested.subject_kind,
                requested.subject_id,
            )
            .await?;
            let row = sqlx::query("INSERT INTO permission_grants(id,workspace_id,document_id,subject_kind,subject_id,access,can_manage,granted_by,created_at,updated_at) VALUES($1,$2,$3,$4::subject_kind,$5,$6::document_access,$7,$8,$9,$9) ON CONFLICT(id) DO UPDATE SET access=EXCLUDED.access,can_manage=EXCLUDED.can_manage,revision=permission_grants.revision+1,updated_at=EXCLUDED.updated_at WHERE permission_grants.workspace_id=EXCLUDED.workspace_id AND permission_grants.document_id=EXCLUDED.document_id AND permission_grants.subject_kind=EXCLUDED.subject_kind AND permission_grants.subject_id=EXCLUDED.subject_id RETURNING id,subject_kind::text,subject_id,access::text,can_manage,revision")
                .bind(input.grant_id).bind(input.workspace_id).bind(input.document_id)
                .bind(subject_kind_text(requested.subject_kind)).bind(requested.subject_id)
                .bind(access_text(requested.access)).bind(requested.manage)
                .bind(input.command.actor_id).bind(input.command.now)
                .fetch_optional(&mut *tx).await.map_err(map_permission_store)?
                .ok_or(GovernanceError::PermissionGrantConflict)?;
            let result = permission_grant(&row)?;
            let new_revision =
                increment_permission_revision(&mut tx, input.workspace_id, input.document_id)
                    .await?;
            ensure_subtree_managers(&mut tx, input.workspace_id, input.document_id).await?;
            append_event(&mut tx, OutboxEvent {
                workspace_id: input.workspace_id,
                aggregate_kind: "Permission",
                aggregate_id: input.document_id,
                sequence: new_revision + 1,
                event_type: "PermissionChanged.v1",
                payload: json!({"documentId":input.document_id,"affectedRootId":input.document_id,"revision":new_revision,"before":before,"after":result}),
                occurred_at: input.command.now,
            }).await?;
            audit_permission(
                &mut tx,
                &input.command,
                input.workspace_id,
                AuditAction::PermissionChanged,
                AuditTargetKind::Permission,
                input.grant_id,
            )
            .await?;
            complete_workspace(&mut tx, input.workspace_id, &input.command, 200, &result).await?;
            tx.commit().await.map_err(map_store)?;
            Ok(result)
        })
    }

    fn delete_permission<'a>(
        &'a self,
        input: PermissionMutation,
    ) -> BoxFuture<'a, Result<(), GovernanceError>> {
        Box::pin(async move {
            let mut tx = self.pool.begin().await.map_err(map_store)?;
            lock_access_stamp(&mut tx, input.workspace_id).await?;
            require_manager(
                &mut tx,
                input.command.actor_id,
                input.workspace_id,
                input.document_id,
            )
            .await?;
            if begin_workspace::<Option<PermissionGrant>>(
                &mut tx,
                input.workspace_id,
                &input.command,
            )
            .await?
            .is_some()
            {
                tx.commit().await.map_err(map_store)?;
                return Ok(());
            }
            let current_revision =
                lock_permission_revision(&mut tx, input.workspace_id, input.document_id).await?;
            check_revision(current_revision, input.expected_revision)?;
            let row = sqlx::query("DELETE FROM permission_grants WHERE workspace_id=$1 AND document_id=$2 AND id=$3 RETURNING id,subject_kind::text,subject_id,access::text,can_manage,revision")
                .bind(input.workspace_id).bind(input.document_id).bind(input.grant_id)
                .fetch_optional(&mut *tx).await.map_err(map_store)?.ok_or(GovernanceError::DocumentNotFound)?;
            let before = permission_grant(&row)?;
            let new_revision =
                increment_permission_revision(&mut tx, input.workspace_id, input.document_id)
                    .await?;
            ensure_subtree_managers(&mut tx, input.workspace_id, input.document_id).await?;
            append_event(&mut tx, OutboxEvent {
                workspace_id: input.workspace_id,
                aggregate_kind: "Permission",
                aggregate_id: input.document_id,
                sequence: new_revision + 1,
                event_type: "PermissionChanged.v1",
                payload: json!({"documentId":input.document_id,"affectedRootId":input.document_id,"revision":new_revision,"before":before,"after":Value::Null}),
                occurred_at: input.command.now,
            }).await?;
            audit_permission(
                &mut tx,
                &input.command,
                input.workspace_id,
                AuditAction::PermissionChanged,
                AuditTargetKind::Permission,
                input.grant_id,
            )
            .await?;
            let empty: Option<PermissionGrant> = None;
            complete_workspace(&mut tx, input.workspace_id, &input.command, 204, &empty).await?;
            tx.commit().await.map_err(map_store)?;
            Ok(())
        })
    }

    fn effective_policy<'a>(
        &'a self,
        workspace_id: Uuid,
        document_id: Uuid,
    ) -> BoxFuture<'a, Result<PublishPolicy, GovernanceError>> {
        Box::pin(async move { load_effective_policy(&self.pool, workspace_id, document_id).await })
    }

    fn set_policy<'a>(
        &'a self,
        input: PolicyMutation,
    ) -> BoxFuture<'a, Result<PublishPolicy, GovernanceError>> {
        Box::pin(async move {
            let mut tx = self.pool.begin().await.map_err(map_store)?;
            lock_access_stamp(&mut tx, input.workspace_id).await?;
            require_manager(
                &mut tx,
                input.command.actor_id,
                input.workspace_id,
                input.document_id,
            )
            .await?;
            if let Some(replay) =
                begin_workspace::<PublishPolicy>(&mut tx, input.workspace_id, &input.command)
                    .await?
            {
                tx.commit().await.map_err(map_store)?;
                return Ok(replay);
            }
            let current_revision =
                lock_policy_revision(&mut tx, input.workspace_id, input.document_id).await?;
            check_revision(current_revision, input.expected_revision)?;
            validate_policy_candidates(
                &mut tx,
                input.workspace_id,
                input.document_id,
                &input.input.reviewer_rule,
                input.input.required_approvals,
            )
            .await?;
            let rule = serde_json::to_value(&input.input.reviewer_rule)
                .map_err(|_| GovernanceError::Internal)?;
            sqlx::query("INSERT INTO publish_policies(document_id,workspace_id,mode,required_approvals,reviewer_rule,updated_by,updated_at) VALUES($1,$2,$3::publish_mode,$4,$5,$6,$7) ON CONFLICT(document_id) DO UPDATE SET mode=EXCLUDED.mode,required_approvals=EXCLUDED.required_approvals,reviewer_rule=EXCLUDED.reviewer_rule,revision=publish_policies.revision+1,updated_by=EXCLUDED.updated_by,updated_at=EXCLUDED.updated_at")
                .bind(input.document_id).bind(input.workspace_id).bind(publish_mode_text(input.input.mode))
                .bind(input.input.required_approvals).bind(rule).bind(input.command.actor_id).bind(input.command.now)
                .execute(&mut *tx).await.map_err(map_policy_store)?;
            let new_revision =
                increment_policy_revision(&mut tx, input.workspace_id, input.document_id).await?;
            let result = PublishPolicy {
                document_id: input.document_id,
                mode: input.input.mode,
                required_approvals: input.input.required_approvals,
                reviewer_rule: input.input.reviewer_rule,
                inherited_from_document_id: None,
                revision: new_revision,
            };
            append_event(&mut tx, OutboxEvent {
                workspace_id: input.workspace_id,
                aggregate_kind: "PublishPolicy",
                aggregate_id: input.document_id,
                sequence: new_revision + 1,
                event_type: "PublishPolicyChanged.v1",
                payload: json!({"documentId":input.document_id,"revision":new_revision,"effectivePolicy":result}),
                occurred_at: input.command.now,
            }).await?;
            audit_permission(
                &mut tx,
                &input.command,
                input.workspace_id,
                AuditAction::PublishPolicyChanged,
                AuditTargetKind::PublishPolicy,
                input.document_id,
            )
            .await?;
            complete_workspace(&mut tx, input.workspace_id, &input.command, 200, &result).await?;
            tx.commit().await.map_err(map_store)?;
            Ok(result)
        })
    }
}

async fn audit_permission(
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

async fn access_stamp_tx(
    tx: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
    workspace_id: Uuid,
    document_id: Uuid,
) -> Result<AccessStamp, GovernanceError> {
    let row = sqlx::query("SELECT m.revision AS membership_revision,COALESCE(r.permission_revision,0) AS permission_revision,COALESCE(r.policy_revision,0) AS policy_revision FROM memberships m JOIN workspaces w ON w.id=m.workspace_id JOIN documents d ON d.workspace_id=m.workspace_id AND d.id=$3 LEFT JOIN workspace_access_revisions r ON r.workspace_id=m.workspace_id WHERE m.workspace_id=$1 AND m.user_id=$2 AND m.status='ACTIVE' AND w.status IN ('ACTIVE','DELETION_SCHEDULED') AND d.status<>'PURGING'")
        .bind(workspace_id).bind(user_id).bind(document_id).fetch_optional(&mut **tx).await.map_err(map_store)?.ok_or(GovernanceError::DocumentNotFound)?;
    Ok(AccessStamp {
        permission_revision: row.get("permission_revision"),
        policy_revision: row.get("policy_revision"),
        membership_revision: row.get("membership_revision"),
    })
}

pub(super) async fn point_snapshot_tx(
    tx: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
    workspace_id: Uuid,
    document_id: Uuid,
) -> Result<PointSnapshot, GovernanceError> {
    let stamp = access_stamp_tx(tx, user_id, workspace_id, document_id).await?;
    let nodes = load_point_nodes(tx, workspace_id, document_id, SubjectKind::User, user_id).await?;
    Ok(PointSnapshot {
        workspace_id,
        user_id,
        document_id,
        stamp,
        nodes,
    })
}

async fn group_snapshot_tx(
    tx: &mut Transaction<'_, Postgres>,
    group_id: Uuid,
    workspace_id: Uuid,
    document_id: Uuid,
) -> Result<PointSnapshot, GovernanceError> {
    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM groups WHERE workspace_id=$1 AND id=$2 AND deleted_at IS NULL)")
        .bind(workspace_id).bind(group_id).fetch_one(&mut **tx).await.map_err(map_store)?;
    if !exists {
        return Err(GovernanceError::PermissionSubjectInvalid);
    }
    let document_exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM documents WHERE workspace_id=$1 AND id=$2 AND status<>'PURGING')")
        .bind(workspace_id).bind(document_id).fetch_one(&mut **tx).await.map_err(map_store)?;
    if !document_exists {
        return Err(GovernanceError::DocumentNotFound);
    }
    let revision = sqlx::query("SELECT COALESCE(permission_revision,0) AS permission_revision,COALESCE(policy_revision,0) AS policy_revision FROM workspace_access_revisions WHERE workspace_id=$1")
        .bind(workspace_id).fetch_optional(&mut **tx).await.map_err(map_store)?;
    let stamp = revision.map_or(
        AccessStamp {
            permission_revision: 0,
            policy_revision: 0,
            membership_revision: 0,
        },
        |row| AccessStamp {
            permission_revision: row.get("permission_revision"),
            policy_revision: row.get("policy_revision"),
            membership_revision: 0,
        },
    );
    let nodes =
        load_point_nodes(tx, workspace_id, document_id, SubjectKind::Group, group_id).await?;
    Ok(PointSnapshot {
        workspace_id,
        user_id: group_id,
        document_id,
        stamp,
        nodes,
    })
}

pub(super) async fn scope_snapshot_tx(
    tx: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
    workspace_id: Uuid,
) -> Result<ScopeSnapshot, GovernanceError> {
    let row = sqlx::query("SELECT m.revision AS membership_revision,COALESCE(r.permission_revision,0) AS permission_revision,COALESCE(r.policy_revision,0) AS policy_revision FROM memberships m JOIN workspaces w ON w.id=m.workspace_id LEFT JOIN workspace_access_revisions r ON r.workspace_id=m.workspace_id WHERE m.workspace_id=$1 AND m.user_id=$2 AND m.status='ACTIVE' AND w.status IN ('ACTIVE','DELETION_SCHEDULED')")
        .bind(workspace_id).bind(user_id).fetch_optional(&mut **tx).await.map_err(map_store)?.ok_or(GovernanceError::WorkspaceNotFound)?;
    let stamp = AccessStamp {
        permission_revision: row.get("permission_revision"),
        policy_revision: row.get("policy_revision"),
        membership_revision: row.get("membership_revision"),
    };
    let nodes = load_scope_nodes(tx, workspace_id, user_id).await?;
    Ok(ScopeSnapshot {
        workspace_id,
        user_id,
        stamp,
        nodes,
    })
}

async fn load_point_nodes(
    tx: &mut Transaction<'_, Postgres>,
    workspace_id: Uuid,
    document_id: Uuid,
    kind: SubjectKind,
    subject_id: Uuid,
) -> Result<Vec<PermissionNode>, GovernanceError> {
    let rows = match kind {
        SubjectKind::User => sqlx::query("WITH RECURSIVE path AS (SELECT id,parent_id,0 AS depth FROM documents WHERE workspace_id=$1 AND id=$2 AND status<>'PURGING' UNION ALL SELECT d.id,d.parent_id,p.depth+1 FROM documents d JOIN path p ON p.parent_id=d.id WHERE d.workspace_id=$1 AND d.status<>'PURGING') SELECT p.id AS document_id,p.parent_id,p.depth,g.id AS grant_id,g.subject_kind::text AS subject_kind,g.subject_id,g.access::text AS access,g.can_manage,g.revision FROM path p LEFT JOIN permission_grants g ON g.workspace_id=$1 AND g.document_id=p.id AND ((g.subject_kind='USER' AND g.subject_id=$3) OR (g.subject_kind='GROUP' AND EXISTS(SELECT 1 FROM group_members gm JOIN groups ag ON ag.workspace_id=gm.workspace_id AND ag.id=gm.group_id AND ag.deleted_at IS NULL WHERE gm.workspace_id=$1 AND gm.user_id=$3 AND gm.group_id=g.subject_id))) ORDER BY p.depth,g.subject_kind,g.subject_id,g.id")
            .bind(workspace_id).bind(document_id).bind(subject_id).fetch_all(&mut **tx).await.map_err(map_store)?,
        SubjectKind::Group => sqlx::query("WITH RECURSIVE path AS (SELECT id,parent_id,0 AS depth FROM documents WHERE workspace_id=$1 AND id=$2 AND status<>'PURGING' UNION ALL SELECT d.id,d.parent_id,p.depth+1 FROM documents d JOIN path p ON p.parent_id=d.id WHERE d.workspace_id=$1 AND d.status<>'PURGING') SELECT p.id AS document_id,p.parent_id,p.depth,g.id AS grant_id,g.subject_kind::text AS subject_kind,g.subject_id,g.access::text AS access,g.can_manage,g.revision FROM path p LEFT JOIN permission_grants g ON g.workspace_id=$1 AND g.document_id=p.id AND g.subject_kind='GROUP' AND g.subject_id=$3 ORDER BY p.depth,g.id")
            .bind(workspace_id).bind(document_id).bind(subject_id).fetch_all(&mut **tx).await.map_err(map_store)?,
    };
    nodes_from_rows(&rows)
}

async fn load_scope_nodes(
    tx: &mut Transaction<'_, Postgres>,
    workspace_id: Uuid,
    user_id: Uuid,
) -> Result<Vec<PermissionNode>, GovernanceError> {
    let rows = sqlx::query("WITH RECURSIVE tree AS (SELECT id,parent_id,0 AS depth FROM documents WHERE workspace_id=$1 AND parent_id IS NULL AND status<>'PURGING' UNION ALL SELECT d.id,d.parent_id,t.depth+1 FROM documents d JOIN tree t ON d.parent_id=t.id WHERE d.workspace_id=$1 AND d.status<>'PURGING') SELECT t.id AS document_id,t.parent_id,t.depth,g.id AS grant_id,g.subject_kind::text AS subject_kind,g.subject_id,g.access::text AS access,g.can_manage,g.revision FROM tree t LEFT JOIN permission_grants g ON g.workspace_id=$1 AND g.document_id=t.id AND ((g.subject_kind='USER' AND g.subject_id=$2) OR (g.subject_kind='GROUP' AND EXISTS(SELECT 1 FROM group_members gm JOIN groups ag ON ag.workspace_id=gm.workspace_id AND ag.id=gm.group_id AND ag.deleted_at IS NULL WHERE gm.workspace_id=$1 AND gm.user_id=$2 AND gm.group_id=g.subject_id))) ORDER BY t.depth,t.id,g.subject_kind,g.subject_id,g.id")
        .bind(workspace_id).bind(user_id).fetch_all(&mut **tx).await.map_err(map_store)?;
    nodes_from_rows(&rows)
}

async fn load_all_user_scope_nodes(
    tx: &mut Transaction<'_, Postgres>,
    workspace_id: Uuid,
) -> Result<BTreeMap<Uuid, Vec<PermissionNode>>, GovernanceError> {
    let rows = sqlx::query("WITH RECURSIVE tree AS (SELECT id,parent_id,0 AS depth FROM documents WHERE workspace_id=$1 AND parent_id IS NULL AND status<>'PURGING' UNION ALL SELECT d.id,d.parent_id,t.depth+1 FROM documents d JOIN tree t ON d.parent_id=t.id WHERE d.workspace_id=$1 AND d.status<>'PURGING'), active_users AS (SELECT user_id FROM memberships WHERE workspace_id=$1 AND status='ACTIVE') SELECT u.user_id,t.id AS document_id,t.parent_id,t.depth,g.id AS grant_id,g.subject_kind::text AS subject_kind,g.subject_id,g.access::text AS access,g.can_manage,g.revision FROM active_users u CROSS JOIN tree t LEFT JOIN permission_grants g ON g.workspace_id=$1 AND g.document_id=t.id AND ((g.subject_kind='USER' AND g.subject_id=u.user_id) OR (g.subject_kind='GROUP' AND EXISTS(SELECT 1 FROM group_members gm JOIN groups ag ON ag.workspace_id=gm.workspace_id AND ag.id=gm.group_id AND ag.deleted_at IS NULL WHERE gm.workspace_id=$1 AND gm.user_id=u.user_id AND gm.group_id=g.subject_id))) ORDER BY u.user_id,t.depth,t.id,g.subject_kind,g.subject_id,g.id")
        .bind(workspace_id)
        .fetch_all(&mut **tx)
        .await
        .map_err(map_store)?;
    let mut grouped = BTreeMap::<Uuid, Vec<PgRow>>::new();
    for row in rows {
        grouped.entry(row.get("user_id")).or_default().push(row);
    }
    grouped
        .into_iter()
        .map(|(user_id, rows)| Ok((user_id, nodes_from_rows(&rows)?)))
        .collect()
}

fn nodes_from_rows(rows: &[PgRow]) -> Result<Vec<PermissionNode>, GovernanceError> {
    let mut nodes = Vec::<PermissionNode>::new();
    for row in rows {
        let document_id: Uuid = row.get("document_id");
        if nodes
            .last()
            .is_none_or(|node| node.document_id != document_id)
        {
            nodes.push(PermissionNode {
                document_id,
                parent_id: row.get("parent_id"),
                user_grant: None,
                group_grants: Vec::new(),
            });
        }
        if row.get::<Option<Uuid>, _>("grant_id").is_some() {
            let grant = permission_grant_optional(row)?;
            let node = nodes.last_mut().ok_or(GovernanceError::Internal)?;
            if grant.subject_kind == SubjectKind::User {
                node.user_grant = Some(grant);
            } else {
                node.group_grants.push(grant);
            }
        }
    }
    if nodes.is_empty() {
        Err(GovernanceError::DocumentNotFound)
    } else {
        Ok(nodes)
    }
}

async fn lock_access_stamp(
    tx: &mut Transaction<'_, Postgres>,
    workspace_id: Uuid,
) -> Result<(), GovernanceError> {
    sqlx::query("INSERT INTO workspace_access_revisions(workspace_id) SELECT id FROM workspaces WHERE id=$1 ON CONFLICT DO NOTHING").bind(workspace_id).execute(&mut **tx).await.map_err(map_store)?;
    sqlx::query(
        "SELECT workspace_id FROM workspace_access_revisions WHERE workspace_id=$1 FOR UPDATE",
    )
    .bind(workspace_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(map_store)?
    .ok_or(GovernanceError::WorkspaceNotFound)?;
    Ok(())
}

async fn require_manager(
    tx: &mut Transaction<'_, Postgres>,
    actor: Uuid,
    workspace: Uuid,
    document: Uuid,
) -> Result<(), GovernanceError> {
    let snapshot = point_snapshot_tx(tx, actor, workspace, document).await?;
    let effective = resolve_permission_path(&snapshot.nodes)
        .map_err(|_| GovernanceError::Internal)?
        .0;
    if effective.manage {
        Ok(())
    } else if effective.access.can_view() {
        Err(GovernanceError::PermissionDenied)
    } else {
        Err(GovernanceError::DocumentNotFound)
    }
}

async fn lock_permission_revision(
    tx: &mut Transaction<'_, Postgres>,
    workspace: Uuid,
    document: Uuid,
) -> Result<i64, GovernanceError> {
    sqlx::query_scalar("SELECT permission_revision FROM documents WHERE workspace_id=$1 AND id=$2 AND status<>'PURGING' FOR UPDATE").bind(workspace).bind(document).fetch_optional(&mut **tx).await.map_err(map_store)?.ok_or(GovernanceError::DocumentNotFound)
}
async fn lock_policy_revision(
    tx: &mut Transaction<'_, Postgres>,
    workspace: Uuid,
    document: Uuid,
) -> Result<i64, GovernanceError> {
    sqlx::query_scalar("SELECT policy_revision FROM documents WHERE workspace_id=$1 AND id=$2 AND status<>'PURGING' FOR UPDATE").bind(workspace).bind(document).fetch_optional(&mut **tx).await.map_err(map_store)?.ok_or(GovernanceError::DocumentNotFound)
}
async fn increment_permission_revision(
    tx: &mut Transaction<'_, Postgres>,
    workspace: Uuid,
    document: Uuid,
) -> Result<i64, GovernanceError> {
    sqlx::query_scalar("UPDATE documents SET permission_revision=permission_revision+1 WHERE workspace_id=$1 AND id=$2 RETURNING permission_revision").bind(workspace).bind(document).fetch_one(&mut **tx).await.map_err(map_store)
}
async fn increment_policy_revision(
    tx: &mut Transaction<'_, Postgres>,
    workspace: Uuid,
    document: Uuid,
) -> Result<i64, GovernanceError> {
    sqlx::query_scalar("UPDATE documents SET policy_revision=policy_revision+1 WHERE workspace_id=$1 AND id=$2 RETURNING policy_revision").bind(workspace).bind(document).fetch_one(&mut **tx).await.map_err(map_store)
}

async fn validate_subject(
    tx: &mut Transaction<'_, Postgres>,
    workspace: Uuid,
    kind: SubjectKind,
    id: Uuid,
) -> Result<(), GovernanceError> {
    let exists:bool=match kind{
        SubjectKind::User=>sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM memberships WHERE workspace_id=$1 AND user_id=$2 AND status='ACTIVE')").bind(workspace).bind(id).fetch_one(&mut **tx).await.map_err(map_store)?,
        SubjectKind::Group=>sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM groups WHERE workspace_id=$1 AND id=$2 AND deleted_at IS NULL)").bind(workspace).bind(id).fetch_one(&mut **tx).await.map_err(map_store)?,
    };
    if exists {
        Ok(())
    } else {
        Err(GovernanceError::PermissionSubjectInvalid)
    }
}

async fn lock_grant_identity(
    tx: &mut Transaction<'_, Postgres>,
    workspace: Uuid,
    document: Uuid,
    grant_id: Uuid,
    kind: SubjectKind,
    subject: Uuid,
) -> Result<Option<PermissionGrant>, GovernanceError> {
    let by_id=sqlx::query("SELECT workspace_id,document_id,id,subject_kind::text,subject_id,access::text,can_manage,revision FROM permission_grants WHERE id=$1 FOR UPDATE").bind(grant_id).fetch_optional(&mut **tx).await.map_err(map_store)?;
    if let Some(row) = by_id {
        if row.get::<Uuid, _>("workspace_id") != workspace
            || row.get::<Uuid, _>("document_id") != document
            || parse_subject(row.get::<String, _>("subject_kind").as_str())? != kind
            || row.get::<Uuid, _>("subject_id") != subject
        {
            return Err(GovernanceError::DocumentNotFound);
        }
        return Ok(Some(permission_grant(&row)?));
    }
    let duplicate:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM permission_grants WHERE workspace_id=$1 AND document_id=$2 AND subject_kind=$3::subject_kind AND subject_id=$4)").bind(workspace).bind(document).bind(subject_kind_text(kind)).bind(subject).fetch_one(&mut **tx).await.map_err(map_store)?;
    if duplicate {
        Err(GovernanceError::PermissionGrantConflict)
    } else {
        Ok(None)
    }
}

async fn ensure_subtree_managers(
    tx: &mut Transaction<'_, Postgres>,
    workspace: Uuid,
    root: Uuid,
) -> Result<(), GovernanceError> {
    let subtree=sqlx::query_scalar::<_,Uuid>("WITH RECURSIVE subtree AS (SELECT id FROM documents WHERE workspace_id=$1 AND id=$2 AND status<>'PURGING' UNION ALL SELECT d.id FROM documents d JOIN subtree s ON d.parent_id=s.id WHERE d.workspace_id=$1 AND d.status<>'PURGING') SELECT id FROM subtree").bind(workspace).bind(root).fetch_all(&mut **tx).await.map_err(map_store)?.into_iter().collect::<BTreeSet<_>>();
    let user_scopes = load_all_user_scope_nodes(tx, workspace).await?;
    let mut managed = BTreeSet::new();
    for nodes in user_scopes.values() {
        let scope = compile_permission_scope(nodes).map_err(|_| GovernanceError::Internal)?;
        managed.extend(scope.into_iter().filter_map(|(id, effective)| {
            (effective.manage && subtree.contains(&id)).then_some(id)
        }));
        if managed.len() == subtree.len() {
            return Ok(());
        }
    }
    Err(GovernanceError::PermissionLastManager)
}

async fn validate_policy_candidates(
    tx: &mut Transaction<'_, Postgres>,
    workspace: Uuid,
    document: Uuid,
    rule: &ReviewerRule,
    required: i16,
) -> Result<(), GovernanceError> {
    if required == 0 {
        return Ok(());
    }
    let candidates = match rule {
        ReviewerRule::AnyEditor => sqlx::query_scalar::<_, Uuid>(
            "SELECT user_id FROM memberships WHERE workspace_id=$1 AND status='ACTIVE'",
        )
        .bind(workspace)
        .fetch_all(&mut **tx)
        .await
        .map_err(map_store)?,
        ReviewerRule::Users { user_ids } => {
            let active=sqlx::query_scalar::<_,Uuid>("SELECT user_id FROM memberships WHERE workspace_id=$1 AND status='ACTIVE' AND user_id=ANY($2) ORDER BY user_id").bind(workspace).bind(user_ids).fetch_all(&mut **tx).await.map_err(map_store)?;
            if active.len() != user_ids.len() {
                return Err(GovernanceError::PublishPolicyInvalid);
            }
            active
        }
        ReviewerRule::Groups { group_ids } => {
            let count:i64=sqlx::query_scalar("SELECT count(*) FROM groups WHERE workspace_id=$1 AND deleted_at IS NULL AND id=ANY($2)").bind(workspace).bind(group_ids).fetch_one(&mut **tx).await.map_err(map_store)?;
            if count != group_ids.len() as i64 {
                return Err(GovernanceError::PublishPolicyInvalid);
            }
            sqlx::query_scalar::<_,Uuid>("SELECT DISTINCT gm.user_id FROM group_members gm JOIN memberships m ON m.workspace_id=gm.workspace_id AND m.user_id=gm.user_id AND m.status='ACTIVE' WHERE gm.workspace_id=$1 AND gm.group_id=ANY($2) ORDER BY gm.user_id").bind(workspace).bind(group_ids).fetch_all(&mut **tx).await.map_err(map_store)?
        }
    };
    let user_scopes = load_all_user_scope_nodes(tx, workspace).await?;
    let mut eligible = 0_i16;
    for user in candidates {
        let nodes = user_scopes
            .get(&user)
            .ok_or(GovernanceError::PublishPolicyInvalid)?;
        let effective = compile_permission_scope(nodes)
            .map_err(|_| GovernanceError::Internal)?
            .remove(&document)
            .ok_or(GovernanceError::DocumentNotFound)?;
        let allowed = match rule {
            ReviewerRule::AnyEditor => effective.access == Access::Editor,
            _ => effective.access.can_contribute(),
        };
        if allowed {
            eligible += 1
        }
    }
    if eligible >= required {
        Ok(())
    } else {
        Err(GovernanceError::PublishPolicyInvalid)
    }
}

pub(super) async fn load_effective_policy<'e, E>(
    executor: E,
    workspace: Uuid,
    document: Uuid,
) -> Result<PublishPolicy, GovernanceError>
where
    E: sqlx::Executor<'e, Database = Postgres>,
{
    let row=sqlx::query("WITH RECURSIVE path AS (SELECT id,parent_id,0 AS depth,policy_revision FROM documents WHERE workspace_id=$1 AND id=$2 AND status<>'PURGING' UNION ALL SELECT d.id,d.parent_id,p.depth+1,p.policy_revision FROM documents d JOIN path p ON p.parent_id=d.id WHERE d.workspace_id=$1 AND d.status<>'PURGING') SELECT p.id AS source_document_id,p.depth,p.policy_revision,pp.mode::text,pp.required_approvals,pp.reviewer_rule,w.default_publish_mode::text AS default_mode FROM path p JOIN workspaces w ON w.id=$1 LEFT JOIN publish_policies pp ON pp.workspace_id=$1 AND pp.document_id=p.id ORDER BY (pp.document_id IS NOT NULL) DESC,p.depth LIMIT 1")
        .bind(workspace).bind(document).fetch_optional(executor).await.map_err(map_store)?.ok_or(GovernanceError::DocumentNotFound)?;
    let target_revision: i64 = row.get("policy_revision");
    if let Some(mode) = row.get::<Option<String>, _>("mode") {
        let source: Uuid = row.get("source_document_id");
        Ok(PublishPolicy {
            document_id: document,
            mode: parse_publish_mode(&mode)?,
            required_approvals: row.get("required_approvals"),
            reviewer_rule: serde_json::from_value(row.get("reviewer_rule"))
                .map_err(|_| GovernanceError::Internal)?,
            inherited_from_document_id: (source != document).then_some(source),
            revision: target_revision,
        })
    } else {
        Ok(PublishPolicy {
            document_id: document,
            mode: parse_publish_mode(row.get::<String, _>("default_mode").as_str())?,
            required_approvals: 0,
            reviewer_rule: ReviewerRule::AnyEditor,
            inherited_from_document_id: None,
            revision: target_revision,
        })
    }
}

fn permission_grant(row: &PgRow) -> Result<PermissionGrant, GovernanceError> {
    Ok(PermissionGrant {
        id: row.try_get("id").map_err(|_| GovernanceError::Internal)?,
        subject_kind: parse_subject(
            row.try_get::<String, _>("subject_kind")
                .map_err(|_| GovernanceError::Internal)?
                .as_str(),
        )?,
        subject_id: row
            .try_get("subject_id")
            .map_err(|_| GovernanceError::Internal)?,
        access: parse_access(
            row.try_get::<String, _>("access")
                .map_err(|_| GovernanceError::Internal)?
                .as_str(),
        )?,
        manage: row
            .try_get("can_manage")
            .map_err(|_| GovernanceError::Internal)?,
        revision: row
            .try_get("revision")
            .map_err(|_| GovernanceError::Internal)?,
    })
}
fn permission_grant_optional(row: &PgRow) -> Result<PermissionGrant, GovernanceError> {
    Ok(PermissionGrant {
        id: row
            .try_get::<Option<Uuid>, _>("grant_id")
            .map_err(|_| GovernanceError::Internal)?
            .ok_or(GovernanceError::Internal)?,
        subject_kind: parse_subject(
            row.try_get::<Option<String>, _>("subject_kind")
                .map_err(|_| GovernanceError::Internal)?
                .ok_or(GovernanceError::Internal)?
                .as_str(),
        )?,
        subject_id: row
            .try_get::<Option<Uuid>, _>("subject_id")
            .map_err(|_| GovernanceError::Internal)?
            .ok_or(GovernanceError::Internal)?,
        access: parse_access(
            row.try_get::<Option<String>, _>("access")
                .map_err(|_| GovernanceError::Internal)?
                .ok_or(GovernanceError::Internal)?
                .as_str(),
        )?,
        manage: row
            .try_get::<Option<bool>, _>("can_manage")
            .map_err(|_| GovernanceError::Internal)?
            .ok_or(GovernanceError::Internal)?,
        revision: row
            .try_get::<Option<i64>, _>("revision")
            .map_err(|_| GovernanceError::Internal)?
            .ok_or(GovernanceError::Internal)?,
    })
}
fn parse_subject(value: &str) -> Result<SubjectKind, GovernanceError> {
    match value {
        "USER" => Ok(SubjectKind::User),
        "GROUP" => Ok(SubjectKind::Group),
        _ => Err(GovernanceError::Internal),
    }
}
fn subject_kind_text(value: SubjectKind) -> &'static str {
    match value {
        SubjectKind::User => "USER",
        SubjectKind::Group => "GROUP",
    }
}
fn parse_access(value: &str) -> Result<Access, GovernanceError> {
    match value {
        "NO_ACCESS" => Ok(Access::NoAccess),
        "VIEWER" => Ok(Access::Viewer),
        "CONTRIBUTOR" => Ok(Access::Contributor),
        "EDITOR" => Ok(Access::Editor),
        _ => Err(GovernanceError::Internal),
    }
}
fn access_text(value: Access) -> &'static str {
    match value {
        Access::NoAccess => "NO_ACCESS",
        Access::Viewer => "VIEWER",
        Access::Contributor => "CONTRIBUTOR",
        Access::Editor => "EDITOR",
    }
}
fn parse_publish_mode(value: &str) -> Result<PublishMode, GovernanceError> {
    match value {
        "DIRECT" => Ok(PublishMode::Direct),
        "REVIEW_REQUIRED" => Ok(PublishMode::ReviewRequired),
        _ => Err(GovernanceError::Internal),
    }
}
fn publish_mode_text(value: PublishMode) -> &'static str {
    match value {
        PublishMode::Direct => "DIRECT",
        PublishMode::ReviewRequired => "REVIEW_REQUIRED",
    }
}

fn map_permission_store(error: sqlx::Error) -> GovernanceError {
    if let sqlx::Error::Database(database) = &error {
        if database.constraint()
            == Some("permission_grants_workspace_id_document_id_subject_kind_subject_id_key")
        {
            return GovernanceError::PermissionGrantConflict;
        }
        if database.code().as_deref() == Some("23514") {
            return GovernanceError::PermissionSubjectInvalid;
        }
    }
    map_store(error)
}
fn map_policy_store(error: sqlx::Error) -> GovernanceError {
    if let sqlx::Error::Database(database) = &error
        && database.code().as_deref() == Some("23514")
    {
        return GovernanceError::PublishPolicyInvalid;
    }
    map_store(error)
}
