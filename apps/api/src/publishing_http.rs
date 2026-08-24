use std::sync::Arc;

use adoc_adapters::{
    identity::{SystemClock, SystemSecureRandom},
    postgres::{PostgresPublishingRepository, PostgresStore},
};
use adoc_application::publishing::{
    CreatePublicLinkInput, PublishDocumentInput, PublishingService,
};
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
    identity_http::{Authenticated, Problem, expected_revision, idempotency_key, validate_command},
};

#[derive(Clone)]
pub(crate) struct PublishingRuntime {
    pub(crate) service: Arc<PublishingService>,
}
impl PublishingRuntime {
    pub(crate) fn new(store: &PostgresStore) -> Self {
        Self {
            service: Arc::new(PublishingService::new(
                Arc::new(PostgresPublishingRepository::new(store)),
                Arc::new(SystemClock),
                Arc::new(SystemSecureRandom),
            )),
        }
    }
}

pub(crate) fn publishing_routes() -> Router<HealthState> {
    Router::new()
        .route(
            "/workspaces/{workspace_id}/documents/{document_id}/versions",
            get(list_versions),
        )
        .route(
            "/workspaces/{workspace_id}/documents/{document_id}/versions/{version_id}",
            get(get_version),
        )
        .route(
            "/workspaces/{workspace_id}/documents/{document_id}/versions/{version_id}/restore",
            post(restore_version),
        )
        .route(
            "/workspaces/{workspace_id}/documents/{document_id}/version-diff",
            get(compare_versions),
        )
        .route(
            "/workspaces/{workspace_id}/documents/{document_id}/publish",
            post(publish_document),
        )
        .route(
            "/workspaces/{workspace_id}/documents/{document_id}/public-links",
            get(list_public_links).post(create_public_link),
        )
        .route(
            "/workspaces/{workspace_id}/documents/{document_id}/public-links/{link_id}",
            axum::routing::delete(revoke_public_link),
        )
}
pub(crate) fn public_routes() -> Router<HealthState> {
    Router::new().route("/public/v1/documents/{token}", get(get_public_document))
}

#[derive(Deserialize)]
struct CursorQuery {
    cursor: Option<String>,
}
#[derive(Deserialize)]
struct DiffQuery {
    from: Uuid,
    to: Uuid,
}

async fn list_versions(
    State(state): State<HealthState>,
    auth: Authenticated,
    Path((workspace, document)): Path<(Uuid, Uuid)>,
    Query(query): Query<CursorQuery>,
) -> Result<Json<serde_json::Value>, Problem> {
    json(
        state
            .publishing
            .service
            .list_versions(auth.principal.user.id, workspace, document, query.cursor)
            .await
            .map_err(Problem::from)?,
    )
}
async fn get_version(
    State(state): State<HealthState>,
    auth: Authenticated,
    Path((workspace, document, version)): Path<(Uuid, Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, Problem> {
    json(
        state
            .publishing
            .service
            .get_version(auth.principal.user.id, workspace, document, version)
            .await
            .map_err(Problem::from)?,
    )
}
async fn compare_versions(
    State(state): State<HealthState>,
    auth: Authenticated,
    Path((workspace, document)): Path<(Uuid, Uuid)>,
    Query(query): Query<DiffQuery>,
) -> Result<Json<serde_json::Value>, Problem> {
    json(
        state
            .publishing
            .service
            .compare_versions(
                auth.principal.user.id,
                workspace,
                document,
                query.from,
                query.to,
            )
            .await
            .map_err(Problem::from)?,
    )
}
async fn publish_document(
    State(state): State<HealthState>,
    headers: HeaderMap,
    auth: Authenticated,
    Path((workspace, document)): Path<(Uuid, Uuid)>,
    Json(input): Json<PublishDocumentInput>,
) -> Result<Response, Problem> {
    validate_command(&state.identity, &headers, &auth)?;
    let result = state
        .publishing
        .service
        .publish(
            auth.principal.user.id,
            workspace,
            document,
            expected_revision(&headers)?,
            input,
            idempotency_key(&headers)?,
        )
        .await
        .map_err(Problem::from)?;
    Ok((StatusCode::CREATED, Json(result)).into_response())
}
async fn restore_version(
    State(state): State<HealthState>,
    headers: HeaderMap,
    auth: Authenticated,
    Path((workspace, document, version)): Path<(Uuid, Uuid, Uuid)>,
) -> Result<Response, Problem> {
    validate_command(&state.identity, &headers, &auth)?;
    let result = state
        .publishing
        .service
        .restore_version(
            auth.principal.user.id,
            workspace,
            document,
            version,
            expected_revision(&headers)?,
            idempotency_key(&headers)?,
        )
        .await
        .map_err(Problem::from)?;
    Ok((StatusCode::CREATED, Json(result)).into_response())
}
async fn list_public_links(
    State(state): State<HealthState>,
    auth: Authenticated,
    Path((workspace, document)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, Problem> {
    json(
        state
            .publishing
            .service
            .list_public_links(auth.principal.user.id, workspace, document)
            .await
            .map_err(Problem::from)?,
    )
}
async fn create_public_link(
    State(state): State<HealthState>,
    headers: HeaderMap,
    auth: Authenticated,
    Path((workspace, document)): Path<(Uuid, Uuid)>,
    Json(input): Json<CreatePublicLinkInput>,
) -> Result<Response, Problem> {
    validate_command(&state.identity, &headers, &auth)?;
    let result = state
        .publishing
        .service
        .create_public_link(
            auth.principal.user.id,
            workspace,
            document,
            expected_revision(&headers)?,
            input,
            idempotency_key(&headers)?,
        )
        .await
        .map_err(Problem::from)?;
    Ok((StatusCode::CREATED, Json(result)).into_response())
}
async fn revoke_public_link(
    State(state): State<HealthState>,
    headers: HeaderMap,
    auth: Authenticated,
    Path((workspace, document, link)): Path<(Uuid, Uuid, Uuid)>,
) -> Result<StatusCode, Problem> {
    validate_command(&state.identity, &headers, &auth)?;
    state
        .publishing
        .service
        .revoke_public_link(
            auth.principal.user.id,
            workspace,
            document,
            link,
            expected_revision(&headers)?,
            idempotency_key(&headers)?,
        )
        .await
        .map_err(Problem::from)?;
    Ok(StatusCode::NO_CONTENT)
}
async fn get_public_document(
    State(state): State<HealthState>,
    Path(token): Path<String>,
) -> Result<Json<serde_json::Value>, Problem> {
    json(
        state
            .publishing
            .service
            .public_document(&token)
            .await
            .map_err(Problem::from)?,
    )
}
fn json<T: serde::Serialize>(value: T) -> Result<Json<serde_json::Value>, Problem> {
    serde_json::to_value(value)
        .map(Json)
        .map_err(|_| Problem::internal())
}
