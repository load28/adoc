use std::sync::Arc;

use adoc_ports::BoxFuture;
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use serde::Serialize;
use uuid::Uuid;

pub use adoc_operations::{EventAudienceKind, StreamAccess, StreamCursor, WorkspaceStreamEvent};

use crate::{
    governance::{GovernanceError, MembershipRole},
    permission::{Access, PermissionService},
};

#[derive(Clone, Debug)]
pub struct StreamMembership {
    pub role: MembershipRole,
    pub revision: i64,
    pub high_sequence: i64,
    pub minimum_sequence: Option<i64>,
}

pub trait StreamRepository: Send + Sync {
    fn membership<'a>(
        &'a self,
        actor: Uuid,
        workspace: Uuid,
    ) -> BoxFuture<'a, Result<StreamMembership, GovernanceError>>;
    fn page<'a>(
        &'a self,
        workspace: Uuid,
        after_sequence: i64,
        limit: i64,
    ) -> BoxFuture<'a, Result<Vec<WorkspaceStreamEvent>, GovernanceError>>;
}

#[derive(Clone, Debug)]
pub struct StreamSession {
    pub actor_id: Uuid,
    pub workspace_id: Uuid,
    pub membership_revision: i64,
    pub role: MembershipRole,
    pub cursor: StreamCursor,
}

#[derive(Clone, Debug)]
pub struct StreamOpen {
    pub session: StreamSession,
    pub reset_required: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamEnvelope {
    pub event_id: Uuid,
    pub workspace_id: Uuid,
    pub aggregate_id: Uuid,
    pub sequence: i64,
    #[serde(rename = "type")]
    pub event_type: String,
    pub version: i32,
    pub occurred_at: chrono::DateTime<chrono::Utc>,
    pub correlation_id: String,
    pub payload: serde_json::Value,
}

#[derive(Clone, Debug)]
pub struct StreamDelivery {
    pub cursor: String,
    pub event_type: String,
    pub envelope: StreamEnvelope,
}

#[derive(Clone, Debug)]
pub struct StreamPage {
    pub deliveries: Vec<StreamDelivery>,
    pub cursor: String,
    pub reset_required: bool,
}

#[derive(Clone)]
pub struct StreamService {
    repository: Arc<dyn StreamRepository>,
    permission: Arc<PermissionService>,
}

impl StreamService {
    pub fn new(repository: Arc<dyn StreamRepository>, permission: Arc<PermissionService>) -> Self {
        Self {
            repository,
            permission,
        }
    }

    pub async fn open(
        &self,
        actor: Uuid,
        workspace: Uuid,
        cursor: Option<&str>,
    ) -> Result<StreamOpen, GovernanceError> {
        let membership = self.repository.membership(actor, workspace).await?;
        let cursor = match cursor {
            Some(value) => decode_cursor(value, workspace)?,
            None => StreamCursor {
                version: 1,
                workspace_id: workspace,
                sequence: membership.high_sequence,
                event_id: Uuid::nil(),
            },
        };
        if cursor.sequence > membership.high_sequence {
            return Err(GovernanceError::StreamCursorInvalid);
        }
        let reset_required = membership
            .minimum_sequence
            .is_some_and(|minimum| cursor.sequence < minimum.saturating_sub(1));
        Ok(StreamOpen {
            session: StreamSession {
                actor_id: actor,
                workspace_id: workspace,
                membership_revision: membership.revision,
                role: membership.role,
                cursor,
            },
            reset_required,
        })
    }

    pub async fn next_page(
        &self,
        session: &mut StreamSession,
    ) -> Result<StreamPage, GovernanceError> {
        let membership = self
            .repository
            .membership(session.actor_id, session.workspace_id)
            .await?;
        if membership.revision != session.membership_revision || membership.role != session.role {
            return Ok(StreamPage {
                deliveries: Vec::new(),
                cursor: encode_cursor(&session.cursor)?,
                reset_required: true,
            });
        }
        let events = self
            .repository
            .page(session.workspace_id, session.cursor.sequence, 100)
            .await?;
        let mut deliveries = Vec::new();
        for event in events {
            session.cursor.sequence = event.sequence;
            session.cursor.event_id = event.id;
            let cursor = encode_cursor(&session.cursor)?;
            if self.visible(session, &event).await? {
                deliveries.push(StreamDelivery {
                    cursor,
                    event_type: event.event_type.clone(),
                    envelope: StreamEnvelope {
                        event_id: event.id,
                        workspace_id: event.workspace_id,
                        aggregate_id: event.aggregate_id,
                        sequence: event.sequence,
                        event_type: event.event_type,
                        version: event.version,
                        occurred_at: event.occurred_at,
                        correlation_id: event.correlation_id,
                        payload: event.payload,
                    },
                });
            }
        }
        Ok(StreamPage {
            deliveries,
            cursor: encode_cursor(&session.cursor)?,
            reset_required: false,
        })
    }

    async fn visible(
        &self,
        session: &StreamSession,
        event: &WorkspaceStreamEvent,
    ) -> Result<bool, GovernanceError> {
        match event.audience.kind {
            EventAudienceKind::Internal => Ok(false),
            EventAudienceKind::Workspace => Ok(true),
            EventAudienceKind::Admin => Ok(session.role.can_administer()),
            EventAudienceKind::User => Ok(event.audience.id == Some(session.actor_id)),
            EventAudienceKind::Document => {
                let (Some(document), Some(required)) =
                    (event.audience.id, event.audience.minimum_access)
                else {
                    return Err(GovernanceError::Internal);
                };
                let permission = match self
                    .permission
                    .point(session.actor_id, session.workspace_id, document)
                    .await
                {
                    Ok(value) => value,
                    Err(GovernanceError::DocumentNotFound | GovernanceError::PermissionDenied) => {
                        return Ok(false);
                    }
                    Err(error) => return Err(error),
                };
                Ok(permission.access >= access(required))
            }
        }
    }
}

fn access(value: StreamAccess) -> Access {
    match value {
        StreamAccess::Viewer => Access::Viewer,
        StreamAccess::Contributor => Access::Contributor,
        StreamAccess::Editor => Access::Editor,
    }
}

fn encode_cursor(cursor: &StreamCursor) -> Result<String, GovernanceError> {
    let value = serde_json::to_vec(cursor).map_err(|_| GovernanceError::Internal)?;
    Ok(URL_SAFE_NO_PAD.encode(value))
}

fn decode_cursor(value: &str, workspace: Uuid) -> Result<StreamCursor, GovernanceError> {
    if value.is_empty() || value.len() > 512 {
        return Err(GovernanceError::StreamCursorInvalid);
    }
    let decoded = URL_SAFE_NO_PAD
        .decode(value)
        .map_err(|_| GovernanceError::StreamCursorInvalid)?;
    let cursor: StreamCursor =
        serde_json::from_slice(&decoded).map_err(|_| GovernanceError::StreamCursorInvalid)?;
    if cursor.version != 1 || cursor.workspace_id != workspace || cursor.sequence < 0 {
        return Err(GovernanceError::StreamCursorInvalid);
    }
    Ok(cursor)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cursor_is_workspace_bound_and_versioned() {
        let workspace = Uuid::now_v7();
        let cursor = StreamCursor {
            version: 1,
            workspace_id: workspace,
            sequence: 42,
            event_id: Uuid::now_v7(),
        };
        let encoded = encode_cursor(&cursor).unwrap();
        assert_eq!(decode_cursor(&encoded, workspace).unwrap(), cursor);
        assert!(decode_cursor(&encoded, Uuid::now_v7()).is_err());
    }
}
