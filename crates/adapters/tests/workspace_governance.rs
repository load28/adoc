use std::{env, sync::Arc};

use adoc_adapters::{
    identity::{SystemClock, SystemSecureRandom},
    postgres::{DatabaseSettings, PostgresGovernanceRepository, PostgresStore},
};
use adoc_application::{
    governance::{
        CreateGroupInput, CreateWorkspaceInput, GovernanceError, GovernanceService,
        InviteMemberInput, MembershipRole, UpdateMemberRoleInput,
    },
    identity::{KeyRing, SigningKey},
};
use uuid::Uuid;

#[tokio::test]
#[ignore = "requires ADOC_TEST_DATABASE_URL pointing to an isolated PostgreSQL 16 database"]
async fn workspace_governance_postgres_contract() {
    let url = test_database_url();
    let store = PostgresStore::connect(DatabaseSettings {
        url: &url,
        max_connections: 8,
        application_name: "adoc-governance-test",
    })
    .await
    .unwrap();
    store.migrate().await.unwrap();

    let owner = seed_user(&store, "owner").await;
    let invited = seed_user(&store, "invited").await;
    let outsider = seed_user(&store, "outsider").await;
    let service = Arc::new(service(&store));

    let workspace = service
        .create_workspace(
            owner,
            CreateWorkspaceInput {
                name: "Platform Team".into(),
            },
            "workspace-create-0001",
        )
        .await
        .unwrap();
    assert_eq!(
        service.list_workspaces(owner).await.unwrap()[0].id,
        workspace.id
    );
    assert!(matches!(
        service.get_workspace(outsider, workspace.id).await,
        Err(GovernanceError::WorkspaceNotFound)
    ));

    let invite_input = InviteMemberInput {
        email: user_email(invited),
        role: adoc_application::governance::InvitationRole::Member,
    };
    let invitation = service
        .invite_member(
            owner,
            workspace.id,
            invite_input.clone(),
            "invitation-create-01",
        )
        .await
        .unwrap();
    let replay = service
        .invite_member(owner, workspace.id, invite_input, "invitation-create-01")
        .await
        .unwrap();
    assert_eq!(invitation.invitation, replay.invitation);
    assert_eq!(invitation.delivery_token(), replay.delivery_token());
    assert!(matches!(
        service
            .preview_invitation(&user_email(outsider), invitation.delivery_token())
            .await,
        Err(GovernanceError::InvitationInvalid)
    ));
    let preview = service
        .preview_invitation(&user_email(invited), invitation.delivery_token())
        .await
        .unwrap();
    assert_eq!(preview.workspace_id, workspace.id);
    assert_eq!(preview.workspace_name, workspace.name);
    assert_eq!(preview.workspace_slug, workspace.slug);
    assert!(matches!(
        service
            .accept_invitation(
                outsider,
                &user_email(outsider),
                invitation.delivery_token(),
                "invitation-accept-01",
            )
            .await,
        Err(GovernanceError::InvitationInvalid)
    ));
    let membership = service
        .accept_invitation(
            invited,
            &user_email(invited),
            invitation.delivery_token(),
            "invitation-accept-02",
        )
        .await
        .unwrap();
    assert_eq!(membership.role, MembershipRole::Member);
    assert!(matches!(
        service
            .preview_invitation(&user_email(invited), invitation.delivery_token())
            .await,
        Err(GovernanceError::InvitationInvalid)
    ));

    let promoted = service
        .update_member_role(
            owner,
            workspace.id,
            invited,
            membership.revision,
            UpdateMemberRoleInput {
                role: MembershipRole::Owner,
            },
            "membership-role-0001",
        )
        .await
        .unwrap();
    let first = service.clone();
    let second = service.clone();
    let (left, right) = tokio::join!(
        first.update_member_role(
            owner,
            workspace.id,
            invited,
            promoted.revision,
            UpdateMemberRoleInput {
                role: MembershipRole::Member,
            },
            "membership-role-0002",
        ),
        second.update_member_role(
            invited,
            workspace.id,
            owner,
            0,
            UpdateMemberRoleInput {
                role: MembershipRole::Member,
            },
            "membership-role-0003",
        )
    );
    assert_eq!(usize::from(left.is_ok()) + usize::from(right.is_ok()), 1);
    let members = service.list_members(owner, workspace.id).await.unwrap();
    assert_eq!(
        members
            .iter()
            .filter(|member| member.role == MembershipRole::Owner)
            .count(),
        1
    );
    let remaining_owner = members
        .iter()
        .find(|member| member.role == MembershipRole::Owner)
        .unwrap()
        .user_id;

    let group = service
        .create_group(
            remaining_owner,
            workspace.id,
            CreateGroupInput {
                name: "Reviewers".into(),
                member_ids: vec![owner, invited, invited],
            },
            "group-create-000001",
        )
        .await
        .unwrap();
    assert_eq!(group.member_ids.len(), 2);
    let group_replay = service
        .create_group(
            remaining_owner,
            workspace.id,
            CreateGroupInput {
                name: "Reviewers".into(),
                member_ids: vec![owner, invited, invited],
            },
            "group-create-000001",
        )
        .await
        .unwrap();
    assert_eq!(group, group_replay);
    assert!(matches!(
        service
            .change_group_member(adoc_application::governance::GroupMemberCommand {
                add: true,
                actor_id: remaining_owner,
                workspace_id: workspace.id,
                group_id: group.id,
                user_id: outsider,
                expected_revision: group.revision,
                idempotency_key: "group-member-00001".into(),
            })
            .await,
        Err(GovernanceError::GroupMemberInvalid)
    ));

    let event_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM outbox_events WHERE workspace_id=$1 AND published_at IS NULL",
    )
    .bind(workspace.id)
    .fetch_one(store.pool())
    .await
    .unwrap();
    assert!(event_count >= 5);
    store.close().await;
}

fn service(store: &PostgresStore) -> GovernanceService {
    GovernanceService::new(
        Arc::new(PostgresGovernanceRepository::new(store)),
        Arc::new(SystemClock),
        Arc::new(SystemSecureRandom),
        KeyRing::new(
            SigningKey {
                id: "current".into(),
                value: Arc::from([7_u8; 32]),
            },
            Some(SigningKey {
                id: "previous".into(),
                value: Arc::from([8_u8; 32]),
            }),
        )
        .unwrap(),
    )
}

async fn seed_user(store: &PostgresStore, label: &str) -> Uuid {
    let id = Uuid::now_v7();
    sqlx::query("INSERT INTO users(id,google_subject,email,display_name) VALUES($1,$2,$3,$4)")
        .bind(id)
        .bind(format!("task-014-{label}-{id}"))
        .bind(user_email(id))
        .bind(format!("Task 014 {label}"))
        .execute(store.pool())
        .await
        .unwrap();
    id
}

fn user_email(id: Uuid) -> String {
    format!("{id}@example.test")
}

fn test_database_url() -> String {
    if let Ok(url) = env::var("ADOC_TEST_DATABASE_URL") {
        return url;
    }
    let path = env::var("ADOC_TEST_DATABASE_URL_FILE")
        .expect("ADOC_TEST_DATABASE_URL or ADOC_TEST_DATABASE_URL_FILE is required");
    std::fs::read_to_string(path).unwrap().trim().to_owned()
}
