use std::{env, fs};

use adoc_adapters::postgres::{DatabaseSettings, PostgresAiContextRepository, PostgresStore};
use adoc_application::ai::{
    AiContextRepository, AiJobRepository, AiTarget, AiTask, AiTaskKind, ContextSourceKind,
};
use chrono::Utc;
use serde_json::{Value, json};
use uuid::Uuid;

#[tokio::test]
#[ignore = "requires ADOC_TEST_DATABASE_URL pointing to an isolated PostgreSQL 16 database"]
async fn ai_context_admission_is_permission_safe_and_payload_bounded() {
    let store = PostgresStore::connect(DatabaseSettings {
        url: &secret("ADOC_TEST_DATABASE_URL", "ADOC_TEST_DATABASE_URL_FILE"),
        max_connections: 8,
        application_name: "adoc-ai-context-test",
    })
    .await
    .unwrap();
    store.migrate().await.unwrap();
    let owner = Uuid::now_v7();
    let denied = Uuid::now_v7();
    let workspace = Uuid::now_v7();
    let root = Uuid::now_v7();
    let document = Uuid::now_v7();
    let target = Uuid::now_v7();
    let block = Uuid::now_v7();
    seed(
        &store, owner, denied, workspace, root, document, target, block,
    )
    .await;

    let repository = PostgresAiContextRepository::new(&store);
    let task = AiTask {
        kind: AiTaskKind::Review,
        workspace_id: workspace,
        actor_id: owner,
        target: AiTarget::Document {
            document_id: document,
        },
        expected_revision: 0,
        external_web_enabled: false,
        instruction: Some("검토".to_owned()),
    };
    let prepared = repository.prepare(&task).await.unwrap();
    let mut artifact = repository.materialize(&task, &prepared, &[]).await.unwrap();
    assert!(artifact.sources.iter().any(|source| {
        source.kind == ContextSourceKind::Draft && source.snapshot_text == "private-context"
    }));
    assert!(
        artifact
            .sources
            .iter()
            .any(|source| source.kind == ContextSourceKind::Discussion)
    );
    assert!(
        artifact
            .sources
            .iter()
            .any(|source| source.kind == ContextSourceKind::Vocabulary)
    );
    assert!(artifact.sources.iter().any(|source| {
        source.kind == ContextSourceKind::PublishedRegion && source.document_id == Some(target)
    }));
    let fingerprint = artifact.normalize_and_fingerprint(4 * 1024 * 1024).unwrap();
    let first = repository
        .admit(
            &task,
            &artifact,
            &fingerprint,
            "ai-context-contract-001",
            Utc::now(),
        )
        .await
        .unwrap();
    let replay = repository
        .admit(
            &task,
            &artifact,
            &fingerprint,
            "ai-context-contract-001",
            Utc::now(),
        )
        .await
        .unwrap();
    assert_eq!(first.view.id, replay.view.id);
    assert_eq!(first.view.target, task.target);
    assert_eq!(replay.view.target, task.target);
    let payload: Value = sqlx::query_scalar("SELECT payload_json FROM jobs WHERE id=$1")
        .bind(first.signal.id)
        .fetch_one(store.pool())
        .await
        .unwrap();
    assert_eq!(payload, json!({"aiJobId": first.view.id}));
    assert!(!payload.to_string().contains("private-context"));
    let stored: String = sqlx::query_scalar(
        "SELECT snapshot_text FROM ai_context_sources WHERE job_id=$1 AND source_kind='DRAFT'",
    )
    .bind(first.view.id)
    .fetch_one(store.pool())
    .await
    .unwrap();
    assert_eq!(stored, "private-context");
    repository
        .cancel(
            owner,
            workspace,
            first.view.id,
            0,
            "ai-context-cancel-001",
            Utc::now(),
        )
        .await
        .unwrap();
    repository
        .cancel(
            owner,
            workspace,
            first.view.id,
            0,
            "ai-context-cancel-001",
            Utc::now(),
        )
        .await
        .unwrap();
    let status: String = sqlx::query_scalar("SELECT status::text FROM ai_jobs WHERE id=$1")
        .bind(first.view.id)
        .fetch_one(store.pool())
        .await
        .unwrap();
    assert_eq!(status, "CANCELLED");
    let events: Vec<Value> = sqlx::query_scalar(
        "SELECT payload_json FROM outbox_events WHERE aggregate_kind='AIJob' AND aggregate_id=$1 ORDER BY sequence",
    )
    .bind(first.view.id)
    .fetch_all(store.pool())
    .await
    .unwrap();
    assert_eq!(events.len(), 2);
    assert!(
        events
            .iter()
            .all(|payload| !payload.to_string().contains("private-context"))
    );

    let denied_task = AiTask {
        actor_id: denied,
        ..task
    };
    assert!(repository.prepare(&denied_task).await.is_err());
    store.close().await;
}

#[allow(clippy::too_many_arguments)]
async fn seed(
    store: &PostgresStore,
    owner: Uuid,
    denied: Uuid,
    workspace: Uuid,
    root: Uuid,
    document: Uuid,
    target: Uuid,
    block: Uuid,
) {
    for (user, name) in [(owner, "Owner"), (denied, "Denied")] {
        sqlx::query("INSERT INTO users(id,google_subject,email,display_name) VALUES($1,$2,$3,$4)")
            .bind(user)
            .bind(format!("subject-{user}"))
            .bind(format!("{user}@example.test"))
            .bind(name)
            .execute(store.pool())
            .await
            .unwrap();
    }
    sqlx::query(
        "INSERT INTO workspaces(id,name,slug,created_by) VALUES($1,'AI','ai-context-contract',$2)",
    )
    .bind(workspace)
    .bind(owner)
    .execute(store.pool())
    .await
    .unwrap();
    for (user, role) in [(owner, "OWNER"), (denied, "MEMBER")] {
        sqlx::query("INSERT INTO memberships(workspace_id,user_id,role,status) VALUES($1,$2,$3::membership_role,'ACTIVE')")
            .bind(workspace).bind(user).bind(role).execute(store.pool()).await.unwrap();
    }
    sqlx::query("INSERT INTO documents(id,workspace_id,parent_id,rank,title,created_by,permission_revision) VALUES($1,$2,NULL,'00000000000000000000000000000001','Root',$3,1),($4,$2,$1,'00000000000000000000000000000002','Private',$3,1),($5,$2,$1,'00000000000000000000000000000003','Reference',$3,1)")
        .bind(root).bind(workspace).bind(owner).bind(document).bind(target).execute(store.pool()).await.unwrap();
    sqlx::query("INSERT INTO permission_grants(id,workspace_id,document_id,subject_kind,subject_id,access,can_manage,granted_by) VALUES($1,$2,$3,'USER',$4,'EDITOR',true,$4),($5,$2,$3,'USER',$6,'NO_ACCESS',false,$4)")
        .bind(Uuid::now_v7()).bind(workspace).bind(root).bind(owner).bind(Uuid::now_v7()).bind(denied).execute(store.pool()).await.unwrap();
    sqlx::query("INSERT INTO drafts(id,workspace_id,document_id,content_json,schema_version,updated_by) VALUES($1,$2,$3,$4,1,$5)")
        .bind(Uuid::now_v7()).bind(workspace).bind(document).bind(json!({"schemaVersion":1,"root":{"type":"doc","children":[{"id":block,"type":"paragraph","children":[{"type":"text","text":"private-context"}]}]}})).bind(owner)
        .execute(store.pool()).await.unwrap();
    let version = Uuid::now_v7();
    let target_block = Uuid::now_v7();
    sqlx::query("INSERT INTO published_versions(id,workspace_id,document_id,number,content_json,schema_version,content_fingerprint,source_draft_revision,publisher_id,summary) VALUES($1,$2,$3,1,$4,1,$5,0,$6,'AI context reference')")
        .bind(version).bind(workspace).bind(target).bind(json!({"schemaVersion":1,"root":{"type":"doc","children":[{"id":target_block,"type":"paragraph","children":[{"type":"text","text":"official-reference"}]}]}})).bind("a".repeat(64)).bind(owner)
        .execute(store.pool()).await.unwrap();
    sqlx::query("UPDATE documents SET current_version_id=$2 WHERE id=$1")
        .bind(target)
        .bind(version)
        .execute(store.pool())
        .await
        .unwrap();
    sqlx::query("INSERT INTO references_graph(id,workspace_id,source_kind,source_id,target_kind,target_id,source_region_json,snapshot_json,created_by) VALUES($1,$2,'DOCUMENT',$3,'DOCUMENT',$4,'{}'::jsonb,'{}'::jsonb,$5)")
        .bind(Uuid::now_v7()).bind(workspace).bind(document).bind(target.to_string()).bind(owner).execute(store.pool()).await.unwrap();
    let discussion = Uuid::now_v7();
    sqlx::query("INSERT INTO discussions(id,workspace_id,document_id,title,created_by) VALUES($1,$2,$3,'Context',$4)")
        .bind(discussion).bind(workspace).bind(document).bind(owner).execute(store.pool()).await.unwrap();
    sqlx::query("INSERT INTO messages(id,workspace_id,discussion_id,author_id,body_json) VALUES($1,$2,$3,$4,$5)")
        .bind(Uuid::now_v7()).bind(workspace).bind(discussion).bind(owner).bind(json!({"root":{"children":[{"type":"paragraph","children":[{"type":"text","text":"discussion-context"}]}]}}))
        .execute(store.pool()).await.unwrap();
    let concept = Uuid::now_v7();
    sqlx::query("INSERT INTO vocabulary_concepts(id,workspace_id,canonical_term,definition,created_by) VALUES($1,$2,'Context','Definition',$3)")
        .bind(concept).bind(workspace).bind(owner).execute(store.pool()).await.unwrap();
    sqlx::query("INSERT INTO vocabulary_terms(workspace_id,concept_id,term,normalized_term,kind) VALUES($1,$2,'Context','context','CANONICAL')")
        .bind(workspace).bind(concept).execute(store.pool()).await.unwrap();
    sqlx::query("INSERT INTO ai_configurations(workspace_id,provider,model,user_concurrency_limit,workspace_concurrency_limit,monthly_budget_microunits,updated_by) VALUES($1,'codex_cli','test-model',2,4,1000000,$2)")
        .bind(workspace).bind(owner).execute(store.pool()).await.unwrap();
}

fn secret(plain: &str, file: &str) -> String {
    env::var(plain).unwrap_or_else(|_| {
        fs::read_to_string(env::var(file).unwrap())
            .unwrap()
            .trim()
            .to_owned()
    })
}
