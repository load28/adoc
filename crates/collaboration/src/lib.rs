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
