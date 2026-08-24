#![forbid(unsafe_code)]

use std::{process::ExitCode, sync::Arc};

use adoc_adapters::postgres::{DatabaseSettings, PostgresStore};
use adoc_configuration::{
    AppConfig, ConfigError, ConfigSource, DatabaseBootstrapConfig, ServiceKind,
};
use adoc_telemetry::{SafeEvent, TelemetryConfig};
use axum::{Json, Router, extract::State, http::StatusCode, routing::get};

mod collaboration_http;
mod document_http;
mod governance_http;
mod identity_http;
mod permission_http;
mod publishing_http;

use collaboration_http::{CollaborationRuntime, collaboration_routes};
use document_http::{DocumentRuntime, document_routes};
use governance_http::{GovernanceRuntime, governance_routes};
use identity_http::{IdentityRuntime, identity_routes};
use permission_http::{PermissionRuntime, permission_routes};
use publishing_http::{PublishingRuntime, public_routes, publishing_routes};

#[tokio::main]
async fn main() -> ExitCode {
    let arguments = std::env::args().skip(1).collect::<Vec<_>>();
    if arguments.iter().any(|argument| argument == "--migrate") {
        return migrate().await;
    }
    if arguments
        .iter()
        .any(|argument| argument == "--check-config")
    {
        return check_config();
    }
    match serve().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => fail(error),
    }
}

async fn migrate() -> ExitCode {
    let source = match ConfigSource::from_process() {
        Ok(source) => source,
        Err(error) => return fail(error),
    };
    let config = match DatabaseBootstrapConfig::parse(&source) {
        Ok(config) => config,
        Err(error) => return fail(error),
    };
    let store = match PostgresStore::connect(DatabaseSettings {
        url: config.database_url.value.expose(),
        max_connections: config.max_connections,
        application_name: "adoc-migrate",
    })
    .await
    {
        Ok(store) => store,
        Err(error) => return fail(error),
    };
    if let Err(error) = store.migrate().await {
        return fail(error);
    }
    match store.preflight().await {
        Ok(preflight) => {
            println!(
                "{{\"status\":\"ok\",\"serverMajorVersion\":{},\"appliedMigrations\":{}}}",
                preflight.server_major_version, preflight.applied_migrations
            );
            store.close().await;
            ExitCode::SUCCESS
        }
        Err(error) => fail(error),
    }
}

fn parse_config() -> Result<AppConfig, ConfigError> {
    let source = ConfigSource::from_process()?;
    AppConfig::parse(&source, ServiceKind::Api)
}

fn check_config() -> ExitCode {
    let config = match parse_config() {
        Ok(config) => config,
        Err(error) => return fail(error),
    };
    println!("{}", config.preflight_json());
    ExitCode::SUCCESS
}

#[derive(Clone)]
struct HealthState {
    store: PostgresStore,
    release_sha: Arc<str>,
    identity: IdentityRuntime,
    governance: GovernanceRuntime,
    document: DocumentRuntime,
    permission: PermissionRuntime,
    publishing: PublishingRuntime,
    collaboration: CollaborationRuntime,
}

async fn serve() -> Result<(), Box<dyn std::error::Error>> {
    let config = parse_config()?;
    let bind = config
        .common
        .http_bind
        .ok_or("API HTTP bind is not configured")?;
    let telemetry = TelemetryConfig::from(&config.common);
    adoc_telemetry::initialize(&telemetry)?;
    let store = PostgresStore::connect(DatabaseSettings {
        url: config.dependencies.database_url.value.expose(),
        max_connections: config.dependencies.db_max_connections,
        application_name: "adoc-api",
    })
    .await?;
    store.preflight().await?;
    let identity = IdentityRuntime::new(&config, &store).await?;
    let governance = GovernanceRuntime::new(&config, &store)?;
    let permission = PermissionRuntime::new(&config, &store).await?;
    let document = DocumentRuntime::new(&store);
    let publishing = PublishingRuntime::new(&store);
    let collaboration = CollaborationRuntime::new(&store);
    let state = HealthState {
        store,
        release_sha: Arc::from(config.common.release_sha.as_str()),
        identity,
        governance,
        document,
        permission,
        publishing,
        collaboration,
    };
    let app = Router::new()
        .route("/health/live", get(live))
        .route("/health/ready", get(ready))
        .nest(
            "/api/v1",
            identity_routes()
                .merge(governance_routes())
                .merge(document_routes())
                .merge(permission_routes())
                .merge(publishing_routes())
                .merge(collaboration_routes()),
        )
        .merge(public_routes())
        .with_state(state.clone());
    let listener = tokio::net::TcpListener::bind(bind).await?;
    SafeEvent::new(&telemetry, "SERVICE_STARTED")
        .field("environment", format!("{:?}", config.common.environment))
        .emit();
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await?;
    state.store.close().await;
    Ok(())
}

async fn live(State(state): State<HealthState>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "service": "api",
        "releaseSha": state.release_sha.as_ref(),
    }))
}

async fn ready(State(state): State<HealthState>) -> (StatusCode, Json<serde_json::Value>) {
    match state.store.preflight().await {
        Ok(preflight) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "status": "ready",
                "core": {"postgres": "ready", "migration": "current"},
                "degraded": {"opensearch": "not_required", "ai": "not_required"},
                "serverMajorVersion": preflight.server_major_version,
            })),
        ),
        Err(_) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({
                "status": "not_ready",
                "core": {"postgres": "unavailable"},
            })),
        ),
    }
}

async fn shutdown_signal() {
    #[cfg(unix)]
    {
        let mut terminate =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                .expect("install SIGTERM handler");
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {},
            _ = terminate.recv() => {},
        }
    }
    #[cfg(not(unix))]
    tokio::signal::ctrl_c()
        .await
        .expect("install Ctrl-C handler");
}

fn fail(error: impl std::fmt::Display) -> ExitCode {
    eprintln!("configuration failed: {error}");
    ExitCode::FAILURE
}
