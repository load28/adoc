#![forbid(unsafe_code)]

use std::process::ExitCode;

use adoc_adapters::postgres::{DatabaseSettings, PostgresStore};
use adoc_configuration::{AppConfig, ConfigError, ConfigSource, ServiceKind};
use adoc_telemetry::{SafeEvent, TelemetryConfig};

#[tokio::main]
async fn main() -> ExitCode {
    let arguments = std::env::args().skip(1).collect::<Vec<_>>();
    if arguments
        .iter()
        .any(|argument| argument == "--check-config")
    {
        return check_config();
    }
    let health_only = arguments.iter().any(|argument| argument == "--health");
    match run(health_only).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => fail(error),
    }
}

fn parse_config() -> Result<AppConfig, ConfigError> {
    let source = ConfigSource::from_process()?;
    AppConfig::parse(&source, ServiceKind::Worker)
}

fn check_config() -> ExitCode {
    let config = match parse_config() {
        Ok(config) => config,
        Err(error) => return fail(error),
    };
    println!("{}", config.preflight_json());
    ExitCode::SUCCESS
}

async fn run(health_only: bool) -> Result<(), Box<dyn std::error::Error>> {
    let config = parse_config()?;
    let telemetry = TelemetryConfig::from(&config.common);
    adoc_telemetry::initialize(&telemetry)?;
    let store = PostgresStore::connect(DatabaseSettings {
        url: config.dependencies.database_url.value.expose(),
        max_connections: config.dependencies.db_max_connections,
        application_name: "adoc-worker",
    })
    .await?;
    store.preflight().await?;
    if health_only {
        store.close().await;
        return Ok(());
    }
    SafeEvent::new(&telemetry, "SERVICE_STARTED")
        .field("environment", format!("{:?}", config.common.environment))
        .emit();
    shutdown_signal().await;
    store.close().await;
    Ok(())
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
