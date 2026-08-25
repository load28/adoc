use std::{env, fs, sync::Arc};

use adoc_adapters::{
    identity::{SystemClock, SystemSecureRandom},
    postgres::{
        DatabaseSettings, PostgresDocumentRepository, PostgresStore,
        PostgresWritingIntelligenceRepository,
    },
};
use adoc_application::{
    ai::{
        ApplyProposalInput, ApplyProposalRequest, ProposalStatus, WritingConfigurationInput,
        WritingIntelligenceService,
    },
    document::{
        AcquireLeaseInput, DocumentOperation, DocumentService, OperationBase,
        OperationPrecondition, OperationScope,
    },
};
use serde_json::json;
use uuid::Uuid;

#[tokio::test]
#[ignore = "requires ADOC_TEST_DATABASE_URL pointing to an isolated PostgreSQL 16 database"]
async fn proposal_apply_is_atomic_dependency_closed_and_idempotent() {
    let store = connect().await;
    let (owner, workspace, document) = seed(&store).await;
    let document_service = DocumentService::new(
        Arc::new(PostgresDocumentRepository::new(&store)),
        Arc::new(SystemClock),
        Arc::new(SystemSecureRandom),
    );
    let client = Uuid::now_v7();
    let lease = document_service
        .acquire_lease(
            owner,
            workspace,
            document,
            0,
            AcquireLeaseInput {
                client_instance_id: client,
                force: false,
                reason: None,
            },
            "proposal-lease-key-001",
        )
        .await
        .unwrap();
    let token = lease.token.expect("lease token");
    let first = Uuid::now_v7();
    let second = Uuid::now_v7();
    let operations = vec![
        insert_operation(first, 1, Vec::new()),
        insert_operation(second, 2, vec![first]),
    ];
    let proposal = insert_proposal(&store, owner, workspace, document, &operations).await;
    let service = WritingIntelligenceService::new(
        Arc::new(PostgresWritingIntelligenceRepository::new(&store)),
        Arc::new(SystemClock),
    );
    let invalid = service
        .apply_proposal(ApplyProposalRequest {
            actor_id: owner,
            workspace_id: workspace,
            proposal_id: proposal,
            client_instance_id: client,
            expected_revision: 0,
            token: &token,
            input: ApplyProposalInput {
                operation_ids: Some(vec![second]),
            },
            idempotency_key: "proposal-apply-invalid-001",
        })
        .await;
    assert!(matches!(
        invalid,
        Err(adoc_application::governance::GovernanceError::ProposalDependencyInvalid)
    ));
    let request = || ApplyProposalRequest {
        actor_id: owner,
        workspace_id: workspace,
        proposal_id: proposal,
        client_instance_id: client,
        expected_revision: 0,
        token: &token,
        input: ApplyProposalInput {
            operation_ids: None,
        },
        idempotency_key: "proposal-apply-valid-0001",
    };
    let applied = service.apply_proposal(request()).await.unwrap();
    let replay = service.apply_proposal(request()).await.unwrap();
    assert_eq!(applied.revision, 1);
    assert_eq!(applied.applied_operation_ids, vec![first, second]);
    assert_eq!(applied.inverse_operations.len(), 2);
    assert_eq!(replay.content_fingerprint, applied.content_fingerprint);
    let view = service
        .get_proposal(owner, workspace, proposal)
        .await
        .unwrap();
    assert_eq!(view.status, ProposalStatus::Applied);
    assert_eq!(view.applied_revision, Some(1));
    let stale_operations = vec![insert_operation(Uuid::now_v7(), 3, Vec::new())];
    let stale_proposal =
        insert_proposal(&store, owner, workspace, document, &stale_operations).await;
    let stale = service
        .apply_proposal(ApplyProposalRequest {
            actor_id: owner,
            workspace_id: workspace,
            proposal_id: stale_proposal,
            client_instance_id: client,
            expected_revision: 0,
            token: &token,
            input: ApplyProposalInput {
                operation_ids: None,
            },
            idempotency_key: "proposal-apply-stale-0001",
        })
        .await;
    assert!(matches!(
        stale,
        Err(adoc_application::governance::GovernanceError::ProposalStale)
    ));
    let draft_revision: i64 =
        sqlx::query_scalar("SELECT revision FROM drafts WHERE document_id=$1")
            .bind(document)
            .fetch_one(store.pool())
            .await
            .unwrap();
    assert_eq!(draft_revision, 1);
    store.close().await;
}

#[tokio::test]
#[ignore = "requires ADOC_TEST_DATABASE_URL pointing to an isolated PostgreSQL 16 database"]
async fn writing_configuration_has_a_closed_versioned_registry() {
    let store = connect().await;
    let (owner, workspace, _) = seed(&store).await;
    let service = WritingIntelligenceService::new(
        Arc::new(PostgresWritingIntelligenceRepository::new(&store)),
        Arc::new(SystemClock),
    );
    let baseline = service
        .get_writing_configuration(owner, workspace)
        .await
        .unwrap();
    assert_eq!(baseline.revision, 0);
    let updated = service
        .update_writing_configuration(
            owner,
            workspace,
            0,
            WritingConfigurationInput {
                baseline_version: "writing-rules-v1".to_owned(),
                overrides: Vec::new(),
            },
            "writing-config-update-001",
        )
        .await
        .unwrap();
    assert_eq!(updated.revision, 1);
    assert!(
        service
            .update_writing_configuration(
                owner,
                workspace,
                0,
                WritingConfigurationInput {
                    baseline_version: "writing-rules-v1".to_owned(),
                    overrides: Vec::new()
                },
                "writing-config-stale-0001"
            )
            .await
            .is_err()
    );
    store.close().await;
}

async fn connect() -> PostgresStore {
    let store = PostgresStore::connect(DatabaseSettings {
        url: &secret("ADOC_TEST_DATABASE_URL", "ADOC_TEST_DATABASE_URL_FILE"),
        max_connections: 8,
        application_name: "adoc-ai-proposal-test",
    })
    .await
    .unwrap();
    store.migrate().await.unwrap();
    store
}

async fn seed(store: &PostgresStore) -> (Uuid, Uuid, Uuid) {
    let owner = Uuid::now_v7();
    let workspace = Uuid::now_v7();
    let document = Uuid::now_v7();
    let block = Uuid::now_v7();
    sqlx::query("INSERT INTO users(id,google_subject,email,display_name) VALUES($1,$2,$3,'Owner')")
        .bind(owner)
        .bind(format!("subject-{owner}"))
        .bind(format!("{owner}@example.test"))
        .execute(store.pool())
        .await
        .unwrap();
    sqlx::query("INSERT INTO workspaces(id,name,slug,created_by) VALUES($1,'Proposal',$2,$3)")
        .bind(workspace)
        .bind(format!("proposal-{workspace}"))
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
    sqlx::query("INSERT INTO documents(id,workspace_id,rank,title,created_by,permission_revision) VALUES($1,$2,'00000000000000000000000000000001','Document',$3,1)").bind(document).bind(workspace).bind(owner).execute(store.pool()).await.unwrap();
    sqlx::query("INSERT INTO permission_grants(id,workspace_id,document_id,subject_kind,subject_id,access,can_manage,granted_by) VALUES($1,$2,$3,'USER',$4,'EDITOR',true,$4)").bind(Uuid::now_v7()).bind(workspace).bind(document).bind(owner).execute(store.pool()).await.unwrap();
    sqlx::query("INSERT INTO drafts(id,workspace_id,document_id,content_json,schema_version,updated_by) VALUES($1,$2,$3,$4,1,$5)").bind(Uuid::now_v7()).bind(workspace).bind(document).bind(json!({"schemaVersion":1,"root":{"type":"doc","children":[{"id":block,"type":"paragraph","children":[{"type":"text","text":"base"}]}]}})).bind(owner).execute(store.pool()).await.unwrap();
    (owner, workspace, document)
}

fn insert_operation(id: Uuid, index: usize, depends_on: Vec<Uuid>) -> DocumentOperation {
    DocumentOperation::InsertBlock {
        base: OperationBase {
            op_id: id,
            scope: OperationScope::Document,
            precondition: OperationPrecondition {
                draft_revision: 0,
                target_hash: None,
            },
            depends_on,
        },
        parent_id: None,
        index,
        block: json!({"id":Uuid::now_v7(),"type":"paragraph","children":[{"type":"text","text":format!("added-{index}")}]}),
    }
}

async fn insert_proposal(
    store: &PostgresStore,
    owner: Uuid,
    workspace: Uuid,
    document: Uuid,
    operations: &[DocumentOperation],
) -> Uuid {
    let job = Uuid::now_v7();
    let proposal = Uuid::now_v7();
    sqlx::query("INSERT INTO ai_jobs(id,workspace_id,user_id,kind,target_json,expected_revision,context_fingerprint,context_metadata_json,request_key,status,priority,provider,model,created_at,completed_at) VALUES($1,$2,$3,'COMPOSE',$4,0,$5,$6,$7,'SUCCEEDED',50,'CODEX_CLI','test',now(),now())").bind(job).bind(workspace).bind(owner).bind(json!({"kind":"COMPOSE","workspaceId":workspace,"actorId":owner,"target":{"kind":"DOCUMENT","documentId":document},"expectedRevision":0,"externalWebEnabled":false,"instruction":null})).bind("a".repeat(64)).bind(json!({"writingRuleVersion":"writing-rules-v1:0","vocabularyRevision":0})).bind(format!("proposal-{job}")).execute(store.pool()).await.unwrap();
    sqlx::query("INSERT INTO proposals(id,workspace_id,job_id,document_id,owner_user_id,base_revision,operations_json,writing_rule_version,vocabulary_revision,validation_json) VALUES($1,$2,$3,$4,$5,0,$6,'writing-rules-v1:0',0,'{}'::jsonb)").bind(proposal).bind(workspace).bind(job).bind(document).bind(owner).bind(serde_json::to_value(operations).unwrap()).execute(store.pool()).await.unwrap();
    proposal
}

fn secret(plain: &str, file: &str) -> String {
    env::var(plain).unwrap_or_else(|_| {
        fs::read_to_string(env::var(file).unwrap())
            .unwrap()
            .trim()
            .to_owned()
    })
}
