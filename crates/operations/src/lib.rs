#![forbid(unsafe_code)]

//! File, audit, and retention bounded context.

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AuditActorKind {
    User,
    System,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuditActor {
    pub kind: AuditActorKind,
    pub user_id: Option<Uuid>,
}

impl AuditActor {
    #[must_use]
    pub fn user(user_id: Uuid) -> Self {
        Self {
            kind: AuditActorKind::User,
            user_id: Some(user_id),
        }
    }

    #[must_use]
    pub fn system() -> Self {
        Self {
            kind: AuditActorKind::System,
            user_id: None,
        }
    }

    #[must_use]
    pub fn is_valid(&self) -> bool {
        matches!(self.kind, AuditActorKind::User) == self.user_id.is_some()
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AuditAction {
    WorkspaceCreated,
    WorkspaceUpdated,
    WorkspaceDeletionScheduled,
    WorkspaceRestored,
    WorkspacePurged,
    MemberInvited,
    MemberAdded,
    MemberRoleChanged,
    MemberRemoved,
    GroupCreated,
    GroupUpdated,
    GroupDeleted,
    GroupMemberAdded,
    GroupMemberRemoved,
    PermissionChanged,
    PublishPolicyChanged,
    DocumentCreated,
    DocumentRenamed,
    DocumentMoved,
    DocumentTrashed,
    DocumentRestored,
    DocumentPurged,
    DraftCreated,
    VersionPublished,
    PublicLinkCreated,
    PublicLinkRevoked,
    DiscussionCreated,
    DiscussionClosed,
    DiscussionReopened,
    ReviewRequested,
    ReviewApproved,
    ReviewChangesRequested,
    VocabularyCreated,
    VocabularyUpdated,
    VocabularyDeprecated,
    FileDeleted,
    AiProposalApplied,
    SecurityActionRecorded,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AuditTargetKind {
    Workspace,
    Membership,
    Invitation,
    Group,
    Permission,
    PublishPolicy,
    Document,
    Draft,
    Version,
    PublicLink,
    Discussion,
    Review,
    Vocabulary,
    File,
    AiProposal,
    Security,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuditTarget {
    pub kind: AuditTargetKind,
    pub id: Uuid,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(untagged)]
pub enum AuditValue {
    String(String),
    Integer(i64),
    Boolean(bool),
    Null,
}

pub type AuditFields = BTreeMap<String, AuditValue>;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuditEvent {
    pub id: Uuid,
    pub sequence: i64,
    pub actor: AuditActor,
    pub action: AuditAction,
    pub target: AuditTarget,
    pub before: Option<AuditFields>,
    pub after: Option<AuditFields>,
    pub metadata: AuditFields,
    pub correlation_id: String,
    pub occurred_at: DateTime<Utc>,
    pub redacted_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuditPage {
    pub items: Vec<AuditEvent>,
    pub next_cursor: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PurgeTargetKind {
    Document,
    Workspace,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PurgeStatus {
    Pending,
    Running,
    Retry,
    Completed,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PurgeStep {
    Pending,
    AccessRevoked,
    ObjectsCaptured,
    DomainPurged,
    ObjectsPurged,
    AuditRedacted,
    Completed,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PurgeRun {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub target_kind: PurgeTargetKind,
    pub target_id: Uuid,
    pub status: PurgeStatus,
    pub step: PurgeStep,
    pub attempt: i32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PurgeObject {
    pub ledger_id: Uuid,
    pub storage_key: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EventAudienceKind {
    Internal,
    Workspace,
    Admin,
    User,
    Document,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StreamAccess {
    Viewer,
    Contributor,
    Editor,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EventAudience {
    pub kind: EventAudienceKind,
    pub id: Option<Uuid>,
    pub minimum_access: Option<StreamAccess>,
}

impl EventAudience {
    #[must_use]
    pub fn internal() -> Self {
        Self {
            kind: EventAudienceKind::Internal,
            id: None,
            minimum_access: None,
        }
    }

    #[must_use]
    pub fn workspace() -> Self {
        Self {
            kind: EventAudienceKind::Workspace,
            id: None,
            minimum_access: None,
        }
    }

    #[must_use]
    pub fn admin() -> Self {
        Self {
            kind: EventAudienceKind::Admin,
            id: None,
            minimum_access: None,
        }
    }

    #[must_use]
    pub fn user(id: Uuid) -> Self {
        Self {
            kind: EventAudienceKind::User,
            id: Some(id),
            minimum_access: None,
        }
    }

    #[must_use]
    pub fn document(id: Uuid, minimum_access: StreamAccess) -> Self {
        Self {
            kind: EventAudienceKind::Document,
            id: Some(id),
            minimum_access: Some(minimum_access),
        }
    }

    #[must_use]
    pub fn is_valid(&self) -> bool {
        match self.kind {
            EventAudienceKind::Internal
            | EventAudienceKind::Workspace
            | EventAudienceKind::Admin => self.id.is_none() && self.minimum_access.is_none(),
            EventAudienceKind::User => self.id.is_some() && self.minimum_access.is_none(),
            EventAudienceKind::Document => self.id.is_some() && self.minimum_access.is_some(),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum JobKind {
    OutboxToStream,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum JobStatus {
    Queued,
    Running,
    CancelRequested,
    Succeeded,
    Failed,
    Cancelled,
    TimedOut,
    DeadLetter,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum JobPriorityBucket {
    Interactive,
    Normal,
    Background,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Job {
    pub id: Uuid,
    pub workspace_id: Option<Uuid>,
    pub kind: JobKind,
    pub payload: serde_json::Value,
    pub status: JobStatus,
    pub priority: i16,
    pub sequence: i64,
    pub attempt: i32,
    pub max_attempts: i32,
    pub correlation_id: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct JobSignal {
    pub id: Uuid,
    pub bucket: JobPriorityBucket,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StreamWake {
    pub workspace_id: Uuid,
    pub sequence: i64,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WorkspaceStreamEvent {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub sequence: i64,
    pub aggregate_id: Uuid,
    pub event_type: String,
    pub version: i32,
    pub payload: serde_json::Value,
    pub audience: EventAudience,
    pub correlation_id: String,
    pub occurred_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StreamCursor {
    pub version: u8,
    pub workspace_id: Uuid,
    pub sequence: i64,
    pub event_id: Uuid,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FileStatus {
    Uploading,
    Validating,
    Ready,
    Failed,
    Deleted,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileAsset {
    pub id: Uuid,
    pub original_name: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub checksum_sha256: String,
    pub status: FileStatus,
    pub failure_code: Option<String>,
    pub ready_at: Option<DateTime<Utc>>,
    pub revision: i64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileUpload {
    pub asset_id: Uuid,
    pub upload_url: String,
    pub upload_token: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ByteRange {
    pub start: u64,
    pub end_inclusive: u64,
}
impl ByteRange {
    pub fn parse(value: &str, size: u64) -> Option<Self> {
        let raw = value.strip_prefix("bytes=")?;
        if raw.contains(',') || size == 0 {
            return None;
        }
        let (left, right) = raw.split_once('-')?;
        let (start, end) = if left.is_empty() {
            let suffix = right.parse::<u64>().ok()?;
            if suffix == 0 {
                return None;
            }
            (size.saturating_sub(suffix), size - 1)
        } else {
            let start = left.parse::<u64>().ok()?;
            let end = if right.is_empty() {
                size - 1
            } else {
                right.parse::<u64>().ok()?
            };
            (start, end)
        };
        (start <= end && end < size).then_some(Self {
            start,
            end_inclusive: end,
        })
    }
    pub fn len(self) -> u64 {
        self.end_inclusive - self.start + 1
    }
    pub fn is_empty(self) -> bool {
        false
    }
}

pub fn sanitize_filename(value: &str) -> Option<String> {
    let cleaned = value
        .chars()
        .filter(|c| !c.is_control())
        .map(|c| if matches!(c, '/' | '\\') { '�' } else { c })
        .collect::<String>();
    let cleaned = cleaned.split_whitespace().collect::<Vec<_>>().join(" ");
    (cleaned.chars().count() >= 1
        && cleaned.chars().count() <= 500
        && cleaned != "."
        && cleaned != "..")
        .then_some(cleaned)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ranges_and_names_are_bounded() {
        assert_eq!(ByteRange::parse("bytes=2-4", 10).unwrap().len(), 3);
        assert_eq!(ByteRange::parse("bytes=-3", 10).unwrap().start, 7);
        assert!(ByteRange::parse("bytes=4-2", 10).is_none());
        assert_eq!(sanitize_filename("../a\n.pdf").unwrap(), "..�a.pdf");
    }
}
