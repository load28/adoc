use adoc_adapters::{
    identity::{SystemClock, SystemSecureRandom},
    postgres::{DatabaseSettings, PostgresCollaborationRepository, PostgresStore},
};
use adoc_application::{
    collaboration::{
        CollaborationService, CreateDiscussionInput, InboxAction, InboxFilter, RichMessage,
        TopicInput, TopicKind,
    },
    governance::GovernanceError,
};
use serde_json::json;
use std::{env, fs, sync::Arc};
use uuid::Uuid;

#[tokio::test]
#[ignore = "requires ADOC_TEST_DATABASE_URL pointing to an isolated PostgreSQL 16 database"]
async fn discussion_message_inbox_postgres_contract() {
    let store = PostgresStore::connect(DatabaseSettings {
        url: &secret("ADOC_TEST_DATABASE_URL", "ADOC_TEST_DATABASE_URL_FILE"),
        max_connections: 8,
        application_name: "adoc-collaboration-contract",
    })
    .await
    .unwrap();
    store.migrate().await.unwrap();
    let actor = Uuid::now_v7();
    let workspace = Uuid::now_v7();
    let document = Uuid::now_v7();
    seed(&store, actor, workspace, document).await;
    let service = CollaborationService::new(
        Arc::new(PostgresCollaborationRepository::new(&store)),
        Arc::new(SystemClock),
        Arc::new(SystemSecureRandom),
    );
    let body = content("first");
    let discussion = service
        .create_discussion(
            actor,
            workspace,
            document,
            "discussion-create-01",
            CreateDiscussionInput {
                title: " Design question ".into(),
                message: RichMessage {
                    body: body.clone(),
                    mention_user_ids: vec![actor],
                    attachment_ids: vec![],
                },
                topics: vec![TopicInput {
                    kind: TopicKind::Text,
                    label: "Decision".into(),
                    text: Some("Choose a design".into()),
                    target_id: None,
                    region: None,
                    url: None,
                }],
            },
        )
        .await
        .unwrap();
    assert_eq!(discussion.title, "Design question");
    assert_eq!(discussion.topics.len(), 1);
    let replay = service
        .create_discussion(
            actor,
            workspace,
            document,
            "discussion-create-01",
            CreateDiscussionInput {
                title: " Design question ".into(),
                message: RichMessage {
                    body: body.clone(),
                    mention_user_ids: vec![actor],
                    attachment_ids: vec![],
                },
                topics: vec![TopicInput {
                    kind: TopicKind::Text,
                    label: "Decision".into(),
                    text: Some("Choose a design".into()),
                    target_id: None,
                    region: None,
                    url: None,
                }],
            },
        )
        .await
        .unwrap();
    assert_eq!(replay.id, discussion.id);
    let detail = service
        .get_discussion(actor, workspace, discussion.id, None)
        .await
        .unwrap();
    let message = &detail.messages[0];
    assert!(message.attachment_ids.is_empty());
    let updated = service
        .update_message(
            actor,
            workspace,
            discussion.id,
            message.id,
            message.revision,
            "message-update-001",
            RichMessage {
                body: content("edited"),
                mention_user_ids: vec![],
                attachment_ids: vec![],
            },
        )
        .await
        .unwrap();
    assert_eq!(updated.revision, 1);
    assert!(updated.attachment_ids.is_empty());
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM message_revisions WHERE message_id=$1")
            .bind(message.id)
            .fetch_one(store.pool())
            .await
            .unwrap(),
        1
    );
    let inbox = service
        .list_inbox(actor, workspace, None, InboxFilter::Resolved)
        .await
        .unwrap();
    assert_eq!(inbox.items.len(), 1);
    assert!(inbox.items[0].read_at.is_none());
    service
        .inbox(
            actor,
            workspace,
            Some(inbox.items[0].id),
            None,
            "inbox-read-0001",
            InboxAction::Read,
        )
        .await
        .unwrap();
    let closed = service
        .close(
            actor,
            workspace,
            discussion.id,
            discussion.revision,
            "discussion-close1",
        )
        .await
        .unwrap();
    assert!(matches!(
        service
            .create_message(
                actor,
                workspace,
                discussion.id,
                "message-after-close",
                RichMessage {
                    body: content("blocked"),
                    mention_user_ids: vec![],
                    attachment_ids: vec![]
                }
            )
            .await,
        Err(GovernanceError::DiscussionClosed)
    ));
    assert_eq!(
        closed.status,
        adoc_application::collaboration::DiscussionStatus::Closed
    );
    let reopened = service
        .reopen(
            actor,
            workspace,
            discussion.id,
            closed.revision,
            "discussion-reopen1",
        )
        .await
        .unwrap();
    assert_eq!(
        reopened.status,
        adoc_application::collaboration::DiscussionStatus::Open
    );
    let history = service
        .get_discussion(actor, workspace, discussion.id, None)
        .await
        .unwrap();
    assert_eq!(history.messages.len(), 1);
    let lifecycle_audit: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM audit_events WHERE workspace_id=$1 AND target_json->>'id'=$2 AND action IN ('DISCUSSION_CLOSED','DISCUSSION_REOPENED')",
    )
    .bind(workspace)
    .bind(discussion.id.to_string())
    .fetch_one(store.pool())
    .await
    .unwrap();
    assert_eq!(lifecycle_audit, 2);
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
    sqlx::query(
        "INSERT INTO memberships(workspace_id,user_id,role,status) VALUES($1,$2,'MEMBER','ACTIVE')",
    )
    .bind(workspace)
    .bind(outsider)
    .execute(store.pool())
    .await
    .unwrap();
    assert!(matches!(
        service
            .get_discussion(outsider, workspace, discussion.id, None)
            .await,
        Err(GovernanceError::DiscussionNotFound)
    ));
    sqlx::query("UPDATE memberships SET status='SUSPENDED' WHERE workspace_id=$1 AND user_id=$2")
        .bind(workspace)
        .bind(actor)
        .execute(store.pool())
        .await
        .unwrap();
    assert!(matches!(
        service
            .list_inbox(actor, workspace, None, InboxFilter::All)
            .await,
        Err(GovernanceError::WorkspaceNotFound)
    ));
    assert!(
        sqlx::query("UPDATE message_revisions SET body_json='{}' WHERE message_id=$1")
            .bind(message.id)
            .execute(store.pool())
            .await
            .is_err()
    );
    store.close().await;
}
fn content(text: &str) -> serde_json::Value {
    json!({"schemaVersion":1,"root":{"type":"doc","children":[{"id":Uuid::now_v7(),"type":"paragraph","children":[{"type":"text","text":text}]}]}})
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
    sqlx::query("INSERT INTO users(id,google_subject,email,display_name) VALUES($1,$2,$3,'Collaboration Test')").bind(actor).bind(actor.to_string()).bind(format!("{actor}@example.test")).execute(store.pool()).await.unwrap();
    sqlx::query(
        "INSERT INTO workspaces(id,slug,name,created_by) VALUES($1,$2,'Collaboration Test',$3)",
    )
    .bind(workspace)
    .bind(format!("collaboration-{workspace}"))
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
    sqlx::query("INSERT INTO documents(id,workspace_id,rank,title,created_by) VALUES($1,$2,'00000000000000000000000000000001','Document',$3)").bind(document).bind(workspace).bind(actor).execute(store.pool()).await.unwrap();
    sqlx::query("INSERT INTO permission_grants(id,workspace_id,document_id,subject_kind,subject_id,access,can_manage,granted_by) VALUES($1,$2,$3,'USER',$4,'EDITOR',true,$4)").bind(Uuid::now_v7()).bind(workspace).bind(document).bind(actor).execute(store.pool()).await.unwrap();
}
