use std::sync::Arc;

use adoc_adapters::{
    identity::{SystemClock, SystemSecureRandom},
    postgres::{PostgresKnowledgeRepository, PostgresSearchRetrievalRepository, PostgresStore},
    search_retrieval::OpenSearchRetrievalIndex,
};
use adoc_application::{
    document::DocumentService,
    knowledge::{
        CreateReferenceInput, DeprecateVocabularyConceptInput, KnowledgeService,
        WriteVocabularyConceptInput,
    },
    search::{KnowledgeRetrievalService, SearchRetrievalError},
};
use adoc_configuration::AppConfig;
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    HealthState,
    document_http::{client, lease_token},
    identity_http::{Authenticated, Problem, expected_revision, idempotency_key, validate_command},
};

#[derive(Clone)]
pub(crate) struct KnowledgeRuntime {
    pub(crate) service: Arc<KnowledgeService>,
    pub(crate) retrieval: Arc<KnowledgeRetrievalService>,
}
impl KnowledgeRuntime {
    pub(crate) fn new(
        config: &AppConfig,
        store: &PostgresStore,
        documents: Arc<DocumentService>,
    ) -> Result<Self, SearchRetrievalError> {
        let retrieval_repository = Arc::new(PostgresSearchRetrievalRepository::new(store));
        let index = Arc::new(OpenSearchRetrievalIndex::new(
            config.dependencies.opensearch_url.clone(),
            config.dependencies.search_index_prefix.clone(),
            config
                .dependencies
                .opensearch_credential
                .as_ref()
                .map(|value| value.value.expose()),
        )?);
        Ok(Self {
            service: Arc::new(KnowledgeService::new(
                Arc::new(PostgresKnowledgeRepository::new(store)),
                documents,
                Arc::new(SystemClock),
                Arc::new(SystemSecureRandom),
            )),
            retrieval: Arc::new(KnowledgeRetrievalService::new(
                retrieval_repository.clone(),
                index,
                retrieval_repository,
                config.dependencies.embedding_dimension as usize,
            )?),
        })
    }
}

pub(crate) fn knowledge_routes() -> Router<HealthState> {
    Router::new()
        .route("/workspaces/{workspace_id}/search", get(search_knowledge))
        .route(
            "/workspaces/{workspace_id}/documents/{document_id}/backlinks",
            get(list_backlinks),
        )
        .route(
            "/workspaces/{workspace_id}/documents/{document_id}/references",
            post(create_reference),
        )
        .route(
            "/workspaces/{workspace_id}/documents/{document_id}/references/{reference_id}",
            axum::routing::delete(delete_reference),
        )
        .route(
            "/workspaces/{workspace_id}/vocabulary",
            get(list_vocabulary).post(create_vocabulary),
        )
        .route(
            "/workspaces/{workspace_id}/vocabulary/{concept_id}",
            get(get_vocabulary).put(update_vocabulary),
        )
        .route(
            "/workspaces/{workspace_id}/vocabulary/{concept_id}/deprecate",
            post(deprecate_vocabulary),
        )
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SearchQuery {
    q: String,
    #[serde(default = "default_true")]
    include_drafts: bool,
    #[serde(default = "default_search_limit")]
    limit: usize,
    cursor: Option<String>,
}

async fn search_knowledge(
    State(state): State<HealthState>,
    auth: Authenticated,
    Path(workspace): Path<Uuid>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<serde_json::Value>, Problem> {
    json(
        state
            .knowledge
            .retrieval
            .search(
                auth.principal.user.id,
                workspace,
                &query.q,
                None,
                query.include_drafts,
                query.limit,
                query.cursor.as_deref(),
                chrono::Utc::now(),
            )
            .await
            .map_err(Problem::from)?,
    )
}

const fn default_true() -> bool {
    true
}

const fn default_search_limit() -> usize {
    20
}

#[derive(Deserialize)]
struct CursorQuery {
    cursor: Option<String>,
}
async fn list_backlinks(
    State(state): State<HealthState>,
    auth: Authenticated,
    Path((workspace, document)): Path<(Uuid, Uuid)>,
    Query(query): Query<CursorQuery>,
) -> Result<Json<serde_json::Value>, Problem> {
    json(
        state
            .knowledge
            .service
            .list_backlinks(auth.principal.user.id, workspace, document, query.cursor)
            .await
            .map_err(Problem::from)?,
    )
}
async fn create_reference(
    State(state): State<HealthState>,
    headers: HeaderMap,
    auth: Authenticated,
    Path((workspace, document)): Path<(Uuid, Uuid)>,
    Json(input): Json<CreateReferenceInput>,
) -> Result<Response, Problem> {
    command(&state, &headers, &auth)?;
    let result = state
        .knowledge
        .service
        .create_reference(
            auth.principal.user.id,
            workspace,
            document,
            client(&headers)?,
            expected_revision(&headers)?,
            lease_token(&headers)?,
            idempotency_key(&headers)?,
            input,
        )
        .await
        .map_err(Problem::from)?;
    Ok((StatusCode::CREATED, Json(result)).into_response())
}
async fn delete_reference(
    State(state): State<HealthState>,
    headers: HeaderMap,
    auth: Authenticated,
    Path((workspace, document, reference)): Path<(Uuid, Uuid, Uuid)>,
) -> Result<StatusCode, Problem> {
    command(&state, &headers, &auth)?;
    state
        .knowledge
        .service
        .delete_reference(
            auth.principal.user.id,
            workspace,
            document,
            reference,
            client(&headers)?,
            expected_revision(&headers)?,
            lease_token(&headers)?,
            idempotency_key(&headers)?,
        )
        .await
        .map_err(Problem::from)?;
    Ok(StatusCode::NO_CONTENT)
}
async fn list_vocabulary(
    State(state): State<HealthState>,
    auth: Authenticated,
    Path(workspace): Path<Uuid>,
    Query(query): Query<CursorQuery>,
) -> Result<Json<serde_json::Value>, Problem> {
    json(
        state
            .knowledge
            .service
            .list_vocabulary(auth.principal.user.id, workspace, query.cursor)
            .await
            .map_err(Problem::from)?,
    )
}
async fn get_vocabulary(
    State(state): State<HealthState>,
    auth: Authenticated,
    Path((workspace, concept)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, Problem> {
    json(
        state
            .knowledge
            .service
            .get_vocabulary(auth.principal.user.id, workspace, concept)
            .await
            .map_err(Problem::from)?,
    )
}
async fn create_vocabulary(
    State(state): State<HealthState>,
    headers: HeaderMap,
    auth: Authenticated,
    Path(workspace): Path<Uuid>,
    Json(input): Json<WriteVocabularyConceptInput>,
) -> Result<Response, Problem> {
    command(&state, &headers, &auth)?;
    let result = state
        .knowledge
        .service
        .create_vocabulary(
            auth.principal.user.id,
            workspace,
            idempotency_key(&headers)?,
            input,
        )
        .await
        .map_err(Problem::from)?;
    Ok((StatusCode::CREATED, Json(result)).into_response())
}
async fn update_vocabulary(
    State(state): State<HealthState>,
    headers: HeaderMap,
    auth: Authenticated,
    Path((workspace, concept)): Path<(Uuid, Uuid)>,
    Json(input): Json<WriteVocabularyConceptInput>,
) -> Result<Json<serde_json::Value>, Problem> {
    command(&state, &headers, &auth)?;
    json(
        state
            .knowledge
            .service
            .update_vocabulary(
                auth.principal.user.id,
                workspace,
                concept,
                expected_revision(&headers)?,
                idempotency_key(&headers)?,
                input,
            )
            .await
            .map_err(Problem::from)?,
    )
}
async fn deprecate_vocabulary(
    State(state): State<HealthState>,
    headers: HeaderMap,
    auth: Authenticated,
    Path((workspace, concept)): Path<(Uuid, Uuid)>,
    Json(input): Json<DeprecateVocabularyConceptInput>,
) -> Result<Json<serde_json::Value>, Problem> {
    command(&state, &headers, &auth)?;
    json(
        state
            .knowledge
            .service
            .deprecate_vocabulary(
                auth.principal.user.id,
                workspace,
                concept,
                expected_revision(&headers)?,
                idempotency_key(&headers)?,
                input,
            )
            .await
            .map_err(Problem::from)?,
    )
}
fn command(state: &HealthState, headers: &HeaderMap, auth: &Authenticated) -> Result<(), Problem> {
    validate_command(&state.identity, headers, auth)
}
fn json<T: serde::Serialize>(value: T) -> Result<Json<serde_json::Value>, Problem> {
    Ok(Json(
        serde_json::to_value(value).map_err(|_| Problem::internal())?,
    ))
}

impl From<SearchRetrievalError> for Problem {
    fn from(error: SearchRetrievalError) -> Self {
        match error {
            SearchRetrievalError::Validation => Self {
                status: StatusCode::UNPROCESSABLE_ENTITY,
                code: "VALIDATION_FAILED",
                retryable: false,
                current_revision: None,
                reference_count: None,
                publish_conflict: None,
            },
            SearchRetrievalError::CursorExpired => Self {
                status: StatusCode::CONFLICT,
                code: "SEARCH_CURSOR_EXPIRED",
                retryable: false,
                current_revision: None,
                reference_count: None,
                publish_conflict: None,
            },
            SearchRetrievalError::WorkspaceNotFound => Self {
                status: StatusCode::NOT_FOUND,
                code: "RESOURCE_NOT_FOUND",
                retryable: false,
                current_revision: None,
                reference_count: None,
                publish_conflict: None,
            },
            SearchRetrievalError::Unavailable => Self {
                status: StatusCode::SERVICE_UNAVAILABLE,
                code: "SEARCH_UNAVAILABLE",
                retryable: true,
                current_revision: None,
                reference_count: None,
                publish_conflict: None,
            },
            SearchRetrievalError::Internal => Self::internal(),
        }
    }
}
