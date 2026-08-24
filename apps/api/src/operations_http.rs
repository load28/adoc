use std::sync::Arc;

use adoc_adapters::{
    identity::SystemClock,
    object_storage::LocalObjectStorage,
    postgres::{PostgresAuditRepository, PostgresRetentionRepository, PostgresStore},
};
use adoc_application::operations::{AuditService, RetentionService};
use adoc_configuration::{AppConfig, ObjectStorageDriver};
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::get,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    HealthState,
    identity_http::{Authenticated, Problem},
};

#[derive(Clone)]
pub(crate) struct OperationsRuntime {
    service: Arc<AuditService>,
    pub(crate) retention: Arc<RetentionService>,
}

impl OperationsRuntime {
    pub(crate) fn new(
        config: &AppConfig,
        store: &PostgresStore,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        if config.storage.driver != ObjectStorageDriver::Local {
            return Err("S3 object storage adapter is not configured in this release".into());
        }
        let root = config
            .storage
            .local_root
            .clone()
            .ok_or("local object storage root is missing")?;
        let root = if root.is_absolute() {
            root
        } else {
            std::env::current_dir()?.join(root)
        };
        Ok(Self {
            service: Arc::new(AuditService::new(Arc::new(PostgresAuditRepository::new(
                store,
            )))),
            retention: Arc::new(RetentionService::new(
                Arc::new(PostgresRetentionRepository::new(store)),
                Arc::new(
                    LocalObjectStorage::new(root)
                        .map_err(|_| "invalid local object storage root")?,
                ),
                Arc::new(SystemClock),
                Arc::from("adoc-api-request-only"),
            )),
        })
    }
}

#[derive(Deserialize)]
struct CursorQuery {
    cursor: Option<String>,
}

pub(crate) fn operations_routes() -> Router<HealthState> {
    Router::new().route(
        "/workspaces/{workspace_id}/audit-events",
        get(list_audit_events),
    )
}

async fn list_audit_events(
    State(state): State<HealthState>,
    auth: Authenticated,
    Path(workspace): Path<Uuid>,
    Query(query): Query<CursorQuery>,
) -> Result<Json<serde_json::Value>, Problem> {
    let page = state
        .operations
        .service
        .list(auth.principal.user.id, workspace, query.cursor)
        .await
        .map_err(Problem::from)?;
    Ok(Json(
        serde_json::to_value(page).map_err(|_| Problem::internal())?,
    ))
}
