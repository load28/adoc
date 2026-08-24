use std::{env, fs, sync::Arc};

use adoc_adapters::{
    identity::SystemClock,
    object_storage::{EicarMalwareScanner, LocalObjectStorage},
    postgres::{DatabaseSettings, PostgresFileRepository, PostgresStore},
};
use adoc_application::{
    governance::GovernanceError,
    identity::{KeyRing, SigningKey},
    operations::{
        ByteRange, ByteStream, CompleteFileUploadInput, CreateFileUploadInput,
        FileGarbageCollector, FileService, FileStatus, ObjectStorage,
    },
};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use futures_util::{StreamExt, stream};
use sha2::{Digest, Sha256};
use uuid::Uuid;

#[tokio::test]
#[ignore = "requires ADOC_TEST_DATABASE_URL pointing to an isolated PostgreSQL 16 database"]
async fn file_storage_postgres_contract() {
    let store = PostgresStore::connect(DatabaseSettings {
        url: &secret("ADOC_TEST_DATABASE_URL", "ADOC_TEST_DATABASE_URL_FILE"),
        max_connections: 8,
        application_name: "adoc-file-contract",
    })
    .await
    .unwrap();
    store.migrate().await.unwrap();
    let owner = Uuid::now_v7();
    let workspace = Uuid::now_v7();
    seed(&store, owner, workspace).await;
    let temporary = env::temp_dir().join(format!("adoc-file-contract-{}", Uuid::now_v7()));
    fs::create_dir_all(&temporary).unwrap();
    let storage = Arc::new(LocalObjectStorage::new(temporary.clone()).unwrap());
    let repository = Arc::new(PostgresFileRepository::new(&store));
    let service = FileService::new(
        repository.clone(),
        storage.clone(),
        Arc::new(EicarMalwareScanner),
        Arc::new(SystemClock),
        Arc::new(keys()),
        1024 * 1024,
        Arc::from("http://localhost/api/v1"),
    );

    let png = hex::decode("89504e470d0a1a0a0000000d49484452000000010000000108060000001f15c4890000000d49444154789c63600000020001e221bc330000000049454e44ae426082").unwrap();
    let checksum = digest(&png);
    let create = CreateFileUploadInput {
        name: "avatar.png".into(),
        mime_type: "image/png".into(),
        size: png.len() as u64,
        checksum: checksum.clone(),
    };
    let ticket = service
        .create_upload(owner, workspace, "file-upload-create-0001", create.clone())
        .await
        .unwrap();
    let replay = service
        .create_upload(owner, workspace, "file-upload-create-0001", create)
        .await
        .unwrap();
    assert_eq!(ticket.asset_id, replay.asset_id);
    assert_eq!(ticket.upload_token, replay.upload_token);
    assert!(matches!(
        service
            .upload(
                owner,
                workspace,
                ticket.asset_id,
                "invalid",
                png.len() as u64,
                bytes(png.clone())
            )
            .await,
        Err(GovernanceError::UploadTokenInvalid)
    ));
    service
        .upload(
            owner,
            workspace,
            ticket.asset_id,
            &ticket.upload_token,
            png.len() as u64,
            bytes(png.clone()),
        )
        .await
        .unwrap();
    let ready = service
        .complete(
            owner,
            workspace,
            ticket.asset_id,
            0,
            "file-upload-complete-001",
            CompleteFileUploadInput {
                checksum_sha256: checksum,
                size_bytes: png.len() as u64,
            },
        )
        .await
        .unwrap();
    assert_eq!(ready.status, FileStatus::Ready);
    assert_eq!(ready.revision, 2);
    let (_, mut range) = service
        .download(
            owner,
            workspace,
            ticket.asset_id,
            Some(ByteRange {
                start: 0,
                end_inclusive: 7,
            }),
        )
        .await
        .unwrap();
    assert_eq!(range.next().await.unwrap().unwrap().as_ref(), &png[..8]);

    let malware = b"X5O!P%@AP[4\\PZX54(P^)7CC)7}$EICAR-STANDARD-ANTIVIRUS-TEST-FILE!$H+H*".to_vec();
    let malware_ticket = service
        .create_upload(
            owner,
            workspace,
            "file-malware-create-001",
            CreateFileUploadInput {
                name: "sample.txt".into(),
                mime_type: "text/plain".into(),
                size: malware.len() as u64,
                checksum: digest(&malware),
            },
        )
        .await
        .unwrap();
    service
        .upload(
            owner,
            workspace,
            malware_ticket.asset_id,
            &malware_ticket.upload_token,
            malware.len() as u64,
            bytes(malware.clone()),
        )
        .await
        .unwrap();
    let failed = service
        .complete(
            owner,
            workspace,
            malware_ticket.asset_id,
            0,
            "file-malware-complete-01",
            CompleteFileUploadInput {
                checksum_sha256: digest(&malware),
                size_bytes: malware.len() as u64,
            },
        )
        .await
        .unwrap();
    assert_eq!(failed.status, FileStatus::Failed);
    assert_eq!(failed.failure_code.as_deref(), Some("MALWARE_DETECTED"));

    let document = Uuid::now_v7();
    let version = Uuid::now_v7();
    sqlx::query("INSERT INTO documents(id,workspace_id,rank,title,created_by) VALUES($1,$2,$3,'File document',$4)")
        .bind(document).bind(workspace).bind("0".repeat(32)).bind(owner).execute(store.pool()).await.unwrap();
    sqlx::query("INSERT INTO published_versions(id,workspace_id,document_id,number,content_json,schema_version,content_fingerprint,source_draft_revision,publisher_id,summary) VALUES($1,$2,$3,1,$4,1,$5,0,$6,'File version')")
        .bind(version).bind(workspace).bind(document).bind(serde_json::json!({"schemaVersion":1,"root":{"type":"doc","children":[]}})).bind(digest(b"version")).bind(owner).execute(store.pool()).await.unwrap();
    sqlx::query("UPDATE documents SET current_version_id=$2 WHERE id=$1")
        .bind(document)
        .bind(version)
        .execute(store.pool())
        .await
        .unwrap();
    sqlx::query("INSERT INTO file_references(workspace_id,asset_id,owner_kind,owner_id) VALUES($1,$2,'PUBLISHED_VERSION',$3)")
        .bind(workspace).bind(ticket.asset_id).bind(version).execute(store.pool()).await.unwrap();
    let public_token = URL_SAFE_NO_PAD.encode([9_u8; 32]);
    sqlx::query("INSERT INTO public_links(id,workspace_id,document_id,token_hash,created_by) VALUES($1,$2,$3,$4,$5)")
        .bind(Uuid::now_v7()).bind(workspace).bind(document).bind(Sha256::digest(public_token.as_bytes()).to_vec()).bind(owner).execute(store.pool()).await.unwrap();
    let (_, mut public_range, parsed_range) = service
        .public_download(&public_token, ticket.asset_id, Some("bytes=0-7"))
        .await
        .unwrap();
    assert_eq!(parsed_range.unwrap().len(), 8);
    assert_eq!(
        public_range.next().await.unwrap().unwrap().as_ref(),
        &png[..8]
    );
    assert!(matches!(
        service
            .public_download(&public_token, Uuid::now_v7(), None)
            .await,
        Err(GovernanceError::PublicLinkInvalid)
    ));
    assert!(matches!(
        service
            .delete(
                owner,
                workspace,
                ticket.asset_id,
                ready.revision,
                "file-delete-in-use-001"
            )
            .await,
        Err(GovernanceError::FileInUse { reference_count: 1 })
    ));
    sqlx::query("DELETE FROM file_references WHERE asset_id=$1")
        .bind(ticket.asset_id)
        .execute(store.pool())
        .await
        .unwrap();

    service
        .delete(
            owner,
            workspace,
            ticket.asset_id,
            ready.revision,
            "file-delete-ready-0001",
        )
        .await
        .unwrap();
    sqlx::query("UPDATE file_assets SET purge_after=now() WHERE id=$1")
        .bind(ticket.asset_id)
        .execute(store.pool())
        .await
        .unwrap();
    let gc = FileGarbageCollector::new(repository, storage.clone(), Arc::new(SystemClock));
    assert!(gc.run_once(100).await.unwrap() >= 1);
    let key: String = sqlx::query_scalar("SELECT storage_key FROM file_assets WHERE id=$1")
        .bind(ticket.asset_id)
        .fetch_one(store.pool())
        .await
        .unwrap();
    assert!(storage.stat(&key).await.is_err());
    assert!(
        sqlx::query_scalar::<_, Option<chrono::DateTime<chrono::Utc>>>(
            "SELECT byte_deleted_at FROM file_assets WHERE id=$1"
        )
        .bind(ticket.asset_id)
        .fetch_one(store.pool())
        .await
        .unwrap()
        .is_some()
    );
    fs::remove_dir_all(temporary).unwrap();
    store.close().await;
}

fn bytes(value: Vec<u8>) -> ByteStream {
    Box::pin(stream::once(async move { Ok(value.into()) }))
}
fn digest(value: &[u8]) -> String {
    hex::encode(Sha256::digest(value))
}
fn keys() -> KeyRing {
    KeyRing::new(
        SigningKey {
            id: "test-key".into(),
            value: Arc::from([7_u8; 32].as_slice()),
        },
        None,
    )
    .unwrap()
}
fn secret(value_key: &str, file_key: &str) -> String {
    if let Ok(value) = env::var(value_key) {
        return value;
    }
    let path =
        env::var(file_key).unwrap_or_else(|_| panic!("{value_key} or {file_key} is required"));
    fs::read_to_string(path).unwrap().trim().to_owned()
}
async fn seed(store: &PostgresStore, owner: Uuid, workspace: Uuid) {
    sqlx::query("INSERT INTO users(id,google_subject,email,display_name) VALUES($1,$2,$3,'Owner')")
        .bind(owner)
        .bind(owner.to_string())
        .bind(format!("{owner}@example.test"))
        .execute(store.pool())
        .await
        .unwrap();
    sqlx::query("INSERT INTO workspaces(id,slug,name,created_by) VALUES($1,$2,'File Test',$3)")
        .bind(workspace)
        .bind(format!("file-{workspace}"))
        .bind(owner)
        .execute(store.pool())
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO memberships(workspace_id,user_id,role,status) VALUES($1,$2,'OWNER','ACTIVE')",
    )
    .bind(workspace)
    .bind(owner)
    .execute(store.pool())
    .await
    .unwrap();
}
