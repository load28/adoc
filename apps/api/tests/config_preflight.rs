use std::{fs, path::Path, process::Command};

#[test]
fn check_config_reports_metadata_without_secret_material() {
    let directory = tempfile::tempdir().unwrap();
    let database = secret(
        directory.path(),
        "database",
        "postgresql://adoc:pass@localhost/adoc",
    );
    let redis = secret(directory.path(), "redis", "redis://localhost:6379/0");
    let google_id = secret(directory.path(), "google-id", "google-client-id");
    let google_secret = secret(directory.path(), "google-secret", "google-client-secret");
    let session = rotating(directory.path(), "session", "session-current");
    let csrf = rotating(directory.path(), "csrf", "csrf-current");
    let pepper = rotating(directory.path(), "pepper", "pepper-current");
    let mime = secret(directory.path(), "mime", "image/png");

    let output = Command::new(env!("CARGO_BIN_EXE_adoc-api"))
        .arg("--check-config")
        .env_clear()
        .env("ADOC_ENV", "test")
        .env("ADOC_RELEASE_SHA", "test-release")
        .env("ADOC_DATABASE_URL_FILE", &database)
        .env("ADOC_REDIS_URL_FILE", &redis)
        .env("ADOC_GOOGLE_CLIENT_ID_FILE", &google_id)
        .env("ADOC_GOOGLE_CLIENT_SECRET_FILE", &google_secret)
        .env("ADOC_SESSION_HMAC_KEY_FILE", &session)
        .env("ADOC_CSRF_HMAC_KEY_FILE", &csrf)
        .env("ADOC_TOKEN_HASH_PEPPER_FILE", &pepper)
        .env("ADOC_LOCAL_OBJECT_ROOT", directory.path())
        .env("ADOC_ALLOWED_MIME_FILE", &mime)
        .env("ADOC_CODEX_EXECUTABLE", "/usr/bin/true")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    let report: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(report["service"], "api");
    assert_eq!(report["status"], "valid");
    assert!(stdout.contains("session-current"));
    for forbidden in ["postgresql://", "google-client-secret", &path(&database)] {
        assert!(!stdout.contains(forbidden));
    }
}

fn secret(directory: &Path, name: &str, value: &str) -> std::path::PathBuf {
    let path = directory.join(name);
    fs::write(&path, value).unwrap();
    path
}

fn rotating(directory: &Path, name: &str, id: &str) -> std::path::PathBuf {
    secret(
        directory,
        name,
        &serde_json::json!({"current": {"id": id, "value": id.repeat(4)}}).to_string(),
    )
}

fn path(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}
