use std::{
    collections::{BTreeMap, BTreeSet},
    env,
    ffi::OsString,
    net::SocketAddr,
    path::PathBuf,
    str::FromStr,
    time::Duration,
};

use ipnet::IpNet;
use thiserror::Error;
use url::Url;

use crate::secret::load_secret;
use crate::{LoadedSecret, RotatingSecret, SecretMetadata};

const ALL_KEYS: &[&str] = &[
    "ADOC_ENV",
    "ADOC_RELEASE_SHA",
    "ADOC_HTTP_BIND",
    "ADOC_PUBLIC_ORIGIN",
    "ADOC_SHUTDOWN_GRACE",
    "ADOC_LOG_LEVEL",
    "ADOC_OTEL_ENDPOINT",
    "ADOC_DATABASE_URL_FILE",
    "ADOC_RETENTION_DATABASE_URL_FILE",
    "ADOC_DB_MAX_CONNECTIONS",
    "ADOC_REDIS_URL_FILE",
    "ADOC_QUEUE_NAMESPACE",
    "ADOC_OPENSEARCH_URL",
    "ADOC_OPENSEARCH_CREDENTIAL_FILE",
    "ADOC_SEARCH_INDEX_PREFIX",
    "ADOC_EMBEDDING_DIMENSION",
    "ADOC_GOOGLE_CLIENT_ID_FILE",
    "ADOC_GOOGLE_CLIENT_SECRET_FILE",
    "ADOC_SESSION_HMAC_KEY_FILE",
    "ADOC_CSRF_HMAC_KEY_FILE",
    "ADOC_TOKEN_HASH_PEPPER_FILE",
    "ADOC_TRUSTED_PROXY_CIDRS",
    "ADOC_SESSION_TTL",
    "ADOC_PUBLIC_LINK_MAX_TTL",
    "ADOC_OBJECT_STORAGE_DRIVER",
    "ADOC_LOCAL_OBJECT_ROOT",
    "ADOC_S3_BUCKET",
    "ADOC_S3_REGION",
    "ADOC_S3_ENDPOINT",
    "ADOC_S3_CREDENTIAL_FILE",
    "ADOC_UPLOAD_MAX_BYTES",
    "ADOC_ALLOWED_MIME_FILE",
    "ADOC_AI_DRIVER",
    "ADOC_CODEX_EXECUTABLE",
    "ADOC_OPENAI_API_KEY_FILE",
    "ADOC_AI_REQUEST_TIMEOUT",
    "ADOC_AI_KILL_GRACE",
    "ADOC_AI_MAX_CONTEXT_TOKENS",
    "ADOC_TRASH_RETENTION",
    "ADOC_WORKSPACE_RETENTION",
    "ADOC_JOB_LEASE",
    "ADOC_JOB_MAX_ATTEMPTS",
    "ADOC_OUTBOX_BATCH_SIZE",
    "ADOC_RECONCILE_INTERVAL",
];

const PLAIN_SECRET_KEYS: &[&str] = &[
    "ADOC_DATABASE_URL",
    "ADOC_RETENTION_DATABASE_URL",
    "ADOC_REDIS_URL",
    "ADOC_OPENSEARCH_CREDENTIAL",
    "ADOC_GOOGLE_CLIENT_ID",
    "ADOC_GOOGLE_CLIENT_SECRET",
    "ADOC_SESSION_HMAC_KEY",
    "ADOC_CSRF_HMAC_KEY",
    "ADOC_TOKEN_HASH_PEPPER",
    "ADOC_S3_CREDENTIAL",
    "ADOC_OPENAI_API_KEY",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ServiceKind {
    Api,
    Worker,
}

impl ServiceKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Api => "api",
            Self::Worker => "worker",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Environment {
    Development,
    Test,
    Staging,
    Production,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Trace => "trace",
            Self::Debug => "debug",
            Self::Info => "info",
            Self::Warn => "warn",
            Self::Error => "error",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ObjectStorageDriver {
    Local,
    S3,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AiDriver {
    CodexCli,
    OpenAiResponses,
}

#[derive(Debug)]
pub struct CommonConfig {
    pub environment: Environment,
    pub release_sha: String,
    pub service: ServiceKind,
    pub http_bind: Option<SocketAddr>,
    pub public_origin: Option<Url>,
    pub shutdown_grace: Duration,
    pub log_level: LogLevel,
    pub otel_endpoint: Option<Url>,
}

#[derive(Debug)]
pub struct DependencyConfig {
    pub database_url: LoadedSecret,
    pub retention_database_url: Option<LoadedSecret>,
    pub redis_url: LoadedSecret,
    pub db_max_connections: u32,
    pub queue_namespace: String,
    pub opensearch_url: Url,
    pub opensearch_credential: Option<LoadedSecret>,
    pub search_index_prefix: String,
    pub embedding_dimension: u32,
}

#[derive(Debug)]
pub struct DatabaseBootstrapConfig {
    pub environment: Environment,
    pub release_sha: String,
    pub database_url: LoadedSecret,
    pub max_connections: u32,
}

impl DatabaseBootstrapConfig {
    pub fn parse(source: &ConfigSource) -> Result<Self, ConfigError> {
        validate_keys(source)?;
        let environment = parse_environment(required(source, "ADOC_ENV")?)?;
        Ok(Self {
            environment,
            release_sha: nonempty(required(source, "ADOC_RELEASE_SHA")?, "ADOC_RELEASE_SHA")?
                .to_owned(),
            database_url: secret_url(
                source,
                "ADOC_DATABASE_URL_FILE",
                environment,
                &["postgres", "postgresql"],
            )?,
            max_connections: integer_range(
                source.get("ADOC_DB_MAX_CONNECTIONS").unwrap_or("5"),
                "ADOC_DB_MAX_CONNECTIONS",
                1,
                200,
            )?,
        })
    }
}

#[derive(Debug)]
pub struct AuthConfig {
    pub google_client_id: Option<LoadedSecret>,
    pub google_client_secret: Option<LoadedSecret>,
    pub session_hmac: Option<RotatingSecret>,
    pub csrf_hmac: Option<RotatingSecret>,
    pub token_hash_pepper: RotatingSecret,
    pub trusted_proxy_cidrs: Vec<IpNet>,
    pub session_ttl: Duration,
    pub public_link_max_ttl: Duration,
}

#[derive(Debug)]
pub struct StorageConfig {
    pub driver: ObjectStorageDriver,
    pub local_root: Option<PathBuf>,
    pub s3_bucket: Option<String>,
    pub s3_region: Option<String>,
    pub s3_endpoint: Option<Url>,
    pub s3_credential: Option<LoadedSecret>,
    pub upload_max_bytes: u64,
    pub allowed_mime_file: PathBuf,
}

#[derive(Debug)]
pub struct AiConfig {
    pub driver: AiDriver,
    pub codex_executable: Option<PathBuf>,
    pub openai_api_key: Option<LoadedSecret>,
    pub request_timeout: Duration,
    pub kill_grace: Duration,
    pub max_context_tokens: u32,
}

#[derive(Debug)]
pub struct WorkerConfig {
    pub trash_retention: Duration,
    pub workspace_retention: Duration,
    pub job_lease: Duration,
    pub job_max_attempts: u32,
    pub outbox_batch_size: u32,
    pub reconcile_interval: Duration,
}

#[derive(Debug)]
pub struct AppConfig {
    pub common: CommonConfig,
    pub dependencies: DependencyConfig,
    pub auth: AuthConfig,
    pub storage: StorageConfig,
    pub ai: AiConfig,
    pub worker: Option<WorkerConfig>,
}

impl AppConfig {
    pub fn parse(source: &ConfigSource, service: ServiceKind) -> Result<Self, ConfigError> {
        validate_keys(source)?;
        let environment = parse_environment(required(source, "ADOC_ENV")?)?;
        let common = parse_common(source, service, environment)?;
        let dependencies = parse_dependencies(source, service, environment)?;
        let auth = parse_auth(source, service, environment)?;
        let storage = parse_storage(source, environment)?;
        let ai = parse_ai(source, environment)?;
        let worker = (service == ServiceKind::Worker)
            .then(|| parse_worker(source, environment))
            .transpose()?;
        Ok(Self {
            common,
            dependencies,
            auth,
            storage,
            ai,
            worker,
        })
    }

    pub fn secret_metadata(&self) -> Vec<SecretMetadata> {
        let mut metadata = vec![
            self.dependencies.database_url.metadata.clone(),
            self.dependencies.redis_url.metadata.clone(),
            self.auth.token_hash_pepper.metadata().clone(),
        ];
        if let Some(value) = &self.dependencies.retention_database_url {
            metadata.push(value.metadata.clone());
        }
        if let Some(value) = &self.dependencies.opensearch_credential {
            metadata.push(value.metadata.clone());
        }
        for value in [&self.auth.google_client_id, &self.auth.google_client_secret]
            .into_iter()
            .flatten()
        {
            metadata.push(value.metadata.clone());
        }
        for value in [&self.auth.session_hmac, &self.auth.csrf_hmac]
            .into_iter()
            .flatten()
        {
            metadata.push(value.metadata().clone());
        }
        if let Some(value) = &self.storage.s3_credential {
            metadata.push(value.metadata.clone());
        }
        if let Some(value) = &self.ai.openai_api_key {
            metadata.push(value.metadata.clone());
        }
        metadata.sort_by_key(|value| value.source_key);
        metadata
    }

    pub fn preflight_json(&self) -> serde_json::Value {
        serde_json::json!({
            "status": "valid",
            "service": self.common.service.as_str(),
            "environment": format!("{:?}", self.common.environment).to_lowercase(),
            "releaseSha": self.common.release_sha,
            "secrets": self.secret_metadata().into_iter().map(|metadata| serde_json::json!({"source": metadata.source_key, "keyIds": metadata.key_ids})).collect::<Vec<_>>(),
            "connectivity": "deferred",
        })
    }
}

#[derive(Clone, Debug, Default)]
pub struct ConfigSource {
    values: BTreeMap<String, String>,
}

impl ConfigSource {
    pub fn new(values: impl IntoIterator<Item = (String, String)>) -> Self {
        Self {
            values: values.into_iter().collect(),
        }
    }

    pub fn from_process() -> Result<Self, ConfigError> {
        let mut values = BTreeMap::new();
        for (key, value) in env::vars_os() {
            if key.to_string_lossy().starts_with("ADOC_") {
                values.insert(os_string(key)?, os_string(value)?);
            }
        }
        Ok(Self { values })
    }

    pub fn with(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.values.insert(key.into(), value.into());
        self
    }

    pub fn without(mut self, key: &str) -> Self {
        self.values.remove(key);
        self
    }

    fn get(&self, key: &str) -> Option<&str> {
        self.values.get(key).map(String::as_str)
    }
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("unknown configuration key: {0}")]
    UnknownKey(String),
    #[error("plain secret environment key is forbidden: {0}")]
    PlainSecret(String),
    #[error("missing required configuration key: {0}")]
    Missing(&'static str),
    #[error("invalid configuration for {key}: {reason}")]
    Invalid { key: &'static str, reason: String },
    #[error("unable to read secret source for {key}: {reason}")]
    SecretFile { key: &'static str, reason: String },
    #[error("ADOC process environment contains non-UTF-8 data")]
    NonUtf8,
}

impl ConfigError {
    pub(crate) fn invalid(key: &'static str, reason: impl Into<String>) -> Self {
        Self::Invalid {
            key,
            reason: reason.into(),
        }
    }
    pub(crate) fn secret_file(key: &'static str, error: std::io::Error) -> Self {
        Self::SecretFile {
            key,
            reason: error.kind().to_string(),
        }
    }
}

fn validate_keys(source: &ConfigSource) -> Result<(), ConfigError> {
    let allowed: BTreeSet<_> = ALL_KEYS.iter().copied().collect();
    let plain: BTreeSet<_> = PLAIN_SECRET_KEYS.iter().copied().collect();
    for key in source.values.keys() {
        if plain.contains(key.as_str()) {
            return Err(ConfigError::PlainSecret(key.clone()));
        }
        if key.starts_with("ADOC_") && !allowed.contains(key.as_str()) {
            return Err(ConfigError::UnknownKey(key.clone()));
        }
    }
    Ok(())
}

fn parse_common(
    source: &ConfigSource,
    service: ServiceKind,
    environment: Environment,
) -> Result<CommonConfig, ConfigError> {
    let public_origin = optional_url(source, "ADOC_PUBLIC_ORIGIN")?;
    if environment == Environment::Production
        && public_origin
            .as_ref()
            .is_none_or(|url| url.scheme() != "https")
    {
        return Err(ConfigError::invalid(
            "ADOC_PUBLIC_ORIGIN",
            "production requires an absolute HTTPS origin",
        ));
    }
    let otel_endpoint = optional_url(source, "ADOC_OTEL_ENDPOINT")?;
    if let Some(endpoint) = &otel_endpoint
        && environment == Environment::Production
        && endpoint.scheme() != "https"
    {
        return Err(ConfigError::invalid(
            "ADOC_OTEL_ENDPOINT",
            "production endpoint must use HTTPS",
        ));
    }
    Ok(CommonConfig {
        environment,
        release_sha: nonempty(required(source, "ADOC_RELEASE_SHA")?, "ADOC_RELEASE_SHA")?
            .to_owned(),
        service,
        http_bind: (service == ServiceKind::Api)
            .then(|| {
                parse_socket(
                    source.get("ADOC_HTTP_BIND").unwrap_or("0.0.0.0:8081"),
                    "ADOC_HTTP_BIND",
                )
            })
            .transpose()?,
        public_origin,
        shutdown_grace: duration_range(
            source.get("ADOC_SHUTDOWN_GRACE").unwrap_or("30s"),
            "ADOC_SHUTDOWN_GRACE",
            Duration::from_secs(5),
            Duration::from_secs(120),
        )?,
        log_level: parse_log_level(source.get("ADOC_LOG_LEVEL").unwrap_or("info"))?,
        otel_endpoint,
    })
}

fn parse_dependencies(
    source: &ConfigSource,
    service: ServiceKind,
    environment: Environment,
) -> Result<DependencyConfig, ConfigError> {
    let database_url = secret_url(
        source,
        "ADOC_DATABASE_URL_FILE",
        environment,
        &["postgres", "postgresql"],
    )?;
    let retention_database_url = (service == ServiceKind::Worker)
        .then(|| {
            secret_url(
                source,
                "ADOC_RETENTION_DATABASE_URL_FILE",
                environment,
                &["postgres", "postgresql"],
            )
        })
        .transpose()?;
    let redis_url = secret_url(
        source,
        "ADOC_REDIS_URL_FILE",
        environment,
        &["redis", "rediss"],
    )?;
    let opensearch_url = parse_url(
        source
            .get("ADOC_OPENSEARCH_URL")
            .unwrap_or("http://127.0.0.1:9200"),
        "ADOC_OPENSEARCH_URL",
    )?;
    if environment == Environment::Production && opensearch_url.scheme() != "https" {
        return Err(ConfigError::invalid(
            "ADOC_OPENSEARCH_URL",
            "production endpoint must use HTTPS",
        ));
    }
    let opensearch_credential =
        optional_secret(source, "ADOC_OPENSEARCH_CREDENTIAL_FILE", environment, 1)?;
    if environment == Environment::Production && opensearch_credential.is_none() {
        return Err(ConfigError::Missing("ADOC_OPENSEARCH_CREDENTIAL_FILE"));
    }
    Ok(DependencyConfig {
        database_url,
        retention_database_url,
        redis_url,
        db_max_connections: integer_range(
            source
                .get("ADOC_DB_MAX_CONNECTIONS")
                .unwrap_or(if service == ServiceKind::Api {
                    "30"
                } else {
                    "20"
                }),
            "ADOC_DB_MAX_CONNECTIONS",
            1,
            200,
        )?,
        queue_namespace: pattern(
            source.get("ADOC_QUEUE_NAMESPACE").unwrap_or("adoc"),
            "ADOC_QUEUE_NAMESPACE",
            |value| {
                !value.is_empty()
                    && value.len() <= 64
                    && value
                        .bytes()
                        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
            },
        )?
        .to_owned(),
        opensearch_url,
        opensearch_credential,
        search_index_prefix: pattern(
            source.get("ADOC_SEARCH_INDEX_PREFIX").unwrap_or("adoc"),
            "ADOC_SEARCH_INDEX_PREFIX",
            |value| {
                !value.is_empty()
                    && value.bytes().all(|byte| {
                        byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-'
                    })
            },
        )?
        .to_owned(),
        embedding_dimension: integer_range(
            source.get("ADOC_EMBEDDING_DIMENSION").unwrap_or("1536"),
            "ADOC_EMBEDDING_DIMENSION",
            1,
            65536,
        )?,
    })
}

fn parse_auth(
    source: &ConfigSource,
    service: ServiceKind,
    environment: Environment,
) -> Result<AuthConfig, ConfigError> {
    let api = service == ServiceKind::Api;
    let session_hmac = api
        .then(|| required_rotating(source, "ADOC_SESSION_HMAC_KEY_FILE", environment))
        .transpose()?;
    let csrf_hmac = api
        .then(|| required_rotating(source, "ADOC_CSRF_HMAC_KEY_FILE", environment))
        .transpose()?;
    if let (Some(session), Some(csrf)) = (&session_hmac, &csrf_hmac)
        && session.current().expose() == csrf.current().expose()
    {
        return Err(ConfigError::invalid(
            "ADOC_CSRF_HMAC_KEY_FILE",
            "CSRF and session signing keys must differ",
        ));
    }
    Ok(AuthConfig {
        google_client_id: api
            .then(|| required_secret(source, "ADOC_GOOGLE_CLIENT_ID_FILE", environment, 1))
            .transpose()?,
        google_client_secret: api
            .then(|| required_secret(source, "ADOC_GOOGLE_CLIENT_SECRET_FILE", environment, 1))
            .transpose()?,
        session_hmac,
        csrf_hmac,
        token_hash_pepper: required_rotating(source, "ADOC_TOKEN_HASH_PEPPER_FILE", environment)?,
        trusted_proxy_cidrs: parse_cidrs(source.get("ADOC_TRUSTED_PROXY_CIDRS").unwrap_or(""))?,
        session_ttl: duration_range(
            source.get("ADOC_SESSION_TTL").unwrap_or("12h"),
            "ADOC_SESSION_TTL",
            Duration::from_secs(60),
            Duration::from_secs(7 * 86_400),
        )?,
        public_link_max_ttl: duration_range(
            source.get("ADOC_PUBLIC_LINK_MAX_TTL").unwrap_or("365d"),
            "ADOC_PUBLIC_LINK_MAX_TTL",
            Duration::from_secs(60),
            Duration::from_secs(3650 * 86_400),
        )?,
    })
}

fn parse_storage(
    source: &ConfigSource,
    environment: Environment,
) -> Result<StorageConfig, ConfigError> {
    let driver = match source.get("ADOC_OBJECT_STORAGE_DRIVER").unwrap_or("local") {
        "local" => ObjectStorageDriver::Local,
        "s3" => ObjectStorageDriver::S3,
        _ => {
            return Err(ConfigError::invalid(
                "ADOC_OBJECT_STORAGE_DRIVER",
                "expected local or s3",
            ));
        }
    };
    let local_root = source.get("ADOC_LOCAL_OBJECT_ROOT").map(PathBuf::from);
    if driver == ObjectStorageDriver::Local
        && local_root.as_ref().is_none_or(|path| !path.is_absolute())
    {
        return Err(ConfigError::invalid(
            "ADOC_LOCAL_OBJECT_ROOT",
            "local driver requires an absolute path",
        ));
    }
    let s3_bucket = source.get("ADOC_S3_BUCKET").map(str::to_owned);
    let s3_region = source.get("ADOC_S3_REGION").map(str::to_owned);
    if driver == ObjectStorageDriver::S3 && (s3_bucket.is_none() || s3_region.is_none()) {
        return Err(ConfigError::invalid(
            "ADOC_OBJECT_STORAGE_DRIVER",
            "s3 driver requires bucket and region",
        ));
    }
    Ok(StorageConfig {
        driver,
        local_root,
        s3_bucket,
        s3_region,
        s3_endpoint: optional_url(source, "ADOC_S3_ENDPOINT")?,
        s3_credential: optional_secret(source, "ADOC_S3_CREDENTIAL_FILE", environment, 1)?,
        upload_max_bytes: integer_range_u64(
            source.get("ADOC_UPLOAD_MAX_BYTES").unwrap_or("104857600"),
            "ADOC_UPLOAD_MAX_BYTES",
            1_048_576,
            5 * 1024 * 1024 * 1024,
        )?,
        allowed_mime_file: absolute_path(
            required(source, "ADOC_ALLOWED_MIME_FILE")?,
            "ADOC_ALLOWED_MIME_FILE",
        )?,
    })
}

fn parse_ai(source: &ConfigSource, environment: Environment) -> Result<AiConfig, ConfigError> {
    let driver = match source.get("ADOC_AI_DRIVER").unwrap_or("codex_cli") {
        "codex_cli" => AiDriver::CodexCli,
        "openai_responses" => AiDriver::OpenAiResponses,
        _ => {
            return Err(ConfigError::invalid(
                "ADOC_AI_DRIVER",
                "expected codex_cli or openai_responses",
            ));
        }
    };
    let codex_executable = source
        .get("ADOC_CODEX_EXECUTABLE")
        .map(|value| absolute_path(value, "ADOC_CODEX_EXECUTABLE"))
        .transpose()?;
    if environment == Environment::Production
        && driver == AiDriver::CodexCli
        && codex_executable.as_ref().is_some_and(|path| {
            path != std::path::Path::new("/usr/local/bin/codex")
                && path != std::path::Path::new("/opt/adoc/bin/codex")
        })
    {
        return Err(ConfigError::invalid(
            "ADOC_CODEX_EXECUTABLE",
            "production executable is not allowlisted",
        ));
    }
    let openai_api_key = optional_secret(source, "ADOC_OPENAI_API_KEY_FILE", environment, 1)?;
    match driver {
        AiDriver::CodexCli if codex_executable.is_none() => {
            return Err(ConfigError::Missing("ADOC_CODEX_EXECUTABLE"));
        }
        AiDriver::OpenAiResponses if openai_api_key.is_none() => {
            return Err(ConfigError::Missing("ADOC_OPENAI_API_KEY_FILE"));
        }
        _ => {}
    }
    Ok(AiConfig {
        driver,
        codex_executable,
        openai_api_key,
        request_timeout: duration_range(
            source.get("ADOC_AI_REQUEST_TIMEOUT").unwrap_or("180s"),
            "ADOC_AI_REQUEST_TIMEOUT",
            Duration::from_secs(10),
            Duration::from_secs(600),
        )?,
        kill_grace: duration_range(
            source.get("ADOC_AI_KILL_GRACE").unwrap_or("5s"),
            "ADOC_AI_KILL_GRACE",
            Duration::from_secs(1),
            Duration::from_secs(30),
        )?,
        max_context_tokens: integer_range(
            source.get("ADOC_AI_MAX_CONTEXT_TOKENS").unwrap_or("128000"),
            "ADOC_AI_MAX_CONTEXT_TOKENS",
            1,
            2_000_000,
        )?,
    })
}

fn parse_worker(
    source: &ConfigSource,
    environment: Environment,
) -> Result<WorkerConfig, ConfigError> {
    let trash = duration_range(
        source.get("ADOC_TRASH_RETENTION").unwrap_or("30d"),
        "ADOC_TRASH_RETENTION",
        Duration::from_secs(86_400),
        Duration::from_secs(3650 * 86_400),
    )?;
    let workspace = duration_range(
        source.get("ADOC_WORKSPACE_RETENTION").unwrap_or("30d"),
        "ADOC_WORKSPACE_RETENTION",
        Duration::from_secs(86_400),
        Duration::from_secs(3650 * 86_400),
    )?;
    if environment == Environment::Production
        && (trash != Duration::from_secs(30 * 86_400)
            || workspace != Duration::from_secs(30 * 86_400))
    {
        return Err(ConfigError::invalid(
            "ADOC_TRASH_RETENTION",
            "production retention policy is fixed at 30d",
        ));
    }
    Ok(WorkerConfig {
        trash_retention: trash,
        workspace_retention: workspace,
        job_lease: duration_range(
            source.get("ADOC_JOB_LEASE").unwrap_or("60s"),
            "ADOC_JOB_LEASE",
            Duration::from_secs(5),
            Duration::from_secs(600),
        )?,
        job_max_attempts: integer_range(
            source.get("ADOC_JOB_MAX_ATTEMPTS").unwrap_or("5"),
            "ADOC_JOB_MAX_ATTEMPTS",
            1,
            100,
        )?,
        outbox_batch_size: integer_range(
            source.get("ADOC_OUTBOX_BATCH_SIZE").unwrap_or("100"),
            "ADOC_OUTBOX_BATCH_SIZE",
            1,
            1000,
        )?,
        reconcile_interval: duration_range(
            source.get("ADOC_RECONCILE_INTERVAL").unwrap_or("30s"),
            "ADOC_RECONCILE_INTERVAL",
            Duration::from_secs(5),
            Duration::from_secs(600),
        )?,
    })
}

fn required<'a>(source: &'a ConfigSource, key: &'static str) -> Result<&'a str, ConfigError> {
    source.get(key).ok_or(ConfigError::Missing(key))
}
fn nonempty<'a>(value: &'a str, key: &'static str) -> Result<&'a str, ConfigError> {
    if value.trim().is_empty() {
        Err(ConfigError::invalid(key, "must not be empty"))
    } else {
        Ok(value)
    }
}
fn parse_environment(value: &str) -> Result<Environment, ConfigError> {
    match value {
        "development" => Ok(Environment::Development),
        "test" => Ok(Environment::Test),
        "staging" => Ok(Environment::Staging),
        "production" => Ok(Environment::Production),
        _ => Err(ConfigError::invalid(
            "ADOC_ENV",
            "expected development, test, staging, or production",
        )),
    }
}
fn parse_log_level(value: &str) -> Result<LogLevel, ConfigError> {
    match value {
        "trace" => Ok(LogLevel::Trace),
        "debug" => Ok(LogLevel::Debug),
        "info" => Ok(LogLevel::Info),
        "warn" => Ok(LogLevel::Warn),
        "error" => Ok(LogLevel::Error),
        _ => Err(ConfigError::invalid(
            "ADOC_LOG_LEVEL",
            "expected trace, debug, info, warn, or error",
        )),
    }
}
fn parse_socket(value: &str, key: &'static str) -> Result<SocketAddr, ConfigError> {
    value
        .parse()
        .map_err(|_| ConfigError::invalid(key, "expected IP socket address"))
}
fn parse_url(value: &str, key: &'static str) -> Result<Url, ConfigError> {
    Url::parse(value)
        .map_err(|_| ConfigError::invalid(key, "expected absolute URL"))
        .and_then(|url| {
            if url.has_host() {
                Ok(url)
            } else {
                Err(ConfigError::invalid(key, "expected absolute URL"))
            }
        })
}
fn optional_url(source: &ConfigSource, key: &'static str) -> Result<Option<Url>, ConfigError> {
    source
        .get(key)
        .map(|value| parse_url(value, key))
        .transpose()
}
fn duration_range(
    value: &str,
    key: &'static str,
    minimum: Duration,
    maximum: Duration,
) -> Result<Duration, ConfigError> {
    let parsed = humantime::parse_duration(value)
        .map_err(|_| ConfigError::invalid(key, "expected duration"))?;
    if (minimum..=maximum).contains(&parsed) {
        Ok(parsed)
    } else {
        Err(ConfigError::invalid(
            key,
            format!(
                "duration must be between {} and {}",
                humantime::format_duration(minimum),
                humantime::format_duration(maximum)
            ),
        ))
    }
}
fn integer_range(
    value: &str,
    key: &'static str,
    minimum: u32,
    maximum: u32,
) -> Result<u32, ConfigError> {
    value
        .parse::<u32>()
        .ok()
        .filter(|number| (minimum..=maximum).contains(number))
        .ok_or_else(|| {
            ConfigError::invalid(
                key,
                format!("integer must be between {minimum} and {maximum}"),
            )
        })
}
fn integer_range_u64(
    value: &str,
    key: &'static str,
    minimum: u64,
    maximum: u64,
) -> Result<u64, ConfigError> {
    value
        .parse::<u64>()
        .ok()
        .filter(|number| (minimum..=maximum).contains(number))
        .ok_or_else(|| {
            ConfigError::invalid(
                key,
                format!("integer must be between {minimum} and {maximum}"),
            )
        })
}
fn pattern<'a>(
    value: &'a str,
    key: &'static str,
    predicate: impl FnOnce(&str) -> bool,
) -> Result<&'a str, ConfigError> {
    if predicate(value) {
        Ok(value)
    } else {
        Err(ConfigError::invalid(
            key,
            "value does not match the required pattern",
        ))
    }
}
fn parse_cidrs(value: &str) -> Result<Vec<IpNet>, ConfigError> {
    if value.trim().is_empty() {
        return Ok(Vec::new());
    }
    value
        .split(',')
        .map(|item| {
            IpNet::from_str(item.trim()).map_err(|_| {
                ConfigError::invalid(
                    "ADOC_TRUSTED_PROXY_CIDRS",
                    "expected comma-separated CIDR list",
                )
            })
        })
        .collect()
}
fn absolute_path(value: &str, key: &'static str) -> Result<PathBuf, ConfigError> {
    let path = PathBuf::from(value);
    if path.is_absolute() {
        Ok(path)
    } else {
        Err(ConfigError::invalid(key, "expected absolute path"))
    }
}
fn required_secret(
    source: &ConfigSource,
    key: &'static str,
    environment: Environment,
    minimum: usize,
) -> Result<LoadedSecret, ConfigError> {
    load_secret(
        &absolute_path(required(source, key)?, key)?,
        key,
        environment,
        minimum,
    )
}
fn optional_secret(
    source: &ConfigSource,
    key: &'static str,
    environment: Environment,
    minimum: usize,
) -> Result<Option<LoadedSecret>, ConfigError> {
    source
        .get(key)
        .map(|value| {
            absolute_path(value, key).and_then(|path| load_secret(&path, key, environment, minimum))
        })
        .transpose()
}
fn required_rotating(
    source: &ConfigSource,
    key: &'static str,
    environment: Environment,
) -> Result<RotatingSecret, ConfigError> {
    crate::secret::load_rotating_secret(
        &absolute_path(required(source, key)?, key)?,
        key,
        environment,
    )
}
fn secret_url(
    source: &ConfigSource,
    key: &'static str,
    environment: Environment,
    schemes: &[&str],
) -> Result<LoadedSecret, ConfigError> {
    let secret = required_secret(source, key, environment, 1)?;
    let url = parse_url(secret.value.expose(), key)?;
    if !schemes.contains(&url.scheme()) {
        Err(ConfigError::invalid(
            key,
            "secret URL uses a forbidden scheme",
        ))
    } else if environment == Environment::Production
        && key == "ADOC_REDIS_URL_FILE"
        && url.scheme() != "rediss"
    {
        Err(ConfigError::invalid(
            key,
            "production Redis URL must use TLS",
        ))
    } else if environment == Environment::Production
        && matches!(
            key,
            "ADOC_DATABASE_URL_FILE" | "ADOC_RETENTION_DATABASE_URL_FILE"
        )
        && !url.query_pairs().any(|(name, value)| {
            name == "sslmode" && matches!(value.as_ref(), "require" | "verify-ca" | "verify-full")
        })
    {
        Err(ConfigError::invalid(
            key,
            "production PostgreSQL URL must require TLS",
        ))
    } else {
        Ok(secret)
    }
}
fn os_string(value: OsString) -> Result<String, ConfigError> {
    value.into_string().map_err(|_| ConfigError::NonUtf8)
}
