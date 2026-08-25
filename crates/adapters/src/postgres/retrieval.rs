use std::collections::{BTreeMap, BTreeSet};

use adoc_application::{
    permission::{Access, compile_permission_scope},
    search::{
        CompiledSearchScope, SearchDrift, SearchDriftRepair, SearchPermissionKey,
        SearchRetrievalError, SearchScopeCompiler, SearchSourceKind, permission_composite_key,
        permission_fingerprint, permission_scope_token,
    },
};
use adoc_ports::BoxFuture;
use chrono::{DateTime, Utc};
use serde::Serialize;
use sha2::{Digest, Sha256};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use super::{PostgresStore, permission::scope_snapshot_tx};

#[derive(Clone)]
pub struct PostgresSearchRetrievalRepository {
    pool: PgPool,
}

impl PostgresSearchRetrievalRepository {
    pub fn new(store: &PostgresStore) -> Self {
        Self {
            pool: store.pool().clone(),
        }
    }
}

impl SearchScopeCompiler for PostgresSearchRetrievalRepository {
    fn compile<'a>(
        &'a self,
        actor_id: Uuid,
        workspace_id: Uuid,
        include_drafts: bool,
    ) -> BoxFuture<'a, Result<CompiledSearchScope, SearchRetrievalError>> {
        Box::pin(async move {
            let mut tx = self
                .pool
                .begin()
                .await
                .map_err(|_| SearchRetrievalError::Internal)?;
            sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ READ ONLY")
                .execute(&mut *tx)
                .await
                .map_err(|_| SearchRetrievalError::Internal)?;
            let snapshot = scope_snapshot_tx(&mut tx, actor_id, workspace_id)
                .await
                .map_err(|error| match error {
                    adoc_application::governance::GovernanceError::WorkspaceNotFound => {
                        SearchRetrievalError::WorkspaceNotFound
                    }
                    _ => SearchRetrievalError::Internal,
                })?;
            let resolved = compile_permission_scope(&snapshot.nodes)
                .map_err(|_| SearchRetrievalError::Internal)?;
            let rows = sqlx::query("SELECT id,parent_id,status::text,permission_revision FROM documents WHERE workspace_id=$1 AND status<>'PURGING' ORDER BY id")
                .bind(workspace_id)
                .fetch_all(&mut *tx)
                .await
                .map_err(|_| SearchRetrievalError::Internal)?;
            let metadata = rows
                .iter()
                .map(|row| {
                    (
                        row.get::<Uuid, _>("id"),
                        (
                            row.get::<Option<Uuid>, _>("parent_id"),
                            row.get::<String, _>("status"),
                            row.get::<i64, _>("permission_revision"),
                        ),
                    )
                })
                .collect::<BTreeMap<_, _>>();
            let mut paths =
                BTreeMap::<Uuid, Vec<adoc_application::search::PermissionPathNode>>::new();
            for node in &snapshot.nodes {
                let Some((parent_id, _, revision)) = metadata.get(&node.document_id) else {
                    return Err(SearchRetrievalError::Internal);
                };
                if *parent_id != node.parent_id {
                    return Err(SearchRetrievalError::Internal);
                }
                let mut path = match parent_id {
                    Some(parent) => paths
                        .get(parent)
                        .cloned()
                        .ok_or(SearchRetrievalError::Internal)?,
                    None => Vec::new(),
                };
                path.push(adoc_application::search::PermissionPathNode {
                    document_id: node.document_id,
                    parent_id: *parent_id,
                    permission_revision: *revision,
                });
                paths.insert(node.document_id, path);
            }
            let mut published_keys = Vec::new();
            let mut draft_keys = Vec::new();
            for (document_id, permission) in resolved {
                let Some((_, status, _)) = metadata.get(&document_id) else {
                    return Err(SearchRetrievalError::Internal);
                };
                if status != "ACTIVE" || permission.access < Access::Viewer {
                    continue;
                }
                let ancestry_fingerprint = permission_fingerprint(
                    paths
                        .get(&document_id)
                        .ok_or(SearchRetrievalError::Internal)?,
                )
                .ok_or(SearchRetrievalError::Internal)?;
                let scope_token = permission_scope_token(workspace_id, document_id);
                published_keys.push(SearchPermissionKey {
                    document_id,
                    source_kind: SearchSourceKind::Published,
                    composite_key: permission_composite_key(&scope_token, &ancestry_fingerprint),
                    scope_token: scope_token.clone(),
                    ancestry_fingerprint: ancestry_fingerprint.clone(),
                });
                if include_drafts && permission.access >= Access::Contributor {
                    draft_keys.push(SearchPermissionKey {
                        document_id,
                        source_kind: SearchSourceKind::Draft,
                        composite_key: permission_composite_key(
                            &scope_token,
                            &ancestry_fingerprint,
                        ),
                        scope_token,
                        ancestry_fingerprint,
                    });
                }
            }
            published_keys.sort_by_key(|key| key.document_id);
            draft_keys.sort_by_key(|key| key.document_id);
            let fingerprint = scope_fingerprint(
                workspace_id,
                actor_id,
                snapshot.stamp.permission_revision,
                snapshot.stamp.membership_revision,
                &published_keys,
                &draft_keys,
            )?;
            tx.commit()
                .await
                .map_err(|_| SearchRetrievalError::Internal)?;
            Ok(CompiledSearchScope {
                workspace_id,
                actor_id,
                published_keys,
                draft_keys,
                fingerprint,
            })
        })
    }
}

impl SearchDriftRepair for PostgresSearchRetrievalRepository {
    fn schedule<'a>(
        &'a self,
        workspace_id: Uuid,
        drift: &'a [SearchDrift],
        now: DateTime<Utc>,
    ) -> BoxFuture<'a, Result<(), SearchRetrievalError>> {
        Box::pin(async move {
            let values = drift
                .iter()
                .filter(|item| valid_hash(&item.detected_fingerprint))
                .map(|item| (item.document_id, item.detected_fingerprint.as_str()))
                .collect::<BTreeSet<_>>();
            for (document_id, detected_fingerprint) in values {
                let event_id = repair_event_id(workspace_id, document_id, detected_fingerprint);
                let mut tx = self
                    .pool
                    .begin()
                    .await
                    .map_err(|_| SearchRetrievalError::Internal)?;
                sqlx::query("INSERT INTO workspace_sequences(workspace_id) VALUES($1) ON CONFLICT(workspace_id) DO NOTHING")
                    .bind(workspace_id)
                    .execute(&mut *tx)
                    .await
                    .map_err(|_| SearchRetrievalError::Internal)?;
                let projection_sequence: i64 = sqlx::query_scalar("UPDATE workspace_sequences SET next_projection_sequence=next_projection_sequence+1 WHERE workspace_id=$1 RETURNING next_projection_sequence-1")
                    .bind(workspace_id)
                    .fetch_one(&mut *tx)
                    .await
                    .map_err(|_| SearchRetrievalError::Internal)?;
                let correlation_id = event_id.to_string();
                let inserted = sqlx::query("INSERT INTO outbox_events(id,workspace_id,aggregate_kind,aggregate_id,sequence,event_type,event_version,projection_sequence,payload_json,audience_kind,correlation_id,occurred_at) VALUES($1,$2,'SearchProjection',$3,$4,'SearchProjectionRepairScheduled.v1',1,$4,$5,'INTERNAL',$6,$7) ON CONFLICT(id) DO NOTHING")
                    .bind(event_id)
                    .bind(workspace_id)
                    .bind(document_id)
                    .bind(projection_sequence)
                    .bind(serde_json::json!({"documentId":document_id,"detectedFingerprint":detected_fingerprint}))
                    .bind(&correlation_id)
                    .bind(now)
                    .execute(&mut *tx)
                    .await
                    .map_err(|_| SearchRetrievalError::Internal)?;
                if inserted.rows_affected() == 1 {
                    sqlx::query("INSERT INTO jobs(id,workspace_id,kind,payload_json,dedupe_key,status,priority,sequence,attempt,max_attempts,run_after,correlation_id,created_at,updated_at) VALUES($1,$2,'OUTBOX_TO_SEARCH',$3,$4,'QUEUED',25,1,0,5,$5,$6,$5,$5) ON CONFLICT(kind,dedupe_key) DO NOTHING")
                        .bind(Uuid::now_v7())
                        .bind(workspace_id)
                        .bind(serde_json::json!({"outboxEventId":event_id}))
                        .bind(format!("search-projection:{event_id}"))
                        .bind(now)
                        .bind(&correlation_id)
                        .execute(&mut *tx)
                        .await
                        .map_err(|_| SearchRetrievalError::Internal)?;
                }
                tx.commit()
                    .await
                    .map_err(|_| SearchRetrievalError::Internal)?;
            }
            Ok(())
        })
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ScopeFingerprint<'a> {
    version: i32,
    workspace_id: Uuid,
    actor_id: Uuid,
    permission_revision: i64,
    membership_revision: i64,
    published_keys: &'a [SearchPermissionKey],
    draft_keys: &'a [SearchPermissionKey],
}

fn scope_fingerprint(
    workspace_id: Uuid,
    actor_id: Uuid,
    permission_revision: i64,
    membership_revision: i64,
    published_keys: &[SearchPermissionKey],
    draft_keys: &[SearchPermissionKey],
) -> Result<String, SearchRetrievalError> {
    let value = serde_json::to_vec(&ScopeFingerprint {
        version: 1,
        workspace_id,
        actor_id,
        permission_revision,
        membership_revision,
        published_keys,
        draft_keys,
    })
    .map_err(|_| SearchRetrievalError::Internal)?;
    Ok(hex::encode(Sha256::digest(value)))
}

fn repair_event_id(workspace_id: Uuid, document_id: Uuid, fingerprint: &str) -> Uuid {
    let hash = Sha256::digest(
        format!("search-repair:v1:{workspace_id}:{document_id}:{fingerprint}").as_bytes(),
    );
    let mut bytes = [0_u8; 16];
    bytes.copy_from_slice(&hash[..16]);
    Uuid::from_bytes(bytes)
}

fn valid_hash(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}
