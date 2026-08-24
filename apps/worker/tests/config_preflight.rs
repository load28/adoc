use std::{fs, path::Path, process::Command};

#[test]
fn check_config_reports_worker_metadata_without_secret_material() {
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
    let pepper = rotating(directory.path(), "pepper", "pepper-current");
    let mime = secret(directory.path(), "mime", "image/png");

    let output = Command::new(env!("CARGO_BIN_EXE_adoc-worker"))
        .arg("--check-config")
        .env_clear()
        .env("ADOC_ENV", "test")
        .env("ADOC_RELEASE_SHA", "test-release")
        .env("ADOC_DATABASE_URL_FILE", &database)
        .env("ADOC_RETENTION_DATABASE_URL_FILE", &retention)
        .env("ADOC_REDIS_URL_FILE", &redis)
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
    assert_eq!(report["service"], "worker");
    assert_eq!(report["status"], "valid");
    assert!(stdout.contains("pepper-current"));
    for forbidden in ["postgresql://", "pass@", &path(&retention)] {
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
