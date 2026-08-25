use adoc_application::{
    governance::{GovernanceError, MembershipRole},
    operations::{EventAudience, EventAudienceKind, StreamAccess, WorkspaceStreamEvent},
    stream::{StreamMembership, StreamRepository},
};
use adoc_ports::BoxFuture;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use super::{PostgresStore, governance::map_store};

#[derive(Clone)]
pub struct PostgresStreamRepository {
    pool: PgPool,
}

impl PostgresStreamRepository {
    #[must_use]
    pub fn new(store: &PostgresStore) -> Self {
        Self {
            pool: store.pool().clone(),
        }
    }
}

impl StreamRepository for PostgresStreamRepository {
    fn membership<'a>(
        &'a self,
        actor: Uuid,
        workspace: Uuid,
    ) -> BoxFuture<'a, Result<StreamMembership, GovernanceError>> {
        Box::pin(async move {
            let row = sqlx::query(
                "SELECT m.role::text,m.revision,COALESCE(s.next_stream_sequence-1,0) AS high_sequence, \
                 CASE WHEN COALESCE(s.next_stream_sequence-1,0)=0 THEN NULL \
                      ELSE COALESCE((SELECT min(e.sequence) FROM workspace_stream_events e WHERE e.workspace_id=$1 AND e.expires_at>now()),s.next_stream_sequence) END AS minimum_sequence \
                 FROM memberships m JOIN workspaces w ON w.id=m.workspace_id \
                 LEFT JOIN workspace_sequences s ON s.workspace_id=m.workspace_id \
                 WHERE m.workspace_id=$1 AND m.user_id=$2 AND m.status='ACTIVE' AND w.status IN ('ACTIVE','DELETION_SCHEDULED')",
            )
            .bind(workspace)
            .bind(actor)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_store)?
            .ok_or(GovernanceError::WorkspaceNotFound)?;
            Ok(StreamMembership {
                role: match row.get::<String, _>("role").as_str() {
                    "MEMBER" => MembershipRole::Member,
                    "ADMIN" => MembershipRole::Admin,
                    "OWNER" => MembershipRole::Owner,
                    _ => return Err(GovernanceError::Internal),
                },
                revision: row.get("revision"),
                high_sequence: row.get("high_sequence"),
                minimum_sequence: row.get("minimum_sequence"),
            })
        })
    }

    fn page<'a>(
        &'a self,
        workspace: Uuid,
        after_sequence: i64,
        limit: i64,
    ) -> BoxFuture<'a, Result<Vec<WorkspaceStreamEvent>, GovernanceError>> {
        Box::pin(async move {
            if after_sequence < 0 || !(1..=100).contains(&limit) {
                return Err(GovernanceError::StreamCursorInvalid);
            }
            let rows = sqlx::query(
                "SELECT id,workspace_id,sequence,aggregate_id,event_type,event_version,payload_json,audience_kind::text,audience_id,minimum_access::text,correlation_id,occurred_at \
                 FROM workspace_stream_events WHERE workspace_id=$1 AND sequence>$2 AND expires_at>now() \
                 ORDER BY sequence,id LIMIT $3",
            )
            .bind(workspace)
            .bind(after_sequence)
            .bind(limit)
            .fetch_all(&self.pool)
            .await
            .map_err(map_store)?;
            rows.iter().map(stream_event).collect()
        })
    }
}

fn stream_event(row: &sqlx::postgres::PgRow) -> Result<WorkspaceStreamEvent, GovernanceError> {
    let kind = match row.get::<String, _>("audience_kind").as_str() {
        "WORKSPACE" => EventAudienceKind::Workspace,
        "ADMIN" => EventAudienceKind::Admin,
        "USER" => EventAudienceKind::User,
        "DOCUMENT" => EventAudienceKind::Document,
        _ => return Err(GovernanceError::Internal),
    };
    let minimum_access = row
        .get::<Option<String>, _>("minimum_access")
        .map(|value| match value.as_str() {
            "VIEWER" => Ok(StreamAccess::Viewer),
            "CONTRIBUTOR" => Ok(StreamAccess::Contributor),
            "EDITOR" => Ok(StreamAccess::Editor),
            _ => Err(GovernanceError::Internal),
        })
        .transpose()?;
    let audience = EventAudience {
        kind,
        id: row.get("audience_id"),
        minimum_access,
    };
    if !audience.is_valid() {
        return Err(GovernanceError::Internal);
    }
    Ok(WorkspaceStreamEvent {
        id: row.get("id"),
        workspace_id: row.get("workspace_id"),
        sequence: row.get("sequence"),
        aggregate_id: row.get("aggregate_id"),
        event_type: row.get("event_type"),
        version: row.get("event_version"),
        payload: row.get("payload_json"),
        audience,
        correlation_id: row.get("correlation_id"),
        occurred_at: row.get("occurred_at"),
    })
}
