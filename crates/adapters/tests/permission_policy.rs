use std::{env, sync::Arc};

use adoc_adapters::{
    identity::SystemClock,
    permission_cache::{RedisPermissionCache, UnavailablePermissionCache},
    postgres::{DatabaseSettings, PostgresPermissionRepository, PostgresStore},
};
use adoc_application::{
    governance::GovernanceError,
    permission::{
        Access, PermissionGrantInput, PermissionService, PublishMode, ReviewerRule,
        SetPermissionCommand, SetPublishPolicyInput, SubjectKind,
    },
};
use uuid::Uuid;

#[tokio::test]
#[ignore = "requires isolated PostgreSQL 16 and Redis"]
async fn permission_policy_postgres_redis_contract() {
    let store = PostgresStore::connect(DatabaseSettings {
        url: &secret("ADOC_TEST_DATABASE_URL", "ADOC_TEST_DATABASE_URL_FILE"),
        max_connections: 8,
        application_name: "adoc-permission-test",
    })
    .await
    .unwrap();
    store.migrate().await.unwrap();
    let redis_url = secret("ADOC_TEST_REDIS_URL", "ADOC_TEST_REDIS_URL_FILE");
    let cache = RedisPermissionCache::connect(&redis_url, "task015")
        .await
        .unwrap();
    let service = PermissionService::new(
        Arc::new(PostgresPermissionRepository::new(&store)),
        Arc::new(cache),
        Arc::new(SystemClock),
    );

    let owner = Uuid::now_v7();
    let viewer = Uuid::now_v7();
    let denied = Uuid::now_v7();
    let outsider = Uuid::now_v7();
    for (id, label) in [
        (owner, "owner"),
        (viewer, "viewer"),
        (denied, "denied"),
        (outsider, "outsider"),
    ] {
        seed_user(&store, id, label).await;
    }
    let workspace = Uuid::now_v7();
    let other_workspace = Uuid::now_v7();
    seed_workspace(&store, workspace, owner, "permission-alpha").await;
    seed_workspace(&store, other_workspace, outsider, "permission-beta").await;
    for user in [viewer, denied] {
        sqlx::query("INSERT INTO memberships(workspace_id,user_id,role,status) VALUES($1,$2,'MEMBER','ACTIVE')")
            .bind(workspace).bind(user).execute(store.pool()).await.unwrap();
    }

    let root = Uuid::now_v7();
    let child = Uuid::now_v7();
    let grandchild = Uuid::now_v7();
    seed_document(&store, workspace, root, None, owner, "a").await;
    seed_document(&store, workspace, child, Some(root), owner, "b").await;
    seed_document(&store, workspace, grandchild, Some(child), owner, "c").await;
    let all = seed_group(&store, workspace, "All", &[owner, viewer, denied]).await;
    let denied_group = seed_group(&store, workspace, "Denied", &[denied]).await;
    let owner_grant = seed_grant(
        &store,
        workspace,
        root,
        SubjectKind::User,
        owner,
        Access::Editor,
        true,
    )
    .await;
    seed_grant(
        &store,
        workspace,
        root,
        SubjectKind::Group,
        all,
        Access::Viewer,
        false,
    )
    .await;
    seed_grant(
        &store,
        workspace,
        child,
        SubjectKind::Group,
        denied_group,
        Access::NoAccess,
        false,
    )
    .await;

    let stamp: (i64, i64, i64) = sqlx::query_as("SELECT r.permission_revision,r.policy_revision,m.revision FROM workspace_access_revisions r JOIN memberships m ON m.workspace_id=r.workspace_id AND m.user_id=$2 WHERE r.workspace_id=$1")
        .bind(workspace).bind(viewer).fetch_one(store.pool()).await.unwrap();
    let cache_key = format!(
        "task015:adoc:permission:v1:{workspace}:{viewer}:{child}:{}:{}:{}",
        stamp.0, stamp.1, stamp.2
    );
    let client = redis::Client::open(redis_url).unwrap();
    let mut redis = client.get_connection_manager().await.unwrap();
    let _: String = redis::cmd("SET")
        .arg(&cache_key)
        .arg("{corrupt")
        .arg("EX")
        .arg(300)
        .query_async(&mut redis)
        .await
        .unwrap();

    assert_eq!(
        service
            .point(viewer, workspace, child)
            .await
            .unwrap()
            .access,
        Access::Viewer
    );
    let ttl: i64 = redis::cmd("TTL")
        .arg(&cache_key)
        .query_async(&mut redis)
        .await
        .unwrap();
    assert!((1..=300).contains(&ttl));
    let unavailable_service = PermissionService::new(
        Arc::new(PostgresPermissionRepository::new(&store)),
        Arc::new(UnavailablePermissionCache),
        Arc::new(SystemClock),
    );
    assert_eq!(
        unavailable_service
            .point(viewer, workspace, child)
            .await
            .unwrap()
            .access,
        Access::Viewer
    );
    assert_eq!(
        service
            .point(denied, workspace, child)
            .await
            .unwrap()
            .access,
        Access::NoAccess
    );
    let scope = service.scope(viewer, workspace).await.unwrap();
    assert_eq!(scope.accessible_document_ids, vec![root, child, grandchild]);
    assert!(matches!(
        service.point(outsider, workspace, root).await,
        Err(GovernanceError::DocumentNotFound)
    ));

    let viewer_grant = Uuid::now_v7();
    let changed = service
        .set_permission(SetPermissionCommand {
            actor_id: owner,
            workspace_id: workspace,
            document_id: child,
            grant_id: viewer_grant,
            expected_revision: 0,
            input: PermissionGrantInput {
                subject_kind: SubjectKind::User,
                subject_id: viewer,
                access: Access::Editor,
                manage: true,
            },
            idempotency_key: "permission-set-0001".to_owned(),
        })
        .await
        .unwrap();
    let replay = service
        .set_permission(SetPermissionCommand {
            actor_id: owner,
            workspace_id: workspace,
            document_id: child,
            grant_id: viewer_grant,
            expected_revision: 0,
            input: PermissionGrantInput {
                subject_kind: SubjectKind::User,
                subject_id: viewer,
                access: Access::Editor,
                manage: true,
            },
            idempotency_key: "permission-set-0001".to_owned(),
        })
        .await
        .unwrap();
    assert_eq!(changed, replay);
    assert!(
        service
            .point(viewer, workspace, child)
            .await
            .unwrap()
            .manage
    );
    assert!(matches!(
        service
            .set_permission(SetPermissionCommand {
                actor_id: owner,
                workspace_id: workspace,
                document_id: child,
                grant_id: Uuid::now_v7(),
                expected_revision: 1,
                input: PermissionGrantInput {
                    subject_kind: SubjectKind::User,
                    subject_id: outsider,
                    access: Access::Viewer,
                    manage: false
                },
                idempotency_key: "permission-set-0002".to_owned(),
            })
            .await,
        Err(GovernanceError::PermissionSubjectInvalid)
    ));
    assert!(matches!(
        service
            .delete_permission(
                owner,
                workspace,
                root,
                owner_grant,
                0,
                "permission-del-0001",
            )
            .await,
        Err(GovernanceError::PermissionLastManager)
    ));

    let policy = service
        .set_policy(
            owner,
            workspace,
            child,
            0,
            SetPublishPolicyInput {
                mode: PublishMode::ReviewRequired,
                required_approvals: 1,
                reviewer_rule: ReviewerRule::Users {
                    user_ids: vec![viewer],
                },
            },
            "publish-policy-0001",
        )
        .await
        .unwrap();
    assert_eq!(policy.revision, 1);
    let inherited = service
        .get_policy(viewer, workspace, grandchild)
        .await
        .unwrap();
    assert_eq!(inherited.mode, PublishMode::ReviewRequired);
    assert_eq!(inherited.inherited_from_document_id, Some(child));
    assert_eq!(inherited.revision, 0);

    let revisions:(i64,i64)=sqlx::query_as("SELECT permission_revision,policy_revision FROM workspace_access_revisions WHERE workspace_id=$1")
        .bind(workspace).fetch_one(store.pool()).await.unwrap();
    assert!(revisions.0 > 0 && revisions.1 > 0);
    let events:i64=sqlx::query_scalar("SELECT count(*) FROM outbox_events WHERE workspace_id=$1 AND event_type IN ('PermissionChanged.v1','PublishPolicyChanged.v1')")
        .bind(workspace).fetch_one(store.pool()).await.unwrap();
    assert_eq!(events, 2);
    store.close().await;
}

async fn seed_user(store: &PostgresStore, id: Uuid, label: &str) {
    sqlx::query("INSERT INTO users(id,google_subject,email,display_name) VALUES($1,$2,$3,$4)")
        .bind(id)
        .bind(format!("task-015-{label}-{id}"))
        .bind(format!("{id}@example.test"))
        .bind(label)
        .execute(store.pool())
        .await
        .unwrap();
}
async fn seed_workspace(store: &PostgresStore, id: Uuid, owner: Uuid, slug: &str) {
    sqlx::query("INSERT INTO workspaces(id,slug,name,created_by) VALUES($1,$2,$3,$4)")
        .bind(id)
        .bind(format!("{slug}-{}", &id.simple().to_string()[..8]))
        .bind(slug)
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
async fn seed_document(
    store: &PostgresStore,
    workspace: Uuid,
    id: Uuid,
    parent: Option<Uuid>,
    creator: Uuid,
    rank: &str,
) {
    let canonical_rank = format!("{rank:0>32}");
    sqlx::query("INSERT INTO documents(id,workspace_id,parent_id,rank,title,created_by) VALUES($1,$2,$3,$4,$5,$6)").bind(id).bind(workspace).bind(parent).bind(canonical_rank).bind(format!("Document {rank}")).bind(creator).execute(store.pool()).await.unwrap();
}
async fn seed_group(store: &PostgresStore, workspace: Uuid, name: &str, members: &[Uuid]) -> Uuid {
    let id = Uuid::now_v7();
    sqlx::query("INSERT INTO groups(id,workspace_id,name) VALUES($1,$2,$3)")
        .bind(id)
        .bind(workspace)
        .bind(name)
        .execute(store.pool())
        .await
        .unwrap();
    for member in members {
        sqlx::query("INSERT INTO group_members(workspace_id,group_id,user_id) VALUES($1,$2,$3)")
            .bind(workspace)
            .bind(id)
            .bind(member)
            .execute(store.pool())
            .await
            .unwrap();
    }
    id
}
async fn seed_grant(
    store: &PostgresStore,
    workspace: Uuid,
    document: Uuid,
    kind: SubjectKind,
    subject: Uuid,
    access: Access,
    manage: bool,
) -> Uuid {
    let id = Uuid::now_v7();
    let kind = match kind {
        SubjectKind::User => "USER",
        SubjectKind::Group => "GROUP",
    };
    let access = match access {
        Access::NoAccess => "NO_ACCESS",
        Access::Viewer => "VIEWER",
        Access::Contributor => "CONTRIBUTOR",
        Access::Editor => "EDITOR",
    };
    let actor: Uuid = sqlx::query_scalar(
        "SELECT user_id FROM memberships WHERE workspace_id=$1 AND role='OWNER' LIMIT 1",
    )
    .bind(workspace)
    .fetch_one(store.pool())
    .await
    .unwrap();
    sqlx::query("INSERT INTO permission_grants(id,workspace_id,document_id,subject_kind,subject_id,access,can_manage,granted_by) VALUES($1,$2,$3,$4::subject_kind,$5,$6::document_access,$7,$8)").bind(id).bind(workspace).bind(document).bind(kind).bind(subject).bind(access).bind(manage).bind(actor).execute(store.pool()).await.unwrap();
    id
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
