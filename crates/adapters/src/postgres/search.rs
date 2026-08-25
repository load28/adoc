use adoc_application::{
    jobs::JobExecution,
    search::{
        PermissionPathNode, ProjectionMutation, ProjectionWork, SEARCH_PROJECTION_SCHEMA,
        SearchProjection, SearchProjectionError, SearchProjectionRepository, SearchSourceKind,
        extract_search_regions, permission_composite_key, permission_fingerprint,
        permission_scope_token, snapshot_hash,
    },
};
use adoc_ports::BoxFuture;
use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::{PgPool, Row};
use unicode_normalization::UnicodeNormalization;
use uuid::Uuid;

use super::PostgresStore;

#[derive(Clone)]
pub struct PostgresSearchProjectionRepository {
    pool: PgPool,
}

pub struct SearchRebuildRun {
    pub id: Uuid,
    pub generation: i64,
}

impl PostgresSearchProjectionRepository {
    #[must_use]
    pub fn new(store: &PostgresStore) -> Self {
        Self {
            pool: store.pool().clone(),
        }
    }

    pub async fn begin_rebuild(
        &self,
        now: DateTime<Utc>,
    ) -> Result<SearchRebuildRun, SearchProjectionError> {
        let mut tx = self.pool.begin().await.map_err(db)?;
        sqlx::query("SELECT pg_advisory_xact_lock(825241630251)")
            .execute(&mut *tx)
            .await
            .map_err(db)?;
        let generation: i64 = sqlx::query_scalar(
            "SELECT COALESCE(max(generation),0)+1 FROM search_projection_rebuilds",
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(db)?;
        let generation = generation.max(2);
        let id = Uuid::now_v7();
        sqlx::query("INSERT INTO search_projection_rebuilds(id,schema_version,generation,status,snapshot_watermark_json,replayed_through_json,started_at,updated_at) VALUES($1,$2,$3,'BUILDING','{}'::jsonb,'{}'::jsonb,$4,$4)")
            .bind(id).bind(SEARCH_PROJECTION_SCHEMA).bind(generation).bind(now)
            .execute(&mut *tx).await.map_err(db)?;
        tx.commit().await.map_err(db)?;

        Ok(SearchRebuildRun { id, generation })
    }

    pub async fn capture_rebuild_snapshot(
        &self,
        run: &SearchRebuildRun,
        now: DateTime<Utc>,
    ) -> Result<Vec<ProjectionMutation>, SearchProjectionError> {
        let watermark_rows = sqlx::query(
            "SELECT w.id,COALESCE(s.next_projection_sequence-1,0) AS watermark FROM workspaces w LEFT JOIN workspace_sequences s ON s.workspace_id=w.id WHERE w.status IN ('ACTIVE','DELETION_SCHEDULED') ORDER BY w.id",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(db)?;
        let watermarks = watermark_rows
            .iter()
            .map(|row| {
                (
                    row.get::<Uuid, _>("id").to_string(),
                    Value::from(row.get::<i64, _>("watermark")),
                )
            })
            .collect::<serde_json::Map<_, _>>();
        let changed = sqlx::query("UPDATE search_projection_rebuilds SET status='CATCHING_UP',snapshot_watermark_json=$2,replayed_through_json=$2,updated_at=$3 WHERE id=$1 AND status='BUILDING'")
            .bind(run.id).bind(Value::Object(watermarks)).bind(now).execute(&self.pool).await.map_err(db)?;
        if changed.rows_affected() != 1 {
            return Err(SearchProjectionError::Permanent(
                "SEARCH_REBUILD_STATE_INVALID",
            ));
        }
        let mut mutations = Vec::new();
        for row in watermark_rows {
            let workspace: Uuid = row.get("id");
            let watermark: i64 = row.get("watermark");
            let documents = all_active_documents(&self.pool, workspace).await?;
            mutations
                .extend(materialize_documents(&self.pool, workspace, documents, watermark).await?);
        }
        sqlx::query("UPDATE search_projection_rebuilds SET status='VALIDATING',updated_at=$2 WHERE id=$1 AND status='CATCHING_UP'")
            .bind(run.id).bind(now).execute(&self.pool).await.map_err(db)?;
        Ok(mutations)
    }

    pub async fn finish_rebuild(
        &self,
        id: Uuid,
        error_code: Option<&str>,
        now: DateTime<Utc>,
    ) -> Result<(), SearchProjectionError> {
        let status = if error_code.is_some() {
            "FAILED"
        } else {
            "ACTIVE"
        };
        let allowed = if error_code.is_some() {
            vec!["BUILDING", "CATCHING_UP", "VALIDATING"]
        } else {
            vec!["VALIDATING"]
        };
        let changed = sqlx::query("UPDATE search_projection_rebuilds SET status=$2,error_code=$3,updated_at=$4,completed_at=$4 WHERE id=$1 AND status=ANY($5)")
            .bind(id).bind(status).bind(error_code).bind(now).bind(allowed).execute(&self.pool).await.map_err(db)?;
        if changed.rows_affected() == 1 {
            Ok(())
        } else {
            Err(SearchProjectionError::Permanent(
                "SEARCH_REBUILD_STATE_INVALID",
            ))
        }
    }
}

impl SearchProjectionRepository for PostgresSearchProjectionRepository {
    fn prepare<'a>(
        &'a self,
        outbox_event_id: Uuid,
    ) -> BoxFuture<'a, Result<ProjectionWork, SearchProjectionError>> {
        Box::pin(async move {
            let row = sqlx::query(
                "SELECT workspace_id,event_type,payload_json,projection_sequence, \
                 EXISTS(SELECT 1 FROM consumer_receipts r WHERE r.consumer='search-projection-v1' AND r.event_id=o.id) AS received \
                 FROM outbox_events o WHERE id=$1",
            )
            .bind(outbox_event_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(db)?
            .ok_or(SearchProjectionError::Permanent("OUTBOX_EVENT_NOT_FOUND"))?;
            if row.get::<bool, _>("received") {
                return Ok(ProjectionWork {
                    outbox_event_id,
                    already_completed: true,
                    mutations: Vec::new(),
                });
            }
            let workspace_id = row.get("workspace_id");
            let event_type: String = row.get("event_type");
            let payload: Value = row.get("payload_json");
            let sequence = row.get("projection_sequence");
            let mutations = match event_type.as_str() {
                "DocumentChanged.v1"
                | "DocumentMoved.v1"
                | "DraftChanged.v1"
                | "VersionPublished.v1" => {
                    let document_id = uuid_field(&payload, "documentId")?;
                    materialize_documents(&self.pool, workspace_id, vec![document_id], sequence)
                        .await?
                }
                "PermissionChanged.v1" => {
                    let root = uuid_field(&payload, "entityId")?;
                    let documents = subtree(&self.pool, workspace_id, root).await?;
                    materialize_documents(&self.pool, workspace_id, documents, sequence).await?
                }
                "VocabularyChanged.v1" => {
                    let documents = all_active_documents(&self.pool, workspace_id).await?;
                    materialize_documents(&self.pool, workspace_id, documents, sequence).await?
                }
                "SearchProjectionRepairScheduled.v1" => {
                    let document_id = uuid_field(&payload, "documentId")?;
                    materialize_documents(&self.pool, workspace_id, vec![document_id], sequence)
                        .await?
                }
                "PurgeChanged.v1" => {
                    let target = uuid_field(&payload, "targetId")?;
                    match payload.get("targetKind").and_then(Value::as_str) {
                        Some("WORKSPACE") => vec![ProjectionMutation::DeleteWorkspace {
                            workspace_id,
                            sequence,
                        }],
                        Some("DOCUMENT") => vec![ProjectionMutation::DeleteTree {
                            workspace_id,
                            root_document_id: target,
                            sequence,
                        }],
                        _ => return Err(SearchProjectionError::Permanent("EVENT_PAYLOAD_INVALID")),
                    }
                }
                _ => return Err(SearchProjectionError::Permanent("EVENT_TYPE_UNSUPPORTED")),
            };
            Ok(ProjectionWork {
                outbox_event_id,
                already_completed: false,
                mutations,
            })
        })
    }

    fn complete<'a>(
        &'a self,
        outbox_event_id: Uuid,
        job_id: Uuid,
        worker: &'a str,
        job_sequence: i64,
        now: DateTime<Utc>,
    ) -> BoxFuture<'a, Result<JobExecution, SearchProjectionError>> {
        Box::pin(async move {
            let mut tx = self.pool.begin().await.map_err(db)?;
            let status = sqlx::query_scalar::<_, String>(
                "SELECT status::text FROM jobs WHERE id=$1 AND sequence=$2 AND lease_owner=$3 FOR UPDATE",
            )
            .bind(job_id)
            .bind(job_sequence)
            .bind(worker)
            .fetch_optional(&mut *tx)
            .await
            .map_err(db)?
            .ok_or(SearchProjectionError::Transient("JOB_LEASE_LOST"))?;
            if status == "CANCEL_REQUESTED" {
                sqlx::query("UPDATE jobs SET status='CANCELLED',sequence=sequence+1,completed_at=$2,updated_at=$2,lease_owner=NULL,lease_until=NULL WHERE id=$1")
                    .bind(job_id).bind(now).execute(&mut *tx).await.map_err(db)?;
                tx.commit().await.map_err(db)?;
                return Ok(JobExecution::Cancelled);
            }
            if status != "RUNNING" {
                return Err(SearchProjectionError::Transient("JOB_LEASE_LOST"));
            }
            sqlx::query("INSERT INTO consumer_receipts(consumer,event_id,processed_at) VALUES('search-projection-v1',$1,$2) ON CONFLICT DO NOTHING")
                .bind(outbox_event_id).bind(now).execute(&mut *tx).await.map_err(db)?;
            sqlx::query("UPDATE jobs SET status='SUCCEEDED',sequence=sequence+1,completed_at=$2,updated_at=$2,lease_owner=NULL,lease_until=NULL,last_error_code=NULL WHERE id=$1 AND status='RUNNING'")
                .bind(job_id).bind(now).execute(&mut *tx).await.map_err(db)?;
            tx.commit().await.map_err(db)?;
            Ok(JobExecution::Delivered(None))
        })
    }
}

async fn materialize_documents(
    pool: &PgPool,
    workspace_id: Uuid,
    documents: Vec<Uuid>,
    sequence: i64,
) -> Result<Vec<ProjectionMutation>, SearchProjectionError> {
    let terms = active_terms(pool, workspace_id).await?;
    let mut mutations = Vec::with_capacity(documents.len() * 2);
    for document_id in documents {
        let context = document_context(pool, workspace_id, document_id).await?;
        for source_kind in [SearchSourceKind::Published, SearchSourceKind::Draft] {
            let regions = context
                .as_ref()
                .and_then(|context| context.source(source_kind))
                .map(|source| {
                    projections(
                        context.as_ref().expect("source requires context"),
                        source_kind,
                        source,
                        &terms,
                        sequence,
                    )
                })
                .transpose()?
                .unwrap_or_default();
            mutations.push(ProjectionMutation::Replace {
                workspace_id,
                document_id,
                source_kind,
                sequence,
                regions,
            });
        }
    }
    Ok(mutations)
}

struct DocumentContext {
    workspace_id: Uuid,
    document_id: Uuid,
    title: String,
    status: String,
    updated_at: DateTime<Utc>,
    ancestors: Vec<Uuid>,
    permission_fingerprint: String,
    published: Option<SourceSnapshot>,
    draft: Option<SourceSnapshot>,
}

struct SourceSnapshot {
    revision: i64,
    version_number: Option<i64>,
    content: Value,
}

impl DocumentContext {
    fn source(&self, kind: SearchSourceKind) -> Option<&SourceSnapshot> {
        if self.status != "ACTIVE" {
            return None;
        }
        match kind {
            SearchSourceKind::Published => self.published.as_ref(),
            SearchSourceKind::Draft => self.draft.as_ref(),
        }
    }
}

async fn document_context(
    pool: &PgPool,
    workspace: Uuid,
    document: Uuid,
) -> Result<Option<DocumentContext>, SearchProjectionError> {
    let row = sqlx::query(
        "SELECT d.title,d.status::text,d.updated_at, \
         pv.number,pv.content_json AS published_content,pv.published_at, \
         dr.revision AS draft_revision,dr.content_json AS draft_content,dr.updated_at AS draft_updated_at \
         FROM documents d \
         LEFT JOIN published_versions pv ON pv.workspace_id=d.workspace_id AND pv.id=d.current_version_id \
         LEFT JOIN drafts dr ON dr.workspace_id=d.workspace_id AND dr.document_id=d.id \
         WHERE d.workspace_id=$1 AND d.id=$2",
    )
    .bind(workspace)
    .bind(document)
    .fetch_optional(pool)
    .await
    .map_err(db)?;
    let Some(row) = row else { return Ok(None) };
    let path_rows = sqlx::query(
        "WITH RECURSIVE path AS ( \
         SELECT id,parent_id,permission_revision,0 AS depth FROM documents WHERE workspace_id=$1 AND id=$2 \
         UNION ALL SELECT d.id,d.parent_id,d.permission_revision,p.depth+1 FROM documents d JOIN path p ON p.parent_id=d.id WHERE d.workspace_id=$1 \
         ) SELECT id,parent_id,permission_revision FROM path ORDER BY depth DESC",
    )
    .bind(workspace)
    .bind(document)
    .fetch_all(pool)
    .await
    .map_err(db)?;
    let path = path_rows
        .iter()
        .map(|row| PermissionPathNode {
            document_id: row.get("id"),
            parent_id: row.get("parent_id"),
            permission_revision: row.get("permission_revision"),
        })
        .collect::<Vec<_>>();
    let fingerprint = permission_fingerprint(&path)
        .ok_or(SearchProjectionError::Permanent("PERMISSION_PATH_INVALID"))?;
    let ancestors = path
        .iter()
        .filter_map(|node| (node.document_id != document).then_some(node.document_id))
        .collect();
    Ok(Some(DocumentContext {
        workspace_id: workspace,
        document_id: document,
        title: row.get("title"),
        status: row.get("status"),
        updated_at: row.get("updated_at"),
        ancestors,
        permission_fingerprint: fingerprint,
        published: row
            .get::<Option<Value>, _>("published_content")
            .map(|content| SourceSnapshot {
                revision: row.get::<i64, _>("number"),
                version_number: Some(row.get("number")),
                content,
            }),
        draft: row
            .get::<Option<Value>, _>("draft_content")
            .map(|content| SourceSnapshot {
                revision: row.get("draft_revision"),
                version_number: None,
                content,
            }),
    }))
}

fn projections(
    context: &DocumentContext,
    source_kind: SearchSourceKind,
    source: &SourceSnapshot,
    vocabulary: &[String],
    sequence: i64,
) -> Result<Vec<SearchProjection>, SearchProjectionError> {
    let hash = snapshot_hash(&source.content);
    let permission_scope = permission_scope_token(context.workspace_id, context.document_id);
    let permission_key =
        permission_composite_key(&permission_scope, &context.permission_fingerprint);
    let regions = extract_search_regions(&source.content)
        .ok_or(SearchProjectionError::Permanent("CONTENT_SCHEMA_INVALID"))?;
    Ok(regions
        .into_iter()
        .map(|region| {
            let normalized = normalize(&format!("{} {}", context.title, region.body));
            let terms = vocabulary
                .iter()
                .filter(|term| normalized.contains(term.as_str()))
                .cloned()
                .collect();
            SearchProjection {
                projection_schema: SEARCH_PROJECTION_SCHEMA,
                workspace_id: context.workspace_id,
                document_id: context.document_id,
                document_status: context.status.clone(),
                source_kind: source_kind.as_str().to_owned(),
                source_revision: source.revision,
                version_number: source.version_number,
                region_id: region.id,
                region_kind: region.kind,
                ancestor_ids: context.ancestors.clone(),
                title: context.title.clone(),
                body: region.body,
                terms,
                embedding: None,
                permission_scope: permission_scope.clone(),
                permission_fingerprint: context.permission_fingerprint.clone(),
                permission_key: permission_key.clone(),
                snapshot_hash: hash.clone(),
                authority: if source_kind == SearchSourceKind::Published {
                    "OFFICIAL"
                } else {
                    "DRAFT"
                }
                .to_owned(),
                updated_at: context.updated_at,
                outbox_sequence: sequence,
                deleted: false,
            }
        })
        .collect())
}

async fn subtree(
    pool: &PgPool,
    workspace: Uuid,
    root: Uuid,
) -> Result<Vec<Uuid>, SearchProjectionError> {
    sqlx::query_scalar("WITH RECURSIVE tree AS (SELECT id FROM documents WHERE workspace_id=$1 AND id=$2 UNION ALL SELECT d.id FROM documents d JOIN tree t ON d.parent_id=t.id WHERE d.workspace_id=$1) SELECT id FROM tree ORDER BY id")
        .bind(workspace).bind(root).fetch_all(pool).await.map_err(db)
}

async fn all_active_documents(
    pool: &PgPool,
    workspace: Uuid,
) -> Result<Vec<Uuid>, SearchProjectionError> {
    sqlx::query_scalar(
        "SELECT id FROM documents WHERE workspace_id=$1 AND status='ACTIVE' ORDER BY id",
    )
    .bind(workspace)
    .fetch_all(pool)
    .await
    .map_err(db)
}

async fn active_terms(
    pool: &PgPool,
    workspace: Uuid,
) -> Result<Vec<String>, SearchProjectionError> {
    let values = sqlx::query_scalar::<_, String>("SELECT vt.normalized_term FROM vocabulary_terms vt JOIN vocabulary_concepts vc ON vc.workspace_id=vt.workspace_id AND vc.id=vt.concept_id WHERE vt.workspace_id=$1 AND vc.status='ACTIVE' AND vt.kind IN ('CANONICAL','SYNONYM') ORDER BY vt.normalized_term")
        .bind(workspace).fetch_all(pool).await.map_err(db)?;
    Ok(values.into_iter().map(|value| normalize(&value)).collect())
}

fn normalize(value: &str) -> String {
    value.nfc().flat_map(char::to_lowercase).collect::<String>()
}

fn uuid_field(payload: &Value, key: &str) -> Result<Uuid, SearchProjectionError> {
    payload
        .get(key)
        .and_then(Value::as_str)
        .and_then(|value| Uuid::parse_str(value).ok())
        .ok_or(SearchProjectionError::Permanent("EVENT_PAYLOAD_INVALID"))
}

fn db(_: sqlx::Error) -> SearchProjectionError {
    SearchProjectionError::Transient("SEARCH_PROJECTION_STORE_UNAVAILABLE")
}
