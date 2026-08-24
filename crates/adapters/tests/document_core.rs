use std::{env, fs, sync::Arc};

use adoc_adapters::{
    identity::{SystemClock, SystemSecureRandom},
    postgres::{DatabaseSettings, PostgresDocumentRepository, PostgresStore},
};
use adoc_application::{
    document::{
        AcquireLeaseInput, ApplyOperationsInput, ApplyOperationsRequest, CreateDocumentInput,
        DocumentOperation, DocumentService, LeaseCommandRequest, MoveDocumentInput,
    },
    governance::GovernanceError,
};
use serde_json::json;
use tokio::sync::Barrier;
use uuid::Uuid;

#[tokio::test]
#[ignore = "requires ADOC_TEST_DATABASE_URL pointing to an isolated PostgreSQL 16 database"]
async fn document_tree_draft_lease_postgres_contract() {
    let store = PostgresStore::connect(DatabaseSettings {
        url: &secret("ADOC_TEST_DATABASE_URL", "ADOC_TEST_DATABASE_URL_FILE"),
        max_connections: 8,
        application_name: "adoc-document-contract",
    })
    .await
    .unwrap();
    store.migrate().await.unwrap();
    let actor = Uuid::now_v7();
    let workspace = Uuid::now_v7();
    seed(&store, actor, workspace).await;
    let service = DocumentService::new(
        Arc::new(PostgresDocumentRepository::new(&store)),
        Arc::new(SystemClock),
        Arc::new(SystemSecureRandom),
    );

    let root = service
        .create(
            actor,
            workspace,
            CreateDocumentInput {
                title: "Root".into(),
                parent_id: None,
                after_document_id: None,
            },
            "document-create-1",
        )
        .await
        .unwrap();
    let child = service
        .create(
            actor,
            workspace,
            CreateDocumentInput {
                title: "Child".into(),
                parent_id: Some(root.id),
                after_document_id: None,
            },
            "document-create-2",
        )
        .await
        .unwrap();
    let tree = service.tree(actor, workspace).await.unwrap();
    assert_eq!(tree.nodes[0].children[0].document.id, child.id);
    assert!(matches!(
        service
            .preview_move(
                actor,
                workspace,
                root.id,
                root.revision,
                MoveDocumentInput {
                    new_parent_id: Some(child.id),
                    after_document_id: None,
                },
            )
            .await,
        Err(GovernanceError::DocumentTreeCycle)
    ));

    let draft = service
        .create_draft(actor, workspace, child.id, "document-draft-1")
        .await
        .unwrap();
    let client = Uuid::now_v7();
    let lease = service
        .acquire_lease(
            actor,
            workspace,
            child.id,
            child.revision,
            AcquireLeaseInput {
                client_instance_id: client,
                force: false,
                reason: None,
            },
            "document-lease-1",
        )
        .await
        .unwrap();
    let token = lease.token.clone().unwrap();
    let block = Uuid::now_v7();
    let operation: DocumentOperation = serde_json::from_value(json!({
        "opId":Uuid::now_v7(),"kind":"INSERT_BLOCK","scope":{"kind":"DOCUMENT"},
        "precondition":{"draftRevision":0,"targetHash":null},"dependsOn":[],
        "parentId":null,"index":0,
        "block":{"id":block,"type":"paragraph","children":[{"type":"text","text":"hello"}]}
    }))
    .unwrap();
    let result = service
        .apply_operations(ApplyOperationsRequest {
            actor_id: actor,
            workspace_id: workspace,
            document_id: child.id,
            client_instance_id: client,
            expected_revision: draft.revision,
            token: &token,
            input: ApplyOperationsInput {
                operations: vec![operation],
            },
            idempotency_key: "document-save-001",
        })
        .await
        .unwrap();
    assert_eq!(result.revision, 1);
    let replay = service
        .create_draft(actor, workspace, child.id, "document-draft-1")
        .await
        .unwrap();
    assert_eq!(replay.revision, 0);
    let renewed = service
        .mutate_lease(LeaseCommandRequest {
            actor_id: actor,
            workspace_id: workspace,
            document_id: child.id,
            client_instance_id: client,
            expected_revision: lease.revision,
            token: &token,
            release: false,
            idempotency_key: "document-renew-01",
        })
        .await
        .unwrap()
        .unwrap();
    service
        .mutate_lease(LeaseCommandRequest {
            actor_id: actor,
            workspace_id: workspace,
            document_id: child.id,
            client_instance_id: client,
            expected_revision: renewed.revision,
            token: &token,
            release: true,
            idempotency_key: "document-release1",
        })
        .await
        .unwrap();

    let barrier = Arc::new(Barrier::new(3));
    let first_service = service.clone();
    let first_barrier = barrier.clone();
    let first = tokio::spawn(async move {
        first_barrier.wait().await;
        first_service
            .acquire_lease(
                actor,
                workspace,
                child.id,
                child.revision,
                AcquireLeaseInput {
                    client_instance_id: Uuid::now_v7(),
                    force: false,
                    reason: None,
                },
                "document-race-lease-01",
            )
            .await
    });
    let second_service = service.clone();
    let second_barrier = barrier.clone();
    let second = tokio::spawn(async move {
        second_barrier.wait().await;
        second_service
            .acquire_lease(
                actor,
                workspace,
                child.id,
                child.revision,
                AcquireLeaseInput {
                    client_instance_id: Uuid::now_v7(),
                    force: false,
                    reason: None,
                },
                "document-race-lease-02",
            )
            .await
    });
    barrier.wait().await;
    let outcomes = [first.await.unwrap(), second.await.unwrap()];
    assert_eq!(outcomes.iter().filter(|result| result.is_ok()).count(), 1);
    assert_eq!(
        outcomes
            .iter()
            .filter(|result| matches!(result, Err(GovernanceError::EditLeaseHeld { .. })))
            .count(),
        1
    );

    store.close().await;
}

fn secret(value_key: &str, file_key: &str) -> String {
    if let Ok(value) = env::var(value_key) {
        return value;
    }
    let path =
        env::var(file_key).unwrap_or_else(|_| panic!("{value_key} or {file_key} is required"));
    fs::read_to_string(path).unwrap().trim().to_owned()
}

async fn seed(store: &PostgresStore, actor: Uuid, workspace: Uuid) {
    sqlx::query(
        "INSERT INTO users(id,google_subject,email,display_name) VALUES($1,$2,$3,'Document Test')",
    )
    .bind(actor)
    .bind(actor.to_string())
    .bind(format!("{actor}@example.test"))
    .execute(store.pool())
    .await
    .unwrap();
    sqlx::query("INSERT INTO workspaces(id,slug,name,created_by) VALUES($1,$2,'Document Test',$3)")
        .bind(workspace)
        .bind(format!("doc-{workspace}"))
        .bind(actor)
        .execute(store.pool())
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO memberships(workspace_id,user_id,role,status) VALUES($1,$2,'OWNER','ACTIVE')",
    )
    .bind(workspace)
    .bind(actor)
    .execute(store.pool())
    .await
    .unwrap();
}
