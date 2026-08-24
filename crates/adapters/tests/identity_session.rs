use std::env;

use adoc_adapters::postgres::{DatabaseSettings, PostgresIdentityRepository, PostgresStore};
use adoc_application::identity::{
    HashCandidate, IdentityError, IdentityRepository, LoginFlowRecord, NewSessionRecord,
    PreferenceInput, SessionLifetime, Theme, TokenHash, UserCommandReceipt,
    VerifiedExternalIdentity,
};
use chrono::{Duration, Utc};
use uuid::Uuid;

#[tokio::test]
#[ignore = "requires ADOC_TEST_DATABASE_URL pointing to an isolated PostgreSQL 16 database"]
async fn identity_session_postgres_contract() {
    let url = test_database_url();
    let store = PostgresStore::connect(DatabaseSettings {
        url: &url,
        max_connections: 5,
        application_name: "adoc-identity-test",
    })
    .await
    .unwrap();
    store.migrate().await.unwrap();
    let repository = PostgresIdentityRepository::new(&store);
    let now = Utc::now();
    let state = candidate("current", 1);
    let marker = candidate("current", 2);
    repository
        .create_login_flow(LoginFlowRecord {
            state_hash: state.clone(),
            marker_hash: marker.hash.clone(),
            nonce_hash: TokenHash([3; 32]),
            pkce_verifier: "v".repeat(43),
            return_to: "/workspaces".into(),
            created_at: now,
            expires_at: now + Duration::minutes(10),
        })
        .await
        .unwrap();
    let consumed = repository
        .consume_login_flow(vec![state.clone()], vec![marker.clone()], now)
        .await
        .unwrap();
    assert_eq!(consumed.return_to, "/workspaces");
    assert!(matches!(
        repository
            .consume_login_flow(vec![state], vec![marker], now)
            .await,
        Err(IdentityError::InvalidCallback)
    ));
    let concurrent_state = candidate("current", 11);
    let concurrent_marker = candidate("current", 12);
    repository
        .create_login_flow(LoginFlowRecord {
            state_hash: concurrent_state.clone(),
            marker_hash: concurrent_marker.hash.clone(),
            nonce_hash: TokenHash([13; 32]),
            pkce_verifier: "c".repeat(43),
            return_to: "/concurrent".into(),
            created_at: now,
            expires_at: now + Duration::minutes(10),
        })
        .await
        .unwrap();
    let first_repository = repository.clone();
    let second_repository = repository.clone();
    let first_state = concurrent_state.clone();
    let first_marker = concurrent_marker.clone();
    let (first, second) = tokio::join!(
        first_repository.consume_login_flow(vec![first_state], vec![first_marker], now),
        second_repository.consume_login_flow(vec![concurrent_state], vec![concurrent_marker], now)
    );
    assert_eq!(usize::from(first.is_ok()) + usize::from(second.is_ok()), 1);

    let user_id = Uuid::now_v7();
    let session = candidate("current", 4);
    let identity = VerifiedExternalIdentity::google(
        "https://accounts.google.com",
        &format!("task-013-{user_id}"),
        "task-013@example.test",
        "Task 013",
    )
    .unwrap();
    let user = repository
        .establish_identity(
            identity,
            user_id,
            NewSessionRecord {
                hash: session.clone(),
                lifetime: SessionLifetime::new(now, Duration::hours(12)),
            },
            vec![],
            now,
        )
        .await
        .unwrap();
    assert_eq!(user.id, user_id);
    let principal = repository
        .authenticate(
            vec![session.clone()],
            now + Duration::minutes(6),
            now + Duration::hours(12) + Duration::minutes(6),
        )
        .await
        .unwrap();
    assert_eq!(principal.user.id, user_id);

    let preferences = repository.preferences(user_id).await.unwrap();
    let updated = repository
        .update_preferences(
            user_id,
            preferences.revision,
            PreferenceInput {
                locale: adoc_application::identity::Locale::En,
                timezone: "America/New_York".into(),
                theme: Theme::Dark,
            },
            receipt("preference-key-00000001", "a".repeat(64), now),
            now,
        )
        .await
        .unwrap();
    assert_eq!(updated.revision, preferences.revision + 1);
    let replay = repository
        .update_preferences(
            user_id,
            preferences.revision,
            PreferenceInput {
                locale: adoc_application::identity::Locale::En,
                timezone: "America/New_York".into(),
                theme: Theme::Dark,
            },
            receipt("preference-key-00000001", "a".repeat(64), now),
            now,
        )
        .await
        .unwrap();
    assert_eq!(replay, updated);
    assert!(matches!(
        repository
            .update_preferences(
                user_id,
                preferences.revision,
                PreferenceInput {
                    locale: adoc_application::identity::Locale::En,
                    timezone: "America/New_York".into(),
                    theme: Theme::Dark,
                },
                receipt("preference-key-00000001", "c".repeat(64), now),
                now,
            )
            .await,
        Err(IdentityError::IdempotencyKeyReused)
    ));
    assert!(matches!(
        repository
            .update_preferences(
                user_id,
                preferences.revision,
                PreferenceInput {
                    locale: adoc_application::identity::Locale::Ko,
                    timezone: "Asia/Seoul".into(),
                    theme: Theme::System,
                },
                receipt("preference-key-00000002", "b".repeat(64), now),
                now,
            )
            .await,
        Err(IdentityError::RevisionConflict { .. })
    ));

    let first_repository = repository.clone();
    let second_repository = repository.clone();
    let (first, second) = tokio::join!(
        first_repository.update_preferences(
            user_id,
            updated.revision,
            PreferenceInput {
                locale: adoc_application::identity::Locale::Ko,
                timezone: "Asia/Seoul".into(),
                theme: Theme::System,
            },
            receipt("preference-key-00000003", "d".repeat(64), now),
            now,
        ),
        second_repository.update_preferences(
            user_id,
            updated.revision,
            PreferenceInput {
                locale: adoc_application::identity::Locale::En,
                timezone: "Europe/London".into(),
                theme: Theme::Light,
            },
            receipt("preference-key-00000004", "e".repeat(64), now),
            now,
        )
    );
    assert_eq!(usize::from(first.is_ok()) + usize::from(second.is_ok()), 1);
    assert!(
        [first, second]
            .iter()
            .any(|result| matches!(result, Err(IdentityError::RevisionConflict { .. })))
    );

    let rotated_session = candidate("current", 14);
    let rotated_identity = VerifiedExternalIdentity::google(
        "https://accounts.google.com",
        &format!("task-013-{user_id}"),
        "task-013@example.test",
        "Task 013",
    )
    .unwrap();
    repository
        .establish_identity(
            rotated_identity,
            Uuid::now_v7(),
            NewSessionRecord {
                hash: rotated_session.clone(),
                lifetime: SessionLifetime::new(now + Duration::minutes(7), Duration::hours(12)),
            },
            vec![session.clone()],
            now + Duration::minutes(7),
        )
        .await
        .unwrap();
    assert!(matches!(
        repository
            .authenticate(
                vec![session],
                now + Duration::minutes(8),
                now + Duration::hours(12),
            )
            .await,
        Err(IdentityError::AuthenticationRequired)
    ));
    assert!(
        repository
            .authenticate(
                vec![rotated_session.clone()],
                now + Duration::minutes(8),
                now + Duration::hours(12),
            )
            .await
            .is_ok()
    );

    repository
        .revoke(vec![rotated_session.clone()], now + Duration::minutes(9))
        .await
        .unwrap();
    assert!(matches!(
        repository
            .authenticate(
                vec![rotated_session],
                now + Duration::minutes(10),
                now + Duration::hours(12),
            )
            .await,
        Err(IdentityError::AuthenticationRequired)
    ));
    sqlx::query("DELETE FROM users WHERE id=$1")
        .bind(user_id)
        .execute(store.pool())
        .await
        .unwrap();
    store.close().await;
}

fn candidate(key_id: &str, byte: u8) -> HashCandidate {
    HashCandidate {
        key_id: key_id.into(),
        hash: TokenHash([byte; 32]),
    }
}

fn receipt(key: &str, request_hash: String, now: chrono::DateTime<Utc>) -> UserCommandReceipt {
    UserCommandReceipt {
        operation_id: "updateUserPreferences",
        key: key.into(),
        request_hash,
        created_at: now,
        expires_at: now + Duration::hours(24),
    }
}

fn test_database_url() -> String {
    if let Ok(url) = env::var("ADOC_TEST_DATABASE_URL") {
        return url;
    }
    let path = env::var("ADOC_TEST_DATABASE_URL_FILE")
        .expect("ADOC_TEST_DATABASE_URL or ADOC_TEST_DATABASE_URL_FILE is required");
    std::fs::read_to_string(path).unwrap().trim().to_owned()
}
