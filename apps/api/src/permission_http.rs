use std::sync::Arc;

use adoc_adapters::{
    permission_cache::{RedisPermissionCache, UnavailablePermissionCache},
    postgres::{PostgresPermissionRepository, PostgresStore},
};
use adoc_application::{
    governance::GovernanceError,
    permission::{
        PermissionGrantInput, PermissionService, SetPermissionCommand, SetPublishPolicyInput,
        SubjectKind,
    },
};
use adoc_configuration::AppConfig;
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    routing::get,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    HealthState,
    identity_http::{Authenticated, Problem, expected_revision, idempotency_key, validate_command},
};

#[derive(Clone)]
pub(crate) struct PermissionRuntime {
    pub(crate) service: Arc<PermissionService>,
}

impl PermissionRuntime {
    pub(crate) async fn new(
        config: &AppConfig,
        store: &PostgresStore,
    ) -> Result<Self, GovernanceError> {
        let cache: Arc<dyn adoc_application::permission::PermissionCache> =
            match RedisPermissionCache::connect(
                config.dependencies.redis_url.value.expose(),
                &config.dependencies.queue_namespace,
            )
            .await
            {
                Ok(cache) => Arc::new(cache),
                Err(()) => Arc::new(UnavailablePermissionCache),
            };
        Ok(Self {
            service: Arc::new(PermissionService::new(
                Arc::new(PostgresPermissionRepository::new(store)),
                cache,
                Arc::new(adoc_adapters::identity::SystemClock),
            )),
        })
    }
}

pub(crate) fn permission_routes() -> Router<HealthState> {
    Router::new()
        .route(
            "/workspaces/{workspace_id}/documents/{document_id}/permissions",
            get(get_document_permissions),
        )
        .route(
            "/workspaces/{workspace_id}/documents/{document_id}/permissions/{grant_id}",
            axum::routing::put(set_document_permission).delete(delete_document_permission),
        )
        .route(
            "/workspaces/{workspace_id}/documents/{document_id}/permission-explanation",
            get(explain_effective_permission),
        )
        .route(
            "/workspaces/{workspace_id}/documents/{document_id}/publish-policy",
            get(get_publish_policy).put(set_publish_policy),
        )
}

async fn get_document_permissions(
    State(state): State<HealthState>,
    auth: Authenticated,
    Path((workspace, document)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, Problem> {
    json_value(
        state
            .permission
            .service
            .get_permissions(auth.principal.user.id, workspace, document)
            .await
            .map_err(Problem::from)?,
    )
}

async fn set_document_permission(
    State(state): State<HealthState>,
    headers: HeaderMap,
    auth: Authenticated,
    Path((workspace, document, grant)): Path<(Uuid, Uuid, Uuid)>,
    Json(input): Json<PermissionGrantInput>,
) -> Result<Json<serde_json::Value>, Problem> {
    validate_command(&state.identity, &headers, &auth)?;
    json_value(
        state
            .permission
            .service
            .set_permission(SetPermissionCommand {
                actor_id: auth.principal.user.id,
                workspace_id: workspace,
                document_id: document,
                grant_id: grant,
                expected_revision: expected_revision(&headers)?,
                input,
                idempotency_key: idempotency_key(&headers)?.to_owned(),
            })
            .await
            .map_err(Problem::from)?,
    )
}

async fn delete_document_permission(
    State(state): State<HealthState>,
    headers: HeaderMap,
    auth: Authenticated,
    Path((workspace, document, grant)): Path<(Uuid, Uuid, Uuid)>,
) -> Result<StatusCode, Problem> {
    validate_command(&state.identity, &headers, &auth)?;
    state
        .permission
        .service
        .delete_permission(
            auth.principal.user.id,
            workspace,
            document,
            grant,
            expected_revision(&headers)?,
            idempotency_key(&headers)?,
        )
        .await
        .map_err(Problem::from)?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ExplanationQuery {
    subject_kind: SubjectKind,
    subject_id: Uuid,
}

async fn explain_effective_permission(
    State(state): State<HealthState>,
    auth: Authenticated,
    Path((workspace, document)): Path<(Uuid, Uuid)>,
    Query(query): Query<ExplanationQuery>,
) -> Result<Json<serde_json::Value>, Problem> {
    json_value(
        state
            .permission
            .service
            .explain(
                auth.principal.user.id,
                workspace,
                document,
                query.subject_kind,
                query.subject_id,
            )
            .await
            .map_err(Problem::from)?,
    )
}

async fn get_publish_policy(
    State(state): State<HealthState>,
    auth: Authenticated,
    Path((workspace, document)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, Problem> {
    json_value(
        state
            .permission
            .service
            .get_policy(auth.principal.user.id, workspace, document)
            .await
            .map_err(Problem::from)?,
    )
}

async fn set_publish_policy(
    State(state): State<HealthState>,
    headers: HeaderMap,
    auth: Authenticated,
    Path((workspace, document)): Path<(Uuid, Uuid)>,
    Json(input): Json<SetPublishPolicyInput>,
) -> Result<Json<serde_json::Value>, Problem> {
    validate_command(&state.identity, &headers, &auth)?;
    json_value(
        state
            .permission
            .service
            .set_policy(
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

fn json_value<T: serde::Serialize>(value: T) -> Result<Json<serde_json::Value>, Problem> {
    Ok(Json(
        serde_json::to_value(value).map_err(|_| Problem::internal())?,
    ))
}
