use adoc_application::{
    governance::GovernanceError,
    operations::{AuditEvent, AuditEventInput, AuditPage, AuditRepository},
};
use adoc_ports::BoxFuture;
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::{DateTime, Utc};
use serde::de::DeserializeOwned;
use serde_json::Value;
use sqlx::{PgPool, Postgres, Row, Transaction};
use uuid::Uuid;

use super::{PostgresStore, error::map_sqlx};

#[derive(Clone)]
pub struct PostgresAuditRepository {
    pool: PgPool,
}

impl PostgresAuditRepository {
    #[must_use]
    pub fn new(store: &PostgresStore) -> Self {
        Self {
            pool: store.pool().clone(),
        }
    }
}

pub async fn append_audit_event(
    tx: &mut Transaction<'_, Postgres>,
    input: AuditEventInput,
) -> Result<AuditEvent, GovernanceError> {
    if !input.is_valid() {
        return Err(GovernanceError::Internal);
    }
    sqlx::query(
        "INSERT INTO workspace_sequences(workspace_id,next_audit_sequence) VALUES($1,1) \
         ON CONFLICT(workspace_id) DO NOTHING",
    )
    .bind(input.workspace_id)
    .execute(&mut **tx)
    .await
    .map_err(map_store)?;
    let sequence: i64 = sqlx::query_scalar(
        "UPDATE workspace_sequences SET next_audit_sequence=next_audit_sequence+1 \
         WHERE workspace_id=$1 RETURNING next_audit_sequence-1",
    )
    .bind(input.workspace_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(map_store)?;
    let id = Uuid::now_v7();
    let actor = serde_json::to_value(&input.actor).map_err(|_| GovernanceError::Internal)?;
    let action = enum_name(&input.action)?;
    let target = serde_json::to_value(&input.target).map_err(|_| GovernanceError::Internal)?;
    let before = input
        .before
        .as_ref()
        .map(serde_json::to_value)
        .transpose()
        .map_err(|_| GovernanceError::Internal)?;
    let after = input
        .after
        .as_ref()
        .map(serde_json::to_value)
        .transpose()
        .map_err(|_| GovernanceError::Internal)?;
    let metadata = serde_json::to_value(&input.metadata).map_err(|_| GovernanceError::Internal)?;
    sqlx::query(
        "INSERT INTO audit_events(id,workspace_id,sequence,actor_json,action,target_json,before_json,after_json,metadata_json,correlation_id,occurred_at) \
         VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)",
    )
    .bind(id)
    .bind(input.workspace_id)
    .bind(sequence)
    .bind(actor)
    .bind(action)
    .bind(target)
    .bind(before)
    .bind(after)
    .bind(metadata)
    .bind(&input.correlation_id)
    .bind(input.occurred_at)
    .execute(&mut **tx)
    .await
    .map_err(map_store)?;
    Ok(AuditEvent {
        id,
        sequence,
        actor: input.actor,
        action: input.action,
        target: input.target,
        before: input.before,
        after: input.after,
        metadata: input.metadata,
        correlation_id: input.correlation_id,
        occurred_at: input.occurred_at,
        redacted_at: None,
    })
}

impl AuditRepository for PostgresAuditRepository {
    fn list<'a>(
        &'a self,
        actor: Uuid,
        workspace: Uuid,
        cursor: Option<String>,
    ) -> BoxFuture<'a, Result<AuditPage, GovernanceError>> {
        Box::pin(async move {
            let allowed: bool = sqlx::query_scalar(
                "SELECT EXISTS(SELECT 1 FROM memberships m JOIN workspaces w ON w.id=m.workspace_id \
                 WHERE m.workspace_id=$1 AND m.user_id=$2 AND m.status='ACTIVE' \
                 AND m.role IN ('ADMIN','OWNER') AND w.status IN ('ACTIVE','DELETION_SCHEDULED'))",
            )
            .bind(workspace)
            .bind(actor)
            .fetch_one(&self.pool)
            .await
            .map_err(map_store)?;
            if !allowed {
                return Err(GovernanceError::WorkspaceNotFound);
            }
            let (cursor_sequence, cursor_id) = cursor
                .as_deref()
                .map(parse_cursor)
                .transpose()?
                .map_or((None, None), |(sequence, id)| (Some(sequence), Some(id)));
            let rows = sqlx::query(
                "SELECT id,sequence,actor_json,action,target_json,before_json,after_json,metadata_json,correlation_id,occurred_at,redacted_at \
                 FROM audit_events WHERE workspace_id=$1 AND \
                 ($2::bigint IS NULL OR (sequence,id)<($2,$3)) \
                 ORDER BY sequence DESC,id DESC LIMIT 51",
            )
            .bind(workspace)
            .bind(cursor_sequence)
            .bind(cursor_id)
            .fetch_all(&self.pool)
            .await
            .map_err(map_store)?;
            let mut items = rows
                .iter()
                .take(50)
                .map(event)
                .collect::<Result<Vec<_>, _>>()?;
            let next_cursor = if rows.len() > 50 {
                items
                    .last()
                    .map(|item| format_cursor(item.sequence, item.id))
            } else {
                None
            };
            items.shrink_to_fit();
            Ok(AuditPage { items, next_cursor })
        })
    }
}

fn event(row: &sqlx::postgres::PgRow) -> Result<AuditEvent, GovernanceError> {
    Ok(AuditEvent {
        id: row.get("id"),
        sequence: row.get("sequence"),
        actor: from_json(row.get("actor_json"))?,
        action: serde_json::from_value(Value::String(row.get("action")))
            .map_err(|_| GovernanceError::Internal)?,
        target: from_json(row.get("target_json"))?,
        before: row
            .get::<Option<Value>, _>("before_json")
            .map(from_json)
            .transpose()?,
        after: row
            .get::<Option<Value>, _>("after_json")
            .map(from_json)
            .transpose()?,
        metadata: from_json(row.get("metadata_json"))?,
        correlation_id: row.get("correlation_id"),
        occurred_at: row.get::<DateTime<Utc>, _>("occurred_at"),
        redacted_at: row.get("redacted_at"),
    })
}

fn enum_name<T: serde::Serialize>(value: &T) -> Result<String, GovernanceError> {
    serde_json::to_value(value)
        .ok()
        .and_then(|value| value.as_str().map(str::to_owned))
        .ok_or(GovernanceError::Internal)
}

fn from_json<T: DeserializeOwned>(value: Value) -> Result<T, GovernanceError> {
    serde_json::from_value(value).map_err(|_| GovernanceError::Internal)
}

fn parse_cursor(value: &str) -> Result<(i64, Uuid), GovernanceError> {
    let decoded = URL_SAFE_NO_PAD
        .decode(value)
        .map_err(|_| GovernanceError::Validation)?;
    let decoded = std::str::from_utf8(&decoded).map_err(|_| GovernanceError::Validation)?;
    let (sequence, id) = decoded.split_once(':').ok_or(GovernanceError::Validation)?;
    let sequence = sequence
        .parse::<i64>()
        .map_err(|_| GovernanceError::Validation)?;
    let id = Uuid::parse_str(id).map_err(|_| GovernanceError::Validation)?;
    if sequence <= 0 {
        return Err(GovernanceError::Validation);
    }
    Ok((sequence, id))
}

fn format_cursor(sequence: i64, id: Uuid) -> String {
    URL_SAFE_NO_PAD.encode(format!("{sequence}:{id}"))
}

fn map_store(error: sqlx::Error) -> GovernanceError {
    let _ = map_sqlx(error);
    GovernanceError::Internal
}
