use std::str::FromStr;

use adoc_application::identity::{
    ConsumedLoginFlow, HashCandidate, IdentityError, IdentityRepository, Locale, LoginFlowRecord,
    NewSessionRecord, PreferenceInput, SessionPrincipal, Theme, TokenHash, UserCommandReceipt,
    UserPreferences, UserSummary, VerifiedExternalIdentity,
};
use adoc_ports::BoxFuture;
use chrono::{DateTime, Duration, Utc};
use sqlx::{PgPool, Postgres, Row, Transaction};
use uuid::Uuid;

use super::PostgresStore;

#[derive(Clone)]
pub struct PostgresIdentityRepository {
    pool: PgPool,
}

impl PostgresIdentityRepository {
    pub fn new(store: &PostgresStore) -> Self {
        Self {
            pool: store.pool().clone(),
        }
    }
}

impl IdentityRepository for PostgresIdentityRepository {
    fn create_login_flow<'a>(
        &'a self,
        flow: LoginFlowRecord,
    ) -> BoxFuture<'a, Result<(), IdentityError>> {
        Box::pin(async move {
            sqlx::query(
                "INSERT INTO login_flows
                 (state_hash, marker_hash, hash_key_id, nonce_hash, pkce_verifier, return_to,
                  created_at, expires_at)
                 VALUES ($1,$2,$3,$4,$5,$6,$7,$8)",
            )
            .bind(flow.state_hash.hash.0.as_slice())
            .bind(flow.marker_hash.0.as_slice())
            .bind(flow.state_hash.key_id)
            .bind(flow.nonce_hash.0.as_slice())
            .bind(flow.pkce_verifier)
            .bind(flow.return_to)
            .bind(flow.created_at)
            .bind(flow.expires_at)
            .execute(&self.pool)
            .await
            .map_err(map_store)?;
            Ok(())
        })
    }

    fn consume_login_flow<'a>(
        &'a self,
        state: Vec<HashCandidate>,
        marker: Vec<HashCandidate>,
        now: DateTime<Utc>,
    ) -> BoxFuture<'a, Result<ConsumedLoginFlow, IdentityError>> {
        Box::pin(async move {
            let state = hash_bytes(&state);
            let marker = hash_bytes(&marker);
            let row = sqlx::query(
                "UPDATE login_flows
                 SET consumed_at=$3
                 WHERE state_hash = ANY($1) AND marker_hash = ANY($2)
                   AND consumed_at IS NULL AND expires_at > $3
                 RETURNING nonce_hash, pkce_verifier, return_to",
            )
            .bind(state)
            .bind(marker)
            .bind(now)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_store)?
            .ok_or(IdentityError::InvalidCallback)?;
            Ok(ConsumedLoginFlow {
                nonce_hash: fixed_hash(
                    row.try_get("nonce_hash")
                        .map_err(|_| IdentityError::Internal)?,
                )?,
                pkce_verifier: row
                    .try_get("pkce_verifier")
                    .map_err(|_| IdentityError::Internal)?,
                return_to: row
                    .try_get("return_to")
                    .map_err(|_| IdentityError::Internal)?,
            })
        })
    }

    fn establish_identity<'a>(
        &'a self,
        identity: VerifiedExternalIdentity,
        proposed_user_id: Uuid,
        session: NewSessionRecord,
        revoke: Vec<HashCandidate>,
        now: DateTime<Utc>,
    ) -> BoxFuture<'a, Result<UserSummary, IdentityError>> {
        Box::pin(async move {
            let mut transaction = self.pool.begin().await.map_err(map_store)?;
            let rotated_from = revoke_sessions(&mut transaction, &revoke, now).await?;
            let row = sqlx::query(
                "INSERT INTO users
                 (id, identity_issuer, google_subject, email, display_name, updated_at)
                 VALUES ($1,$2,$3,$4,$5,$6)
                 ON CONFLICT (identity_issuer, google_subject) DO UPDATE
                 SET email=EXCLUDED.email, display_name=EXCLUDED.display_name, updated_at=EXCLUDED.updated_at
                 RETURNING id, email, display_name, locale, timezone",
            )
            .bind(proposed_user_id)
            .bind(identity.issuer)
            .bind(identity.subject)
            .bind(identity.email.as_str())
            .bind(identity.display_name.as_str())
            .bind(now)
            .fetch_one(&mut *transaction)
            .await
            .map_err(map_store)?;
            let user = user_summary(&row)?;
            sqlx::query(
                "INSERT INTO sessions
                 (id_hash, hash_key_id, user_id, rotated_from_hash, created_at, last_seen_at,
                  idle_expires_at, absolute_expires_at)
                 VALUES ($1,$2,$3,$4,$5,$6,$7,$8)",
            )
            .bind(session.hash.hash.0.as_slice())
            .bind(session.hash.key_id)
            .bind(user.id)
            .bind(rotated_from)
            .bind(session.lifetime.created_at)
            .bind(session.lifetime.last_seen_at)
            .bind(session.lifetime.idle_expires_at)
            .bind(session.lifetime.absolute_expires_at)
            .execute(&mut *transaction)
            .await
            .map_err(map_store)?;
            transaction.commit().await.map_err(map_store)?;
            Ok(user)
        })
    }

    fn authenticate<'a>(
        &'a self,
        candidates: Vec<HashCandidate>,
        now: DateTime<Utc>,
        proposed_idle_expires_at: DateTime<Utc>,
    ) -> BoxFuture<'a, Result<SessionPrincipal, IdentityError>> {
        Box::pin(async move {
            let hashes = hash_bytes(&candidates);
            let mut transaction = self.pool.begin().await.map_err(map_store)?;
            let row = sqlx::query(
                "SELECT s.id_hash, s.last_seen_at, s.absolute_expires_at,
                        u.id, u.email, u.display_name, u.locale, u.timezone
                 FROM sessions s JOIN users u ON u.id=s.user_id
                 WHERE s.id_hash = ANY($1) AND s.revoked_at IS NULL
                   AND s.idle_expires_at > $2 AND s.absolute_expires_at > $2
                 FOR UPDATE OF s",
            )
            .bind(hashes)
            .bind(now)
            .fetch_optional(&mut *transaction)
            .await
            .map_err(map_store)?
            .ok_or(IdentityError::AuthenticationRequired)?;
            let session_hash = fixed_hash(
                row.try_get("id_hash")
                    .map_err(|_| IdentityError::Internal)?,
            )?;
            let last_seen_at: DateTime<Utc> = row
                .try_get("last_seen_at")
                .map_err(|_| IdentityError::Internal)?;
            let absolute_expires_at: DateTime<Utc> = row
                .try_get("absolute_expires_at")
                .map_err(|_| IdentityError::Internal)?;
            if now - last_seen_at >= Duration::minutes(5) {
                sqlx::query(
                    "UPDATE sessions SET last_seen_at=$2,
                     idle_expires_at=LEAST($3, $4)
                     WHERE id_hash=$1 AND revoked_at IS NULL",
                )
                .bind(session_hash.0.as_slice())
                .bind(now)
                .bind(proposed_idle_expires_at)
                .bind(absolute_expires_at)
                .execute(&mut *transaction)
                .await
                .map_err(map_store)?;
            }
            let user = user_summary(&row)?;
            transaction.commit().await.map_err(map_store)?;
            Ok(SessionPrincipal { user, session_hash })
        })
    }

    fn revoke<'a>(
        &'a self,
        candidates: Vec<HashCandidate>,
        now: DateTime<Utc>,
    ) -> BoxFuture<'a, Result<(), IdentityError>> {
        Box::pin(async move {
            sqlx::query(
                "UPDATE sessions SET revoked_at=COALESCE(revoked_at,$2)
                 WHERE id_hash = ANY($1)",
            )
            .bind(hash_bytes(&candidates))
            .bind(now)
            .execute(&self.pool)
            .await
            .map_err(map_store)?;
            Ok(())
        })
    }

    fn preferences<'a>(
        &'a self,
        user_id: Uuid,
    ) -> BoxFuture<'a, Result<UserPreferences, IdentityError>> {
        Box::pin(async move {
            let row =
                sqlx::query("SELECT locale, timezone, theme, revision FROM users WHERE id=$1")
                    .bind(user_id)
                    .fetch_optional(&self.pool)
                    .await
                    .map_err(map_store)?
                    .ok_or(IdentityError::AuthenticationRequired)?;
            preferences(&row)
        })
    }

    fn update_preferences<'a>(
        &'a self,
        user_id: Uuid,
        expected_revision: i64,
        input: PreferenceInput,
        command: UserCommandReceipt,
        now: DateTime<Utc>,
    ) -> BoxFuture<'a, Result<UserPreferences, IdentityError>> {
        Box::pin(async move {
            let mut transaction = self.pool.begin().await.map_err(map_store)?;
            sqlx::query(
                "INSERT INTO user_command_receipts
                 (user_id, operation_id, key, request_hash, created_at, expires_at)
                 VALUES ($1,$2,$3,$4,$5,$6)
                 ON CONFLICT (user_id, operation_id, key) DO NOTHING",
            )
            .bind(user_id)
            .bind(command.operation_id)
            .bind(&command.key)
            .bind(&command.request_hash)
            .bind(command.created_at)
            .bind(command.expires_at)
            .execute(&mut *transaction)
            .await
            .map_err(map_store)?;
            let receipt = sqlx::query(
                "SELECT request_hash, response_json FROM user_command_receipts
                 WHERE user_id=$1 AND operation_id=$2 AND key=$3 FOR UPDATE",
            )
            .bind(user_id)
            .bind(command.operation_id)
            .bind(&command.key)
            .fetch_one(&mut *transaction)
            .await
            .map_err(map_store)?;
            let stored_hash: String = receipt
                .try_get("request_hash")
                .map_err(|_| IdentityError::Internal)?;
            if stored_hash != command.request_hash {
                return Err(IdentityError::IdempotencyKeyReused);
            }
            if let Some(response) = receipt
                .try_get::<Option<serde_json::Value>, _>("response_json")
                .map_err(|_| IdentityError::Internal)?
            {
                let response =
                    serde_json::from_value(response).map_err(|_| IdentityError::Internal)?;
                transaction.commit().await.map_err(map_store)?;
                return Ok(response);
            }
            let row = sqlx::query(
                "UPDATE users SET locale=$3, timezone=$4, theme=$5,
                 revision=revision+1, updated_at=$6
                 WHERE id=$1 AND revision=$2
                 RETURNING locale, timezone, theme, revision",
            )
            .bind(user_id)
            .bind(expected_revision)
            .bind(input.locale.as_str())
            .bind(input.timezone)
            .bind(input.theme.as_str())
            .bind(now)
            .fetch_optional(&mut *transaction)
            .await
            .map_err(map_store)?;
            if let Some(row) = row {
                let response = preferences(&row)?;
                sqlx::query(
                    "UPDATE user_command_receipts SET response_json=$4
                     WHERE user_id=$1 AND operation_id=$2 AND key=$3",
                )
                .bind(user_id)
                .bind(command.operation_id)
                .bind(command.key)
                .bind(serde_json::to_value(&response).map_err(|_| IdentityError::Internal)?)
                .execute(&mut *transaction)
                .await
                .map_err(map_store)?;
                transaction.commit().await.map_err(map_store)?;
                return Ok(response);
            }
            let current = sqlx::query_scalar::<_, i64>("SELECT revision FROM users WHERE id=$1")
                .bind(user_id)
                .fetch_optional(&mut *transaction)
                .await
                .map_err(map_store)?
                .ok_or(IdentityError::AuthenticationRequired)?;
            Err(IdentityError::RevisionConflict {
                current_revision: current,
            })
        })
    }
}

async fn revoke_sessions(
    transaction: &mut Transaction<'_, Postgres>,
    candidates: &[HashCandidate],
    now: DateTime<Utc>,
) -> Result<Option<Vec<u8>>, IdentityError> {
    if candidates.is_empty() {
        return Ok(None);
    }
    let rows = sqlx::query(
        "UPDATE sessions SET revoked_at=COALESCE(revoked_at,$2)
         WHERE id_hash = ANY($1) AND revoked_at IS NULL RETURNING id_hash",
    )
    .bind(hash_bytes(candidates))
    .bind(now)
    .fetch_all(&mut **transaction)
    .await
    .map_err(map_store)?;
    Ok(rows
        .first()
        .and_then(|row| row.try_get::<Vec<u8>, _>("id_hash").ok()))
}

fn hash_bytes(candidates: &[HashCandidate]) -> Vec<Vec<u8>> {
    candidates
        .iter()
        .map(|candidate| candidate.hash.0.to_vec())
        .collect()
}

fn fixed_hash(bytes: Vec<u8>) -> Result<TokenHash, IdentityError> {
    let bytes: [u8; 32] = bytes.try_into().map_err(|_| IdentityError::Internal)?;
    Ok(TokenHash(bytes))
}

fn user_summary(row: &sqlx::postgres::PgRow) -> Result<UserSummary, IdentityError> {
    let locale: String = row.try_get("locale").map_err(|_| IdentityError::Internal)?;
    Ok(UserSummary {
        id: row.try_get("id").map_err(|_| IdentityError::Internal)?,
        email: row.try_get("email").map_err(|_| IdentityError::Internal)?,
        display_name: row
            .try_get("display_name")
            .map_err(|_| IdentityError::Internal)?,
        locale: Locale::from_str(&locale).map_err(|_| IdentityError::Internal)?,
        timezone: row
            .try_get("timezone")
            .map_err(|_| IdentityError::Internal)?,
    })
}

fn preferences(row: &sqlx::postgres::PgRow) -> Result<UserPreferences, IdentityError> {
    let locale: String = row.try_get("locale").map_err(|_| IdentityError::Internal)?;
    let theme: String = row.try_get("theme").map_err(|_| IdentityError::Internal)?;
    Ok(UserPreferences {
        locale: Locale::from_str(&locale).map_err(|_| IdentityError::Internal)?,
        timezone: row
            .try_get("timezone")
            .map_err(|_| IdentityError::Internal)?,
        theme: Theme::from_str(&theme).map_err(|_| IdentityError::Internal)?,
        revision: row
            .try_get("revision")
            .map_err(|_| IdentityError::Internal)?,
    })
}

fn map_store(error: sqlx::Error) -> IdentityError {
    if matches!(
        error,
        sqlx::Error::PoolTimedOut | sqlx::Error::PoolClosed | sqlx::Error::Io(_)
    ) {
        IdentityError::StorageUnavailable
    } else {
        IdentityError::Internal
    }
}
