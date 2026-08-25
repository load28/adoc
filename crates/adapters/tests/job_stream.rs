use std::{env, sync::Arc};

use adoc_adapters::{
    identity::SystemClock,
    job_queue::RedisJobSignalQueue,
    permission_cache::UnavailablePermissionCache,
    postgres::{
        DatabaseSettings, OutboxEventInput, PgUnitOfWork, PostgresJobRepository,
        PostgresPermissionRepository, PostgresStore, PostgresStreamRepository, append_outbox_event,
    },
};
use adoc_application::{
    jobs::{JobRepository, JobRuntime, JobSignalQueue},
    operations::{EventAudience, JobPriorityBucket, JobSignal, StreamAccess},
    permission::PermissionService,
    stream::StreamService,
};
use adoc_ports::UnitOfWork;
use chrono::{Duration, Utc};
use serde_json::{Value, json};
use uuid::Uuid;

#[tokio::test]
#[ignore = "requires isolated PostgreSQL 16 and Redis"]
async fn job_runtime_and_resumable_stream_contract() {
    let store = PostgresStore::connect(DatabaseSettings {
        url: &secret("ADOC_TEST_DATABASE_URL", "ADOC_TEST_DATABASE_URL_FILE"),
        max_connections: 8,
        application_name: "adoc-job-stream-test",
    })
    .await
    .unwrap();
    store.migrate().await.unwrap();

    let owner = Uuid::now_v7();
    let viewer = Uuid::now_v7();
    let member = Uuid::now_v7();
    for (id, label) in [(owner, "owner"), (viewer, "viewer"), (member, "member")] {
        seed_user(&store, id, label).await;
    }
    let workspace = Uuid::now_v7();
    seed_workspace(&store, workspace, owner).await;
    for user in [viewer, member] {
        sqlx::query("INSERT INTO memberships(workspace_id,user_id,role,status) VALUES($1,$2,'MEMBER','ACTIVE')")
            .bind(workspace).bind(user).execute(store.pool()).await.unwrap();
    }
    let document = Uuid::now_v7();
    sqlx::query("INSERT INTO documents(id,workspace_id,rank,title,created_by) VALUES($1,$2,$3,'Runtime contract',$4)")
        .bind(document).bind(workspace).bind("00000000000000000000000000000001").bind(owner)
        .execute(store.pool()).await.unwrap();
    for (user, access, manage) in [(owner, "EDITOR", true), (viewer, "VIEWER", false)] {
        sqlx::query("INSERT INTO permission_grants(id,workspace_id,document_id,subject_kind,subject_id,access,can_manage,granted_by) VALUES($1,$2,$3,'USER',$4,$5::document_access,$6,$7)")
            .bind(Uuid::now_v7()).bind(workspace).bind(document).bind(user).bind(access).bind(manage).bind(owner)
            .execute(store.pool()).await.unwrap();
    }

    let permission = Arc::new(PermissionService::new(
        Arc::new(PostgresPermissionRepository::new(&store)),
        Arc::new(UnavailablePermissionCache),
        Arc::new(SystemClock),
    ));
    let stream = StreamService::new(Arc::new(PostgresStreamRepository::new(&store)), permission);
    let mut owner_session = stream.open(owner, workspace, None).await.unwrap().session;
    let mut viewer_session = stream.open(viewer, workspace, None).await.unwrap().session;
    let mut member_session = stream.open(member, workspace, None).await.unwrap().session;
    let expired_cursor = stream.next_page(&mut viewer_session).await.unwrap().cursor;

    let uow = PgUnitOfWork::new(store.pool().clone());
    append(
        &uow,
        workspace,
        "WorkspaceChanged.v1",
        json!({"entityId": workspace, "revision": 1, "action": "UPDATED"}),
        EventAudience::workspace(),
    )
    .await;
    append(
        &uow,
        workspace,
        "GroupChanged.v1",
        json!({"entityId": Uuid::now_v7(), "revision": 1, "action": "UPDATED"}),
        EventAudience::admin(),
    )
    .await;
    append(
        &uow,
        workspace,
        "InboxChanged.v1",
        json!({"entityId": Uuid::now_v7(), "revision": 1, "action": "CREATED"}),
        EventAudience::user(viewer),
    )
    .await;
    append(
        &uow,
        workspace,
        "DocumentChanged.v1",
        json!({"documentId": document, "revision": 1, "treeRevision": 1, "action": "UPDATED"}),
        EventAudience::document(document, StreamAccess::Viewer),
    )
    .await;

    let redis_url = secret("ADOC_TEST_REDIS_URL", "ADOC_TEST_REDIS_URL_FILE");
    let namespace = format!("task024-{}", workspace.simple());
    let queue = Arc::new(
        RedisJobSignalQueue::connect(&redis_url, &namespace)
            .await
            .unwrap(),
    );
    let repository = Arc::new(PostgresJobRepository::new(&store));
    let runtime = JobRuntime::new(
        repository.clone(),
        repository.clone(),
        queue.clone(),
        Arc::new(SystemClock),
        Arc::from("job-stream-test"),
        Duration::seconds(30),
    );

    // Redis is initially empty: PostgreSQL reconciliation must recover every committed job.
    assert!(runtime.run_once(100, true).await.unwrap() >= 4);
    let sequences: Vec<i64> = sqlx::query_scalar(
        "SELECT sequence FROM workspace_stream_events WHERE workspace_id=$1 ORDER BY sequence",
    )
    .bind(workspace)
    .fetch_all(store.pool())
    .await
    .unwrap();
    assert_eq!(sequences, vec![1, 2, 3, 4]);
    let receipt_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM consumer_receipts r JOIN outbox_events o ON o.id=r.event_id WHERE r.consumer='workspace-stream' AND o.workspace_id=$1",
    )
    .bind(workspace).fetch_one(store.pool()).await.unwrap();
    assert_eq!(receipt_count, 4);

    let owner_page = stream.next_page(&mut owner_session).await.unwrap();
    let viewer_page = stream.next_page(&mut viewer_session).await.unwrap();
    let member_page = stream.next_page(&mut member_session).await.unwrap();
    assert_eq!(owner_page.deliveries.len(), 3);
    assert_eq!(viewer_page.deliveries.len(), 3);
    assert_eq!(member_page.deliveries.len(), 1);

    let succeeded_job: Uuid = sqlx::query_scalar(
        "SELECT id FROM jobs WHERE workspace_id=$1 AND status='SUCCEEDED' ORDER BY id LIMIT 1",
    )
    .bind(workspace)
    .fetch_one(store.pool())
    .await
    .unwrap();
    queue
        .signal(&[
            JobSignal {
                id: succeeded_job,
                bucket: JobPriorityBucket::Normal,
            },
            JobSignal {
                id: succeeded_job,
                bucket: JobPriorityBucket::Normal,
            },
        ])
        .await
        .unwrap();
    assert_eq!(runtime.run_once(100, false).await.unwrap(), 0);
    let stream_count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM workspace_stream_events WHERE workspace_id=$1")
            .bind(workspace)
            .fetch_one(store.pool())
            .await
            .unwrap();
    assert_eq!(stream_count, 4);

    let resume_cursor = viewer_page.cursor;
    append(
        &uow,
        workspace,
        "WorkspaceChanged.v1",
        json!({"entityId": workspace, "revision": 2, "action": "UPDATED"}),
        EventAudience::workspace(),
    )
    .await;
    assert_eq!(runtime.run_once(100, true).await.unwrap(), 1);
    let mut resumed = stream
        .open(viewer, workspace, Some(&resume_cursor))
        .await
        .unwrap()
        .session;
    assert_eq!(
        stream
            .next_page(&mut resumed)
            .await
            .unwrap()
            .deliveries
            .len(),
        1
    );
    assert!(
        stream
            .open(viewer, Uuid::now_v7(), Some(&resume_cursor))
            .await
            .is_err()
    );

    sqlx::query("UPDATE memberships SET revision=revision+1 WHERE workspace_id=$1 AND user_id=$2")
        .bind(workspace)
        .bind(viewer)
        .execute(store.pool())
        .await
        .unwrap();
    assert!(stream.next_page(&mut resumed).await.unwrap().reset_required);

    sqlx::query("DELETE FROM workspace_stream_events WHERE workspace_id=$1 AND sequence<4")
        .bind(workspace)
        .execute(store.pool())
        .await
        .unwrap();
    assert!(
        stream
            .open(viewer, workspace, Some(&expired_cursor))
            .await
            .unwrap()
            .reset_required
    );

    append(
        &uow,
        workspace,
        "WorkspaceChanged.v1",
        json!({"entityId": workspace, "revision": 3, "action": "UPDATED"}),
        EventAudience::internal(),
    )
    .await;
    let (cancel_job, cancel_sequence): (Uuid, i64) = sqlx::query_as(
        "SELECT id,sequence FROM jobs WHERE workspace_id=$1 AND status='QUEUED' ORDER BY created_at DESC LIMIT 1",
    )
    .bind(workspace).fetch_one(store.pool()).await.unwrap();
    repository
        .request_cancel(cancel_job, cancel_sequence, Utc::now())
        .await
        .unwrap();
    runtime.run_once(100, true).await.unwrap();
    let cancelled: String = sqlx::query_scalar("SELECT status::text FROM jobs WHERE id=$1")
        .bind(cancel_job)
        .fetch_one(store.pool())
        .await
        .unwrap();
    assert_eq!(cancelled, "CANCELLED");

    append(
        &uow,
        workspace,
        "Unsupported.v1",
        json!({"entityId": workspace, "revision": 4, "action": "UPDATED"}),
        EventAudience::workspace(),
    )
    .await;
    runtime.run_once(100, true).await.unwrap();
    let failed: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM jobs WHERE workspace_id=$1 AND status='FAILED' AND last_error_code='EVENT_TYPE_INVALID'",
    )
    .bind(workspace).fetch_one(store.pool()).await.unwrap();
    assert_eq!(failed, 1);

    let recoverable_event = append(
        &uow,
        workspace,
        "WorkspaceChanged.v1",
        json!({"entityId": workspace, "revision": 5, "action": "UPDATED"}),
        EventAudience::internal(),
    )
    .await;
    sqlx::query(
        "UPDATE jobs SET status='RUNNING',sequence=sequence+1,attempt=1,lease_owner='lost-worker',lease_until=now()-interval '1 second',updated_at=now() WHERE payload_json->>'outboxEventId'=$1",
    )
    .bind(recoverable_event.to_string()).execute(store.pool()).await.unwrap();
    runtime.run_once(100, true).await.unwrap();
    let recovered: String =
        sqlx::query_scalar("SELECT status::text FROM jobs WHERE payload_json->>'outboxEventId'=$1")
            .bind(recoverable_event.to_string())
            .fetch_one(store.pool())
            .await
            .unwrap();
    assert_eq!(recovered, "SUCCEEDED");

    let exhausted_event = append(
        &uow,
        workspace,
        "WorkspaceChanged.v1",
        json!({"entityId": workspace, "revision": 6, "action": "UPDATED"}),
        EventAudience::internal(),
    )
    .await;
    sqlx::query(
        "UPDATE jobs SET status='RUNNING',sequence=sequence+1,attempt=max_attempts,lease_owner='lost-worker',lease_until=now()-interval '1 second',updated_at=now() WHERE payload_json->>'outboxEventId'=$1",
    )
    .bind(exhausted_event.to_string()).execute(store.pool()).await.unwrap();
    runtime.run_once(100, true).await.unwrap();
    let exhausted: (String, Option<String>) = sqlx::query_as(
        "SELECT status::text,last_error_code FROM jobs WHERE payload_json->>'outboxEventId'=$1",
    )
    .bind(exhausted_event.to_string())
    .fetch_one(store.pool())
    .await
    .unwrap();
    assert_eq!(
        exhausted,
        ("DEAD_LETTER".to_owned(), Some("LEASE_EXPIRED".to_owned()))
    );

    sqlx::query("DELETE FROM permission_grants WHERE workspace_id=$1")
        .bind(workspace)
        .execute(store.pool())
        .await
        .unwrap();
    sqlx::query("DELETE FROM workspaces WHERE id=$1")
        .bind(workspace)
        .execute(store.pool())
        .await
        .unwrap();
    for user in [owner, viewer, member] {
        sqlx::query("DELETE FROM users WHERE id=$1")
            .bind(user)
            .execute(store.pool())
            .await
            .unwrap();
    }
    store.close().await;
}

async fn append(
    uow: &PgUnitOfWork,
    workspace: Uuid,
    event_type: &'static str,
    payload: Value,
    audience: EventAudience,
) -> Uuid {
    let event_id = Uuid::now_v7();
    uow.execute(|transaction| {
        Box::pin(async move {
            let correlation_id = event_id.to_string();
            append_outbox_event(
                transaction,
                OutboxEventInput {
                    id: event_id,
                    workspace_id: workspace,
                    aggregate_kind: "RuntimeTest",
                    aggregate_id: event_id,
                    sequence: 1,
                    event_type,
                    event_version: 1,
                    payload,
                    audience,
                    correlation_id: &correlation_id,
                    occurred_at: Utc::now(),
                },
            )
            .await
        })
    })
    .await
    .unwrap();
    event_id
}

async fn seed_user(store: &PostgresStore, id: Uuid, label: &str) {
    sqlx::query("INSERT INTO users(id,google_subject,email,display_name) VALUES($1,$2,$3,$4)")
        .bind(id)
        .bind(format!("task-024-{label}-{id}"))
        .bind(format!("{id}@example.test"))
        .bind(label)
        .execute(store.pool())
        .await
        .unwrap();
}

async fn seed_workspace(store: &PostgresStore, id: Uuid, owner: Uuid) {
    sqlx::query("INSERT INTO workspaces(id,slug,name,created_by) VALUES($1,$2,'Job stream',$3)")
        .bind(id)
        .bind(format!("job-stream-{}", &id.simple().to_string()[..8]))
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

fn secret(value: &str, file: &str) -> String {
    if let Ok(value) = env::var(value) {
        return value;
    }
    std::fs::read_to_string(env::var(file).unwrap())
        .unwrap()
        .trim()
        .to_owned()
}
