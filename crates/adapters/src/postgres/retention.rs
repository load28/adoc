use adoc_application::{
    governance::GovernanceError,
    operations::{
        AuditAction, AuditEventInput, AuditTarget, AuditTargetKind, DocumentPurgeCommand,
        PurgeAdvance, PurgeJobReference, PurgeObject, PurgeRun, PurgeStatus, PurgeStep,
        PurgeTargetKind, RetentionRepository,
    },
};
use adoc_ports::BoxFuture;
use chrono::{DateTime, Duration, Utc};
use serde_json::json;
use sha2::{Digest, Sha256};
use sqlx::{PgPool, Postgres, Row, Transaction, postgres::PgRow};
use uuid::Uuid;

use super::{
    PostgresStore, append_audit_event,
    governance::{
        OutboxEvent, append_event, begin_workspace, check_revision, complete_workspace, map_store,
    },
};

#[derive(Clone)]
pub struct PostgresRetentionRepository {
    pool: PgPool,
}

impl PostgresRetentionRepository {
    #[must_use]
    pub fn new(store: &PostgresStore) -> Self {
        Self {
            pool: store.pool().clone(),
        }
    }
}

impl RetentionRepository for PostgresRetentionRepository {
    fn request_document<'a>(
        &'a self,
        input: DocumentPurgeCommand,
    ) -> BoxFuture<'a, Result<PurgeJobReference, GovernanceError>> {
        Box::pin(async move {
            let mut tx = self.pool.begin().await.map_err(map_store)?;
            require_admin(&mut tx, input.command.actor_id, input.workspace_id).await?;
            if let Some(replay) =
                begin_workspace::<PurgeJobReference>(&mut tx, input.workspace_id, &input.command)
                    .await?
            {
                tx.commit().await.map_err(map_store)?;
                return Ok(replay);
            }
            let row = sqlx::query(
                "SELECT status::text,revision,purge_after FROM documents \
                 WHERE workspace_id=$1 AND id=$2 FOR UPDATE",
            )
            .bind(input.workspace_id)
            .bind(input.document_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(map_store)?
            .ok_or(GovernanceError::DocumentNotFound)?;
            check_revision(row.get("revision"), input.expected_revision)?;
            if row.get::<String, _>("status") != "TRASHED" {
                return Err(GovernanceError::DocumentStateInvalid);
            }
            if row
                .get::<Option<DateTime<Utc>>, _>("purge_after")
                .is_none_or(|value| value > input.command.now)
            {
                return Err(GovernanceError::PurgeNotEligible);
            }
            let ledger_id = Uuid::now_v7();
            sqlx::query(
                "INSERT INTO purge_ledger(id,workspace_id,target_kind,target_id,reason,status,step,attempt,run_after,started_at,updated_at) \
                 VALUES($1,$2,'DOCUMENT',$3,$4,'PENDING','PENDING',0,$5,$5,$5) \
                 ON CONFLICT(target_kind,target_id) DO NOTHING",
            )
            .bind(ledger_id)
            .bind(input.workspace_id)
            .bind(input.document_id)
            .bind(input.reason)
            .bind(input.command.now)
            .execute(&mut *tx)
            .await
            .map_err(map_store)?;
            let job_id: Uuid = sqlx::query_scalar(
                "SELECT id FROM purge_ledger WHERE target_kind='DOCUMENT' AND target_id=$1",
            )
            .bind(input.document_id)
            .fetch_one(&mut *tx)
            .await
            .map_err(map_store)?;
            let result = PurgeJobReference {
                job_id,
                status: "QUEUED".to_owned(),
            };
            complete_workspace(&mut tx, input.workspace_id, &input.command, 202, &result).await?;
            tx.commit().await.map_err(map_store)?;
            Ok(result)
        })
    }

    fn claim_due<'a>(
        &'a self,
        now: DateTime<Utc>,
        worker: &'a str,
        limit: i64,
    ) -> BoxFuture<'a, Result<Vec<PurgeRun>, GovernanceError>> {
        Box::pin(async move {
            if worker.is_empty() || worker.len() > 100 || !(1..=100).contains(&limit) {
                return Err(GovernanceError::Internal);
            }
            let mut tx = self.pool.begin().await.map_err(map_store)?;
            let documents = sqlx::query(
                "SELECT d.workspace_id,d.id FROM documents d WHERE d.status='TRASHED' \
                 AND d.purge_after<=$1 AND NOT EXISTS(SELECT 1 FROM purge_ledger p WHERE p.target_kind='DOCUMENT' AND p.target_id=d.id) \
                 ORDER BY d.purge_after,d.id FOR UPDATE SKIP LOCKED LIMIT $2",
            )
            .bind(now)
            .bind(limit)
            .fetch_all(&mut *tx)
            .await
            .map_err(map_store)?;
            for row in documents {
                insert_discovered(
                    &mut tx,
                    row.get("workspace_id"),
                    PurgeTargetKind::Document,
                    row.get("id"),
                    now,
                )
                .await?;
            }
            let workspaces = sqlx::query(
                "SELECT id FROM workspaces w WHERE w.status='DELETION_SCHEDULED' AND w.delete_after<=$1 \
                 AND NOT EXISTS(SELECT 1 FROM purge_ledger p WHERE p.target_kind='WORKSPACE' AND p.target_id=w.id) \
                 ORDER BY w.delete_after,w.id FOR UPDATE SKIP LOCKED LIMIT $2",
            )
            .bind(now)
            .bind(limit)
            .fetch_all(&mut *tx)
            .await
            .map_err(map_store)?;
            for row in workspaces {
                let workspace: Uuid = row.get("id");
                insert_discovered(
                    &mut tx,
                    workspace,
                    PurgeTargetKind::Workspace,
                    workspace,
                    now,
                )
                .await?;
            }
            let rows = sqlx::query(
                "SELECT id,workspace_id,target_kind,target_id,status,step,attempt FROM purge_ledger \
                 WHERE run_after<=$1 AND (status IN ('PENDING','RETRY') OR (status='RUNNING' AND lease_until<=$1)) \
                 ORDER BY run_after,started_at,id FOR UPDATE SKIP LOCKED LIMIT $2",
            )
            .bind(now)
            .bind(limit)
            .fetch_all(&mut *tx)
            .await
            .map_err(map_store)?;
            let mut claimed = Vec::new();
            for row in rows {
                let mut run = purge_run(&row)?;
                let eligible = if run.step == PurgeStep::Pending {
                    revoke_access(&mut tx, &run, now).await?
                } else {
                    true
                };
                if !eligible {
                    let hash = result_hash(&run, "CANCELLED");
                    sqlx::query("UPDATE purge_ledger SET status='COMPLETED',step='COMPLETED',completed_at=$2,updated_at=$2,result_hash=$3,lease_owner=NULL,lease_until=NULL WHERE id=$1")
                        .bind(run.id).bind(now).bind(hash).execute(&mut *tx).await.map_err(map_store)?;
                    continue;
                }
                sqlx::query(
                    "UPDATE purge_ledger SET status='RUNNING',step=CASE WHEN step='PENDING' THEN 'ACCESS_REVOKED' ELSE step END,attempt=attempt+1,lease_owner=$2,lease_until=$3,updated_at=$4,last_error_code=NULL WHERE id=$1",
                )
                .bind(run.id)
                .bind(worker)
                .bind(now + Duration::minutes(10))
                .bind(now)
                .execute(&mut *tx)
                .await
                .map_err(map_store)?;
                run.status = PurgeStatus::Running;
                if run.step == PurgeStep::Pending {
                    run.step = PurgeStep::AccessRevoked;
                }
                run.attempt += 1;
                claimed.push(run);
            }
            tx.commit().await.map_err(map_store)?;
            Ok(claimed)
        })
    }

    fn advance<'a>(
        &'a self,
        run: &'a PurgeRun,
        worker: &'a str,
        now: DateTime<Utc>,
    ) -> BoxFuture<'a, Result<PurgeAdvance, GovernanceError>> {
        Box::pin(async move {
            let mut tx = self.pool.begin().await.map_err(map_store)?;
            let row = sqlx::query(
                "SELECT id,workspace_id,target_kind,target_id,status,step,attempt FROM purge_ledger \
                 WHERE id=$1 AND status='RUNNING' AND lease_owner=$2 AND lease_until>$3 FOR UPDATE",
            )
            .bind(run.id)
            .bind(worker)
            .bind(now)
            .fetch_optional(&mut *tx)
            .await
            .map_err(map_store)?
            .ok_or(GovernanceError::DependencyUnavailable)?;
            let current = purge_run(&row)?;
            let result = match current.step {
                PurgeStep::AccessRevoked => {
                    capture_objects(&mut tx, &current).await?;
                    advance_step(&mut tx, current.id, "OBJECTS_CAPTURED", now).await?;
                    PurgeAdvance::Continue(PurgeRun {
                        step: PurgeStep::ObjectsCaptured,
                        ..current
                    })
                }
                PurgeStep::ObjectsCaptured => {
                    authorize_retention_mutation(&mut tx).await?;
                    purge_domain(&mut tx, &current, now).await?;
                    advance_step(&mut tx, current.id, "DOMAIN_PURGED", now).await?;
                    PurgeAdvance::Continue(PurgeRun {
                        step: PurgeStep::DomainPurged,
                        ..current
                    })
                }
                PurgeStep::DomainPurged => {
                    let rows = sqlx::query(
                        "SELECT ledger_id,storage_key FROM purge_object_deletions WHERE ledger_id=$1 AND deleted_at IS NULL ORDER BY storage_key",
                    )
                    .bind(current.id)
                    .fetch_all(&mut *tx)
                    .await
                    .map_err(map_store)?;
                    if rows.is_empty() {
                        advance_step(&mut tx, current.id, "OBJECTS_PURGED", now).await?;
                        PurgeAdvance::Continue(PurgeRun {
                            step: PurgeStep::ObjectsPurged,
                            ..current
                        })
                    } else {
                        PurgeAdvance::DeleteObjects(
                            rows.into_iter()
                                .map(|row| PurgeObject {
                                    ledger_id: row.get("ledger_id"),
                                    storage_key: row.get("storage_key"),
                                })
                                .collect(),
                        )
                    }
                }
                PurgeStep::ObjectsPurged => {
                    authorize_retention_mutation(&mut tx).await?;
                    redact_audit(&mut tx, &current, now).await?;
                    let action = match current.target_kind {
                        PurgeTargetKind::Document => AuditAction::DocumentPurged,
                        PurgeTargetKind::Workspace => AuditAction::WorkspacePurged,
                    };
                    let kind = match current.target_kind {
                        PurgeTargetKind::Document => AuditTargetKind::Document,
                        PurgeTargetKind::Workspace => AuditTargetKind::Workspace,
                    };
                    append_audit_event(
                        &mut tx,
                        AuditEventInput::system(
                            current.workspace_id,
                            action,
                            AuditTarget {
                                kind,
                                id: current.target_id,
                            },
                            now,
                            current.id.to_string(),
                        ),
                    )
                    .await?;
                    advance_step(&mut tx, current.id, "AUDIT_REDACTED", now).await?;
                    PurgeAdvance::Continue(PurgeRun {
                        step: PurgeStep::AuditRedacted,
                        ..current
                    })
                }
                PurgeStep::AuditRedacted => {
                    let hash = result_hash(&current, "COMPLETED");
                    sqlx::query(
                        "UPDATE purge_ledger SET status='COMPLETED',step='COMPLETED',reason='retention-policy',completed_at=$2,updated_at=$2,result_hash=$3,lease_owner=NULL,lease_until=NULL WHERE id=$1",
                    )
                    .bind(current.id)
                    .bind(now)
                    .bind(hash)
                    .execute(&mut *tx)
                    .await
                    .map_err(map_store)?;
                    append_event(
                        &mut tx,
                        OutboxEvent {
                            workspace_id: current.workspace_id,
                            aggregate_kind: "Purge",
                            aggregate_id: current.id,
                            sequence: 1,
                            event_type: "PurgeChanged.v1",
                            payload: json!({"targetKind":target_kind_text(current.target_kind),"targetId":current.target_id,"step":"COMPLETED","status":"COMPLETED"}),
                            occurred_at: now,
                        },
                    )
                    .await?;
                    PurgeAdvance::Completed
                }
                PurgeStep::Completed => PurgeAdvance::Completed,
                PurgeStep::Pending => return Err(GovernanceError::Internal),
            };
            tx.commit().await.map_err(map_store)?;
            Ok(result)
        })
    }

    fn finish_object<'a>(
        &'a self,
        object: &'a PurgeObject,
        success: bool,
        error_code: Option<&'a str>,
        now: DateTime<Utc>,
    ) -> BoxFuture<'a, Result<(), GovernanceError>> {
        Box::pin(async move {
            sqlx::query(
                "UPDATE purge_object_deletions SET attempt=attempt+1,last_error_code=$3,deleted_at=CASE WHEN $4 THEN $5 ELSE deleted_at END \
                 WHERE ledger_id=$1 AND storage_key=$2",
            )
            .bind(object.ledger_id)
            .bind(&object.storage_key)
            .bind(error_code)
            .bind(success)
            .bind(now)
            .execute(&self.pool)
            .await
            .map_err(map_store)?;
            Ok(())
        })
    }

    fn fail<'a>(
        &'a self,
        run_id: Uuid,
        worker: &'a str,
        error_code: &'a str,
        now: DateTime<Utc>,
    ) -> BoxFuture<'a, Result<(), GovernanceError>> {
        Box::pin(async move {
            sqlx::query(
                "UPDATE purge_ledger SET status='RETRY',run_after=$3+interval '30 seconds',lease_owner=NULL,lease_until=NULL,last_error_code=$4,updated_at=$3 \
                 WHERE id=$1 AND status='RUNNING' AND lease_owner=$2",
            )
            .bind(run_id)
            .bind(worker)
            .bind(now)
            .bind(error_code)
            .execute(&self.pool)
            .await
            .map_err(map_store)?;
            Ok(())
        })
    }
}

async fn require_admin(
    tx: &mut Transaction<'_, Postgres>,
    actor: Uuid,
    workspace: Uuid,
) -> Result<(), GovernanceError> {
    let allowed: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM memberships m JOIN workspaces w ON w.id=m.workspace_id \
         WHERE m.workspace_id=$1 AND m.user_id=$2 AND m.status='ACTIVE' AND m.role IN ('ADMIN','OWNER') \
         AND w.status='ACTIVE')",
    )
    .bind(workspace)
    .bind(actor)
    .fetch_one(&mut **tx)
    .await
    .map_err(map_store)?;
    if allowed {
        Ok(())
    } else {
        Err(GovernanceError::WorkspaceNotFound)
    }
}

async fn authorize_retention_mutation(
    tx: &mut Transaction<'_, Postgres>,
) -> Result<(), GovernanceError> {
    sqlx::query("SELECT set_config('adoc.retention_context','on',true)")
        .execute(&mut **tx)
        .await
        .map_err(map_store)?;
    Ok(())
}

async fn insert_discovered(
    tx: &mut Transaction<'_, Postgres>,
    workspace: Uuid,
    kind: PurgeTargetKind,
    target: Uuid,
    now: DateTime<Utc>,
) -> Result<(), GovernanceError> {
    sqlx::query(
        "INSERT INTO purge_ledger(id,workspace_id,target_kind,target_id,reason,status,step,attempt,run_after,started_at,updated_at) \
         VALUES($1,$2,$3,$4,'retention-policy','PENDING','PENDING',0,$5,$5,$5) ON CONFLICT(target_kind,target_id) DO NOTHING",
    )
    .bind(Uuid::now_v7())
    .bind(workspace)
    .bind(target_kind_text(kind))
    .bind(target)
    .bind(now)
    .execute(&mut **tx)
    .await
    .map_err(map_store)?;
    Ok(())
}

async fn revoke_access(
    tx: &mut Transaction<'_, Postgres>,
    run: &PurgeRun,
    now: DateTime<Utc>,
) -> Result<bool, GovernanceError> {
    let changed = match run.target_kind {
        PurgeTargetKind::Document => sqlx::query(
            "UPDATE documents SET status='PURGING',revision=revision+1,updated_at=$3 \
             WHERE workspace_id=$1 AND id=$2 AND status='TRASHED' AND purge_after<=$3",
        )
        .bind(run.workspace_id)
        .bind(run.target_id)
        .bind(now)
        .execute(&mut **tx)
        .await
        .map_err(map_store)?
        .rows_affected(),
        PurgeTargetKind::Workspace => sqlx::query(
            "UPDATE workspaces SET status='PURGING',revision=revision+1,updated_at=$2 \
             WHERE id=$1 AND status='DELETION_SCHEDULED' AND delete_after<=$2",
        )
        .bind(run.target_id)
        .bind(now)
        .execute(&mut **tx)
        .await
        .map_err(map_store)?
        .rows_affected(),
    };
    Ok(changed == 1)
}

async fn capture_objects(
    tx: &mut Transaction<'_, Postgres>,
    run: &PurgeRun,
) -> Result<(), GovernanceError> {
    match run.target_kind {
        PurgeTargetKind::Workspace => {
            sqlx::query(
                "INSERT INTO purge_object_deletions(ledger_id,storage_key) \
                 SELECT $1,storage_key FROM file_assets WHERE workspace_id=$2 ON CONFLICT DO NOTHING",
            )
            .bind(run.id)
            .bind(run.workspace_id)
            .execute(&mut **tx)
            .await
            .map_err(map_store)?;
        }
        PurgeTargetKind::Document => {
            sqlx::query(
                "WITH RECURSIVE subtree AS (\
                   SELECT id FROM documents WHERE workspace_id=$1 AND id=$2 \
                   UNION ALL SELECT d.id FROM documents d JOIN subtree s ON d.parent_id=s.id WHERE d.workspace_id=$1\
                 ), owned AS (\
                   SELECT 'DRAFT'::text AS kind,id FROM drafts WHERE workspace_id=$1 AND document_id IN (SELECT id FROM subtree) \
                   UNION ALL SELECT 'PUBLISHED_VERSION',id FROM published_versions WHERE workspace_id=$1 AND document_id IN (SELECT id FROM subtree) \
                   UNION ALL SELECT 'MESSAGE',m.id FROM messages m JOIN discussions d ON d.workspace_id=m.workspace_id AND d.id=m.discussion_id WHERE d.workspace_id=$1 AND d.document_id IN (SELECT id FROM subtree)\
                 ), candidates AS (SELECT DISTINCT r.asset_id FROM file_references r JOIN owned o ON o.kind=r.owner_kind AND o.id=r.owner_id WHERE r.workspace_id=$1) \
                 INSERT INTO purge_object_deletions(ledger_id,storage_key) \
                 SELECT $3,a.storage_key FROM file_assets a JOIN candidates c ON c.asset_id=a.id \
                 WHERE NOT EXISTS(SELECT 1 FROM file_references r LEFT JOIN owned o ON o.kind=r.owner_kind AND o.id=r.owner_id WHERE r.asset_id=a.id AND o.id IS NULL) \
                 ON CONFLICT DO NOTHING",
            )
            .bind(run.workspace_id)
            .bind(run.target_id)
            .bind(run.id)
            .execute(&mut **tx)
            .await
            .map_err(map_store)?;
        }
    }
    Ok(())
}

async fn purge_domain(
    tx: &mut Transaction<'_, Postgres>,
    run: &PurgeRun,
    now: DateTime<Utc>,
) -> Result<(), GovernanceError> {
    match run.target_kind {
        PurgeTargetKind::Document => purge_document_domain(tx, run).await,
        PurgeTargetKind::Workspace => purge_workspace_domain(tx, run, now).await,
    }
}

async fn purge_document_domain(
    tx: &mut Transaction<'_, Postgres>,
    run: &PurgeRun,
) -> Result<(), GovernanceError> {
    let ids = sqlx::query_scalar::<_, Uuid>(
        "WITH RECURSIVE subtree AS (SELECT id FROM documents WHERE workspace_id=$1 AND id=$2 UNION ALL SELECT d.id FROM documents d JOIN subtree s ON d.parent_id=s.id WHERE d.workspace_id=$1) SELECT id FROM subtree",
    )
    .bind(run.workspace_id)
    .bind(run.target_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(map_store)?;
    if ids.is_empty() {
        return Ok(());
    }
    let draft_ids = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM drafts WHERE workspace_id=$1 AND document_id=ANY($2)",
    )
    .bind(run.workspace_id)
    .bind(&ids)
    .fetch_all(&mut **tx)
    .await
    .map_err(map_store)?;
    let version_ids = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM published_versions WHERE workspace_id=$1 AND document_id=ANY($2)",
    )
    .bind(run.workspace_id)
    .bind(&ids)
    .fetch_all(&mut **tx)
    .await
    .map_err(map_store)?;
    let message_ids = sqlx::query_scalar::<_, Uuid>("SELECT m.id FROM messages m JOIN discussions d ON d.workspace_id=m.workspace_id AND d.id=m.discussion_id WHERE d.workspace_id=$1 AND d.document_id=ANY($2)").bind(run.workspace_id).bind(&ids).fetch_all(&mut **tx).await.map_err(map_store)?;
    delete_file_references(tx, run.workspace_id, "DRAFT", &draft_ids).await?;
    delete_file_references(tx, run.workspace_id, "PUBLISHED_VERSION", &version_ids).await?;
    delete_file_references(tx, run.workspace_id, "MESSAGE", &message_ids).await?;
    let text_ids = ids.iter().map(Uuid::to_string).collect::<Vec<_>>();
    sqlx::query("DELETE FROM references_graph WHERE workspace_id=$1 AND (source_id=ANY($2) OR target_id=ANY($3))").bind(run.workspace_id).bind(&ids).bind(&text_ids).execute(&mut **tx).await.map_err(map_store)?;
    sqlx::query("DELETE FROM reviews WHERE workspace_id=$1 AND document_id=ANY($2)")
        .bind(run.workspace_id)
        .bind(&ids)
        .execute(&mut **tx)
        .await
        .map_err(map_store)?;
    sqlx::query("DELETE FROM discussions WHERE workspace_id=$1 AND document_id=ANY($2)")
        .bind(run.workspace_id)
        .bind(&ids)
        .execute(&mut **tx)
        .await
        .map_err(map_store)?;
    sqlx::query("DELETE FROM drafts WHERE workspace_id=$1 AND document_id=ANY($2)")
        .bind(run.workspace_id)
        .bind(&ids)
        .execute(&mut **tx)
        .await
        .map_err(map_store)?;
    sqlx::query(
        "UPDATE documents SET current_version_id=NULL WHERE workspace_id=$1 AND id=ANY($2)",
    )
    .bind(run.workspace_id)
    .bind(&ids)
    .execute(&mut **tx)
    .await
    .map_err(map_store)?;
    sqlx::query("DELETE FROM published_versions WHERE workspace_id=$1 AND document_id=ANY($2)")
        .bind(run.workspace_id)
        .bind(&ids)
        .execute(&mut **tx)
        .await
        .map_err(map_store)?;
    sqlx::query("DELETE FROM documents WHERE workspace_id=$1 AND id=ANY($2)")
        .bind(run.workspace_id)
        .bind(&ids)
        .execute(&mut **tx)
        .await
        .map_err(map_store)?;
    sqlx::query("DELETE FROM file_assets a USING purge_object_deletions p WHERE p.ledger_id=$1 AND p.storage_key=a.storage_key AND a.workspace_id=$2").bind(run.id).bind(run.workspace_id).execute(&mut **tx).await.map_err(map_store)?;
    Ok(())
}

async fn purge_workspace_domain(
    tx: &mut Transaction<'_, Postgres>,
    run: &PurgeRun,
    now: DateTime<Utc>,
) -> Result<(), GovernanceError> {
    let workspace = run.workspace_id;
    sqlx::query("DELETE FROM file_references WHERE workspace_id=$1")
        .bind(workspace)
        .execute(&mut **tx)
        .await
        .map_err(map_store)?;
    sqlx::query("DELETE FROM file_assets WHERE workspace_id=$1")
        .bind(workspace)
        .execute(&mut **tx)
        .await
        .map_err(map_store)?;
    sqlx::query("DELETE FROM ai_jobs WHERE workspace_id=$1")
        .bind(workspace)
        .execute(&mut **tx)
        .await
        .map_err(map_store)?;
    sqlx::query("DELETE FROM references_graph WHERE workspace_id=$1")
        .bind(workspace)
        .execute(&mut **tx)
        .await
        .map_err(map_store)?;
    sqlx::query("DELETE FROM vocabulary_concepts WHERE workspace_id=$1")
        .bind(workspace)
        .execute(&mut **tx)
        .await
        .map_err(map_store)?;
    sqlx::query("DELETE FROM inbox_items WHERE workspace_id=$1")
        .bind(workspace)
        .execute(&mut **tx)
        .await
        .map_err(map_store)?;
    sqlx::query("DELETE FROM reviews WHERE workspace_id=$1")
        .bind(workspace)
        .execute(&mut **tx)
        .await
        .map_err(map_store)?;
    sqlx::query("DELETE FROM discussions WHERE workspace_id=$1")
        .bind(workspace)
        .execute(&mut **tx)
        .await
        .map_err(map_store)?;
    sqlx::query("DELETE FROM drafts WHERE workspace_id=$1")
        .bind(workspace)
        .execute(&mut **tx)
        .await
        .map_err(map_store)?;
    sqlx::query("UPDATE documents SET current_version_id=NULL WHERE workspace_id=$1")
        .bind(workspace)
        .execute(&mut **tx)
        .await
        .map_err(map_store)?;
    sqlx::query("DELETE FROM published_versions WHERE workspace_id=$1")
        .bind(workspace)
        .execute(&mut **tx)
        .await
        .map_err(map_store)?;
    sqlx::query("DELETE FROM documents WHERE workspace_id=$1")
        .bind(workspace)
        .execute(&mut **tx)
        .await
        .map_err(map_store)?;
    sqlx::query("DELETE FROM groups WHERE workspace_id=$1")
        .bind(workspace)
        .execute(&mut **tx)
        .await
        .map_err(map_store)?;
    sqlx::query("DELETE FROM invitations WHERE workspace_id=$1")
        .bind(workspace)
        .execute(&mut **tx)
        .await
        .map_err(map_store)?;
    for table in [
        "writing_configurations",
        "ai_configurations",
        "ai_usage_daily",
        "jobs",
        "idempotency_keys",
        "outbox_events",
        "workspace_document_revisions",
        "workspace_access_revisions",
    ] {
        let statement = format!("DELETE FROM {table} WHERE workspace_id=$1");
        sqlx::query(&statement)
            .bind(workspace)
            .execute(&mut **tx)
            .await
            .map_err(map_store)?;
    }
    sqlx::query("UPDATE memberships SET status='REMOVED',removed_at=COALESCE(removed_at,$2),revision=revision+1 WHERE workspace_id=$1").bind(workspace).bind(now).execute(&mut **tx).await.map_err(map_store)?;
    sqlx::query("UPDATE workspaces SET status='DELETED',name='Deleted workspace',slug='deleted-'||id::text,updated_at=$2 WHERE id=$1").bind(workspace).bind(now).execute(&mut **tx).await.map_err(map_store)?;
    Ok(())
}

async fn delete_file_references(
    tx: &mut Transaction<'_, Postgres>,
    workspace: Uuid,
    owner_kind: &str,
    owner_ids: &[Uuid],
) -> Result<(), GovernanceError> {
    if !owner_ids.is_empty() {
        sqlx::query("DELETE FROM file_references WHERE workspace_id=$1 AND owner_kind=$2 AND owner_id=ANY($3)")
            .bind(workspace).bind(owner_kind).bind(owner_ids).execute(&mut **tx).await.map_err(map_store)?;
    }
    Ok(())
}

async fn redact_audit(
    tx: &mut Transaction<'_, Postgres>,
    run: &PurgeRun,
    now: DateTime<Utc>,
) -> Result<(), GovernanceError> {
    match run.target_kind {
        PurgeTargetKind::Document => {
            sqlx::query("UPDATE audit_events SET before_json=NULL,after_json=NULL,metadata_json='{}'::jsonb,redacted_at=$3 WHERE workspace_id=$1 AND target_json->>'kind'='DOCUMENT' AND target_json->>'id'=$2 AND redacted_at IS NULL")
                .bind(run.workspace_id).bind(run.target_id.to_string()).bind(now).execute(&mut **tx).await.map_err(map_store)?;
        }
        PurgeTargetKind::Workspace => {
            sqlx::query("UPDATE audit_events SET before_json=NULL,after_json=NULL,metadata_json='{}'::jsonb,redacted_at=$2 WHERE workspace_id=$1 AND redacted_at IS NULL")
                .bind(run.workspace_id).bind(now).execute(&mut **tx).await.map_err(map_store)?;
        }
    }
    Ok(())
}

async fn advance_step(
    tx: &mut Transaction<'_, Postgres>,
    id: Uuid,
    step: &str,
    now: DateTime<Utc>,
) -> Result<(), GovernanceError> {
    sqlx::query("UPDATE purge_ledger SET step=$2,updated_at=$3 WHERE id=$1")
        .bind(id)
        .bind(step)
        .bind(now)
        .execute(&mut **tx)
        .await
        .map_err(map_store)?;
    Ok(())
}

fn purge_run(row: &PgRow) -> Result<PurgeRun, GovernanceError> {
    Ok(PurgeRun {
        id: row.get("id"),
        workspace_id: row.get("workspace_id"),
        target_kind: match row.get::<String, _>("target_kind").as_str() {
            "DOCUMENT" => PurgeTargetKind::Document,
            "WORKSPACE" => PurgeTargetKind::Workspace,
            _ => return Err(GovernanceError::Internal),
        },
        target_id: row.get("target_id"),
        status: match row.get::<String, _>("status").as_str() {
            "PENDING" => PurgeStatus::Pending,
            "RUNNING" => PurgeStatus::Running,
            "RETRY" => PurgeStatus::Retry,
            "COMPLETED" => PurgeStatus::Completed,
            _ => return Err(GovernanceError::Internal),
        },
        step: match row.get::<String, _>("step").as_str() {
            "PENDING" => PurgeStep::Pending,
            "ACCESS_REVOKED" => PurgeStep::AccessRevoked,
            "OBJECTS_CAPTURED" => PurgeStep::ObjectsCaptured,
            "DOMAIN_PURGED" => PurgeStep::DomainPurged,
            "OBJECTS_PURGED" => PurgeStep::ObjectsPurged,
            "AUDIT_REDACTED" => PurgeStep::AuditRedacted,
            "COMPLETED" => PurgeStep::Completed,
            _ => return Err(GovernanceError::Internal),
        },
        attempt: row.get("attempt"),
    })
}

fn target_kind_text(kind: PurgeTargetKind) -> &'static str {
    match kind {
        PurgeTargetKind::Document => "DOCUMENT",
        PurgeTargetKind::Workspace => "WORKSPACE",
    }
}

fn result_hash(run: &PurgeRun, state: &str) -> String {
    let mut digest = Sha256::new();
    digest.update(run.id.as_bytes());
    digest.update(run.target_id.as_bytes());
    digest.update(target_kind_text(run.target_kind).as_bytes());
    digest.update(state.as_bytes());
    hex::encode(digest.finalize())
}
