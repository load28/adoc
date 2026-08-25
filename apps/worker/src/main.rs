#![forbid(unsafe_code)]

use std::{process::ExitCode, sync::Arc, time::Duration};

use adoc_adapters::{
    identity::SystemClock,
    job_executor::WorkerJobExecutor,
    job_queue::RedisJobSignalQueue,
    object_storage::LocalObjectStorage,
    postgres::{
        DatabaseSettings, PostgresFileRepository, PostgresJobRepository,
        PostgresRetentionRepository, PostgresSearchProjectionRepository, PostgresStore,
    },
    search_index::OpenSearchIndex,
};
use adoc_application::{
    jobs::JobRuntime,
    operations::{FileGarbageCollector, RetentionService},
    search::SearchProjectionService,
};
use adoc_configuration::{
    AppConfig, ConfigError, ConfigSource, Environment, ObjectStorageDriver, ServiceKind,
};
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
    let worker = config
        .worker
        .as_ref()
        .ok_or("worker configuration is missing")?;
    let job_lease = chrono::Duration::from_std(worker.job_lease)?;
    let job_batch_size = i64::from(worker.outbox_batch_size);
    let reconcile_every = worker.reconcile_interval;
    let telemetry = TelemetryConfig::from(&config.common);
    adoc_telemetry::initialize(&telemetry)?;
    let store = PostgresStore::connect(DatabaseSettings {
        url: config.dependencies.database_url.value.expose(),
        max_connections: config.dependencies.db_max_connections,
        application_name: "adoc-worker",
    })
    .await?;
    store.preflight().await?;
    let retention_url = config
        .dependencies
        .retention_database_url
        .as_ref()
        .ok_or("retention database URL is missing")?;
    let retention_store = PostgresStore::connect(DatabaseSettings {
        url: retention_url.value.expose(),
        max_connections: config.dependencies.db_max_connections,
        application_name: "adoc-retention-worker",
    })
    .await?;
    retention_store.preflight().await?;
    if matches!(
        config.common.environment,
        Environment::Staging | Environment::Production
    ) && retention_store.current_user().await? != "adoc_retention"
    {
        return Err("retention database credential must use adoc_retention".into());
    }
    if health_only {
        store.close().await;
        retention_store.close().await;
        return Ok(());
    }
    SafeEvent::new(&telemetry, "SERVICE_STARTED")
        .field("environment", format!("{:?}", config.common.environment))
        .emit();
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
    let storage =
        Arc::new(LocalObjectStorage::new(root).map_err(|_| "invalid local object storage root")?);
    let gc = FileGarbageCollector::new(
        Arc::new(PostgresFileRepository::new(&store)),
        storage.clone(),
        Arc::new(SystemClock),
    );
    let retention = RetentionService::new(
        Arc::new(PostgresRetentionRepository::new(&retention_store)),
        storage,
        Arc::new(SystemClock),
        Arc::from(format!("retention-{}", config.common.release_sha)),
    );
    let job_repository = Arc::new(PostgresJobRepository::new(&store));
    let search_index = Arc::new(OpenSearchIndex::new(
        config.dependencies.opensearch_url.clone(),
        config.dependencies.search_index_prefix.clone(),
        config.dependencies.embedding_dimension,
        config
            .dependencies
            .opensearch_credential
            .as_ref()
            .map(|secret| secret.value.expose()),
    )?);
    let job_executor = Arc::new(WorkerJobExecutor::new(
        job_repository.clone(),
        SearchProjectionService::new(
            Arc::new(PostgresSearchProjectionRepository::new(&store)),
            search_index,
        ),
    ));
    let job_runtime = JobRuntime::new(
        job_repository.clone(),
        job_executor,
        Arc::new(
            RedisJobSignalQueue::connect(
                config.dependencies.redis_url.value.expose(),
                &config.dependencies.queue_namespace,
            )
            .await?,
        ),
        Arc::new(SystemClock),
        Arc::from(format!("worker-{}", config.common.release_sha)),
        job_lease,
    );
    let mut job_interval = tokio::time::interval(Duration::from_secs(1));
    let mut reconcile_interval = tokio::time::interval(reconcile_every);
    let mut maintenance_interval = tokio::time::interval(Duration::from_secs(60));
    loop {
        tokio::select! {
            _ = shutdown_signal() => break,
            _ = job_interval.tick() => {
                job_runtime.run_once(job_batch_size, false).await?;
            }
            _ = reconcile_interval.tick() => {
                job_runtime.run_once(job_batch_size, true).await?;
            }
            _ = maintenance_interval.tick() => {
                retention.run_once(25).await?;
                gc.run_once(100).await?;
                job_runtime.cleanup_stream(1_000).await?;
            }
        }
    }
    store.close().await;
    retention_store.close().await;
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
