use std::{collections::BTreeSet, sync::Arc};

use adoc_adapters::{
    ai_runtime::OpenAiRuntime,
    identity::SystemClock,
    job_queue::RedisJobSignalQueue,
    postgres::{PostgresAiContextRepository, PostgresStore, PostgresWritingIntelligenceRepository},
};
use adoc_application::{
    ai::{
        AiContextService, AiJobService, AiTarget, AiTask, AiTaskKind, ApplyProposalInput,
        ApplyProposalRequest, ContextError, ContextSelection, EmbeddingRuntime, NeverCancelled,
        WritingConfigurationInput, WritingIntelligenceService,
    },
    governance::ReasonInput,
    jobs::JobSignalQueue,
    search::{KnowledgeRetrievalService, SearchRetrievalError},
};
use adoc_configuration::{AiDriver, AppConfig};
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    routing::{get, post},
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    HealthState,
    identity_http::{Authenticated, Problem, expected_revision, idempotency_key, validate_command},
};

#[derive(Clone)]
pub(crate) struct AiHttpRuntime {
    context: Arc<AiContextService>,
    jobs: Arc<AiJobService>,
    intelligence: Arc<WritingIntelligenceService>,
    queue: Arc<dyn JobSignalQueue>,
}

impl AiHttpRuntime {
    pub(crate) async fn new(
        config: &AppConfig,
        store: &PostgresStore,
        retrieval: Arc<KnowledgeRetrievalService>,
    ) -> Result<Self, SearchRetrievalError> {
        let embedding: Option<Arc<dyn EmbeddingRuntime>> = match config.ai.driver {
            AiDriver::CodexCli => None,
            AiDriver::OpenAiResponses => {
                let key = config
                    .ai
                    .openai_api_key
                    .as_ref()
                    .ok_or(SearchRetrievalError::Unavailable)?
                    .value
                    .expose();
                let runtime = OpenAiRuntime::new(
                    "https://api.openai.com/"
                        .parse()
                        .map_err(|_| SearchRetrievalError::Internal)?,
                    key,
                    "text-embedding-3-small",
                )
                .map_err(|_| SearchRetrievalError::Unavailable)?;
                Some(Arc::new(runtime))
            }
        };
        let context = Arc::new(
            AiContextService::new(
                Arc::new(PostgresAiContextRepository::new(store)),
                retrieval,
                embedding,
                config.dependencies.embedding_dimension as usize,
                u64::from(config.ai.max_context_tokens),
            )
            .map_err(|_| SearchRetrievalError::Validation)?,
        );
        let jobs = Arc::new(AiJobService::new(
            context.clone(),
            Arc::new(PostgresAiContextRepository::new(store)),
        ));
        let intelligence = Arc::new(WritingIntelligenceService::new(
            Arc::new(PostgresWritingIntelligenceRepository::new(store)),
            Arc::new(SystemClock),
        ));
        let queue = Arc::new(
            RedisJobSignalQueue::connect(
                config.dependencies.redis_url.value.expose(),
                &config.dependencies.queue_namespace,
            )
            .await
            .map_err(|_| SearchRetrievalError::Unavailable)?,
        );
        Ok(Self {
            context,
            jobs,
            intelligence,
            queue,
        })
    }
}

pub(crate) fn ai_routes() -> Router<HealthState> {
    Router::new()
        .route(
            "/workspaces/{workspace_id}/ai/context-preview",
            post(preview_context),
        )
        .route(
            "/workspaces/{workspace_id}/ai/jobs",
            get(list_jobs).post(create_job),
        )
        .route(
            "/workspaces/{workspace_id}/ai/jobs/{job_id}",
            get(get_job).delete(cancel_job),
        )
        .route(
            "/workspaces/{workspace_id}/proposals/{proposal_id}",
            get(get_proposal),
        )
        .route(
            "/workspaces/{workspace_id}/proposals/{proposal_id}/apply",
            post(apply_proposal),
        )
        .route(
            "/workspaces/{workspace_id}/proposals/{proposal_id}/reject",
            post(reject_proposal),
        )
        .route(
            "/workspaces/{workspace_id}/writing-configuration",
            get(get_writing_configuration).put(update_writing_configuration),
        )
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ContextRequest {
    kind: AiTaskKind,
    target: AiTarget,
    expected_revision: i64,
    external_web_enabled: bool,
    instruction: Option<String>,
    #[serde(default)]
    include_source_ids: BTreeSet<Uuid>,
    #[serde(default)]
    exclude_source_ids: BTreeSet<Uuid>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CreateJobRequest {
    kind: AiTaskKind,
    target: AiTarget,
    expected_revision: i64,
    external_web_enabled: bool,
    context_fingerprint: String,
    instruction: Option<String>,
    #[serde(default)]
    include_source_ids: BTreeSet<Uuid>,
    #[serde(default)]
    exclude_source_ids: BTreeSet<Uuid>,
}

#[derive(Deserialize)]
struct CursorQuery {
    cursor: Option<String>,
}

async fn preview_context(
    State(state): State<HealthState>,
    auth: Authenticated,
    Path(workspace): Path<Uuid>,
    Json(input): Json<ContextRequest>,
) -> Result<Json<serde_json::Value>, Problem> {
    let task = AiTask {
        kind: input.kind,
        workspace_id: workspace,
        actor_id: auth.principal.user.id,
        target: input.target,
        expected_revision: input.expected_revision,
        external_web_enabled: input.external_web_enabled,
        instruction: input.instruction,
    };
    let (preview, _) = state
        .ai
        .context
        .preview(
            &task,
            &ContextSelection {
                include_source_ids: input.include_source_ids,
                exclude_source_ids: input.exclude_source_ids,
            },
            chrono::Utc::now(),
            &NeverCancelled,
        )
        .await
        .map_err(Problem::from)?;
    Ok(Json(
        serde_json::to_value(preview).map_err(|_| Problem::internal())?,
    ))
}

async fn create_job(
    State(state): State<HealthState>,
    headers: HeaderMap,
    auth: Authenticated,
    Path(workspace): Path<Uuid>,
    Json(input): Json<CreateJobRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), Problem> {
    validate_command(&state.identity, &headers, &auth)?;
    if expected_revision(&headers)? != input.expected_revision {
        return Err(Problem::from(ContextError::Stale));
    }
    let admission = state
        .ai
        .jobs
        .create(
            &AiTask {
                kind: input.kind,
                workspace_id: workspace,
                actor_id: auth.principal.user.id,
                target: input.target,
                expected_revision: input.expected_revision,
                external_web_enabled: input.external_web_enabled,
                instruction: input.instruction,
            },
            &ContextSelection {
                include_source_ids: input.include_source_ids,
                exclude_source_ids: input.exclude_source_ids,
            },
            &input.context_fingerprint,
            idempotency_key(&headers)?,
            chrono::Utc::now(),
            &NeverCancelled,
        )
        .await
        .map_err(Problem::from)?;
    let _ = state.ai.queue.signal(&[admission.signal]).await;
    Ok((
        StatusCode::ACCEPTED,
        Json(serde_json::to_value(admission.view).map_err(|_| Problem::internal())?),
    ))
}

async fn list_jobs(
    State(state): State<HealthState>,
    auth: Authenticated,
    Path(workspace): Path<Uuid>,
    Query(query): Query<CursorQuery>,
) -> Result<Json<serde_json::Value>, Problem> {
    let page = state
        .ai
        .jobs
        .list(auth.principal.user.id, workspace, query.cursor.as_deref())
        .await
        .map_err(Problem::from)?;
    Ok(Json(
        serde_json::to_value(page).map_err(|_| Problem::internal())?,
    ))
}

async fn get_job(
    State(state): State<HealthState>,
    auth: Authenticated,
    Path((workspace, job)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, Problem> {
    let job = state
        .ai
        .jobs
        .get(auth.principal.user.id, workspace, job)
        .await
        .map_err(Problem::from)?;
    Ok(Json(
        serde_json::to_value(job).map_err(|_| Problem::internal())?,
    ))
}

async fn cancel_job(
    State(state): State<HealthState>,
    headers: HeaderMap,
    auth: Authenticated,
    Path((workspace, job)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, Problem> {
    validate_command(&state.identity, &headers, &auth)?;
    let request_key = idempotency_key(&headers)?;
    state
        .ai
        .jobs
        .cancel(
            auth.principal.user.id,
            workspace,
            job,
            expected_revision(&headers)?,
            request_key,
            chrono::Utc::now(),
        )
        .await
        .map_err(Problem::from)?;
    Ok(StatusCode::ACCEPTED)
}

async fn get_proposal(
    State(state): State<HealthState>,
    auth: Authenticated,
    Path((workspace, proposal)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, Problem> {
    let value = state
        .ai
        .intelligence
        .get_proposal(auth.principal.user.id, workspace, proposal)
        .await
        .map_err(Problem::from)?;
    Ok(Json(
        serde_json::to_value(value).map_err(|_| Problem::internal())?,
    ))
}

async fn apply_proposal(
    State(state): State<HealthState>,
    headers: HeaderMap,
    auth: Authenticated,
    Path((workspace, proposal)): Path<(Uuid, Uuid)>,
    Json(input): Json<ApplyProposalInput>,
) -> Result<Json<serde_json::Value>, Problem> {
    validate_command(&state.identity, &headers, &auth)?;
    let value = state
        .ai
        .intelligence
        .apply_proposal(ApplyProposalRequest {
            actor_id: auth.principal.user.id,
            workspace_id: workspace,
            proposal_id: proposal,
            client_instance_id: crate::document_http::client(&headers)?,
            expected_revision: expected_revision(&headers)?,
            token: crate::document_http::lease_token(&headers)?,
            input,
            idempotency_key: idempotency_key(&headers)?,
        })
        .await
        .map_err(Problem::from)?;
    Ok(Json(
        serde_json::to_value(value).map_err(|_| Problem::internal())?,
    ))
}

async fn reject_proposal(
    State(state): State<HealthState>,
    headers: HeaderMap,
    auth: Authenticated,
    Path((workspace, proposal)): Path<(Uuid, Uuid)>,
    Json(input): Json<ReasonInput>,
) -> Result<Json<serde_json::Value>, Problem> {
    validate_command(&state.identity, &headers, &auth)?;
    let value = state
        .ai
        .intelligence
        .reject_proposal(
            auth.principal.user.id,
            workspace,
            proposal,
            expected_revision(&headers)?,
            input.reason,
            idempotency_key(&headers)?,
        )
        .await
        .map_err(Problem::from)?;
    Ok(Json(
        serde_json::to_value(value).map_err(|_| Problem::internal())?,
    ))
}

async fn get_writing_configuration(
    State(state): State<HealthState>,
    auth: Authenticated,
    Path(workspace): Path<Uuid>,
) -> Result<Json<serde_json::Value>, Problem> {
    let value = state
        .ai
        .intelligence
        .get_writing_configuration(auth.principal.user.id, workspace)
        .await
        .map_err(Problem::from)?;
    Ok(Json(
        serde_json::to_value(value).map_err(|_| Problem::internal())?,
    ))
}

async fn update_writing_configuration(
    State(state): State<HealthState>,
    headers: HeaderMap,
    auth: Authenticated,
    Path(workspace): Path<Uuid>,
    Json(input): Json<WritingConfigurationInput>,
) -> Result<Json<serde_json::Value>, Problem> {
    validate_command(&state.identity, &headers, &auth)?;
    let value = state
        .ai
        .intelligence
        .update_writing_configuration(
            auth.principal.user.id,
            workspace,
            expected_revision(&headers)?,
            input,
            idempotency_key(&headers)?,
        )
        .await
        .map_err(Problem::from)?;
    Ok(Json(
        serde_json::to_value(value).map_err(|_| Problem::internal())?,
    ))
}

impl From<ContextError> for Problem {
    fn from(error: ContextError) -> Self {
        let (status, code, retryable) = match error {
            ContextError::Validation => {
                (StatusCode::UNPROCESSABLE_ENTITY, "VALIDATION_FAILED", false)
            }
            ContextError::NotFound | ContextError::PermissionDenied => {
                (StatusCode::NOT_FOUND, "RESOURCE_NOT_FOUND", false)
            }
            ContextError::Stale => (StatusCode::CONFLICT, "AI_CONTEXT_STALE", false),
            ContextError::IdempotencyConflict => {
                (StatusCode::CONFLICT, "IDEMPOTENCY_KEY_REUSED", false)
            }
            ContextError::Limit => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "AI_CONTEXT_LIMIT_EXCEEDED",
                false,
            ),
            ContextError::Quota => (StatusCode::TOO_MANY_REQUESTS, "AI_QUOTA_EXCEEDED", true),
            ContextError::RetrievalUnavailable => (
                StatusCode::SERVICE_UNAVAILABLE,
                "AI_PROVIDER_UNAVAILABLE",
                true,
            ),
            ContextError::Storage | ContextError::StorageAt(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", false)
            }
        };
        Self {
            status,
            code,
            retryable,
            current_revision: None,
            reference_count: None,
            publish_conflict: None,
        }
    }
}
