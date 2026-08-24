use std::{env, fs, sync::Arc};

use adoc_adapters::{
    identity::{SystemClock, SystemSecureRandom},
    postgres::{DatabaseSettings, PostgresPublishingRepository, PostgresStore},
};
use adoc_application::{
    governance::GovernanceError,
    publishing::{CreatePublicLinkInput, PublishDocumentInput, PublishingService},
};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::{Duration, Utc};
use serde_json::json;
use sha2::{Digest, Sha256};
use tokio::sync::Barrier;
use uuid::Uuid;

#[tokio::test]
#[ignore = "requires ADOC_TEST_DATABASE_URL pointing to an isolated PostgreSQL 16 database"]
async fn publish_version_public_link_postgres_contract() {
    let store = PostgresStore::connect(DatabaseSettings {
        url: &secret("ADOC_TEST_DATABASE_URL", "ADOC_TEST_DATABASE_URL_FILE"),
        max_connections: 8,
        application_name: "adoc-publishing-contract",
    })
    .await
    .unwrap();
    store.migrate().await.unwrap();
    let actor = Uuid::now_v7();
    let workspace = Uuid::now_v7();
    let document = Uuid::now_v7();
    seed(&store, actor, workspace, document).await;
    let service = PublishingService::new(
        Arc::new(PostgresPublishingRepository::new(&store)),
        Arc::new(SystemClock),
        Arc::new(SystemSecureRandom),
    );

    let first = service
        .publish(
            actor,
            workspace,
            document,
            0,
            PublishDocumentInput {
                summary: "First publication".into(),
                client_instance_id: None,
                lease_token: None,
            },
            "publish-version-0001",
        )
        .await
        .unwrap();
    assert_eq!(first.number, 1);
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM drafts WHERE document_id=$1")
            .bind(document)
            .fetch_one(store.pool())
            .await
            .unwrap(),
        0
    );
    assert!(
        sqlx::query("UPDATE published_versions SET summary='mutated' WHERE id=$1")
            .bind(first.id)
            .execute(store.pool())
            .await
            .is_err()
    );

    let link_input = CreatePublicLinkInput {
        expires_at: Some(Utc::now() + Duration::days(1)),
    };
    let link = service
        .create_public_link(
            actor,
            workspace,
            document,
            1,
            link_input.clone(),
            "public-link-create1",
        )
        .await
        .unwrap();
    assert!(matches!(
        service
            .create_public_link(
                actor,
                workspace,
                document,
                1,
                link_input,
                "public-link-create1"
            )
            .await,
        Err(GovernanceError::PublicLinkTokenAlreadyIssued)
    ));
    let public = service.public_document(&link.token).await.unwrap();
    assert_eq!(public.version_number, 1);

    let restored = service
        .restore_version(
            actor,
            workspace,
            document,
            first.id,
            1,
            "restore-version-0001",
        )
        .await
        .unwrap();
    let changed = json!({"schemaVersion":1,"root":{"type":"doc","children":[{"id":Uuid::now_v7(),"type":"paragraph","children":[{"type":"text","text":"changed"}]}]}});
    sqlx::query("UPDATE drafts SET content_json=$2,revision=revision+1 WHERE id=$1")
        .bind(restored.id)
        .bind(changed)
        .execute(store.pool())
        .await
        .unwrap();
    let second = service
        .publish(
            actor,
            workspace,
            document,
            1,
            PublishDocumentInput {
                summary: "Second publication".into(),
                client_instance_id: None,
                lease_token: None,
            },
            "publish-version-0002",
        )
        .await
        .unwrap();
    assert_eq!(second.number, 2);
    assert_eq!(
        service
            .public_document(&link.token)
            .await
            .unwrap()
            .version_number,
        2
    );
    let history = service
        .list_versions(actor, workspace, document, None)
        .await
        .unwrap();
    assert_eq!(
        history
            .items
            .iter()
            .map(|item| item.number)
            .collect::<Vec<_>>(),
        vec![2, 1]
    );
    assert_eq!(
        service
            .compare_versions(actor, workspace, document, first.id, second.id)
            .await
            .unwrap()
            .operations
            .len(),
        1
    );

    service
        .revoke_public_link(
            actor,
            workspace,
            document,
            link.id,
            0,
            "public-link-revoke1",
        )
        .await
        .unwrap();
    assert!(matches!(
        service.public_document(&link.token).await,
        Err(GovernanceError::PublicLinkInvalid)
    ));
    let expired_token = URL_SAFE_NO_PAD.encode([7_u8; 32]);
    let expired_hash = Sha256::digest(expired_token.as_bytes());
    let created_at = Utc::now() - Duration::days(2);
    sqlx::query("INSERT INTO public_links(id,workspace_id,document_id,token_hash,expires_at,created_by,created_at) VALUES($1,$2,$3,$4,$5,$6,$7)")
        .bind(Uuid::now_v7()).bind(workspace).bind(document).bind(expired_hash.to_vec()).bind(created_at+Duration::days(1)).bind(actor).bind(created_at).execute(store.pool()).await.unwrap();
    assert!(matches!(
        service.public_document(&expired_token).await,
        Err(GovernanceError::PublicLinkInvalid)
    ));

    let outsider = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO users(id,google_subject,email,display_name) VALUES($1,$2,$3,'Outsider')",
    )
    .bind(outsider)
    .bind(outsider.to_string())
    .bind(format!("{outsider}@example.test"))
    .execute(store.pool())
    .await
    .unwrap();
    assert!(
        service
            .list_versions(outsider, workspace, document, None)
            .await
            .is_err()
    );

    let stale = service
        .restore_version(
            actor,
            workspace,
            document,
            first.id,
            2,
            "restore-version-stale",
        )
        .await
        .unwrap();
    assert!(matches!(
        service
            .publish(
                actor,
                workspace,
                document,
                stale.revision,
                PublishDocumentInput {
                    summary: "Must conflict".into(),
                    client_instance_id: None,
                    lease_token: None
                },
                "publish-version-stale"
            )
            .await,
        Err(GovernanceError::PublishBaseStale { .. })
    ));
    let trash_link = service
        .create_public_link(
            actor,
            workspace,
            document,
            2,
            CreatePublicLinkInput { expires_at: None },
            "public-link-trash01",
        )
        .await
        .unwrap();
    sqlx::query("UPDATE documents SET status='TRASHED',trashed_at=now(),purge_after=now()+interval '30 days' WHERE id=$1")
        .bind(document).execute(store.pool()).await.unwrap();
    assert!(matches!(
        service.public_document(&trash_link.token).await,
        Err(GovernanceError::PublicLinkInvalid)
    ));
    let secret_column_count: i64 = sqlx::query_scalar("SELECT count(*) FROM information_schema.columns WHERE table_name='public_links' AND column_name IN ('token','secret','raw_token')")
        .fetch_one(store.pool()).await.unwrap();
    assert_eq!(secret_column_count, 0);

    let race_document = Uuid::now_v7();
    seed_document(&store, actor, workspace, race_document).await;
    sqlx::query("INSERT INTO edit_leases(document_id,workspace_id,holder_user_id,client_instance_id,token_hash,expires_at) VALUES($1,$2,$3,$4,$5,$6)")
        .bind(race_document).bind(workspace).bind(actor).bind(Uuid::now_v7()).bind([9_u8;32].as_slice()).bind(Utc::now()+Duration::minutes(5)).execute(store.pool()).await.unwrap();
    assert!(matches!(
        service
            .publish(
                actor,
                workspace,
                race_document,
                0,
                PublishDocumentInput {
                    summary: "Lease conflict".into(),
                    client_instance_id: None,
                    lease_token: None
                },
                "publish-lease-denied"
            )
            .await,
        Err(GovernanceError::PublishLeaseConflict)
    ));
    sqlx::query("DELETE FROM edit_leases WHERE document_id=$1")
        .bind(race_document)
        .execute(store.pool())
        .await
        .unwrap();
    let barrier = Arc::new(Barrier::new(3));
    let first_service = service.clone();
    let first_barrier = barrier.clone();
    let first_publish = tokio::spawn(async move {
        first_barrier.wait().await;
        first_service
            .publish(
                actor,
                workspace,
                race_document,
                0,
                PublishDocumentInput {
                    summary: "Race one".into(),
                    client_instance_id: None,
                    lease_token: None,
                },
                "publish-race-key-01",
            )
            .await
    });
    let second_service = service.clone();
    let second_barrier = barrier.clone();
    let second_publish = tokio::spawn(async move {
        second_barrier.wait().await;
        second_service
            .publish(
                actor,
                workspace,
                race_document,
                0,
                PublishDocumentInput {
                    summary: "Race two".into(),
                    client_instance_id: None,
                    lease_token: None,
                },
                "publish-race-key-02",
            )
            .await
    });
    barrier.wait().await;
    let outcomes = [first_publish.await.unwrap(), second_publish.await.unwrap()];
    assert_eq!(outcomes.iter().filter(|outcome| outcome.is_ok()).count(), 1);
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT count(*) FROM published_versions WHERE document_id=$1"
        )
        .bind(race_document)
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
async fn seed(store: &PostgresStore, actor: Uuid, workspace: Uuid, document: Uuid) {
    sqlx::query(
        "INSERT INTO users(id,google_subject,email,display_name) VALUES($1,$2,$3,'Publish Test')",
    )
    .bind(actor)
    .bind(actor.to_string())
    .bind(format!("{actor}@example.test"))
    .execute(store.pool())
    .await
    .unwrap();
    sqlx::query("INSERT INTO workspaces(id,slug,name,created_by) VALUES($1,$2,'Publish Test',$3)")
        .bind(workspace)
        .bind(format!("publish-{workspace}"))
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
    seed_document(store, actor, workspace, document).await;
}

async fn seed_document(store: &PostgresStore, actor: Uuid, workspace: Uuid, document: Uuid) {
    let sequence: i64 =
        sqlx::query_scalar("SELECT count(*)+1 FROM documents WHERE workspace_id=$1")
            .bind(workspace)
            .fetch_one(store.pool())
            .await
            .unwrap();
    let rank = format!("{sequence:032}");
    sqlx::query("INSERT INTO documents(id,workspace_id,rank,title,created_by) VALUES($1,$2,$3,'Publish Document',$4)").bind(document).bind(workspace).bind(rank).bind(actor).execute(store.pool()).await.unwrap();
    sqlx::query("INSERT INTO permission_grants(id,workspace_id,document_id,subject_kind,subject_id,access,can_manage,granted_by) VALUES($1,$2,$3,'USER',$4,'EDITOR',true,$4)").bind(Uuid::now_v7()).bind(workspace).bind(document).bind(actor).execute(store.pool()).await.unwrap();
    let content = json!({"schemaVersion":1,"root":{"type":"doc","children":[{"id":Uuid::now_v7(),"type":"paragraph","children":[{"type":"text","text":"published"}]}]}});
    sqlx::query("INSERT INTO drafts(id,workspace_id,document_id,content_json,schema_version,revision,updated_by) VALUES($1,$2,$3,$4,1,0,$5)").bind(Uuid::now_v7()).bind(workspace).bind(document).bind(content).bind(actor).execute(store.pool()).await.unwrap();
}
