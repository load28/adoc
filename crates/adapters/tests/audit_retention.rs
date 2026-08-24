use std::{
    env, fs,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

use adoc_adapters::{
    identity::SystemClock,
    object_storage::LocalObjectStorage,
    postgres::{
        DatabaseSettings, PostgresAuditRepository, PostgresRetentionRepository, PostgresStore,
        append_audit_event,
    },
};
use adoc_application::{
    governance::GovernanceError,
    operations::{
        AuditAction, AuditEventInput, AuditFields, AuditRepository, AuditTarget, AuditTargetKind,
        AuditValue, ByteRange, ByteStream, ObjectMetadata, ObjectStorage, RetentionService,
        StorageError,
    },
};
use bytes::Bytes;
use futures_util::stream;
use sqlx::Row;
use uuid::Uuid;

#[tokio::test]
#[ignore = "requires ADOC_TEST_DATABASE_URL pointing to an isolated PostgreSQL 16 database"]
async fn audit_retention_postgres_contract() {
    let store = PostgresStore::connect(DatabaseSettings {
        url: &secret("ADOC_TEST_DATABASE_URL", "ADOC_TEST_DATABASE_URL_FILE"),
        max_connections: 12,
        application_name: "adoc-audit-retention-contract",
    })
    .await
    .unwrap();
    store.migrate().await.unwrap();
    let owner = Uuid::now_v7();
    let outsider = Uuid::now_v7();
    let workspace = Uuid::now_v7();
    seed_user(&store, owner, "audit-owner").await;
    seed_user(&store, outsider, "audit-outsider").await;
    seed_workspace(&store, workspace, owner, "audit-main").await;

    let mut tasks = Vec::new();
    for index in 0..12 {
        let pool = store.pool().clone();
        tasks.push(tokio::spawn(async move {
            let mut tx = pool.begin().await.unwrap();
            append_audit_event(
                &mut tx,
                AuditEventInput::user(
                    workspace,
                    owner,
                    AuditAction::WorkspaceUpdated,
                    AuditTarget {
                        kind: AuditTargetKind::Workspace,
                        id: workspace,
                    },
                    chrono::Utc::now(),
                    format!("audit-concurrent-{index:02}"),
                ),
            )
            .await
            .unwrap();
            tx.commit().await.unwrap();
        }));
    }
    for task in tasks {
        task.await.unwrap();
    }
    let sequences = sqlx::query_scalar::<_, i64>(
        "SELECT sequence FROM audit_events WHERE workspace_id=$1 ORDER BY sequence",
    )
    .bind(workspace)
    .fetch_all(store.pool())
    .await
    .unwrap();
    assert_eq!(sequences, (1..=12).collect::<Vec<_>>());
    assert!(
        sqlx::query("DELETE FROM audit_events WHERE workspace_id=$1")
            .bind(workspace)
            .execute(store.pool())
            .await
            .is_err()
    );
    let audit = PostgresAuditRepository::new(&store);
    assert_eq!(
        audit
            .list(owner, workspace, None)
            .await
            .unwrap()
            .items
            .len(),
        12
    );
    assert!(matches!(
        audit.list(outsider, workspace, None).await,
        Err(GovernanceError::WorkspaceNotFound)
    ));

    let temporary = env::temp_dir().join(format!("adoc-retention-{}", Uuid::now_v7()));
    fs::create_dir_all(&temporary).unwrap();
    let local = Arc::new(LocalObjectStorage::new(temporary.clone()).unwrap());
    let flaky = Arc::new(FlakyStorage {
        inner: local.clone(),
        remaining_failures: AtomicUsize::new(1),
    });
    let retention = RetentionService::new(
        Arc::new(PostgresRetentionRepository::new(&store)),
        flaky,
        Arc::new(SystemClock),
        Arc::from("audit-retention-test"),
    );

    let document = Uuid::now_v7();
    let draft = Uuid::now_v7();
    let asset = Uuid::now_v7();
    let storage_key = "1".repeat(64);
    seed_trashed_document(&store, workspace, owner, document, draft).await;
    seed_file(&store, workspace, owner, asset, &storage_key).await;
    sqlx::query("INSERT INTO file_references(workspace_id,asset_id,owner_kind,owner_id) VALUES($1,$2,'DRAFT',$3)")
        .bind(workspace).bind(asset).bind(draft).execute(store.pool()).await.unwrap();
    local
        .write(&storage_key, bytes(b"retained-byte"), 100)
        .await
        .unwrap();
    let mut metadata = AuditFields::new();
    metadata.insert("revision".into(), AuditValue::Integer(0));
    let mut tx = store.pool().begin().await.unwrap();
    let mut event = AuditEventInput::user(
        workspace,
        owner,
        AuditAction::DocumentTrashed,
        AuditTarget {
            kind: AuditTargetKind::Document,
            id: document,
        },
        chrono::Utc::now(),
        "document-trash-audit-01",
    );
    event.metadata = metadata;
    append_audit_event(&mut tx, event).await.unwrap();
    tx.commit().await.unwrap();

    let job = retention
        .request_document_purge(
            owner,
            workspace,
            document,
            0,
            "retention elapsed".into(),
            "document-purge-0001",
        )
        .await
        .unwrap();
    assert_eq!(job.status, "QUEUED");
    assert_eq!(retention.run_once(25).await.unwrap(), 0);
    let (status, step): (String, String) =
        sqlx::query_as("SELECT status::text,step::text FROM purge_ledger WHERE id=$1")
            .bind(job.job_id)
            .fetch_one(store.pool())
            .await
            .unwrap();
    assert_eq!(status, "RETRY");
    assert_eq!(step, "DOMAIN_PURGED");
    sqlx::query("UPDATE purge_ledger SET run_after=now() WHERE id=$1")
        .bind(job.job_id)
        .execute(store.pool())
        .await
        .unwrap();
    assert_eq!(retention.run_once(25).await.unwrap(), 1);
    assert!(
        !sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM documents WHERE id=$1)")
            .bind(document)
            .fetch_one(store.pool())
            .await
            .unwrap()
    );
    assert!(local.stat(&storage_key).await.is_err());
    let redacted = sqlx::query("SELECT metadata_json,redacted_at FROM audit_events WHERE workspace_id=$1 AND target_json->>'id'=$2 AND action='DOCUMENT_TRASHED'")
        .bind(workspace).bind(document.to_string()).fetch_one(store.pool()).await.unwrap();
    assert_eq!(
        redacted.get::<serde_json::Value, _>("metadata_json"),
        serde_json::json!({})
    );
    assert!(
        redacted
            .get::<Option<chrono::DateTime<chrono::Utc>>, _>("redacted_at")
            .is_some()
    );
    let purged_count: i64 = sqlx::query_scalar("SELECT count(*) FROM audit_events WHERE workspace_id=$1 AND action='DOCUMENT_PURGED' AND target_json->>'id'=$2")
        .bind(workspace).bind(document.to_string()).fetch_one(store.pool()).await.unwrap();
    assert_eq!(purged_count, 1);

    let restored = Uuid::now_v7();
    let restored_draft = Uuid::now_v7();
    seed_trashed_document(&store, workspace, owner, restored, restored_draft).await;
    let cancelled = retention
        .request_document_purge(
            owner,
            workspace,
            restored,
            0,
            "restore wins".into(),
            "document-purge-0002",
        )
        .await
        .unwrap();
    sqlx::query(
        "UPDATE documents SET status='ACTIVE',trashed_at=NULL,purge_after=NULL WHERE id=$1",
    )
    .bind(restored)
    .execute(store.pool())
    .await
    .unwrap();
    retention.run_once(25).await.unwrap();
    let cancelled_status: String =
        sqlx::query_scalar("SELECT status FROM purge_ledger WHERE id=$1")
            .bind(cancelled.job_id)
            .fetch_one(store.pool())
            .await
            .unwrap();
    assert_eq!(cancelled_status, "COMPLETED");
    assert!(
        sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM documents WHERE id=$1 AND status='ACTIVE')"
        )
        .bind(restored)
        .fetch_one(store.pool())
        .await
        .unwrap()
    );

    let deleted_workspace = Uuid::now_v7();
    seed_workspace(&store, deleted_workspace, owner, "audit-delete").await;
    sqlx::query("UPDATE workspaces SET status='DELETION_SCHEDULED',delete_after=now()-interval '1 second' WHERE id=$1")
        .bind(deleted_workspace).execute(store.pool()).await.unwrap();
    assert_eq!(retention.run_once(25).await.unwrap(), 1);
    let workspace_row = sqlx::query("SELECT status::text,name FROM workspaces WHERE id=$1")
        .bind(deleted_workspace)
        .fetch_one(store.pool())
        .await
        .unwrap();
    assert_eq!(workspace_row.get::<String, _>("status"), "DELETED");
    assert_eq!(workspace_row.get::<String, _>("name"), "Deleted workspace");
    let member_status: String = sqlx::query_scalar(
        "SELECT status::text FROM memberships WHERE workspace_id=$1 AND user_id=$2",
    )
    .bind(deleted_workspace)
    .bind(owner)
    .fetch_one(store.pool())
    .await
    .unwrap();
    assert_eq!(member_status, "REMOVED");

    store.close().await;
    fs::remove_dir_all(temporary).unwrap();
}

struct FlakyStorage {
    inner: Arc<LocalObjectStorage>,
    remaining_failures: AtomicUsize,
}

impl ObjectStorage for FlakyStorage {
    fn write<'a>(
        &'a self,
        key: &'a str,
        stream: ByteStream,
        max_bytes: u64,
    ) -> adoc_ports::BoxFuture<'a, Result<ObjectMetadata, StorageError>> {
        self.inner.write(key, stream, max_bytes)
    }
    fn stat<'a>(
        &'a self,
        key: &'a str,
    ) -> adoc_ports::BoxFuture<'a, Result<ObjectMetadata, StorageError>> {
        self.inner.stat(key)
    }
    fn read<'a>(
        &'a self,
        key: &'a str,
        range: Option<ByteRange>,
    ) -> adoc_ports::BoxFuture<'a, Result<ByteStream, StorageError>> {
        self.inner.read(key, range)
    }
    fn delete<'a>(&'a self, key: &'a str) -> adoc_ports::BoxFuture<'a, Result<(), StorageError>> {
        Box::pin(async move {
            if self
                .remaining_failures
                .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |value| {
                    value.checked_sub(1)
                })
                .is_ok()
            {
                return Err(StorageError::Unavailable);
            }
            self.inner.delete(key).await
        })
    }
}

async fn seed_user(store: &PostgresStore, id: Uuid, suffix: &str) {
    sqlx::query("INSERT INTO users(id,google_subject,email,display_name) VALUES($1,$2,$3,$4)")
        .bind(id)
        .bind(suffix)
        .bind(format!("{suffix}@example.com"))
        .bind(suffix)
        .execute(store.pool())
        .await
        .unwrap();
}

async fn seed_workspace(store: &PostgresStore, id: Uuid, owner: Uuid, slug: &str) {
    sqlx::query("INSERT INTO workspaces(id,slug,name,created_by) VALUES($1,$2,$3,$4)")
        .bind(id)
        .bind(slug)
        .bind(slug)
        .bind(owner)
        .execute(store.pool())
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO memberships(workspace_id,user_id,role,status) VALUES($1,$2,'OWNER','ACTIVE')",
    )
    .bind(id)
    .bind(owner)
    .execute(store.pool())
    .await
    .unwrap();
}

async fn seed_trashed_document(
    store: &PostgresStore,
    workspace: Uuid,
    owner: Uuid,
    document: Uuid,
    draft: Uuid,
) {
    sqlx::query("INSERT INTO documents(id,workspace_id,rank,title,status,trashed_at,purge_after,created_by) VALUES($1,$2,$3,'Trash','TRASHED',now()-interval '31 days',now()-interval '1 day',$4)")
        .bind(document).bind(workspace).bind(format!("{:0>32}", document.as_u128() % 1_000_000)).bind(owner).execute(store.pool()).await.unwrap();
    sqlx::query("INSERT INTO drafts(id,workspace_id,document_id,content_json,schema_version,updated_by) VALUES($1,$2,$3,$4,1,$5)")
        .bind(draft).bind(workspace).bind(document).bind(serde_json::json!({"schemaVersion":1,"root":{"type":"doc","children":[]}})).bind(owner).execute(store.pool()).await.unwrap();
}

async fn seed_file(
    store: &PostgresStore,
    workspace: Uuid,
    owner: Uuid,
    asset: Uuid,
    storage_key: &str,
) {
    sqlx::query("INSERT INTO file_assets(id,workspace_id,storage_key,original_name,mime_type,size_bytes,checksum_sha256,status,detected_mime_type,uploaded_by,ready_at) VALUES($1,$2,$3,'purge.txt','text/plain',13,$4,'READY','text/plain',$5,now())")
        .bind(asset).bind(workspace).bind(storage_key).bind("a".repeat(64)).bind(owner).execute(store.pool()).await.unwrap();
}

fn bytes(value: &'static [u8]) -> ByteStream {
    Box::pin(stream::iter(vec![Ok(Bytes::from_static(value))]))
}

fn secret(direct: &str, file: &str) -> String {
    env::var(direct).unwrap_or_else(|_| {
        fs::read_to_string(env::var(file).unwrap())
            .unwrap()
            .trim()
            .to_owned()
    })
}
