use adoc_ports::{PersistenceError, Transaction};
use chrono::{DateTime, Utc};
use serde_json::Value;
use thiserror::Error;
use uuid::Uuid;

use super::{error::map_sqlx, transaction::connection};

#[derive(Clone, Debug)]
pub struct OutboxEventInput<'a> {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub aggregate_kind: &'a str,
    pub aggregate_id: Uuid,
    pub sequence: i64,
    pub event_type: &'a str,
    pub event_version: i32,
    pub payload: Value,
    pub occurred_at: DateTime<Utc>,
}

#[derive(Debug, Error)]
pub enum OutboxAppendError {
    #[error(transparent)]
    Persistence(#[from] PersistenceError),
    #[error("invalid outbox event input")]
    InvalidInput,
    #[error("aggregate outbox sequence conflict")]
    SequenceConflict,
}

pub async fn append_outbox_event(
    transaction: &mut dyn Transaction,
    event: OutboxEventInput<'_>,
) -> Result<(), OutboxAppendError> {
    if event.aggregate_kind.is_empty()
        || event.aggregate_kind.len() > 100
        || event.event_type.is_empty()
        || event.event_type.len() > 200
        || event.sequence <= 0
        || event.event_version <= 0
    {
        return Err(OutboxAppendError::InvalidInput);
    }
    let connection = connection(transaction)?;
    let result = sqlx::query(
        "INSERT INTO outbox_events \
         (id, workspace_id, aggregate_kind, aggregate_id, sequence, event_type, \
          event_version, payload_json, occurred_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
    )
    .bind(event.id)
    .bind(event.workspace_id)
    .bind(event.aggregate_kind)
    .bind(event.aggregate_id)
    .bind(event.sequence)
    .bind(event.event_type)
    .bind(event.event_version)
    .bind(event.payload)
    .bind(event.occurred_at)
    .execute(&mut *connection)
    .await;

    match result {
        Ok(_) => Ok(()),
        Err(error) if is_sequence_conflict(&error) => Err(OutboxAppendError::SequenceConflict),
        Err(error) => Err(OutboxAppendError::Persistence(map_sqlx(error))),
    }
}

fn is_sequence_conflict(error: &sqlx::Error) -> bool {
    let sqlx::Error::Database(database) = error else {
        return false;
    };
    database.constraint() == Some("outbox_events_aggregate_kind_aggregate_id_sequence_key")
}
