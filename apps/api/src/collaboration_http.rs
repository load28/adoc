use std::sync::Arc;

use adoc_adapters::{
    identity::{SystemClock, SystemSecureRandom},
    postgres::{PostgresCollaborationRepository, PostgresStore},
};
use adoc_application::collaboration::{
    CollaborationService, CreateDiscussionInput, InboxAction, InboxFilter, ReadAllInput,
    RichMessage, TopicInput, UpdateDiscussionInput,
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
pub(crate) struct CollaborationRuntime {
    pub(crate) service: Arc<CollaborationService>,
}
impl CollaborationRuntime {
    pub(crate) fn new(store: &PostgresStore) -> Self {
        Self {
            service: Arc::new(CollaborationService::new(
                Arc::new(PostgresCollaborationRepository::new(store)),
                Arc::new(SystemClock),
                Arc::new(SystemSecureRandom),
            )),
        }
    }
}

pub(crate) fn collaboration_routes() -> Router<HealthState> {
    Router::new()
        .route(
            "/workspaces/{workspace_id}/documents/{document_id}/discussions",
            get(list_discussions).post(create_discussion),
        )
        .route(
            "/workspaces/{workspace_id}/discussions/{discussion_id}",
            get(get_discussion).put(update_discussion),
        )
        .route(
            "/workspaces/{workspace_id}/discussions/{discussion_id}/close",
            post(close_discussion),
        )
        .route(
            "/workspaces/{workspace_id}/discussions/{discussion_id}/reopen",
            post(reopen_discussion),
        )
        .route(
            "/workspaces/{workspace_id}/discussions/{discussion_id}/topics",
            post(add_topic),
        )
        .route(
            "/workspaces/{workspace_id}/discussions/{discussion_id}/topics/{topic_id}",
            axum::routing::delete(remove_topic),
        )
        .route(
            "/workspaces/{workspace_id}/discussions/{discussion_id}/messages",
            post(create_message),
        )
        .route(
            "/workspaces/{workspace_id}/discussions/{discussion_id}/messages/{message_id}",
            axum::routing::put(update_message).delete(redact_message),
        )
        .route("/workspaces/{workspace_id}/inbox", get(list_inbox))
        .route(
            "/workspaces/{workspace_id}/inbox/{item_id}/read",
            post(mark_read),
        )
        .route(
            "/workspaces/{workspace_id}/inbox/read-all",
            post(mark_all_read),
        )
        .route(
            "/workspaces/{workspace_id}/inbox/{item_id}/resolve",
            post(resolve_item),
        )
}

#[derive(Deserialize)]
struct Cursor {
    cursor: Option<String>,
}
#[derive(Deserialize)]
struct InboxQuery {
    cursor: Option<String>,
    status: Option<String>,
}
async fn list_discussions(
    State(s): State<HealthState>,
    auth: Authenticated,
    Path((w, d)): Path<(Uuid, Uuid)>,
    Query(q): Query<Cursor>,
) -> Result<Json<serde_json::Value>, Problem> {
    json(
        s.collaboration
            .service
            .list_discussions(auth.principal.user.id, w, d, q.cursor)
            .await
            .map_err(Problem::from)?,
    )
}
async fn create_discussion(
    State(s): State<HealthState>,
    headers: HeaderMap,
    auth: Authenticated,
    Path((w, d)): Path<(Uuid, Uuid)>,
    Json(input): Json<CreateDiscussionInput>,
) -> Result<Response, Problem> {
    validate_command(&s.identity, &headers, &auth)?;
    let value = s
        .collaboration
        .service
        .create_discussion(
            auth.principal.user.id,
            w,
            d,
            idempotency_key(&headers)?,
            input,
        )
        .await
        .map_err(Problem::from)?;
    Ok((StatusCode::CREATED, Json(value)).into_response())
}
async fn get_discussion(
    State(s): State<HealthState>,
    auth: Authenticated,
    Path((w, d)): Path<(Uuid, Uuid)>,
    Query(q): Query<Cursor>,
) -> Result<Json<serde_json::Value>, Problem> {
    json(
        s.collaboration
            .service
            .get_discussion(auth.principal.user.id, w, d, q.cursor)
            .await
            .map_err(Problem::from)?,
    )
}
async fn update_discussion(
    State(s): State<HealthState>,
    headers: HeaderMap,
    auth: Authenticated,
    Path((w, d)): Path<(Uuid, Uuid)>,
    Json(input): Json<UpdateDiscussionInput>,
) -> Result<Json<serde_json::Value>, Problem> {
    command(&s, &headers, &auth)?;
    json(
        s.collaboration
            .service
            .update_discussion(
                auth.principal.user.id,
                w,
                d,
                expected_revision(&headers)?,
                idempotency_key(&headers)?,
                input,
            )
            .await
            .map_err(Problem::from)?,
    )
}
async fn close_discussion(
    State(s): State<HealthState>,
    headers: HeaderMap,
    auth: Authenticated,
    Path((w, d)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, Problem> {
    command(&s, &headers, &auth)?;
    json(
        s.collaboration
            .service
            .close(
                auth.principal.user.id,
                w,
                d,
                expected_revision(&headers)?,
                idempotency_key(&headers)?,
            )
            .await
            .map_err(Problem::from)?,
    )
}
async fn reopen_discussion(
    State(s): State<HealthState>,
    headers: HeaderMap,
    auth: Authenticated,
    Path((w, d)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, Problem> {
    command(&s, &headers, &auth)?;
    json(
        s.collaboration
            .service
            .reopen(
                auth.principal.user.id,
                w,
                d,
                expected_revision(&headers)?,
                idempotency_key(&headers)?,
            )
            .await
            .map_err(Problem::from)?,
    )
}
async fn add_topic(
    State(s): State<HealthState>,
    headers: HeaderMap,
    auth: Authenticated,
    Path((w, d)): Path<(Uuid, Uuid)>,
    Json(input): Json<TopicInput>,
) -> Result<Response, Problem> {
    command(&s, &headers, &auth)?;
    let value = s
        .collaboration
        .service
        .add_topic(
            auth.principal.user.id,
            w,
            d,
            expected_revision(&headers)?,
            idempotency_key(&headers)?,
            input,
        )
        .await
        .map_err(Problem::from)?;
    Ok((StatusCode::CREATED, Json(value)).into_response())
}
async fn remove_topic(
    State(s): State<HealthState>,
    headers: HeaderMap,
    auth: Authenticated,
    Path((w, d, t)): Path<(Uuid, Uuid, Uuid)>,
) -> Result<StatusCode, Problem> {
    command(&s, &headers, &auth)?;
    s.collaboration
        .service
        .remove_topic(
            auth.principal.user.id,
            w,
            d,
            t,
            expected_revision(&headers)?,
            idempotency_key(&headers)?,
        )
        .await
        .map_err(Problem::from)?;
    Ok(StatusCode::NO_CONTENT)
}
async fn create_message(
    State(s): State<HealthState>,
    headers: HeaderMap,
    auth: Authenticated,
    Path((w, d)): Path<(Uuid, Uuid)>,
    Json(input): Json<RichMessage>,
) -> Result<Response, Problem> {
    command(&s, &headers, &auth)?;
    let value = s
        .collaboration
        .service
        .create_message(
            auth.principal.user.id,
            w,
            d,
            idempotency_key(&headers)?,
            input,
        )
        .await
        .map_err(Problem::from)?;
    Ok((StatusCode::CREATED, Json(value)).into_response())
}
async fn update_message(
    State(s): State<HealthState>,
    headers: HeaderMap,
    auth: Authenticated,
    Path((w, d, m)): Path<(Uuid, Uuid, Uuid)>,
    Json(input): Json<RichMessage>,
) -> Result<Json<serde_json::Value>, Problem> {
    command(&s, &headers, &auth)?;
    json(
        s.collaboration
            .service
            .update_message(
                auth.principal.user.id,
                w,
                d,
                m,
                expected_revision(&headers)?,
                idempotency_key(&headers)?,
                input,
            )
            .await
            .map_err(Problem::from)?,
    )
}
async fn redact_message(
    State(s): State<HealthState>,
    headers: HeaderMap,
    auth: Authenticated,
    Path((w, d, m)): Path<(Uuid, Uuid, Uuid)>,
) -> Result<StatusCode, Problem> {
    command(&s, &headers, &auth)?;
    s.collaboration
        .service
        .redact_message(
            auth.principal.user.id,
            w,
            d,
            m,
            expected_revision(&headers)?,
            idempotency_key(&headers)?,
        )
        .await
        .map_err(Problem::from)?;
    Ok(StatusCode::NO_CONTENT)
}
async fn list_inbox(
    State(s): State<HealthState>,
    auth: Authenticated,
    Path(w): Path<Uuid>,
    Query(q): Query<InboxQuery>,
) -> Result<Json<serde_json::Value>, Problem> {
    let filter = match q.status.as_deref() {
        None | Some("ALL") => InboxFilter::All,
        Some("UNREAD") => InboxFilter::Unread,
        Some("ACTIONABLE") => InboxFilter::Actionable,
        Some("RESOLVED") => InboxFilter::Resolved,
        _ => {
            return Err(Problem::from(
                adoc_application::governance::GovernanceError::Validation,
            ));
        }
    };
    json(
        s.collaboration
            .service
            .list_inbox(auth.principal.user.id, w, q.cursor, filter)
            .await
            .map_err(Problem::from)?,
    )
}
async fn mark_read(
    State(s): State<HealthState>,
    headers: HeaderMap,
    auth: Authenticated,
    Path((w, i)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, Problem> {
    command(&s, &headers, &auth)?;
    Ok(Json(
        s.collaboration
            .service
            .inbox(
                auth.principal.user.id,
                w,
                Some(i),
                None,
                idempotency_key(&headers)?,
                InboxAction::Read,
            )
            .await
            .map_err(Problem::from)?,
    ))
}
async fn mark_all_read(
    State(s): State<HealthState>,
    headers: HeaderMap,
    auth: Authenticated,
    Path(w): Path<Uuid>,
    Json(input): Json<ReadAllInput>,
) -> Result<Json<serde_json::Value>, Problem> {
    command(&s, &headers, &auth)?;
    Ok(Json(
        s.collaboration
            .service
            .inbox(
                auth.principal.user.id,
                w,
                None,
                Some(input.before),
                idempotency_key(&headers)?,
                InboxAction::ReadAll,
            )
            .await
            .map_err(Problem::from)?,
    ))
}
async fn resolve_item(
    State(s): State<HealthState>,
    headers: HeaderMap,
    auth: Authenticated,
    Path((w, i)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, Problem> {
    command(&s, &headers, &auth)?;
    Ok(Json(
        s.collaboration
            .service
            .inbox(
                auth.principal.user.id,
                w,
                Some(i),
                None,
                idempotency_key(&headers)?,
                InboxAction::Resolve,
            )
            .await
            .map_err(Problem::from)?,
    ))
}
fn command(s: &HealthState, h: &HeaderMap, a: &Authenticated) -> Result<(), Problem> {
    validate_command(&s.identity, h, a)
}
fn json<T: serde::Serialize>(value: T) -> Result<Json<serde_json::Value>, Problem> {
    serde_json::to_value(value)
        .map(Json)
        .map_err(|_| Problem::internal())
}
