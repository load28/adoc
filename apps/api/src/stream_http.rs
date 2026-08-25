use std::{collections::VecDeque, convert::Infallible, sync::Arc, time::Duration};

use adoc_adapters::{
    job_queue::RedisStreamWakeHub,
    postgres::{PostgresStore, PostgresStreamRepository},
};
use adoc_application::{
    governance::GovernanceError,
    operations::StreamWake,
    stream::{StreamDelivery, StreamService, StreamSession},
};
use adoc_configuration::AppConfig;
use axum::{
    Router,
    extract::{Query, State},
    http::{HeaderMap, HeaderValue, header},
    response::{IntoResponse, Response, Sse, sse::Event},
    routing::get,
};
use futures_util::stream;
use serde::Deserialize;
use serde_json::json;
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::{
    HealthState,
    identity_http::{Authenticated, Problem},
};

#[derive(Clone)]
pub(crate) struct StreamRuntime {
    service: Arc<StreamService>,
    wake_hub: RedisStreamWakeHub,
}

impl StreamRuntime {
    pub(crate) fn new(
        config: &AppConfig,
        store: &PostgresStore,
        permission: Arc<adoc_application::permission::PermissionService>,
    ) -> Result<Self, GovernanceError> {
        Ok(Self {
            service: Arc::new(StreamService::new(
                Arc::new(PostgresStreamRepository::new(store)),
                permission,
            )),
            wake_hub: RedisStreamWakeHub::start(config.dependencies.redis_url.value.expose(), 256)?,
        })
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct StreamQuery {
    workspace_id: Uuid,
    cursor: Option<String>,
}

pub(crate) fn stream_routes() -> Router<HealthState> {
    Router::new().route("/stream", get(open_stream))
}

async fn open_stream(
    State(state): State<HealthState>,
    auth: Authenticated,
    headers: HeaderMap,
    Query(query): Query<StreamQuery>,
) -> Result<Response, Problem> {
    let header_cursor = headers
        .get("last-event-id")
        .map(|value| value.to_str())
        .transpose()
        .map_err(|_| Problem::from(GovernanceError::StreamCursorInvalid))?;
    if let (Some(query_cursor), Some(header_cursor)) = (query.cursor.as_deref(), header_cursor)
        && query_cursor != header_cursor
    {
        return Err(Problem::from(GovernanceError::StreamCursorInvalid));
    }
    let cursor = query.cursor.as_deref().or(header_cursor);
    let opened = state
        .stream
        .service
        .open(auth.principal.user.id, query.workspace_id, cursor)
        .await
        .map_err(Problem::from)?;
    let stream_state = ConnectionState {
        service: state.stream.service.clone(),
        session: opened.session,
        wake_receiver: state.stream.wake_hub.subscribe(),
        pending: VecDeque::new(),
        reset_required: opened.reset_required,
        finished: false,
    };
    let events = stream::unfold(stream_state, next_event);
    let mut response = Sse::new(events)
        .keep_alive(
            axum::response::sse::KeepAlive::new()
                .interval(Duration::from_secs(15))
                .text("heartbeat"),
        )
        .into_response();
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("no-cache, no-store"),
    );
    response
        .headers_mut()
        .insert("x-accel-buffering", HeaderValue::from_static("no"));
    Ok(response)
}

struct ConnectionState {
    service: Arc<StreamService>,
    session: StreamSession,
    wake_receiver: broadcast::Receiver<StreamWake>,
    pending: VecDeque<StreamDelivery>,
    reset_required: bool,
    finished: bool,
}

async fn next_event(
    mut state: ConnectionState,
) -> Option<(Result<Event, Infallible>, ConnectionState)> {
    if state.finished {
        return None;
    }
    if state.reset_required {
        state.finished = true;
        return Some((Ok(reset_event()), state));
    }
    loop {
        if let Some(delivery) = state.pending.pop_front() {
            return Some((Ok(delivery_event(delivery)), state));
        }
        match state.service.next_page(&mut state.session).await {
            Ok(page) if page.reset_required => {
                state.finished = true;
                return Some((Ok(reset_event()), state));
            }
            Ok(page) if !page.deliveries.is_empty() => {
                state.pending = page.deliveries.into();
                continue;
            }
            Ok(_) => {}
            Err(_) => return None,
        }
        match tokio::time::timeout(Duration::from_secs(15), state.wake_receiver.recv()).await {
            Ok(Ok(wake)) if wake.workspace_id == state.session.workspace_id => continue,
            Ok(Ok(_)) | Err(_) => continue,
            Ok(Err(broadcast::error::RecvError::Lagged(_)))
            | Ok(Err(broadcast::error::RecvError::Closed)) => return None,
        }
    }
}

fn delivery_event(delivery: StreamDelivery) -> Event {
    let data = serde_json::to_string(&delivery.envelope).unwrap_or_else(|_| "{}".to_owned());
    Event::default()
        .id(delivery.cursor)
        .event(delivery.event_type)
        .data(data)
}

fn reset_event() -> Event {
    Event::default()
        .event("STREAM_RESET_REQUIRED")
        .data(json!({"code": "STREAM_RESET_REQUIRED"}).to_string())
}
