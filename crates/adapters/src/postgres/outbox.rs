use adoc_application::operations::{
    EventAudience, EventAudienceKind, JobPriorityBucket, StreamAccess,
};
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
    pub audience: EventAudience,
    pub correlation_id: &'a str,
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
        || !event.audience.is_valid()
        || !(8..=128).contains(&event.correlation_id.len())
        || event.payload.to_string().len() > 65_536
    {
        return Err(OutboxAppendError::InvalidInput);
    }
    let connection = connection(transaction)?;
    sqlx::query("INSERT INTO workspace_sequences(workspace_id) VALUES($1) ON CONFLICT(workspace_id) DO NOTHING")
        .bind(event.workspace_id)
        .execute(&mut *connection)
        .await
        .map_err(|error| OutboxAppendError::Persistence(map_sqlx(error)))?;
    let projection_sequence: i64 = sqlx::query_scalar(
        "UPDATE workspace_sequences SET next_projection_sequence=next_projection_sequence+1 WHERE workspace_id=$1 RETURNING next_projection_sequence-1",
    )
    .bind(event.workspace_id)
    .fetch_one(&mut *connection)
    .await
    .map_err(|error| OutboxAppendError::Persistence(map_sqlx(error)))?;
    let result = sqlx::query(
        "INSERT INTO outbox_events \
         (id, workspace_id, aggregate_kind, aggregate_id, sequence, event_type, projection_sequence, \
          event_version, payload_json, audience_kind, audience_id, minimum_access, correlation_id, occurred_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10::event_audience_kind, $11, $12::document_access, $13, $14)",
    )
    .bind(event.id)
    .bind(event.workspace_id)
    .bind(event.aggregate_kind)
    .bind(event.aggregate_id)
    .bind(event.sequence)
    .bind(event.event_type)
    .bind(projection_sequence)
    .bind(event.event_version)
    .bind(event.payload)
    .bind(audience_kind(event.audience.kind))
    .bind(event.audience.id)
    .bind(event.audience.minimum_access.map(access_text))
    .bind(event.correlation_id)
    .bind(event.occurred_at)
    .execute(&mut *connection)
    .await;

    match result {
        Ok(_) => {
            sqlx::query(
                "INSERT INTO jobs(id,workspace_id,kind,payload_json,dedupe_key,status,priority,sequence,attempt,max_attempts,run_after,correlation_id,created_at,updated_at) \
                 VALUES($1,$2,'OUTBOX_TO_STREAM',$3,$4,'QUEUED',50,1,0,5,$5,$6,$5,$5) ON CONFLICT(kind,dedupe_key) DO NOTHING",
            )
            .bind(Uuid::now_v7())
            .bind(event.workspace_id)
            .bind(serde_json::json!({"outboxEventId": event.id}))
            .bind(format!("outbox:{}", event.id))
            .bind(event.occurred_at)
            .bind(event.correlation_id)
            .execute(&mut *connection)
            .await
            .map_err(|error| OutboxAppendError::Persistence(map_sqlx(error)))?;
            if is_search_projection_event(event.event_type) {
                sqlx::query(
                    "INSERT INTO jobs(id,workspace_id,kind,payload_json,dedupe_key,status,priority,sequence,attempt,max_attempts,run_after,correlation_id,created_at,updated_at) \
                     VALUES($1,$2,'OUTBOX_TO_SEARCH',$3,$4,'QUEUED',25,1,0,5,$5,$6,$5,$5) ON CONFLICT(kind,dedupe_key) DO NOTHING",
                )
                .bind(Uuid::now_v7())
                .bind(event.workspace_id)
                .bind(serde_json::json!({"outboxEventId": event.id}))
                .bind(format!("search-projection:{}", event.id))
                .bind(event.occurred_at)
                .bind(event.correlation_id)
                .execute(&mut *connection)
                .await
                .map_err(|error| OutboxAppendError::Persistence(map_sqlx(error)))?;
            }
            Ok(())
        }
        Err(error) if is_sequence_conflict(&error) => Err(OutboxAppendError::SequenceConflict),
        Err(error) => Err(OutboxAppendError::Persistence(map_sqlx(error))),
    }
}

pub(super) fn is_search_projection_event(value: &str) -> bool {
    matches!(
        value,
        "DocumentChanged.v1"
            | "DocumentMoved.v1"
            | "DraftChanged.v1"
            | "VersionPublished.v1"
            | "PermissionChanged.v1"
            | "VocabularyChanged.v1"
            | "PurgeChanged.v1"
    )
}

pub(super) fn audience_kind(value: EventAudienceKind) -> &'static str {
    match value {
        EventAudienceKind::Internal => "INTERNAL",
        EventAudienceKind::Workspace => "WORKSPACE",
        EventAudienceKind::Admin => "ADMIN",
        EventAudienceKind::User => "USER",
        EventAudienceKind::Document => "DOCUMENT",
    }
}

pub(super) fn access_text(value: StreamAccess) -> &'static str {
    match value {
        StreamAccess::Viewer => "VIEWER",
        StreamAccess::Contributor => "CONTRIBUTOR",
        StreamAccess::Editor => "EDITOR",
    }
}

pub(super) fn priority_bucket(priority: i16) -> JobPriorityBucket {
    if priority >= 75 {
        JobPriorityBucket::Interactive
    } else if priority >= 25 {
        JobPriorityBucket::Normal
    } else {
        JobPriorityBucket::Background
    }
}

fn is_sequence_conflict(error: &sqlx::Error) -> bool {
    let sqlx::Error::Database(database) = error else {
        return false;
    };
    database.constraint() == Some("outbox_events_aggregate_kind_aggregate_id_sequence_key")
}
