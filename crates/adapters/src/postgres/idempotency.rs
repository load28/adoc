use adoc_ports::{PersistenceError, Transaction};
use chrono::{DateTime, Utc};
use serde_json::Value;
use thiserror::Error;
use uuid::Uuid;

use super::{error::map_sqlx, transaction::connection};

#[derive(Clone, Copy, Debug)]
pub struct IdempotencyIdentity<'a> {
    pub workspace_id: Uuid,
    pub actor_id: Uuid,
    pub operation_id: &'a str,
    pub key: &'a str,
}

#[derive(Clone, Copy, Debug)]
pub struct IdempotencyReservation<'a> {
    pub identity: IdempotencyIdentity<'a>,
    pub request_hash: &'a str,
    pub now: DateTime<Utc>,
    pub locked_until: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StoredResponse {
    pub status: i32,
    pub body: Value,
}

#[derive(Clone, Debug, PartialEq)]
pub enum IdempotencyDecision {
    Acquired,
    Replay(StoredResponse),
    Busy { locked_until: DateTime<Utc> },
}

#[derive(Debug, Error)]
pub enum IdempotencyError {
    #[error(transparent)]
    Persistence(#[from] PersistenceError),
    #[error("invalid idempotency input")]
    InvalidInput,
    #[error("idempotency key was reused with a different request")]
    KeyReused,
    #[error("idempotency completion state conflict")]
    CompletionStateConflict,
}

pub async fn reserve_idempotency(
    transaction: &mut dyn Transaction,
    reservation: IdempotencyReservation<'_>,
) -> Result<IdempotencyDecision, IdempotencyError> {
    validate_reservation(reservation)?;
    let connection = connection(transaction)?;
    let identity = reservation.identity;

    let inserted = sqlx::query(
        "INSERT INTO idempotency_keys \
         (workspace_id, actor_id, operation_id, key, request_hash, locked_until, expires_at, created_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8) ON CONFLICT DO NOTHING",
    )
    .bind(identity.workspace_id)
    .bind(identity.actor_id)
    .bind(identity.operation_id)
    .bind(identity.key)
    .bind(reservation.request_hash)
    .bind(reservation.locked_until)
    .bind(reservation.expires_at)
    .bind(reservation.now)
    .execute(&mut *connection)
    .await
    .map_err(map_sqlx)?;
    if inserted.rows_affected() == 1 {
        return Ok(IdempotencyDecision::Acquired);
    }

    let row = sqlx::query_as::<
        _,
        (
            String,
            Option<i32>,
            Option<Value>,
            DateTime<Utc>,
            DateTime<Utc>,
        ),
    >(
        "SELECT request_hash, response_status, response_json, locked_until, expires_at \
         FROM idempotency_keys \
         WHERE workspace_id = $1 AND actor_id = $2 AND operation_id = $3 AND key = $4 \
         FOR UPDATE",
    )
    .bind(identity.workspace_id)
    .bind(identity.actor_id)
    .bind(identity.operation_id)
    .bind(identity.key)
    .fetch_one(&mut *connection)
    .await
    .map_err(map_sqlx)?;

    let (stored_hash, response_status, response_json, locked_until, expires_at) = row;
    if expires_at <= reservation.now {
        sqlx::query(
            "UPDATE idempotency_keys SET request_hash = $5, response_status = NULL, \
             response_json = NULL, locked_until = $6, expires_at = $7, created_at = $8 \
             WHERE workspace_id = $1 AND actor_id = $2 AND operation_id = $3 AND key = $4",
        )
        .bind(identity.workspace_id)
        .bind(identity.actor_id)
        .bind(identity.operation_id)
        .bind(identity.key)
        .bind(reservation.request_hash)
        .bind(reservation.locked_until)
        .bind(reservation.expires_at)
        .bind(reservation.now)
        .execute(&mut *connection)
        .await
        .map_err(map_sqlx)?;
        return Ok(IdempotencyDecision::Acquired);
    }

    if stored_hash != reservation.request_hash {
        return Err(IdempotencyError::KeyReused);
    }
    if let (Some(status), Some(body)) = (response_status, response_json) {
        return Ok(IdempotencyDecision::Replay(StoredResponse { status, body }));
    }
    if locked_until > reservation.now {
        return Ok(IdempotencyDecision::Busy { locked_until });
    }

    sqlx::query(
        "UPDATE idempotency_keys SET locked_until = $5, expires_at = $6 \
         WHERE workspace_id = $1 AND actor_id = $2 AND operation_id = $3 AND key = $4",
    )
    .bind(identity.workspace_id)
    .bind(identity.actor_id)
    .bind(identity.operation_id)
    .bind(identity.key)
    .bind(reservation.locked_until)
    .bind(reservation.expires_at)
    .execute(&mut *connection)
    .await
    .map_err(map_sqlx)?;
    Ok(IdempotencyDecision::Acquired)
}

pub async fn complete_idempotency(
    transaction: &mut dyn Transaction,
    identity: IdempotencyIdentity<'_>,
    request_hash: &str,
    response: StoredResponse,
) -> Result<(), IdempotencyError> {
    validate_identity(identity)?;
    if !is_sha256(request_hash) || !(100..=599).contains(&response.status) {
        return Err(IdempotencyError::InvalidInput);
    }
    let connection = connection(transaction)?;
    let result = sqlx::query(
        "UPDATE idempotency_keys SET response_status = $6, response_json = $7 \
         WHERE workspace_id = $1 AND actor_id = $2 AND operation_id = $3 AND key = $4 \
         AND request_hash = $5 AND response_status IS NULL",
    )
    .bind(identity.workspace_id)
    .bind(identity.actor_id)
    .bind(identity.operation_id)
    .bind(identity.key)
    .bind(request_hash)
    .bind(response.status)
    .bind(response.body)
    .execute(&mut *connection)
    .await
    .map_err(map_sqlx)?;
    if result.rows_affected() != 1 {
        return Err(IdempotencyError::CompletionStateConflict);
    }
    Ok(())
}

fn validate_reservation(reservation: IdempotencyReservation<'_>) -> Result<(), IdempotencyError> {
    validate_identity(reservation.identity)?;
    if !is_sha256(reservation.request_hash)
        || reservation.locked_until <= reservation.now
        || reservation.expires_at <= reservation.locked_until
    {
        return Err(IdempotencyError::InvalidInput);
    }
    Ok(())
}

fn validate_identity(identity: IdempotencyIdentity<'_>) -> Result<(), IdempotencyError> {
    if identity.operation_id.is_empty()
        || identity.operation_id.len() > 200
        || identity.key.is_empty()
        || identity.key.len() > 255
    {
        return Err(IdempotencyError::InvalidInput);
    }
    Ok(())
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, Utc};
    use uuid::Uuid;

    use super::{
        IdempotencyError, IdempotencyIdentity, IdempotencyReservation, validate_reservation,
    };

    #[test]
    fn reservation_requires_canonical_hash_and_ordered_deadlines() {
        let now = Utc::now();
        let identity = IdempotencyIdentity {
            workspace_id: Uuid::nil(),
            actor_id: Uuid::nil(),
            operation_id: "createDocument",
            key: "key-1",
        };
        let invalid = IdempotencyReservation {
            identity,
            request_hash: "ABC",
            now,
            locked_until: now + Duration::seconds(10),
            expires_at: now + Duration::minutes(10),
        };

        assert!(matches!(
            validate_reservation(invalid),
            Err(IdempotencyError::InvalidInput)
        ));
    }
}
