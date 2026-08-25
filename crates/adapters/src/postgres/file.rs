use adoc_application::{
    governance::{Command, GovernanceError},
    identity::TokenHash,
    operations::{
        AuditAction, AuditEventInput, AuditTarget, AuditTargetKind, CreateFileCommand,
        EventAudience, FileAccess, FileAsset, FileMutation, FileRepository, FileStatus,
        GcCandidate, UploadAuthorization,
    },
};
use adoc_ports::BoxFuture;
use chrono::{DateTime, Duration, Utc};
use serde_json::json;
use sha2::{Digest, Sha256};
use sqlx::{PgPool, Postgres, Row, Transaction, postgres::PgRow};
use subtle::ConstantTimeEq;
use uuid::Uuid;

use super::{
    PostgresStore, append_audit_event,
    document::require_access,
    governance::{
        OutboxEvent, append_event, begin_workspace, check_revision, complete_workspace, map_store,
    },
};
use adoc_application::permission::Access;

#[derive(Clone)]
pub struct PostgresFileRepository {
    pool: PgPool,
}
impl PostgresFileRepository {
    pub fn new(store: &PostgresStore) -> Self {
        Self {
            pool: store.pool().clone(),
        }
    }
}
impl FileRepository for PostgresFileRepository {
    fn upload_key_id<'a>(
        &'a self,
        actor: Uuid,
        workspace: Uuid,
        asset: Uuid,
    ) -> BoxFuture<'a, Result<Option<String>, GovernanceError>> {
        Box::pin(async move {
            sqlx::query_scalar("SELECT s.token_key_id FROM file_upload_sessions s JOIN file_assets a ON a.workspace_id=s.workspace_id AND a.id=s.asset_id JOIN memberships m ON m.workspace_id=a.workspace_id AND m.user_id=$1 AND m.status='ACTIVE' WHERE s.workspace_id=$2 AND s.asset_id=$3 AND a.uploaded_by=$1")
                .bind(actor)
                .bind(workspace)
                .bind(asset)
                .fetch_optional(&self.pool)
                .await
                .map_err(map_store)
        })
    }
    fn create<'a>(
        &'a self,
        input: CreateFileCommand,
    ) -> BoxFuture<'a, Result<FileAsset, GovernanceError>> {
        Box::pin(async move {
            let mut tx = self.pool.begin().await.map_err(map_store)?;
            require_member(&mut tx, input.command.actor_id, input.workspace_id).await?;
            if let Some(replay) =
                begin_workspace::<FileAsset>(&mut tx, input.workspace_id, &input.command).await?
            {
                tx.commit().await.map_err(map_store)?;
                return Ok(replay);
            }
            sqlx::query("INSERT INTO file_assets(id,workspace_id,storage_key,original_name,mime_type,size_bytes,checksum_sha256,uploaded_by,created_at) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9)").bind(input.asset_id).bind(input.workspace_id).bind(&input.storage_key).bind(&input.original_name).bind(&input.mime_type).bind(input.size_bytes).bind(&input.checksum_sha256).bind(input.command.actor_id).bind(input.command.now).execute(&mut *tx).await.map_err(map_store)?;
            sqlx::query("INSERT INTO file_upload_sessions(asset_id,workspace_id,token_hash,token_key_id,expires_at) VALUES($1,$2,$3,$4,$5)").bind(input.asset_id).bind(input.workspace_id).bind(input.token_hash.0.as_slice()).bind(&input.token_key_id).bind(input.expires_at).execute(&mut *tx).await.map_err(map_store)?;
            let result = get_asset(&mut tx, input.workspace_id, input.asset_id, false).await?;
            append_file_event(
                &mut tx,
                input.workspace_id,
                input.command.actor_id,
                &result,
                "UPLOAD_CREATED",
                input.command.now,
            )
            .await?;
            complete_workspace(&mut tx, input.workspace_id, &input.command, 201, &result).await?;
            tx.commit().await.map_err(map_store)?;
            Ok(result)
        })
    }
    fn authorize_upload<'a>(
        &'a self,
        actor: Uuid,
        workspace: Uuid,
        asset: Uuid,
        token_hash: TokenHash,
        now: DateTime<Utc>,
    ) -> BoxFuture<'a, Result<UploadAuthorization, GovernanceError>> {
        Box::pin(async move {
            let row=sqlx::query("SELECT a.storage_key,a.size_bytes,a.status::text,a.uploaded_by,s.token_hash,s.expires_at,s.uploaded_at FROM file_assets a JOIN file_upload_sessions s ON s.workspace_id=a.workspace_id AND s.asset_id=a.id JOIN memberships m ON m.workspace_id=a.workspace_id AND m.user_id=$1 AND m.status='ACTIVE' WHERE a.workspace_id=$2 AND a.id=$3").bind(actor).bind(workspace).bind(asset).fetch_optional(&self.pool).await.map_err(map_store)?.ok_or(GovernanceError::FileNotFound)?;
            if row.get::<Uuid, _>("uploaded_by") != actor {
                return Err(GovernanceError::FileNotFound);
            }
            if row.get::<String, _>("status") != "UPLOADING" {
                return Err(GovernanceError::FileStateInvalid);
            }
            if row.get::<DateTime<Utc>, _>("expires_at") <= now {
                return Err(GovernanceError::UploadExpired);
            }
            let stored: Vec<u8> = row.get("token_hash");
            if stored.as_slice().ct_eq(token_hash.0.as_slice()).unwrap_u8() != 1 {
                return Err(GovernanceError::UploadTokenInvalid);
            }
            Ok(UploadAuthorization {
                storage_key: row.get("storage_key"),
                expected_size: u64::try_from(row.get::<i64, _>("size_bytes"))
                    .map_err(|_| GovernanceError::Internal)?,
                uploaded: row.get::<Option<DateTime<Utc>>, _>("uploaded_at").is_some(),
            })
        })
    }
    fn mark_uploaded<'a>(
        &'a self,
        actor: Uuid,
        workspace: Uuid,
        asset: Uuid,
        now: DateTime<Utc>,
    ) -> BoxFuture<'a, Result<(), GovernanceError>> {
        Box::pin(async move {
            let affected=sqlx::query("UPDATE file_upload_sessions s SET uploaded_at=COALESCE(uploaded_at,$4) FROM file_assets a WHERE s.workspace_id=$1 AND s.asset_id=$2 AND a.workspace_id=s.workspace_id AND a.id=s.asset_id AND a.uploaded_by=$3 AND a.status='UPLOADING'").bind(workspace).bind(asset).bind(actor).bind(now).execute(&self.pool).await.map_err(map_store)?.rows_affected();
            if affected == 1 {
                Ok(())
            } else {
                Err(GovernanceError::FileNotFound)
            }
        })
    }
    fn begin_validation<'a>(
        &'a self,
        workspace: Uuid,
        asset: Uuid,
        expected_revision: i64,
        command: &'a Command,
    ) -> BoxFuture<'a, Result<FileAccess, GovernanceError>> {
        Box::pin(async move {
            let mut tx = self.pool.begin().await.map_err(map_store)?;
            require_member(&mut tx, command.actor_id, workspace).await?;
            let row=sqlx::query("SELECT a.id,a.storage_key,a.original_name,a.mime_type,a.size_bytes,a.checksum_sha256,a.status::text,a.failure_code,a.ready_at,a.revision,a.uploaded_by,s.uploaded_at,s.validation_key,s.validation_request_hash FROM file_assets a JOIN file_upload_sessions s ON s.workspace_id=a.workspace_id AND s.asset_id=a.id WHERE a.workspace_id=$1 AND a.id=$2 FOR UPDATE OF a,s")
                .bind(workspace).bind(asset).fetch_optional(&mut *tx).await.map_err(map_store)?.ok_or(GovernanceError::FileNotFound)?;
            if row.get::<Uuid, _>("uploaded_by") != command.actor_id {
                return Err(GovernanceError::FileNotFound);
            }
            let status: String = row.get("status");
            if status == "UPLOADING" {
                check_revision(row.get("revision"), expected_revision)?;
                if row.get::<Option<DateTime<Utc>>, _>("uploaded_at").is_none() {
                    return Err(GovernanceError::FileStateInvalid);
                }
                sqlx::query("UPDATE file_assets SET status='VALIDATING',revision=revision+1 WHERE workspace_id=$1 AND id=$2").bind(workspace).bind(asset).execute(&mut *tx).await.map_err(map_store)?;
                sqlx::query("UPDATE file_upload_sessions SET validation_key=$3,validation_request_hash=$4 WHERE workspace_id=$1 AND asset_id=$2").bind(workspace).bind(asset).bind(&command.idempotency_key).bind(&command.request_hash).execute(&mut *tx).await.map_err(map_store)?;
            } else if status == "VALIDATING" {
                if row.get::<Option<String>, _>("validation_key").as_deref()
                    != Some(command.idempotency_key.as_str())
                    || row
                        .get::<Option<String>, _>("validation_request_hash")
                        .as_deref()
                        != Some(command.request_hash.as_str())
                {
                    return Err(GovernanceError::IdempotencyKeyReused);
                }
            } else if !matches!(status.as_str(), "READY" | "FAILED") {
                return Err(GovernanceError::FileStateInvalid);
            }
            let row=sqlx::query("SELECT id,storage_key,original_name,mime_type,size_bytes,checksum_sha256,status::text,failure_code,ready_at,revision FROM file_assets WHERE workspace_id=$1 AND id=$2").bind(workspace).bind(asset).fetch_one(&mut *tx).await.map_err(map_store)?;
            let result = FileAccess {
                asset: file_asset(&row)?,
                storage_key: row.get("storage_key"),
            };
            tx.commit().await.map_err(map_store)?;
            Ok(result)
        })
    }
    fn access<'a>(
        &'a self,
        actor: Uuid,
        workspace: Uuid,
        asset: Uuid,
    ) -> BoxFuture<'a, Result<FileAccess, GovernanceError>> {
        Box::pin(async move {
            let mut tx = self.pool.begin().await.map_err(map_store)?;
            require_member(&mut tx, actor, workspace).await?;
            let row=sqlx::query("SELECT id,storage_key,original_name,mime_type,size_bytes,checksum_sha256,status::text,failure_code,ready_at,revision,uploaded_by FROM file_assets WHERE workspace_id=$1 AND id=$2").bind(workspace).bind(asset).fetch_optional(&mut *tx).await.map_err(map_store)?.ok_or(GovernanceError::FileNotFound)?;
            if row.get::<Uuid, _>("uploaded_by") != actor {
                let docs=sqlx::query_scalar::<_,Uuid>("SELECT DISTINCT document_id FROM (SELECT d.document_id FROM file_references r JOIN drafts d ON r.owner_kind='DRAFT' AND d.workspace_id=r.workspace_id AND d.id=r.owner_id WHERE r.workspace_id=$1 AND r.asset_id=$2 UNION ALL SELECT v.document_id FROM file_references r JOIN published_versions v ON r.owner_kind='PUBLISHED_VERSION' AND v.workspace_id=r.workspace_id AND v.id=r.owner_id WHERE r.workspace_id=$1 AND r.asset_id=$2 UNION ALL SELECT ds.document_id FROM file_references r JOIN messages m ON r.owner_kind='MESSAGE' AND m.workspace_id=r.workspace_id AND m.id=r.owner_id JOIN discussions ds ON ds.workspace_id=m.workspace_id AND ds.id=m.discussion_id WHERE r.workspace_id=$1 AND r.asset_id=$2) visible_docs").bind(workspace).bind(asset).fetch_all(&mut *tx).await.map_err(map_store)?;
                let mut visible = false;
                for document in docs {
                    if require_access(&mut tx, actor, workspace, document, Access::Viewer, false)
                        .await
                        .is_ok()
                    {
                        visible = true;
                        break;
                    }
                }
                if !visible {
                    return Err(GovernanceError::FileNotFound);
                }
            }
            let result = FileAccess {
                asset: file_asset(&row)?,
                storage_key: row.get("storage_key"),
            };
            tx.commit().await.map_err(map_store)?;
            Ok(result)
        })
    }
    fn public_access<'a>(
        &'a self,
        token_hash: TokenHash,
        asset: Uuid,
        now: DateTime<Utc>,
    ) -> BoxFuture<'a, Result<FileAccess, GovernanceError>> {
        Box::pin(async move {
            let row=sqlx::query("SELECT a.id,a.storage_key,a.original_name,a.mime_type,a.size_bytes,a.checksum_sha256,a.status::text,a.failure_code,a.ready_at,a.revision FROM public_links pl JOIN documents d ON d.workspace_id=pl.workspace_id AND d.id=pl.document_id JOIN file_references r ON r.workspace_id=d.workspace_id AND r.owner_kind='PUBLISHED_VERSION' AND r.owner_id=d.current_version_id JOIN file_assets a ON a.workspace_id=r.workspace_id AND a.id=r.asset_id WHERE pl.token_hash=$1 AND pl.revoked_at IS NULL AND (pl.expires_at IS NULL OR pl.expires_at>$3) AND d.status='ACTIVE' AND a.id=$2 AND a.status='READY'")
                .bind(token_hash.0.as_slice()).bind(asset).bind(now).fetch_optional(&self.pool).await.map_err(map_store)?.ok_or(GovernanceError::PublicLinkInvalid)?;
            Ok(FileAccess {
                asset: file_asset(&row)?,
                storage_key: row.get("storage_key"),
            })
        })
    }
    fn mutate<'a>(
        &'a self,
        input: FileMutation,
    ) -> BoxFuture<'a, Result<FileAsset, GovernanceError>> {
        Box::pin(async move {
            let mut tx = self.pool.begin().await.map_err(map_store)?;
            require_member(&mut tx, input.actor_id, input.workspace_id).await?;
            if let Some(replay) =
                begin_workspace::<FileAsset>(&mut tx, input.workspace_id, &input.command).await?
            {
                tx.commit().await.map_err(map_store)?;
                return Ok(replay);
            }
            let current = get_asset(&mut tx, input.workspace_id, input.asset_id, true).await?;
            check_revision(current.revision, input.expected_revision)?;
            if current.status != FileStatus::Validating {
                return Err(GovernanceError::FileStateInvalid);
            }
            let uploaded:bool=sqlx::query_scalar("SELECT uploaded_at IS NOT NULL FROM file_upload_sessions WHERE workspace_id=$1 AND asset_id=$2 FOR UPDATE").bind(input.workspace_id).bind(input.asset_id).fetch_one(&mut *tx).await.map_err(map_store)?;
            if !uploaded {
                return Err(GovernanceError::FileStateInvalid);
            }
            let status = if input.success { "READY" } else { "FAILED" };
            sqlx::query("UPDATE file_assets SET status=$3::file_status,detected_mime_type=$4,failure_code=$5,ready_at=CASE WHEN $3='READY' THEN $6 ELSE NULL END,purge_after=CASE WHEN $3='FAILED' THEN $6 ELSE purge_after END,revision=revision+1 WHERE workspace_id=$1 AND id=$2").bind(input.workspace_id).bind(input.asset_id).bind(status).bind(input.detected_mime).bind(input.failure_code).bind(input.command.now).execute(&mut *tx).await.map_err(map_store)?;
            sqlx::query("UPDATE file_upload_sessions SET completed_at=$3 WHERE workspace_id=$1 AND asset_id=$2").bind(input.workspace_id).bind(input.asset_id).bind(input.command.now).execute(&mut *tx).await.map_err(map_store)?;
            let result = get_asset(&mut tx, input.workspace_id, input.asset_id, false).await?;
            append_file_event(
                &mut tx,
                input.workspace_id,
                input.command.actor_id,
                &result,
                status,
                input.command.now,
            )
            .await?;
            complete_workspace(&mut tx, input.workspace_id, &input.command, 200, &result).await?;
            tx.commit().await.map_err(map_store)?;
            Ok(result)
        })
    }
    fn delete<'a>(
        &'a self,
        actor: Uuid,
        workspace: Uuid,
        asset: Uuid,
        revision: i64,
        key: &'a str,
        now: DateTime<Utc>,
    ) -> BoxFuture<'a, Result<(), GovernanceError>> {
        Box::pin(async move {
            let command = command(actor, "deleteFile", key, &(asset, revision), now)?;
            let mut tx = self.pool.begin().await.map_err(map_store)?;
            require_member(&mut tx, actor, workspace).await?;
            if begin_workspace::<()>(&mut tx, workspace, &command)
                .await?
                .is_some()
            {
                tx.commit().await.map_err(map_store)?;
                return Ok(());
            }
            let current = get_asset(&mut tx, workspace, asset, true).await?;
            check_revision(current.revision, revision)?;
            if current.status != FileStatus::Ready {
                return Err(GovernanceError::FileStateInvalid);
            }
            let count: i64 = sqlx::query_scalar(
                "SELECT count(*) FROM file_references WHERE workspace_id=$1 AND asset_id=$2",
            )
            .bind(workspace)
            .bind(asset)
            .fetch_one(&mut *tx)
            .await
            .map_err(map_store)?;
            if count > 0 {
                return Err(GovernanceError::FileInUse {
                    reference_count: count,
                });
            }
            sqlx::query("UPDATE file_assets SET status='DELETED',deleted_at=$3,purge_after=$3+interval '7 days',revision=revision+1 WHERE workspace_id=$1 AND id=$2").bind(workspace).bind(asset).bind(now).execute(&mut *tx).await.map_err(map_store)?;
            let result = get_asset(&mut tx, workspace, asset, false).await?;
            append_file_event(&mut tx, workspace, actor, &result, "DELETED", now).await?;
            append_audit_event(
                &mut tx,
                AuditEventInput::user(
                    workspace,
                    actor,
                    AuditAction::FileDeleted,
                    AuditTarget {
                        kind: AuditTargetKind::File,
                        id: asset,
                    },
                    now,
                    key,
                ),
            )
            .await?;
            complete_workspace(&mut tx, workspace, &command, 204, &()).await?;
            tx.commit().await.map_err(map_store)?;
            Ok(())
        })
    }
    fn claim_gc<'a>(
        &'a self,
        now: DateTime<Utc>,
        limit: i64,
    ) -> BoxFuture<'a, Result<Vec<GcCandidate>, GovernanceError>> {
        Box::pin(async move {
            let rows=sqlx::query("WITH candidates AS (SELECT a.id FROM file_assets a WHERE a.status IN ('FAILED','DELETED') AND a.purge_after<=$1 AND a.byte_deleted_at IS NULL AND (a.gc_claimed_at IS NULL OR a.gc_claimed_at<$1-interval '10 minutes') AND NOT EXISTS(SELECT 1 FROM file_references r WHERE r.workspace_id=a.workspace_id AND r.asset_id=a.id) ORDER BY a.purge_after,a.id FOR UPDATE SKIP LOCKED LIMIT $2) UPDATE file_assets a SET gc_claimed_at=$1 FROM candidates c WHERE a.id=c.id RETURNING a.id,a.storage_key")
                .bind(now).bind(limit.clamp(1, 1000)).fetch_all(&self.pool).await.map_err(map_store)?;
            Ok(rows
                .into_iter()
                .map(|row| GcCandidate {
                    asset_id: row.get("id"),
                    storage_key: row.get("storage_key"),
                })
                .collect())
        })
    }
    fn finish_gc<'a>(
        &'a self,
        asset: Uuid,
        success: bool,
        now: DateTime<Utc>,
    ) -> BoxFuture<'a, Result<(), GovernanceError>> {
        Box::pin(async move {
            sqlx::query("UPDATE file_assets SET byte_deleted_at=CASE WHEN $2 THEN $3 ELSE byte_deleted_at END,gc_claimed_at=NULL,purge_after=CASE WHEN $2 THEN purge_after ELSE $3+interval '5 minutes' END WHERE id=$1 AND status IN ('FAILED','DELETED') AND gc_claimed_at IS NOT NULL")
                .bind(asset).bind(success).bind(now).execute(&self.pool).await.map_err(map_store)?;
            Ok(())
        })
    }
}
async fn require_member(
    tx: &mut Transaction<'_, Postgres>,
    actor: Uuid,
    workspace: Uuid,
) -> Result<(), GovernanceError> {
    let ok:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM memberships m JOIN workspaces w ON w.id=m.workspace_id WHERE m.workspace_id=$1 AND m.user_id=$2 AND m.status='ACTIVE' AND w.status='ACTIVE')").bind(workspace).bind(actor).fetch_one(&mut **tx).await.map_err(map_store)?;
    if ok {
        Ok(())
    } else {
        Err(GovernanceError::WorkspaceNotFound)
    }
}
async fn get_asset(
    tx: &mut Transaction<'_, Postgres>,
    workspace: Uuid,
    id: Uuid,
    lock: bool,
) -> Result<FileAsset, GovernanceError> {
    let sql = if lock {
        "SELECT id,original_name,mime_type,size_bytes,checksum_sha256,status::text,failure_code,ready_at,revision FROM file_assets WHERE workspace_id=$1 AND id=$2 FOR UPDATE"
    } else {
        "SELECT id,original_name,mime_type,size_bytes,checksum_sha256,status::text,failure_code,ready_at,revision FROM file_assets WHERE workspace_id=$1 AND id=$2"
    };
    let row = sqlx::query(sql)
        .bind(workspace)
        .bind(id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(map_store)?
        .ok_or(GovernanceError::FileNotFound)?;
    file_asset(&row)
}
fn file_asset(row: &PgRow) -> Result<FileAsset, GovernanceError> {
    Ok(FileAsset {
        id: row.get("id"),
        original_name: row.get("original_name"),
        mime_type: row.get("mime_type"),
        size_bytes: row.get("size_bytes"),
        checksum_sha256: row.get("checksum_sha256"),
        status: match row.get::<String, _>("status").as_str() {
            "UPLOADING" => FileStatus::Uploading,
            "VALIDATING" => FileStatus::Validating,
            "READY" => FileStatus::Ready,
            "FAILED" => FileStatus::Failed,
            "DELETED" => FileStatus::Deleted,
            _ => return Err(GovernanceError::Internal),
        },
        failure_code: row.get("failure_code"),
        ready_at: row.get("ready_at"),
        revision: row.get("revision"),
    })
}
async fn append_file_event(
    tx: &mut Transaction<'_, Postgres>,
    workspace: Uuid,
    actor: Uuid,
    asset: &FileAsset,
    action: &str,
    now: DateTime<Utc>,
) -> Result<(), GovernanceError> {
    append_event(
        tx,
        OutboxEvent {
            workspace_id: workspace,
            aggregate_kind: "FileAsset",
            aggregate_id: asset.id,
            sequence: asset.revision + 1,
            event_type: "FileChanged.v1",
            payload: json!({"entityId":asset.id,"revision":asset.revision,"action":if action=="UPLOAD_CREATED"{"CREATED"}else if action=="DELETED"{"DELETED"}else{"UPDATED"}}),
            audience: EventAudience::user(actor),
            occurred_at: now,
        },
    )
    .await
}
fn command<T: serde::Serialize>(
    actor: Uuid,
    operation_id: &'static str,
    key: &str,
    input: &T,
    now: DateTime<Utc>,
) -> Result<Command, GovernanceError> {
    if !(8..=128).contains(&key.len()) {
        return Err(GovernanceError::Validation);
    }
    Ok(Command {
        actor_id: actor,
        operation_id,
        idempotency_key: key.into(),
        request_hash: hex::encode(Sha256::digest(
            serde_json::to_vec(input).map_err(|_| GovernanceError::Internal)?,
        )),
        now,
        expires_at: now + Duration::hours(24),
    })
}

pub(super) async fn sync_file_references(
    tx: &mut Transaction<'_, Postgres>,
    workspace: Uuid,
    owner_kind: &str,
    owner_id: Uuid,
    content: &serde_json::Value,
) -> Result<(), GovernanceError> {
    let mut assets = Vec::new();
    collect_asset_ids(content, &mut assets)?;
    sync_file_asset_ids(tx, workspace, owner_kind, owner_id, &assets).await
}

pub(super) async fn sync_file_asset_ids(
    tx: &mut Transaction<'_, Postgres>,
    workspace: Uuid,
    owner_kind: &str,
    owner_id: Uuid,
    assets: &[Uuid],
) -> Result<(), GovernanceError> {
    let mut assets = assets.to_vec();
    assets.sort_unstable();
    assets.dedup();
    if !assets.is_empty() {
        let ready: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM file_assets WHERE workspace_id=$1 AND id=ANY($2) AND status='READY'",
        )
        .bind(workspace)
        .bind(&assets)
        .fetch_one(&mut **tx)
        .await
        .map_err(map_store)?;
        if ready != i64::try_from(assets.len()).map_err(|_| GovernanceError::Internal)? {
            return Err(GovernanceError::FileStateInvalid);
        }
    }
    sqlx::query(
        "DELETE FROM file_references WHERE workspace_id=$1 AND owner_kind=$2 AND owner_id=$3",
    )
    .bind(workspace)
    .bind(owner_kind)
    .bind(owner_id)
    .execute(&mut **tx)
    .await
    .map_err(map_store)?;
    for asset in assets {
        sqlx::query("INSERT INTO file_references(workspace_id,asset_id,owner_kind,owner_id) VALUES($1,$2,$3,$4)")
            .bind(workspace)
            .bind(asset)
            .bind(owner_kind)
            .bind(owner_id)
            .execute(&mut **tx)
            .await
            .map_err(map_store)?;
    }
    Ok(())
}

fn collect_asset_ids(
    value: &serde_json::Value,
    output: &mut Vec<Uuid>,
) -> Result<(), GovernanceError> {
    match value {
        serde_json::Value::Object(object) => {
            if object
                .get("type")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|kind| matches!(kind, "image" | "file"))
            {
                output.push(
                    object
                        .get("assetId")
                        .and_then(serde_json::Value::as_str)
                        .and_then(|id| Uuid::parse_str(id).ok())
                        .ok_or(GovernanceError::Validation)?,
                );
            }
            for child in object.values() {
                collect_asset_ids(child, output)?;
            }
        }
        serde_json::Value::Array(items) => {
            for child in items {
                collect_asset_ids(child, output)?;
            }
        }
        _ => {}
    }
    Ok(())
}
