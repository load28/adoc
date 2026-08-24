use std::env;

use adoc_adapters::postgres::{
    DatabaseSettings, IdempotencyDecision, IdempotencyError, IdempotencyIdentity,
    IdempotencyReservation, OutboxAppendError, OutboxEventInput, PgUnitOfWork, PostgresStore,
    StoredResponse, append_outbox_event, complete_idempotency, reserve_idempotency,
};
use adoc_ports::{UnitOfWork, UnitOfWorkError};
use chrono::{Duration, Utc};
use serde_json::json;
use uuid::Uuid;

const HASH_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const HASH_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

#[tokio::test]
#[ignore = "requires ADOC_TEST_DATABASE_URL pointing to an isolated PostgreSQL 16 database"]
async fn postgres_foundation_contract() {
    let url = test_database_url();
    let store = PostgresStore::connect(DatabaseSettings {
        url: &url,
        max_connections: 5,
        application_name: "adoc-integration-test",
    })
    .await
    .expect("connect PostgreSQL");

    store.migrate().await.expect("fresh migration");
    let ledger_before = migration_ledger(&store).await;
    store.migrate().await.expect("second migration run");
    assert_eq!(migration_ledger(&store).await, ledger_before);
    assert_eq!(ledger_before.len(), 3);
    assert_eq!(store.preflight().await.unwrap().server_major_version, 16);

    let table_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM information_schema.tables \
         WHERE table_schema = 'public' AND table_type = 'BASE TABLE' \
         AND table_name <> '_sqlx_migrations'",
    )
    .fetch_one(store.pool())
    .await
    .unwrap();
    assert_eq!(table_count, 43);

    let user_id = uuid("018f0000-0000-7000-8000-000000000001");
    let workspace_id = uuid("018f0000-0000-7000-8000-000000000002");
    seed_identity(&store, user_id, workspace_id).await;
    let unit_of_work = PgUnitOfWork::new(store.pool().clone());

    verify_rollback(&store, &unit_of_work, workspace_id, user_id).await;
    verify_idempotency(&unit_of_work, workspace_id, user_id).await;
    verify_busy_and_takeover(&unit_of_work, workspace_id, user_id).await;
    verify_outbox(&store, &unit_of_work, workspace_id).await;

    sqlx::query("DELETE FROM workspaces WHERE id = $1")
        .bind(workspace_id)
        .execute(store.pool())
        .await
        .unwrap();
    sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user_id)
        .execute(store.pool())
        .await
        .unwrap();
    store.close().await;
}

async fn migration_ledger(store: &PostgresStore) -> Vec<(i64, Vec<u8>, bool)> {
    sqlx::query_as("SELECT version, checksum, success FROM _sqlx_migrations ORDER BY version")
        .fetch_all(store.pool())
        .await
        .unwrap()
}

async fn seed_identity(store: &PostgresStore, user_id: Uuid, workspace_id: Uuid) {
    sqlx::query("DELETE FROM workspaces WHERE id = $1")
        .bind(workspace_id)
        .execute(store.pool())
        .await
        .unwrap();
    sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user_id)
        .execute(store.pool())
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO users (id, google_subject, email, display_name) VALUES ($1, $2, $3, $4)",
    )
    .bind(user_id)
    .bind("google-task-011")
    .bind("task-011@example.test")
    .bind("Task 011")
    .execute(store.pool())
    .await
    .unwrap();
    sqlx::query("INSERT INTO workspaces (id, slug, name, created_by) VALUES ($1, $2, $3, $4)")
        .bind(workspace_id)
        .bind("task-011")
        .bind("Task 011")
        .bind(user_id)
        .execute(store.pool())
        .await
        .unwrap();
}

async fn verify_rollback(
    store: &PostgresStore,
    unit_of_work: &PgUnitOfWork,
    workspace_id: Uuid,
    actor_id: Uuid,
) {
    let identity = identity(workspace_id, actor_id, "rollback");
    let now = Utc::now();
    let result = unit_of_work
        .execute(|transaction| {
            Box::pin(async move {
                reserve_idempotency(transaction, reservation(identity, HASH_A, now))
                    .await
                    .unwrap();
                Err::<(), _>("abort")
            })
        })
        .await;
    assert!(matches!(result, Err(UnitOfWorkError::Operation("abort"))));

    let count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM idempotency_keys \
         WHERE workspace_id = $1 AND actor_id = $2 AND operation_id = $3 AND key = $4",
    )
    .bind(workspace_id)
    .bind(actor_id)
    .bind(identity.operation_id)
    .bind(identity.key)
    .fetch_one(store.pool())
    .await
    .unwrap();
    assert_eq!(count, 0);
}

async fn verify_idempotency(unit_of_work: &PgUnitOfWork, workspace_id: Uuid, actor_id: Uuid) {
    let identity = identity(workspace_id, actor_id, "replay");
    let now = Utc::now();
    let response = StoredResponse {
        status: 201,
        body: json!({"documentId": "018f0000-0000-7000-8000-000000000010"}),
    };
    let expected = response.clone();
    let acquired = unit_of_work
        .execute(|transaction| {
            Box::pin(async move {
                let decision =
                    reserve_idempotency(transaction, reservation(identity, HASH_A, now)).await?;
                complete_idempotency(transaction, identity, HASH_A, response).await?;
                Ok::<_, IdempotencyError>(decision)
            })
        })
        .await
        .unwrap();
    assert_eq!(acquired, IdempotencyDecision::Acquired);

    let replay = unit_of_work
        .execute(|transaction| {
            Box::pin(async move {
                reserve_idempotency(transaction, reservation(identity, HASH_A, Utc::now())).await
            })
        })
        .await
        .unwrap();
    assert_eq!(replay, IdempotencyDecision::Replay(expected));

    let reused = unit_of_work
        .execute(|transaction| {
            Box::pin(async move {
                reserve_idempotency(transaction, reservation(identity, HASH_B, Utc::now())).await
            })
        })
        .await;
    assert!(matches!(
        reused,
        Err(UnitOfWorkError::Operation(IdempotencyError::KeyReused))
    ));
}

async fn verify_busy_and_takeover(unit_of_work: &PgUnitOfWork, workspace_id: Uuid, actor_id: Uuid) {
    let busy_identity = identity(workspace_id, actor_id, "busy");
    let now = Utc::now();
    unit_of_work
        .execute(|transaction| {
            Box::pin(async move {
                reserve_idempotency(transaction, reservation(busy_identity, HASH_A, now)).await
            })
        })
        .await
        .unwrap();
    let busy = unit_of_work
        .execute(|transaction| {
            Box::pin(async move {
                reserve_idempotency(
                    transaction,
                    reservation(busy_identity, HASH_A, now + Duration::seconds(1)),
                )
                .await
            })
        })
        .await
        .unwrap();
    assert!(matches!(busy, IdempotencyDecision::Busy { .. }));

    let expired_identity = identity(workspace_id, actor_id, "expired");
    let old_now = now - Duration::hours(2);
    unit_of_work
        .execute(|transaction| {
            Box::pin(async move {
                reserve_idempotency(transaction, reservation(expired_identity, HASH_A, old_now))
                    .await
            })
        })
        .await
        .unwrap();
    let takeover = unit_of_work
        .execute(|transaction| {
            Box::pin(async move {
                reserve_idempotency(transaction, reservation(expired_identity, HASH_B, now)).await
            })
        })
        .await
        .unwrap();
    assert_eq!(takeover, IdempotencyDecision::Acquired);
}

async fn verify_outbox(store: &PostgresStore, unit_of_work: &PgUnitOfWork, workspace_id: Uuid) {
    let aggregate_id = uuid("018f0000-0000-7000-8000-000000000003");
    unit_of_work
        .execute(|transaction| {
            Box::pin(async move {
                append_outbox_event(
                    transaction,
                    outbox_event(
                        uuid("018f0000-0000-7000-8000-000000000004"),
                        workspace_id,
                        aggregate_id,
                    ),
                )
                .await
            })
        })
        .await
        .unwrap();

    let conflict = unit_of_work
        .execute(|transaction| {
            Box::pin(async move {
                append_outbox_event(
                    transaction,
                    outbox_event(
                        uuid("018f0000-0000-7000-8000-000000000005"),
                        workspace_id,
                        aggregate_id,
                    ),
                )
                .await
            })
        })
        .await;
    assert!(matches!(
        conflict,
        Err(UnitOfWorkError::Operation(
            OutboxAppendError::SequenceConflict
        ))
    ));

    let stored: (String, i64, i32, serde_json::Value) = sqlx::query_as(
        "SELECT event_type, sequence, event_version, payload_json FROM outbox_events \
         WHERE aggregate_kind = 'Document' AND aggregate_id = $1",
    )
    .bind(aggregate_id)
    .fetch_one(store.pool())
    .await
    .unwrap();
    assert_eq!(
        stored,
        ("DocumentCreated".to_owned(), 1, 1, json!({"revision": 0}))
    );
}

fn identity(
    workspace_id: Uuid,
    actor_id: Uuid,
    suffix: &'static str,
) -> IdempotencyIdentity<'static> {
    IdempotencyIdentity {
        workspace_id,
        actor_id,
        operation_id: "createDocument",
        key: suffix,
    }
}

fn reservation(
    identity: IdempotencyIdentity<'static>,
    request_hash: &'static str,
    now: chrono::DateTime<Utc>,
) -> IdempotencyReservation<'static> {
    IdempotencyReservation {
        identity,
        request_hash,
        now,
        locked_until: now + Duration::minutes(1),
        expires_at: now + Duration::hours(1),
    }
}

fn outbox_event(id: Uuid, workspace_id: Uuid, aggregate_id: Uuid) -> OutboxEventInput<'static> {
    OutboxEventInput {
        id,
        workspace_id,
        aggregate_kind: "Document",
        aggregate_id,
        sequence: 1,
        event_type: "DocumentCreated",
        event_version: 1,
        payload: json!({"revision": 0}),
        occurred_at: Utc::now(),
    }
}

fn uuid(value: &str) -> Uuid {
    Uuid::parse_str(value).unwrap()
}

fn test_database_url() -> String {
    if let Ok(url) = env::var("ADOC_TEST_DATABASE_URL") {
        return url;
    }
    let path = env::var("ADOC_TEST_DATABASE_URL_FILE")
        .expect("ADOC_TEST_DATABASE_URL or ADOC_TEST_DATABASE_URL_FILE is required");
    std::fs::read_to_string(path).unwrap().trim().to_owned()
}
