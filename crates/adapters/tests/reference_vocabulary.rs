use std::{env, fs, sync::Arc};

use adoc_adapters::{
    identity::{SystemClock, SystemSecureRandom},
    postgres::{
        DatabaseSettings, PostgresDocumentRepository, PostgresKnowledgeRepository, PostgresStore,
    },
};
use adoc_application::{
    document::{
        AcquireLeaseInput, CreateDocumentInput, DocumentService, OperationScope, ReferenceTarget,
    },
    governance::GovernanceError,
    knowledge::{
        CreateReferenceInput, DeprecateVocabularyConceptInput, KnowledgeService, VocabularyStatus,
        VocabularyTerm, VocabularyTermKind, WriteVocabularyConceptInput,
    },
};
use uuid::Uuid;

#[tokio::test]
#[ignore = "requires ADOC_TEST_DATABASE_URL pointing to an isolated PostgreSQL 16 database"]
async fn reference_and_vocabulary_postgres_contract() {
    let store = PostgresStore::connect(DatabaseSettings {
        url: &secret("ADOC_TEST_DATABASE_URL", "ADOC_TEST_DATABASE_URL_FILE"),
        max_connections: 8,
        application_name: "adoc-knowledge-contract",
    })
    .await
    .unwrap();
    store.migrate().await.unwrap();
    let owner = Uuid::now_v7();
    let viewer = Uuid::now_v7();
    let workspace = Uuid::now_v7();
    seed(&store, owner, viewer, workspace).await;
    let documents = Arc::new(DocumentService::new(
        Arc::new(PostgresDocumentRepository::new(&store)),
        Arc::new(SystemClock),
        Arc::new(SystemSecureRandom),
    ));
    let knowledge = KnowledgeService::new(
        Arc::new(PostgresKnowledgeRepository::new(&store)),
        documents.clone(),
        Arc::new(SystemClock),
        Arc::new(SystemSecureRandom),
    );
    let source = documents
        .create(
            owner,
            workspace,
            CreateDocumentInput {
                title: "Source".into(),
                parent_id: None,
                after_document_id: None,
            },
            "knowledge-source-001",
        )
        .await
        .unwrap();
    let target = documents
        .create(
            owner,
            workspace,
            CreateDocumentInput {
                title: "Target".into(),
                parent_id: None,
                after_document_id: Some(source.id),
            },
            "knowledge-target-001",
        )
        .await
        .unwrap();
    let draft = documents
        .create_draft(owner, workspace, source.id, "knowledge-draft-001")
        .await
        .unwrap();
    let client = Uuid::now_v7();
    let lease = documents
        .acquire_lease(
            owner,
            workspace,
            source.id,
            source.revision,
            AcquireLeaseInput {
                client_instance_id: client,
                force: false,
                reason: None,
            },
            "knowledge-lease-001",
        )
        .await
        .unwrap();
    let token = lease.token.unwrap();
    let reference = knowledge
        .create_reference(
            owner,
            workspace,
            source.id,
            client,
            draft.revision,
            &token,
            "knowledge-reference-create-001",
            CreateReferenceInput {
                source_region: OperationScope::Document,
                target: ReferenceTarget {
                    kind: "DOCUMENT".into(),
                    id: target.id.to_string(),
                    region: None,
                },
            },
        )
        .await
        .unwrap();
    let reference_replay = knowledge
        .create_reference(
            owner,
            workspace,
            source.id,
            client,
            draft.revision,
            &token,
            "knowledge-reference-create-001",
            CreateReferenceInput {
                source_region: OperationScope::Document,
                target: ReferenceTarget {
                    kind: "DOCUMENT".into(),
                    id: target.id.to_string(),
                    region: None,
                },
            },
        )
        .await
        .unwrap();
    assert_eq!(reference_replay.id, reference.id);
    assert_eq!(reference.snapshot.title, "Target");
    assert_eq!(
        documents
            .draft(owner, workspace, source.id)
            .await
            .unwrap()
            .revision,
        1
    );
    for (document, access) in [(target.id, "VIEWER"), (source.id, "NO_ACCESS")] {
        sqlx::query("INSERT INTO permission_grants(id,workspace_id,document_id,subject_kind,subject_id,access,can_manage,granted_by) VALUES($1,$2,$3,'USER',$4,$5::document_access,false,$6)").bind(Uuid::now_v7()).bind(workspace).bind(document).bind(viewer).bind(access).bind(owner).execute(store.pool()).await.unwrap();
    }
    assert!(
        knowledge
            .list_backlinks(viewer, workspace, target.id, None)
            .await
            .unwrap()
            .items
            .is_empty()
    );
    knowledge
        .delete_reference(
            owner,
            workspace,
            source.id,
            reference.id,
            client,
            1,
            &token,
            "knowledge-reference-delete-001",
        )
        .await
        .unwrap();
    knowledge
        .delete_reference(
            owner,
            workspace,
            source.id,
            reference.id,
            client,
            1,
            &token,
            "knowledge-reference-delete-001",
        )
        .await
        .unwrap();
    assert!(
        sqlx::query_scalar::<_, Option<chrono::DateTime<chrono::Utc>>>(
            "SELECT deleted_at FROM references_graph WHERE id=$1"
        )
        .bind(reference.id)
        .fetch_one(store.pool())
        .await
        .unwrap()
        .is_some()
    );
    let external = knowledge
        .create_reference(
            owner,
            workspace,
            source.id,
            client,
            2,
            &token,
            "knowledge-reference-create-002",
            CreateReferenceInput {
                source_region: OperationScope::Document,
                target: ReferenceTarget {
                    kind: "EXTERNAL".into(),
                    id: "https://EXAMPLE.COM/guide".into(),
                    region: None,
                },
            },
        )
        .await
        .unwrap();
    assert_eq!(external.target["id"], "https://example.com/guide");
    assert!(matches!(
        knowledge
            .get_vocabulary(owner, workspace, Uuid::now_v7())
            .await,
        Err(GovernanceError::VocabularyNotFound)
    ));

    let first = knowledge
        .create_vocabulary(
            owner,
            workspace,
            "knowledge-vocabulary-create-01",
            write("CAFÉ 정책", "정책 정의"),
        )
        .await
        .unwrap();
    assert_eq!(first.revision, 0);
    assert!(matches!(
        knowledge
            .create_vocabulary(
                owner,
                workspace,
                "knowledge-vocabulary-create-02",
                write("cafe\u{301}   정책", "중복 정의")
            )
            .await,
        Err(GovernanceError::VocabularyTermConflict)
    ));
    let updated = knowledge
        .update_vocabulary(
            owner,
            workspace,
            first.id,
            0,
            "knowledge-vocabulary-update-01",
            write("CAFÉ 정책", "갱신 정의"),
        )
        .await
        .unwrap();
    assert_eq!(updated.revision, 1);
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT count(*) FROM vocabulary_concept_revisions WHERE concept_id=$1"
        )
        .bind(first.id)
        .fetch_one(store.pool())
        .await
        .unwrap(),
        1
    );
    assert!(
        sqlx::query("DELETE FROM vocabulary_concept_revisions WHERE concept_id=$1")
            .bind(first.id)
            .execute(store.pool())
            .await
            .is_err()
    );
    let deprecated = knowledge
        .deprecate_vocabulary(
            owner,
            workspace,
            first.id,
            1,
            "knowledge-vocabulary-deprecate-01",
            DeprecateVocabularyConceptInput {
                reason: "대체 용어 적용".into(),
                replacement_concept_id: None,
            },
        )
        .await
        .unwrap();
    assert_eq!(deprecated.status, VocabularyStatus::Deprecated);
    assert_eq!(sqlx::query_scalar::<_,Option<String>>("SELECT change_reason FROM vocabulary_concept_revisions WHERE concept_id=$1 AND revision=2").bind(first.id).fetch_one(store.pool()).await.unwrap().as_deref(),Some("대체 용어 적용"));
    store.close().await;
}

fn write(term: &str, definition: &str) -> WriteVocabularyConceptInput {
    WriteVocabularyConceptInput {
        canonical_term: term.into(),
        definition: definition.into(),
        terms: vec![VocabularyTerm {
            term: term.into(),
            kind: VocabularyTermKind::Canonical,
        }],
    }
}
fn secret(value_key: &str, file_key: &str) -> String {
    if let Ok(value) = env::var(value_key) {
        return value;
    }
    let path =
        env::var(file_key).unwrap_or_else(|_| panic!("{value_key} or {file_key} is required"));
    fs::read_to_string(path).unwrap().trim().to_owned()
}
async fn seed(store: &PostgresStore, owner: Uuid, viewer: Uuid, workspace: Uuid) {
    for (user, name) in [(owner, "Owner"), (viewer, "Viewer")] {
        sqlx::query("INSERT INTO users(id,google_subject,email,display_name) VALUES($1,$2,$3,$4)")
            .bind(user)
            .bind(user.to_string())
            .bind(format!("{user}@example.test"))
            .bind(name)
            .execute(store.pool())
            .await
            .unwrap();
    }
    sqlx::query(
        "INSERT INTO workspaces(id,slug,name,created_by) VALUES($1,$2,'Knowledge Test',$3)",
    )
    .bind(workspace)
    .bind(format!("knowledge-{workspace}"))
    .bind(owner)
    .execute(store.pool())
    .await
    .unwrap();
    for (user, role) in [(owner, "OWNER"), (viewer, "MEMBER")] {
        sqlx::query("INSERT INTO memberships(workspace_id,user_id,role,status) VALUES($1,$2,$3::membership_role,'ACTIVE')").bind(workspace).bind(user).bind(role).execute(store.pool()).await.unwrap();
    }
}
