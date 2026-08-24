#![forbid(unsafe_code)]

//! Collaboration bounded context.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DiscussionStatus {
    Open,
    Closed,
}
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TopicKind {
    Text,
    Document,
    Region,
    External,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Topic {
    pub id: Uuid,
    pub kind: TopicKind,
    #[serde(flatten)]
    pub target: Value,
    pub label: String,
    pub rank: String,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Discussion {
    pub id: Uuid,
    pub document_id: Uuid,
    pub title: String,
    pub status: DiscussionStatus,
    pub topics: Vec<Topic>,
    pub revision: i64,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub id: Uuid,
    pub author_id: Uuid,
    pub body: Value,
    pub mention_user_ids: Vec<Uuid>,
    pub revision: i64,
    pub created_at: DateTime<Utc>,
    pub edited_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscussionDetail {
    pub discussion: Discussion,
    pub messages: Vec<Message>,
    pub next_cursor: Option<String>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscussionPage {
    pub items: Vec<Discussion>,
    pub next_cursor: Option<String>,
}
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum InboxKind {
    ReviewRequested,
    ReviewDecided,
    Mentioned,
    DiscussionChanged,
    PermissionChanged,
    AiJobCompleted,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InboxItem {
    pub id: Uuid,
    pub kind: InboxKind,
    pub target: Value,
    pub revision: i64,
    pub created_at: DateTime<Utc>,
    pub read_at: Option<DateTime<Utc>>,
    pub resolved_at: Option<DateTime<Utc>>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InboxPage {
    pub items: Vec<InboxItem>,
    pub next_cursor: Option<String>,
}
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum InboxFilter {
    All,
    Unread,
    Actionable,
    Resolved,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ReviewStatus {
    Requested,
    Approved,
    ChangesRequested,
    Cancelled,
    Invalidated,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ReviewDecision {
    Pending,
    Approved,
    ChangesRequested,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewAssignment {
    pub reviewer_id: Uuid,
    pub decision: ReviewDecision,
    pub discussion_id: Option<Uuid>,
    pub decided_at: Option<DateTime<Utc>>,
    pub revision: i64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Review {
    pub id: Uuid,
    pub document_id: Uuid,
    pub draft_id: Uuid,
    pub draft_revision: i64,
    pub requested_by: Uuid,
    pub policy_snapshot: Value,
    pub policy_outdated: bool,
    pub status: ReviewStatus,
    pub assignments: Vec<ReviewAssignment>,
    pub requested_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub revision: i64,
}

pub fn review_status(assignments: &[ReviewAssignment], required: usize) -> ReviewStatus {
    if assignments
        .iter()
        .any(|assignment| assignment.decision == ReviewDecision::ChangesRequested)
    {
        ReviewStatus::ChangesRequested
    } else if assignments
        .iter()
        .filter(|assignment| assignment.decision == ReviewDecision::Approved)
        .count()
        >= required
    {
        ReviewStatus::Approved
    } else {
        ReviewStatus::Requested
    }
}

pub fn normalized_title(value: &str) -> Option<String> {
    let value = value.trim();
    (1..=500)
        .contains(&value.chars().count())
        .then(|| value.to_owned())
}
pub fn may_edit_message(
    author: Uuid,
    actor: Uuid,
    created_at: DateTime<Utc>,
    now: DateTime<Utc>,
) -> bool {
    author == actor && now >= created_at && now <= created_at + chrono::Duration::minutes(15)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assignment(decision: ReviewDecision) -> ReviewAssignment {
        ReviewAssignment {
            reviewer_id: Uuid::now_v7(),
            decision,
            discussion_id: None,
            decided_at: None,
            revision: 0,
        }
    }

    #[test]
    fn review_threshold_prioritizes_changes_and_counts_distinct_assignments() {
        assert_eq!(
            review_status(
                &[
                    assignment(ReviewDecision::Approved),
                    assignment(ReviewDecision::Pending)
                ],
                2
            ),
            ReviewStatus::Requested
        );
        assert_eq!(
            review_status(
                &[
                    assignment(ReviewDecision::Approved),
                    assignment(ReviewDecision::Approved)
                ],
                2
            ),
            ReviewStatus::Approved
        );
        assert_eq!(
            review_status(
                &[
                    assignment(ReviewDecision::Approved),
                    assignment(ReviewDecision::ChangesRequested)
                ],
                1
            ),
            ReviewStatus::ChangesRequested
        );
    }
}
