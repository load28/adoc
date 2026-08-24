use std::{fs, path::Path, time::Duration};

use adoc_configuration::{
    AiDriver, AppConfig, ConfigError, ConfigSource, Environment, ObjectStorageDriver, ServiceKind,
};
use tempfile::TempDir;

struct Fixture {
    _directory: TempDir,
    source: ConfigSource,
    database_path: String,
}

impl Fixture {
    fn new() -> Self {
        let directory = tempfile::tempdir().unwrap();
        let database = secret(
            directory.path(),
            "database",
            "postgresql://adoc:pass@localhost/adoc",
        );
        let retention = secret(
            directory.path(),
            "retention",
            "postgresql://retention:pass@localhost/adoc",
        );
        let redis = secret(directory.path(), "redis", "redis://localhost:6379/0");
        let google_id = secret(directory.path(), "google-id", "client-id");
        let google_secret = secret(directory.path(), "google-secret", "client-secret");
        let rotating = rotation(directory.path(), "rotation", "current-key", None);
        let session = rotation(
            directory.path(),
            "session",
            "session-current",
            Some("session-old"),
        );
        let csrf = rotation(directory.path(), "csrf", "csrf-current", None);
        let source = ConfigSource::default()
            .with("ADOC_ENV", "test")
            .with("ADOC_RELEASE_SHA", "release-abc123")
            .with("ADOC_DATABASE_URL_FILE", path(&database))
            .with("ADOC_RETENTION_DATABASE_URL_FILE", path(&retention))
            .with("ADOC_REDIS_URL_FILE", path(&redis))
            .with("ADOC_GOOGLE_CLIENT_ID_FILE", path(&google_id))
            .with("ADOC_GOOGLE_CLIENT_SECRET_FILE", path(&google_secret))
            .with("ADOC_SESSION_HMAC_KEY_FILE", path(&session))
            .with("ADOC_CSRF_HMAC_KEY_FILE", path(&csrf))
            .with("ADOC_TOKEN_HASH_PEPPER_FILE", path(&rotating))
            .with("ADOC_LOCAL_OBJECT_ROOT", path(directory.path()))
            .with(
                "ADOC_ALLOWED_MIME_FILE",
                path(&secret(directory.path(), "mime", "image/png")),
            )
            .with("ADOC_CODEX_EXECUTABLE", "/usr/bin/true");
        Self {
            _directory: directory,
            source,
            database_path: path(&database),
        }
    }
}

#[test]
fn parses_api_and_worker_config_from_the_same_catalog() {
    let fixture = Fixture::new();
    let api = AppConfig::parse(&fixture.source, ServiceKind::Api).unwrap();
    assert_eq!(api.common.environment, Environment::Test);
    assert_eq!(api.storage.driver, ObjectStorageDriver::Local);
    assert_eq!(api.ai.driver, AiDriver::CodexCli);
    assert!(api.worker.is_none());
    assert_eq!(
        api.auth.session_hmac.as_ref().unwrap().current_id(),
        "session-current"
    );
    assert!(api.auth.session_hmac.as_ref().unwrap().previous().is_some());

    let worker = AppConfig::parse(&fixture.source, ServiceKind::Worker).unwrap();
    assert_eq!(
        worker.worker.as_ref().unwrap().trash_retention,
        Duration::from_secs(30 * 86_400)
    );
    assert!(worker.dependencies.retention_database_url.is_some());
}

#[test]
fn negative_corpus_rejects_unknown_plain_and_invalid_values() {
    let cases = [
        ("ADOC_UNKNOWN", "value"),
        ("ADOC_DATABASE_URL", "postgresql://plain-secret"),
        ("ADOC_ENV", "invalid"),
        ("ADOC_SHUTDOWN_GRACE", "4s"),
        ("ADOC_PUBLIC_ORIGIN", "relative/path"),
        ("ADOC_DB_MAX_CONNECTIONS", "0"),
        ("ADOC_SEARCH_INDEX_PREFIX", "Upper_Case"),
        ("ADOC_TRUSTED_PROXY_CIDRS", "not-cidr"),
        ("ADOC_LOCAL_OBJECT_ROOT", "relative"),
        ("ADOC_UPLOAD_MAX_BYTES", "1"),
        ("ADOC_AI_REQUEST_TIMEOUT", "9s"),
        ("ADOC_AI_MAX_CONTEXT_TOKENS", "0"),
    ];
    for (key, value) in cases {
        let fixture = Fixture::new();
        assert!(
            AppConfig::parse(&fixture.source.with(key, value), ServiceKind::Api).is_err(),
            "{key} should be rejected",
        );
    }
}

#[test]
fn rejects_missing_and_driver_conditional_keys() {
    let fixture = Fixture::new();
    assert!(matches!(
        AppConfig::parse(
            &fixture.source.clone().without("ADOC_RELEASE_SHA"),
            ServiceKind::Api
        ),
        Err(ConfigError::Missing("ADOC_RELEASE_SHA")),
    ));
    assert!(
        AppConfig::parse(
            &fixture
                .source
                .clone()
                .with("ADOC_OBJECT_STORAGE_DRIVER", "s3")
                .without("ADOC_LOCAL_OBJECT_ROOT"),
            ServiceKind::Api,
        )
        .is_err()
    );
    assert!(
        AppConfig::parse(
            &fixture
                .source
                .clone()
                .with("ADOC_AI_DRIVER", "openai_responses")
                .without("ADOC_CODEX_EXECUTABLE"),
            ServiceKind::Api,
        )
        .is_err()
    );
}

#[test]
fn production_rejects_insecure_secret_permissions_and_policy_drift() {
    let fixture = Fixture::new();
    let production = fixture
        .source
        .clone()
        .with("ADOC_ENV", "production")
        .with("ADOC_PUBLIC_ORIGIN", "https://docs.example.com")
        .with(
            "ADOC_OPENSEARCH_CREDENTIAL_FILE",
            fixture.database_path.clone(),
        );
    let error = AppConfig::parse(&production, ServiceKind::Api)
        .unwrap_err()
        .to_string();
    assert!(error.contains("must not be group/world accessible"));
    assert!(!error.contains("postgresql://"));
    assert!(!error.contains(&fixture.database_path));

    secure_permissions(fixture._directory.path());
    fs::write(
        &fixture.database_path,
        "postgresql://adoc:pass@localhost/adoc?sslmode=require",
    )
    .unwrap();
    let redis_path = fixture._directory.path().join("redis");
    let retention_path = fixture._directory.path().join("retention");
    fs::write(
        &retention_path,
        "postgresql://retention:pass@localhost/adoc?sslmode=require",
    )
    .unwrap();
    fs::write(&redis_path, "rediss://localhost:6379/0").unwrap();
    let policy = production
        .with("ADOC_OPENSEARCH_URL", "https://localhost:9200")
        .with("ADOC_CODEX_EXECUTABLE", "/usr/local/bin/codex")
        .with("ADOC_TRASH_RETENTION", "31d");
    let error = AppConfig::parse(&policy, ServiceKind::Worker)
        .unwrap_err()
        .to_string();
    assert!(error.contains("retention policy is fixed"));
}

#[test]
fn secret_values_and_preflight_never_expose_content_or_paths() {
    let fixture = Fixture::new();
    let config = AppConfig::parse(&fixture.source, ServiceKind::Api).unwrap();
    let debug = format!("{config:?}");
    let preflight = config.preflight_json().to_string();
    for forbidden in ["postgresql://", "client-secret", &fixture.database_path] {
        assert!(!debug.contains(forbidden));
        assert!(!preflight.contains(forbidden));
    }
    assert!(preflight.contains("current-key"));
    assert!(preflight.contains("ADOC_DATABASE_URL_FILE"));
}

#[test]
fn rotation_contract_rejects_duplicate_ids_and_unknown_fields() {
    let directory = tempfile::tempdir().unwrap();
    let duplicate = directory.path().join("duplicate.json");
    fs::write(
        &duplicate,
        serde_json::json!({
            "current": {"id": "same", "value": "a".repeat(32)},
            "previous": {"id": "same", "value": "b".repeat(32)},
        })
        .to_string(),
    )
    .unwrap();
    let fixture = Fixture::new();
    assert!(
        AppConfig::parse(
            &fixture
                .source
                .with("ADOC_TOKEN_HASH_PEPPER_FILE", path(&duplicate)),
            ServiceKind::Api,
        )
        .is_err()
    );
}

fn secret(directory: &Path, name: &str, value: &str) -> std::path::PathBuf {
    let path = directory.join(name);
    fs::write(&path, value).unwrap();
    path
}

fn rotation(
    directory: &Path,
    name: &str,
    current: &str,
    previous: Option<&str>,
) -> std::path::PathBuf {
    let path = directory.join(format!("{name}.json"));
    let previous = previous.map(|id| serde_json::json!({"id": id, "value": id.repeat(4)}));
    fs::write(
        &path,
        serde_json::json!({
            "current": {"id": current, "value": current.repeat(4)},
            "previous": previous,
        })
        .to_string(),
    )
    .unwrap();
    path
}

fn path(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

#[cfg(unix)]
fn secure_permissions(directory: &Path) {
    use std::os::unix::fs::PermissionsExt;

    for entry in fs::read_dir(directory).unwrap() {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_file() {
            fs::set_permissions(entry.path(), fs::Permissions::from_mode(0o600)).unwrap();
        }
    }
}

#[cfg(not(unix))]
fn secure_permissions(_directory: &Path) {}
