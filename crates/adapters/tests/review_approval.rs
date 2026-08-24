use std::{env, fs, sync::Arc};

use adoc_adapters::{
    identity::{SystemClock, SystemSecureRandom},
    postgres::{
        DatabaseSettings, PostgresCollaborationRepository, PostgresPublishingRepository,
        PostgresStore,
    },
};
use adoc_application::{
    collaboration::{
        CollaborationService, ReviewDecisionInput, ReviewDecisionInputKind, ReviewStatus,
    },
    governance::GovernanceError,
    publishing::{PublishDocumentInput, PublishingService},
};
use serde_json::json;
use uuid::Uuid;

#[tokio::test]
#[ignore = "requires ADOC_TEST_DATABASE_URL pointing to an isolated PostgreSQL 16 database"]
async fn review_threshold_history_and_publish_gate_postgres_contract() {
    let store = PostgresStore::connect(DatabaseSettings {
        url: &secret("ADOC_TEST_DATABASE_URL", "ADOC_TEST_DATABASE_URL_FILE"),
        max_connections: 8,
        application_name: "adoc-review-contract",
    })
    .await
    .unwrap();
    store.migrate().await.unwrap();
    let requester = Uuid::now_v7();
    let reviewer = Uuid::now_v7();
    let workspace = Uuid::now_v7();
    let document = Uuid::now_v7();
    seed(&store, requester, reviewer, workspace, document).await;
    let collaboration = CollaborationService::new(
        Arc::new(PostgresCollaborationRepository::new(&store)),
        Arc::new(SystemClock),
        Arc::new(SystemSecureRandom),
    );
    let publishing = PublishingService::new(
        Arc::new(PostgresPublishingRepository::new(&store)),
        Arc::new(SystemClock),
        Arc::new(SystemSecureRandom),
    );

    let requested = collaboration
        .request_review(requester, workspace, document, 0, "review-request-0001")
        .await
        .unwrap();
    assert_eq!(requested.status, ReviewStatus::Requested);
    assert_eq!(requested.assignments.len(), 1);
    assert_eq!(requested.assignments[0].reviewer_id, reviewer);
    assert!(matches!(
        collaboration
            .submit_review_decision(
                requester,
                workspace,
                requested.id,
                0,
                "review-self-000001",
                ReviewDecisionInput {
                    decision: ReviewDecisionInputKind::Approve,
                    discussion_id: None
                }
            )
            .await,
        Err(GovernanceError::ReviewNotEligible)
    ));
    let approved = collaboration
        .submit_review_decision(
            reviewer,
            workspace,
            requested.id,
            0,
            "review-approve-0001",
            ReviewDecisionInput {
                decision: ReviewDecisionInputKind::Approve,
                discussion_id: None,
            },
        )
        .await
        .unwrap();
    assert_eq!(approved.status, ReviewStatus::Approved);
    assert_eq!(approved.revision, 1);
    assert!(matches!(
        collaboration
            .submit_review_decision(
                reviewer,
                workspace,
                requested.id,
                0,
                "review-stale-00001",
                ReviewDecisionInput {
                    decision: ReviewDecisionInputKind::Approve,
                    discussion_id: None,
                },
            )
            .await,
        Err(GovernanceError::RevisionConflict {
            current_revision: 1
        })
    ));
    assert!(
        sqlx::query("UPDATE review_decision_revisions SET revision=2 WHERE review_id=$1")
            .bind(requested.id)
            .execute(store.pool())
            .await
            .is_err()
    );
    let version = publishing
        .publish(
            requester,
            workspace,
            document,
            0,
            PublishDocumentInput {
                summary: "Reviewed publication".into(),
                client_instance_id: None,
                lease_token: None,
            },
            "reviewed-publish001",
        )
        .await
        .unwrap();
    assert_eq!(
        version.review_snapshot["reviewId"],
        requested.id.to_string()
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT count(*) FROM review_decision_revisions WHERE review_id=$1"
        )
        .bind(requested.id)
        .fetch_one(store.pool())
        .await
        .unwrap(),
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

async fn seed(
    store: &PostgresStore,
    requester: Uuid,
    reviewer: Uuid,
    workspace: Uuid,
    document: Uuid,
) {
    for (user, name) in [(requester, "Requester"), (reviewer, "Reviewer")] {
        sqlx::query("INSERT INTO users(id,google_subject,email,display_name) VALUES($1,$2,$3,$4)")
            .bind(user)
            .bind(user.to_string())
            .bind(format!("{user}@example.test"))
            .bind(name)
            .execute(store.pool())
            .await
            .unwrap();
    }
    sqlx::query("INSERT INTO workspaces(id,slug,name,created_by) VALUES($1,$2,'Review Test',$3)")
        .bind(workspace)
        .bind(format!("review-{workspace}"))
        .bind(requester)
        .execute(store.pool())
        .await
        .unwrap();
    for (user, role) in [(requester, "OWNER"), (reviewer, "MEMBER")] {
        sqlx::query("INSERT INTO memberships(workspace_id,user_id,role,status) VALUES($1,$2,$3::membership_role,'ACTIVE')").bind(workspace).bind(user).bind(role).execute(store.pool()).await.unwrap();
    }
    sqlx::query("INSERT INTO documents(id,workspace_id,rank,title,created_by) VALUES($1,$2,'00000000000000000000000000000001','Review Document',$3)").bind(document).bind(workspace).bind(requester).execute(store.pool()).await.unwrap();
    for user in [requester, reviewer] {
        sqlx::query("INSERT INTO permission_grants(id,workspace_id,document_id,subject_kind,subject_id,access,can_manage,granted_by) VALUES($1,$2,$3,'USER',$4,'EDITOR',true,$5)").bind(Uuid::now_v7()).bind(workspace).bind(document).bind(user).bind(requester).execute(store.pool()).await.unwrap();
    }
    sqlx::query("INSERT INTO publish_policies(document_id,workspace_id,mode,required_approvals,reviewer_rule,updated_by) VALUES($1,$2,'REVIEW_REQUIRED',1,$3,$4)").bind(document).bind(workspace).bind(json!({"kind":"ANY_EDITOR"})).bind(requester).execute(store.pool()).await.unwrap();
    let content = json!({"schemaVersion":1,"root":{"type":"doc","children":[{"id":Uuid::now_v7(),"type":"paragraph","children":[{"type":"text","text":"reviewed"}]}]}});
    sqlx::query("INSERT INTO drafts(id,workspace_id,document_id,content_json,schema_version,revision,updated_by) VALUES($1,$2,$3,$4,1,0,$5)").bind(Uuid::now_v7()).bind(workspace).bind(document).bind(content).bind(requester).execute(store.pool()).await.unwrap();
}
