use std::sync::Arc;

use adoc_adapters::{
    identity::{SystemClock, SystemSecureRandom},
    postgres::{PostgresDocumentRepository, PostgresStore},
};
use adoc_application::document::{
    AcquireLeaseInput, ApplyOperationsInput, ApplyOperationsRequest, CreateDocumentInput,
    DocumentService, LeaseCommandRequest, MoveDocumentCommitInput, MoveDocumentInput,
    RestoreDocumentInput, UpdateDocumentMetadataInput,
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
pub(crate) struct DocumentRuntime {
    pub(crate) service: Arc<DocumentService>,
}

impl DocumentRuntime {
    pub(crate) fn new(store: &PostgresStore) -> Self {
        Self {
            service: Arc::new(DocumentService::new(
                Arc::new(PostgresDocumentRepository::new(store)),
                Arc::new(SystemClock),
                Arc::new(SystemSecureRandom),
            )),
        }
    }
}

pub(crate) fn document_routes() -> Router<HealthState> {
    Router::new()
        .route(
            "/workspaces/{workspace_id}/documents",
            post(create_document),
        )
        .route("/workspaces/{workspace_id}/documents/tree", get(get_tree))
        .route(
            "/workspaces/{workspace_id}/documents/trash",
            get(list_trash),
        )
        .route(
            "/workspaces/{workspace_id}/documents/{document_id}",
            get(get_document).put(rename_document),
        )
        .route(
            "/workspaces/{workspace_id}/documents/{document_id}/trash",
            post(trash_document),
        )
        .route(
            "/workspaces/{workspace_id}/documents/{document_id}/restore",
            post(restore_document),
        )
        .route(
            "/workspaces/{workspace_id}/documents/{document_id}/move-preview",
            post(preview_move),
        )
        .route(
            "/workspaces/{workspace_id}/documents/{document_id}/move",
            post(move_document),
        )
        .route(
            "/workspaces/{workspace_id}/documents/{document_id}/draft",
            get(get_draft).post(create_draft),
        )
        .route(
            "/workspaces/{workspace_id}/documents/{document_id}/lease",
            post(acquire_lease).delete(release_lease),
        )
        .route(
            "/workspaces/{workspace_id}/documents/{document_id}/lease/renew",
            post(renew_lease),
        )
        .route(
            "/workspaces/{workspace_id}/documents/{document_id}/draft/operations",
            post(apply_operations),
        )
}

#[derive(Deserialize)]
struct CursorQuery {
    cursor: Option<String>,
}

async fn get_tree(
    State(state): State<HealthState>,
    auth: Authenticated,
    Path(workspace): Path<Uuid>,
) -> Result<Json<serde_json::Value>, Problem> {
    json(
        state
            .document
            .service
            .tree(auth.principal.user.id, workspace)
            .await
            .map_err(Problem::from)?,
    )
}
async fn list_trash(
    State(state): State<HealthState>,
    auth: Authenticated,
    Path(workspace): Path<Uuid>,
    Query(query): Query<CursorQuery>,
) -> Result<Json<serde_json::Value>, Problem> {
    json(
        state
            .document
            .service
            .trash(auth.principal.user.id, workspace, query.cursor)
            .await
            .map_err(Problem::from)?,
    )
}
async fn get_document(
    State(state): State<HealthState>,
    auth: Authenticated,
    Path((workspace, document)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, Problem> {
    json(
        state
            .document
            .service
            .detail(auth.principal.user.id, workspace, document)
            .await
            .map_err(Problem::from)?,
    )
}
async fn get_draft(
    State(state): State<HealthState>,
    auth: Authenticated,
    Path((workspace, document)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, Problem> {
    json(
        state
            .document
            .service
            .draft(auth.principal.user.id, workspace, document)
            .await
            .map_err(Problem::from)?,
    )
}
async fn create_document(
    State(state): State<HealthState>,
    headers: HeaderMap,
    auth: Authenticated,
    Path(workspace): Path<Uuid>,
    Json(input): Json<CreateDocumentInput>,
) -> Result<Response, Problem> {
    command(&state, &headers, &auth)?;
    let result = state
        .document
        .service
        .create(
            auth.principal.user.id,
            workspace,
            input,
            idempotency_key(&headers)?,
        )
        .await
        .map_err(Problem::from)?;
    Ok((StatusCode::CREATED, Json(result)).into_response())
}
async fn rename_document(
    State(state): State<HealthState>,
    headers: HeaderMap,
    auth: Authenticated,
    Path((workspace, document)): Path<(Uuid, Uuid)>,
    Json(input): Json<UpdateDocumentMetadataInput>,
) -> Result<Json<serde_json::Value>, Problem> {
    command(&state, &headers, &auth)?;
    json(
        state
            .document
            .service
            .rename(
                auth.principal.user.id,
                workspace,
                document,
                expected_revision(&headers)?,
                input,
                idempotency_key(&headers)?,
            )
            .await
            .map_err(Problem::from)?,
    )
}
async fn trash_document(
    State(state): State<HealthState>,
    headers: HeaderMap,
    auth: Authenticated,
    Path((workspace, document)): Path<(Uuid, Uuid)>,
    Json(input): Json<adoc_application::governance::ReasonInput>,
) -> Result<Json<serde_json::Value>, Problem> {
    command(&state, &headers, &auth)?;
    json(
        state
            .document
            .service
            .trash_document(
                auth.principal.user.id,
                workspace,
                document,
                expected_revision(&headers)?,
                input,
                idempotency_key(&headers)?,
            )
            .await
            .map_err(Problem::from)?,
    )
}
async fn restore_document(
    State(state): State<HealthState>,
    headers: HeaderMap,
    auth: Authenticated,
    Path((workspace, document)): Path<(Uuid, Uuid)>,
    Json(input): Json<RestoreDocumentInput>,
) -> Result<Json<serde_json::Value>, Problem> {
    command(&state, &headers, &auth)?;
    json(
        state
            .document
            .service
            .restore_document(
                auth.principal.user.id,
                workspace,
                document,
                expected_revision(&headers)?,
                input,
                idempotency_key(&headers)?,
            )
            .await
            .map_err(Problem::from)?,
    )
}
async fn preview_move(
    State(state): State<HealthState>,
    headers: HeaderMap,
    auth: Authenticated,
    Path((workspace, document)): Path<(Uuid, Uuid)>,
    Json(input): Json<MoveDocumentInput>,
) -> Result<Json<serde_json::Value>, Problem> {
    validate_command(&state.identity, &headers, &auth)?;
    json(
        state
            .document
            .service
            .preview_move(
                auth.principal.user.id,
                workspace,
                document,
                expected_revision(&headers)?,
                input,
            )
            .await
            .map_err(Problem::from)?,
    )
}
async fn move_document(
    State(state): State<HealthState>,
    headers: HeaderMap,
    auth: Authenticated,
    Path((workspace, document)): Path<(Uuid, Uuid)>,
    Json(input): Json<MoveDocumentCommitInput>,
) -> Result<Json<serde_json::Value>, Problem> {
    command(&state, &headers, &auth)?;
    json(
        state
            .document
            .service
            .move_document(
                auth.principal.user.id,
                workspace,
                document,
                expected_revision(&headers)?,
                input,
                idempotency_key(&headers)?,
            )
            .await
            .map_err(Problem::from)?,
    )
}
async fn create_draft(
    State(state): State<HealthState>,
    headers: HeaderMap,
    auth: Authenticated,
    Path((workspace, document)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, Problem> {
    command(&state, &headers, &auth)?;
    json(
        state
            .document
            .service
            .create_draft(
                auth.principal.user.id,
                workspace,
                document,
                idempotency_key(&headers)?,
            )
            .await
            .map_err(Problem::from)?,
    )
}
async fn acquire_lease(
    State(state): State<HealthState>,
    headers: HeaderMap,
    auth: Authenticated,
    Path((workspace, document)): Path<(Uuid, Uuid)>,
    Json(input): Json<AcquireLeaseInput>,
) -> Result<Json<serde_json::Value>, Problem> {
    command(&state, &headers, &auth)?;
    json(
        state
            .document
            .service
            .acquire_lease(
                auth.principal.user.id,
                workspace,
                document,
                expected_revision(&headers)?,
                input,
                idempotency_key(&headers)?,
            )
            .await
            .map_err(Problem::from)?,
    )
}
async fn renew_lease(
    State(state): State<HealthState>,
    headers: HeaderMap,
    auth: Authenticated,
    Path((workspace, document)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, Problem> {
    let result = lease_command(&state, &headers, &auth, workspace, document, false)
        .await?
        .ok_or_else(Problem::internal)?;
    json(result)
}
async fn release_lease(
    State(state): State<HealthState>,
    headers: HeaderMap,
    auth: Authenticated,
    Path((workspace, document)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, Problem> {
    lease_command(&state, &headers, &auth, workspace, document, true).await?;
    Ok(StatusCode::NO_CONTENT)
}
async fn apply_operations(
    State(state): State<HealthState>,
    headers: HeaderMap,
    auth: Authenticated,
    Path((workspace, document)): Path<(Uuid, Uuid)>,
    Json(input): Json<ApplyOperationsInput>,
) -> Result<Json<serde_json::Value>, Problem> {
    command(&state, &headers, &auth)?;
    json(
        state
            .document
            .service
            .apply_operations(ApplyOperationsRequest {
                actor_id: auth.principal.user.id,
                workspace_id: workspace,
                document_id: document,
                client_instance_id: client(&headers)?,
                expected_revision: expected_revision(&headers)?,
                token: lease_token(&headers)?,
                input,
                idempotency_key: idempotency_key(&headers)?,
            })
            .await
            .map_err(Problem::from)?,
    )
}

async fn lease_command(
    state: &HealthState,
    headers: &HeaderMap,
    auth: &Authenticated,
    workspace: Uuid,
    document: Uuid,
    release: bool,
) -> Result<Option<adoc_application::document::EditLeaseView>, Problem> {
    command(state, headers, auth)?;
    state
        .document
        .service
        .mutate_lease(LeaseCommandRequest {
            actor_id: auth.principal.user.id,
            workspace_id: workspace,
            document_id: document,
            client_instance_id: client(headers)?,
            expected_revision: expected_revision(headers)?,
            token: lease_token(headers)?,
            release,
            idempotency_key: idempotency_key(headers)?,
        })
        .await
        .map_err(Problem::from)
}
fn client(headers: &HeaderMap) -> Result<Uuid, Problem> {
    headers
        .get("x-client-instance")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| Uuid::parse_str(value).ok())
        .ok_or_else(|| Problem::from(adoc_application::governance::GovernanceError::Validation))
}
fn lease_token(headers: &HeaderMap) -> Result<&str, Problem> {
    headers
        .get("x-edit-lease")
        .and_then(|value| value.to_str().ok())
        .filter(|value| value.len() == 43)
        .ok_or_else(|| Problem::from(adoc_application::governance::GovernanceError::Validation))
}
fn command(state: &HealthState, headers: &HeaderMap, auth: &Authenticated) -> Result<(), Problem> {
    validate_command(&state.identity, headers, auth)
}
fn json<T: serde::Serialize>(value: T) -> Result<Json<serde_json::Value>, Problem> {
    Ok(Json(
        serde_json::to_value(value).map_err(|_| Problem::internal())?,
    ))
}
