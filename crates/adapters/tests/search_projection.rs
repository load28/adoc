use std::{env, fs, sync::Arc};

use adoc_adapters::{
    job_executor::WorkerJobExecutor,
    postgres::{
        DatabaseSettings, OutboxEventInput, PgUnitOfWork, PostgresJobRepository,
        PostgresSearchProjectionRepository, PostgresStore, append_outbox_event,
    },
    search_index::OpenSearchIndex,
    search_rebuild::SearchRebuilder,
};
use adoc_application::{
    jobs::{JobExecutor, JobRepository},
    operations::EventAudience,
    search::{
        ProjectionMutation, SearchIndex, SearchProjectionRepository, SearchProjectionService,
    },
};
use adoc_ports::UnitOfWork;
use chrono::{Duration, Utc};
use reqwest::Url;
use serde_json::{Value, json};
use uuid::Uuid;

#[tokio::test]
#[ignore = "requires isolated PostgreSQL 16 and OpenSearch 3"]
async fn search_projection_prefilter_ordering_and_tombstone_contract() {
    let store = PostgresStore::connect(DatabaseSettings {
        url: &secret("ADOC_TEST_DATABASE_URL", "ADOC_TEST_DATABASE_URL_FILE"),
        max_connections: 8,
        application_name: "adoc-search-projection-test",
    })
    .await
    .unwrap();
    store.migrate().await.unwrap();
    let owner = Uuid::now_v7();
    let workspace = Uuid::now_v7();
    let root = Uuid::now_v7();
    let document = Uuid::now_v7();
    seed(&store, owner, workspace, root, document).await;

    let uow = PgUnitOfWork::new(store.pool().clone());
    let first_event = append(
        &uow,
        workspace,
        document,
        1,
        "DocumentChanged.v1",
        json!({"documentId":document,"revision":1,"treeRevision":1,"action":"UPDATED"}),
    )
    .await;
    let sequences: Vec<i64> = sqlx::query_scalar(
        "SELECT projection_sequence FROM outbox_events WHERE workspace_id=$1 ORDER BY projection_sequence",
    )
    .bind(workspace)
    .fetch_all(store.pool())
    .await
    .unwrap();
    assert_eq!(sequences, vec![1]);

    let prefix = format!("task025-{}", workspace.simple());
    let endpoint = Url::parse(&env::var("ADOC_TEST_OPENSEARCH_URL").unwrap()).unwrap();
    let index =
        Arc::new(OpenSearchIndex::new(endpoint.clone(), prefix.clone(), 1536, None).unwrap());
    let projection_repository = Arc::new(PostgresSearchProjectionRepository::new(&store));
    let job_repository = Arc::new(PostgresJobRepository::new(&store));
    let executor = WorkerJobExecutor::new(
        job_repository.clone(),
        SearchProjectionService::new(projection_repository.clone(), index.clone()),
    );

    let old_work = projection_repository.prepare(first_event).await.unwrap();
    let (scope, old_fingerprint) = projection_identity(&old_work.mutations);
    execute_search_job(&store, &job_repository, &executor, first_event).await;
    assert_eq!(
        hits(
            &endpoint,
            &prefix,
            workspace,
            &scope,
            &old_fingerprint,
            "oldsecret"
        )
        .await,
        1
    );
    assert_eq!(
        hits(&endpoint, &prefix, workspace, &scope, "wrong", "oldsecret").await,
        0
    );
    assert_eq!(
        hits(
            &endpoint,
            &prefix,
            Uuid::now_v7(),
            &scope,
            &old_fingerprint,
            "oldsecret"
        )
        .await,
        0
    );

    let new_block = Uuid::now_v7();
    sqlx::query("UPDATE drafts SET content_json=$2,revision=revision+1,updated_at=$3 WHERE document_id=$1")
        .bind(document)
        .bind(json!({"schemaVersion":1,"root":{"type":"doc","children":[{"id":new_block,"type":"paragraph","children":[{"type":"text","text":"newvalue"}]}]}}))
        .bind(Utc::now())
        .execute(store.pool()).await.unwrap();
    sqlx::query("UPDATE documents SET permission_revision=permission_revision+1 WHERE id=$1")
        .bind(root)
        .execute(store.pool())
        .await
        .unwrap();
    let permission_event = append(
        &uow,
        workspace,
        document,
        2,
        "PermissionChanged.v1",
        json!({"entityId":root,"revision":1,"action":"UPDATED"}),
    )
    .await;
    let new_work = projection_repository
        .prepare(permission_event)
        .await
        .unwrap();
    let (_, new_fingerprint) = projection_identity(&new_work.mutations);
    assert_ne!(old_fingerprint, new_fingerprint);
    execute_search_job(&store, &job_repository, &executor, permission_event).await;

    // A delayed older replacement cannot recreate a Region removed by the newer projection.
    index.apply(&old_work.mutations).await.unwrap();
    assert_eq!(
        hits(
            &endpoint,
            &prefix,
            workspace,
            &scope,
            &old_fingerprint,
            "oldsecret"
        )
        .await,
        0
    );
    assert_eq!(
        hits(
            &endpoint,
            &prefix,
            workspace,
            &scope,
            &new_fingerprint,
            "newvalue"
        )
        .await,
        1
    );

    let generation = SearchRebuilder::new(
        PostgresSearchProjectionRepository::new(&store),
        index.as_ref().clone(),
    )
    .run(Utc::now())
    .await
    .unwrap();
    assert_eq!(generation, 2);
    let aliases: Value = reqwest::get(
        endpoint
            .join(&format!("_alias/{prefix}-draft-read"))
            .unwrap(),
    )
    .await
    .unwrap()
    .json()
    .await
    .unwrap();
    assert!(
        aliases
            .as_object()
            .unwrap()
            .contains_key(&format!("{prefix}-draft-v1-2"))
    );
    assert_eq!(
        sqlx::query_scalar::<_, String>(
            "SELECT status FROM search_projection_rebuilds WHERE generation=2"
        )
        .fetch_one(store.pool())
        .await
        .unwrap(),
        "ACTIVE"
    );

    let sequence = sqlx::query_scalar::<_, i64>(
        "SELECT max(projection_sequence)+1 FROM outbox_events WHERE workspace_id=$1",
    )
    .bind(workspace)
    .fetch_one(store.pool())
    .await
    .unwrap();
    index
        .apply(&[ProjectionMutation::Replace {
            workspace_id: workspace,
            document_id: document,
            source_kind: adoc_application::search::SearchSourceKind::Draft,
            sequence,
            regions: Vec::new(),
        }])
        .await
        .unwrap();
    assert_eq!(
        hits(
            &endpoint,
            &prefix,
            workspace,
            &scope,
            &new_fingerprint,
            "newvalue"
        )
        .await,
        0
    );
    store.close().await;
}

async fn execute_search_job(
    store: &PostgresStore,
    repository: &Arc<PostgresJobRepository>,
    executor: &WorkerJobExecutor,
    event: Uuid,
) {
    let id: Uuid = sqlx::query_scalar(
        "SELECT id FROM jobs WHERE kind='OUTBOX_TO_SEARCH' AND payload_json->>'outboxEventId'=$1",
    )
    .bind(event.to_string())
    .fetch_one(store.pool())
    .await
    .unwrap();
    let now = Utc::now();
    let job = repository
        .claim(id, "search-test", now, now + Duration::seconds(30))
        .await
        .unwrap()
        .unwrap();
    executor.execute(&job, "search-test", now).await.unwrap();
}

fn projection_identity(mutations: &[ProjectionMutation]) -> (String, String) {
    mutations
        .iter()
        .find_map(|mutation| match mutation {
            ProjectionMutation::Replace { regions, .. } => regions.first().map(|region| {
                (
                    region.permission_scope.clone(),
                    region.permission_fingerprint.clone(),
                )
            }),
            _ => None,
        })
        .unwrap()
}

async fn hits(
    endpoint: &Url,
    prefix: &str,
    workspace: Uuid,
    scope: &str,
    fingerprint: &str,
    text: &str,
) -> i64 {
    let response = reqwest::Client::new()
        .post(
            endpoint
                .join(&format!("{prefix}-draft-read/_search?routing={workspace}"))
                .unwrap(),
        )
        .json(&json!({"query":{"bool":{"filter":[
            {"term":{"workspace_id":workspace}},
            {"term":{"permission_scope":scope}},
            {"term":{"permission_fingerprint":fingerprint}},
            {"term":{"deleted":false}}
        ],"must":[{"match":{"body":text}}]}}}))
        .send()
        .await
        .unwrap();
    assert!(
        response.status().is_success(),
        "{}",
        response.text().await.unwrap()
    );
    let value: Value = response.json().await.unwrap();
    value["hits"]["total"]["value"].as_i64().unwrap()
}

async fn append(
    uow: &PgUnitOfWork,
    workspace: Uuid,
    aggregate: Uuid,
    sequence: i64,
    event_type: &'static str,
    payload: Value,
) -> Uuid {
    let id = Uuid::now_v7();
    uow.execute(|tx| {
        Box::pin(async move {
            append_outbox_event(
                tx,
                OutboxEventInput {
                    id,
                    workspace_id: workspace,
                    aggregate_kind: "SearchContract",
                    aggregate_id: aggregate,
                    sequence,
                    event_type,
                    event_version: 1,
                    payload,
                    audience: EventAudience::workspace(),
                    correlation_id: "search-projection-contract",
                    occurred_at: Utc::now(),
                },
            )
            .await
        })
    })
    .await
    .unwrap();
    id
}

async fn seed(store: &PostgresStore, owner: Uuid, workspace: Uuid, root: Uuid, document: Uuid) {
    sqlx::query("INSERT INTO users(id,google_subject,email,display_name) VALUES($1,$2,$3,'Owner')")
        .bind(owner)
        .bind(format!("subject-{owner}"))
        .bind(format!("{owner}@example.test"))
        .execute(store.pool())
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO workspaces(id,name,slug,created_by) VALUES($1,'Search','search-contract',$2)",
    )
    .bind(workspace)
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
    sqlx::query("INSERT INTO documents(id,workspace_id,parent_id,rank,title,created_by,permission_revision) VALUES($1,$2,NULL,$3,'Root',$4,1),($5,$2,$1,$6,'Search document',$4,1)")
        .bind(root).bind(workspace).bind("00000000000000000000000000000001").bind(owner)
        .bind(document).bind("00000000000000000000000000000002")
        .execute(store.pool()).await.unwrap();
    let block = Uuid::now_v7();
    sqlx::query("INSERT INTO drafts(id,workspace_id,document_id,content_json,schema_version,updated_by) VALUES($1,$2,$3,$4,1,$5)")
        .bind(Uuid::now_v7()).bind(workspace).bind(document)
        .bind(json!({"schemaVersion":1,"root":{"type":"doc","children":[{"id":block,"type":"paragraph","children":[{"type":"text","text":"oldsecret"}]}]}}))
        .bind(owner).execute(store.pool()).await.unwrap();
}

fn secret(plain: &str, file: &str) -> String {
    env::var(plain).unwrap_or_else(|_| {
        fs::read_to_string(env::var(file).unwrap())
            .unwrap()
            .trim()
            .to_owned()
    })
}
