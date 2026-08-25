use std::sync::Arc;

pub use adoc_knowledge::{
    PermissionPathNode, SEARCH_PROJECTION_SCHEMA, SearchHit, SearchPermissionKey, SearchProjection,
    SearchResultItem, SearchSource, SearchSourceKind, TOMBSTONE_REGION_ID, extract_search_regions,
    fuse_search_hits, normalize_search_query, permission_composite_key, permission_fingerprint,
    permission_scope_token, projection_id, snapshot_hash, valid_query_vector,
};
use adoc_ports::BoxFuture;
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::jobs::{JobExecution, JobExecutionError};

pub const SEARCH_RANKING_VERSION: &str = "search-ranking-v1";

#[derive(Clone, Debug)]
pub struct CompiledSearchScope {
    pub workspace_id: Uuid,
    pub actor_id: Uuid,
    pub published_keys: Vec<SearchPermissionKey>,
    pub draft_keys: Vec<SearchPermissionKey>,
    pub fingerprint: String,
}

#[derive(Clone, Debug)]
pub struct ScopedSearchRequest {
    pub workspace_id: Uuid,
    pub normalized_query: String,
    pub query_vector: Option<Vec<f32>>,
    pub published_keys: Vec<SearchPermissionKey>,
    pub draft_keys: Vec<SearchPermissionKey>,
    pub now: DateTime<Utc>,
}

#[derive(Clone, Debug)]
pub struct SearchDrift {
    pub document_id: Uuid,
    pub detected_fingerprint: String,
}

#[derive(Clone, Debug)]
pub struct SearchIndexResult {
    pub lexical_hits: Vec<SearchHit>,
    pub vector_hits: Vec<SearchHit>,
    pub drift: Vec<SearchDrift>,
    pub index_generation: String,
    pub index_watermark: i64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchPage {
    pub items: Vec<SearchResultItem>,
    pub next_cursor: Option<String>,
    pub index_watermark: i64,
    pub configuration_version: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum SearchRetrievalError {
    #[error("invalid search request")]
    Validation,
    #[error("search cursor expired")]
    CursorExpired,
    #[error("workspace not found")]
    WorkspaceNotFound,
    #[error("search dependency unavailable")]
    Unavailable,
    #[error("search failed")]
    Internal,
}

pub trait SearchScopeCompiler: Send + Sync {
    fn compile<'a>(
        &'a self,
        actor_id: Uuid,
        workspace_id: Uuid,
        include_drafts: bool,
    ) -> BoxFuture<'a, Result<CompiledSearchScope, SearchRetrievalError>>;
}

pub trait HybridSearchIndex: Send + Sync {
    fn retrieve<'a>(
        &'a self,
        request: &'a ScopedSearchRequest,
    ) -> BoxFuture<'a, Result<SearchIndexResult, SearchRetrievalError>>;
}

pub trait SearchDriftRepair: Send + Sync {
    fn schedule<'a>(
        &'a self,
        workspace_id: Uuid,
        drift: &'a [SearchDrift],
        now: DateTime<Utc>,
    ) -> BoxFuture<'a, Result<(), SearchRetrievalError>>;
}

#[derive(Clone)]
pub struct KnowledgeRetrievalService {
    scope: Arc<dyn SearchScopeCompiler>,
    index: Arc<dyn HybridSearchIndex>,
    repair: Arc<dyn SearchDriftRepair>,
    embedding_dimension: usize,
}

impl KnowledgeRetrievalService {
    pub fn new(
        scope: Arc<dyn SearchScopeCompiler>,
        index: Arc<dyn HybridSearchIndex>,
        repair: Arc<dyn SearchDriftRepair>,
        embedding_dimension: usize,
    ) -> Result<Self, SearchRetrievalError> {
        if embedding_dimension == 0 {
            return Err(SearchRetrievalError::Validation);
        }
        Ok(Self {
            scope,
            index,
            repair,
            embedding_dimension,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn search(
        &self,
        actor_id: Uuid,
        workspace_id: Uuid,
        query: &str,
        query_vector: Option<Vec<f32>>,
        include_drafts: bool,
        limit: usize,
        cursor: Option<&str>,
        now: DateTime<Utc>,
    ) -> Result<SearchPage, SearchRetrievalError> {
        let normalized_query =
            normalize_search_query(query).ok_or(SearchRetrievalError::Validation)?;
        if !(1..=30).contains(&limit)
            || query_vector
                .as_ref()
                .is_some_and(|value| !valid_query_vector(value, self.embedding_dimension))
        {
            return Err(SearchRetrievalError::Validation);
        }
        let scope = self
            .scope
            .compile(actor_id, workspace_id, include_drafts)
            .await?;
        let query_hash =
            search_query_hash(&normalized_query, query_vector.as_deref(), include_drafts);
        let index = if scope.published_keys.is_empty() && scope.draft_keys.is_empty() {
            SearchIndexResult {
                lexical_hits: Vec::new(),
                vector_hits: Vec::new(),
                drift: Vec::new(),
                index_generation: "empty".to_owned(),
                index_watermark: 0,
            }
        } else {
            self.index
                .retrieve(&ScopedSearchRequest {
                    workspace_id,
                    normalized_query: normalized_query.clone(),
                    query_vector,
                    published_keys: scope.published_keys,
                    draft_keys: scope.draft_keys,
                    now,
                })
                .await?
        };
        if !index.drift.is_empty() {
            let _ = self.repair.schedule(workspace_id, &index.drift, now).await;
        }
        let all = fuse_search_hits(
            index.lexical_hits,
            index.vector_hits,
            &normalized_query,
            now,
        )
        .ok_or(SearchRetrievalError::Internal)?;
        let offset = match cursor {
            Some(value) => {
                let value = decode_search_cursor(value)?;
                if value.version != 1
                    || value.workspace_id != workspace_id
                    || value.actor_id != actor_id
                    || value.query_hash != query_hash
                    || value.scope_fingerprint != scope.fingerprint
                    || value.index_generation != index.index_generation
                    || value.index_watermark != index.index_watermark
                    || value.configuration_version != SEARCH_RANKING_VERSION
                    || !(1..=30).contains(&value.offset)
                {
                    return Err(SearchRetrievalError::CursorExpired);
                }
                value.offset
            }
            None => 0,
        };
        if offset > all.len() {
            return Err(SearchRetrievalError::CursorExpired);
        }
        let items = all
            .iter()
            .skip(offset)
            .take(limit)
            .cloned()
            .collect::<Vec<_>>();
        let next_offset = offset + items.len();
        let next_cursor = (next_offset < all.len()).then(|| {
            encode_search_cursor(&SearchCursor {
                version: 1,
                workspace_id,
                actor_id,
                query_hash,
                scope_fingerprint: scope.fingerprint,
                index_generation: index.index_generation,
                index_watermark: index.index_watermark,
                configuration_version: SEARCH_RANKING_VERSION.to_owned(),
                offset: next_offset,
            })
        });
        Ok(SearchPage {
            items,
            next_cursor: next_cursor.transpose()?,
            index_watermark: index.index_watermark,
            configuration_version: SEARCH_RANKING_VERSION,
        })
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SearchCursor {
    version: i32,
    workspace_id: Uuid,
    actor_id: Uuid,
    query_hash: String,
    scope_fingerprint: String,
    index_generation: String,
    index_watermark: i64,
    configuration_version: String,
    offset: usize,
}

fn encode_search_cursor(value: &SearchCursor) -> Result<String, SearchRetrievalError> {
    let bytes = serde_json::to_vec(value).map_err(|_| SearchRetrievalError::Internal)?;
    Ok(URL_SAFE_NO_PAD.encode(bytes))
}

fn decode_search_cursor(value: &str) -> Result<SearchCursor, SearchRetrievalError> {
    if value.is_empty() || value.len() > 2_048 {
        return Err(SearchRetrievalError::CursorExpired);
    }
    let bytes = URL_SAFE_NO_PAD
        .decode(value)
        .map_err(|_| SearchRetrievalError::CursorExpired)?;
    serde_json::from_slice(&bytes).map_err(|_| SearchRetrievalError::CursorExpired)
}

fn search_query_hash(query: &str, vector: Option<&[f32]>, include_drafts: bool) -> String {
    let mut hash = Sha256::new();
    hash.update(b"search-query:v1\0");
    hash.update(query.as_bytes());
    hash.update([u8::from(include_drafts)]);
    for value in vector.unwrap_or_default() {
        hash.update(value.to_le_bytes());
    }
    hex::encode(hash.finalize())
}

#[derive(Clone, Debug)]
pub enum ProjectionMutation {
    Replace {
        workspace_id: Uuid,
        document_id: Uuid,
        source_kind: SearchSourceKind,
        sequence: i64,
        regions: Vec<SearchProjection>,
    },
    DeleteTree {
        workspace_id: Uuid,
        root_document_id: Uuid,
        sequence: i64,
    },
    DeleteWorkspace {
        workspace_id: Uuid,
        sequence: i64,
    },
}

#[derive(Clone, Debug)]
pub struct ProjectionWork {
    pub outbox_event_id: Uuid,
    pub already_completed: bool,
    pub mutations: Vec<ProjectionMutation>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum SearchProjectionError {
    #[error("transient search projection failure: {0}")]
    Transient(&'static str),
    #[error("permanent search projection failure: {0}")]
    Permanent(&'static str),
}

pub trait SearchProjectionRepository: Send + Sync {
    fn prepare<'a>(
        &'a self,
        outbox_event_id: Uuid,
    ) -> BoxFuture<'a, Result<ProjectionWork, SearchProjectionError>>;
    fn complete<'a>(
        &'a self,
        outbox_event_id: Uuid,
        job_id: Uuid,
        worker: &'a str,
        job_sequence: i64,
        now: DateTime<Utc>,
    ) -> BoxFuture<'a, Result<JobExecution, SearchProjectionError>>;
}

pub trait SearchIndex: Send + Sync {
    fn apply<'a>(
        &'a self,
        mutations: &'a [ProjectionMutation],
    ) -> BoxFuture<'a, Result<(), SearchProjectionError>>;
}

pub struct SearchProjectionService {
    repository: Arc<dyn SearchProjectionRepository>,
    index: Arc<dyn SearchIndex>,
}

impl SearchProjectionService {
    pub fn new(
        repository: Arc<dyn SearchProjectionRepository>,
        index: Arc<dyn SearchIndex>,
    ) -> Self {
        Self { repository, index }
    }

    pub async fn execute(
        &self,
        outbox_event_id: Uuid,
        job_id: Uuid,
        worker: &str,
        job_sequence: i64,
        now: DateTime<Utc>,
    ) -> Result<JobExecution, JobExecutionError> {
        let work = self
            .repository
            .prepare(outbox_event_id)
            .await
            .map_err(job_error)?;
        if !work.already_completed {
            self.index.apply(&work.mutations).await.map_err(job_error)?;
        }
        self.repository
            .complete(outbox_event_id, job_id, worker, job_sequence, now)
            .await
            .map_err(job_error)
    }
}

fn job_error(error: SearchProjectionError) -> JobExecutionError {
    match error {
        SearchProjectionError::Transient(code) => JobExecutionError::Transient(code),
        SearchProjectionError::Permanent(code) => JobExecutionError::Permanent(code),
    }
}
