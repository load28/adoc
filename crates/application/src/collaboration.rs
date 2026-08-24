use std::sync::Arc;

pub use adoc_collaboration::*;
use adoc_ports::BoxFuture;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::{
    document::ValidatedContent,
    governance::{Command, GovernanceError},
    identity::{Clock, SecureRandom},
};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RichMessage {
    pub body: Value,
    #[serde(default)]
    pub mention_user_ids: Vec<Uuid>,
    #[serde(default)]
    pub attachment_ids: Vec<Uuid>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TopicInput {
    pub kind: TopicKind,
    pub label: String,
    pub text: Option<String>,
    pub target_id: Option<Uuid>,
    pub region: Option<Value>,
    pub url: Option<String>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateDiscussionInput {
    pub title: String,
    pub message: RichMessage,
    pub topics: Vec<TopicInput>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateDiscussionInput {
    pub title: String,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadAllInput {
    pub before: DateTime<Utc>,
}
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ReviewDecisionInputKind {
    Approve,
    RequestChanges,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReviewDecisionInput {
    pub decision: ReviewDecisionInputKind,
    pub discussion_id: Option<Uuid>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReviewCancelInput {
    pub reason: String,
}

#[derive(Clone, Debug)]
pub struct DiscussionCommand {
    pub workspace_id: Uuid,
    pub document_id: Option<Uuid>,
    pub discussion_id: Uuid,
    pub topic_id: Option<Uuid>,
    pub expected_revision: Option<i64>,
    pub title: Option<String>,
    pub topics: Vec<(Uuid, TopicInput)>,
    pub first_message: Option<(Uuid, RichMessage)>,
    pub command: Command,
    pub action: DiscussionAction,
}
#[derive(Clone, Copy, Debug)]
pub enum DiscussionAction {
    Create,
    Update,
    Close,
    Reopen,
    AddTopic,
    RemoveTopic,
}
#[derive(Clone, Debug)]
pub struct MessageCommand {
    pub workspace_id: Uuid,
    pub discussion_id: Uuid,
    pub message_id: Uuid,
    pub expected_revision: Option<i64>,
    pub message: Option<RichMessage>,
    pub command: Command,
    pub action: MessageAction,
}
#[derive(Clone, Copy, Debug)]
pub enum MessageAction {
    Create,
    Update,
    Redact,
}
#[derive(Clone, Debug)]
pub struct InboxCommand {
    pub workspace_id: Uuid,
    pub item_id: Option<Uuid>,
    pub before: Option<DateTime<Utc>>,
    pub command: Command,
    pub action: InboxAction,
}
#[derive(Clone, Copy, Debug)]
pub enum InboxAction {
    Read,
    ReadAll,
    Resolve,
}
#[derive(Clone, Debug)]
pub struct ReviewCommand {
    pub workspace_id: Uuid,
    pub document_id: Option<Uuid>,
    pub review_id: Uuid,
    pub expected_revision: i64,
    pub decision: Option<ReviewDecisionInput>,
    pub reason: Option<String>,
    pub command: Command,
    pub action: ReviewAction,
}
#[derive(Clone, Copy, Debug)]
pub enum ReviewAction {
    Request,
    Decide,
    Cancel,
}

pub trait CollaborationRepository: Send + Sync {
    fn list_discussions<'a>(
        &'a self,
        actor: Uuid,
        workspace: Uuid,
        document: Uuid,
        cursor: Option<String>,
    ) -> BoxFuture<'a, Result<DiscussionPage, GovernanceError>>;
    fn get_discussion<'a>(
        &'a self,
        actor: Uuid,
        workspace: Uuid,
        discussion: Uuid,
        cursor: Option<String>,
    ) -> BoxFuture<'a, Result<DiscussionDetail, GovernanceError>>;
    fn mutate_discussion<'a>(
        &'a self,
        input: DiscussionCommand,
    ) -> BoxFuture<'a, Result<Discussion, GovernanceError>>;
    fn mutate_message<'a>(
        &'a self,
        input: MessageCommand,
    ) -> BoxFuture<'a, Result<Message, GovernanceError>>;
    fn list_inbox<'a>(
        &'a self,
        actor: Uuid,
        workspace: Uuid,
        cursor: Option<String>,
        filter: InboxFilter,
    ) -> BoxFuture<'a, Result<InboxPage, GovernanceError>>;
    fn mutate_inbox<'a>(
        &'a self,
        input: InboxCommand,
    ) -> BoxFuture<'a, Result<Value, GovernanceError>>;
    fn get_review<'a>(
        &'a self,
        actor: Uuid,
        workspace: Uuid,
        review: Uuid,
    ) -> BoxFuture<'a, Result<Review, GovernanceError>>;
    fn mutate_review<'a>(
        &'a self,
        input: ReviewCommand,
    ) -> BoxFuture<'a, Result<Review, GovernanceError>>;
}

pub struct CollaborationService {
    repository: Arc<dyn CollaborationRepository>,
    clock: Arc<dyn Clock>,
    random: Arc<dyn SecureRandom>,
}
impl CollaborationService {
    pub fn new(
        repository: Arc<dyn CollaborationRepository>,
        clock: Arc<dyn Clock>,
        random: Arc<dyn SecureRandom>,
    ) -> Self {
        Self {
            repository,
            clock,
            random,
        }
    }
    pub async fn list_discussions(
        &self,
        actor: Uuid,
        workspace: Uuid,
        document: Uuid,
        cursor: Option<String>,
    ) -> Result<DiscussionPage, GovernanceError> {
        self.repository
            .list_discussions(actor, workspace, document, cursor)
            .await
    }
    pub async fn get_discussion(
        &self,
        actor: Uuid,
        workspace: Uuid,
        discussion: Uuid,
        cursor: Option<String>,
    ) -> Result<DiscussionDetail, GovernanceError> {
        self.repository
            .get_discussion(actor, workspace, discussion, cursor)
            .await
    }
    pub async fn create_discussion(
        &self,
        actor: Uuid,
        workspace: Uuid,
        document: Uuid,
        key: &str,
        input: CreateDiscussionInput,
    ) -> Result<Discussion, GovernanceError> {
        let title = normalized_title(&input.title).ok_or(GovernanceError::Validation)?;
        validate_topics(&input.topics)?;
        validate_message(&input.message)?;
        let now = self.clock.now();
        let command = command(actor, "createDiscussion", key, &input, now)?;
        let topics = input
            .topics
            .into_iter()
            .map(|topic| Ok((self.uuid(now)?, topic)))
            .collect::<Result<Vec<_>, GovernanceError>>()?;
        self.repository
            .mutate_discussion(DiscussionCommand {
                workspace_id: workspace,
                document_id: Some(document),
                discussion_id: self.uuid(now)?,
                topic_id: None,
                expected_revision: None,
                title: Some(title),
                topics,
                first_message: Some((self.uuid(now)?, input.message)),
                command,
                action: DiscussionAction::Create,
            })
            .await
    }
    pub async fn update_discussion(
        &self,
        actor: Uuid,
        workspace: Uuid,
        id: Uuid,
        rev: i64,
        key: &str,
        input: UpdateDiscussionInput,
    ) -> Result<Discussion, GovernanceError> {
        let title = normalized_title(&input.title).ok_or(GovernanceError::Validation)?;
        self.discussion_mutation(
            actor,
            workspace,
            id,
            rev,
            key,
            "updateDiscussion",
            DiscussionAction::Update,
            Some(title),
            None,
            None,
        )
        .await
    }
    pub async fn close(
        &self,
        actor: Uuid,
        workspace: Uuid,
        id: Uuid,
        rev: i64,
        key: &str,
    ) -> Result<Discussion, GovernanceError> {
        self.discussion_mutation(
            actor,
            workspace,
            id,
            rev,
            key,
            "closeDiscussion",
            DiscussionAction::Close,
            None,
            None,
            None,
        )
        .await
    }
    pub async fn reopen(
        &self,
        actor: Uuid,
        workspace: Uuid,
        id: Uuid,
        rev: i64,
        key: &str,
    ) -> Result<Discussion, GovernanceError> {
        self.discussion_mutation(
            actor,
            workspace,
            id,
            rev,
            key,
            "reopenDiscussion",
            DiscussionAction::Reopen,
            None,
            None,
            None,
        )
        .await
    }
    pub async fn add_topic(
        &self,
        actor: Uuid,
        workspace: Uuid,
        id: Uuid,
        rev: i64,
        key: &str,
        topic: TopicInput,
    ) -> Result<Discussion, GovernanceError> {
        validate_topics(std::slice::from_ref(&topic))?;
        let topic_id = self.uuid(self.clock.now())?;
        self.discussion_mutation(
            actor,
            workspace,
            id,
            rev,
            key,
            "addDiscussionTopic",
            DiscussionAction::AddTopic,
            None,
            Some((topic_id, topic)),
            None,
        )
        .await
    }
    pub async fn remove_topic(
        &self,
        actor: Uuid,
        workspace: Uuid,
        id: Uuid,
        topic: Uuid,
        rev: i64,
        key: &str,
    ) -> Result<Discussion, GovernanceError> {
        self.discussion_mutation(
            actor,
            workspace,
            id,
            rev,
            key,
            "removeDiscussionTopic",
            DiscussionAction::RemoveTopic,
            None,
            None,
            Some(topic),
        )
        .await
    }
    #[allow(clippy::too_many_arguments)]
    async fn discussion_mutation(
        &self,
        actor: Uuid,
        workspace: Uuid,
        id: Uuid,
        rev: i64,
        key: &str,
        op: &'static str,
        action: DiscussionAction,
        title: Option<String>,
        topic: Option<(Uuid, TopicInput)>,
        topic_id: Option<Uuid>,
    ) -> Result<Discussion, GovernanceError> {
        let now = self.clock.now();
        let hash_input = serde_json::json!({"id":id,"revision":rev,"title":title,"topic":topic,"topicId":topic_id});
        let command = command(actor, op, key, &hash_input, now)?;
        self.repository
            .mutate_discussion(DiscussionCommand {
                workspace_id: workspace,
                document_id: None,
                discussion_id: id,
                topic_id,
                expected_revision: Some(rev),
                title,
                topics: topic.into_iter().collect(),
                first_message: None,
                command,
                action,
            })
            .await
    }
    pub async fn create_message(
        &self,
        actor: Uuid,
        workspace: Uuid,
        discussion: Uuid,
        key: &str,
        message: RichMessage,
    ) -> Result<Message, GovernanceError> {
        validate_message(&message)?;
        self.message_mutation(
            actor,
            workspace,
            discussion,
            self.uuid(self.clock.now())?,
            None,
            key,
            "createMessage",
            MessageAction::Create,
            Some(message),
        )
        .await
    }
    #[allow(clippy::too_many_arguments)]
    pub async fn update_message(
        &self,
        actor: Uuid,
        workspace: Uuid,
        discussion: Uuid,
        id: Uuid,
        rev: i64,
        key: &str,
        message: RichMessage,
    ) -> Result<Message, GovernanceError> {
        validate_message(&message)?;
        self.message_mutation(
            actor,
            workspace,
            discussion,
            id,
            Some(rev),
            key,
            "updateMessage",
            MessageAction::Update,
            Some(message),
        )
        .await
    }
    pub async fn redact_message(
        &self,
        actor: Uuid,
        workspace: Uuid,
        discussion: Uuid,
        id: Uuid,
        rev: i64,
        key: &str,
    ) -> Result<Message, GovernanceError> {
        self.message_mutation(
            actor,
            workspace,
            discussion,
            id,
            Some(rev),
            key,
            "deleteMessage",
            MessageAction::Redact,
            None,
        )
        .await
    }
    #[allow(clippy::too_many_arguments)]
    async fn message_mutation(
        &self,
        actor: Uuid,
        workspace: Uuid,
        discussion: Uuid,
        id: Uuid,
        rev: Option<i64>,
        key: &str,
        op: &'static str,
        action: MessageAction,
        message: Option<RichMessage>,
    ) -> Result<Message, GovernanceError> {
        let now = self.clock.now();
        let command = command(
            actor,
            op,
            key,
            &serde_json::json!({"id":id,"revision":rev,"message":message}),
            now,
        )?;
        self.repository
            .mutate_message(MessageCommand {
                workspace_id: workspace,
                discussion_id: discussion,
                message_id: id,
                expected_revision: rev,
                message,
                command,
                action,
            })
            .await
    }
    pub async fn list_inbox(
        &self,
        actor: Uuid,
        workspace: Uuid,
        cursor: Option<String>,
        filter: InboxFilter,
    ) -> Result<InboxPage, GovernanceError> {
        self.repository
            .list_inbox(actor, workspace, cursor, filter)
            .await
    }
    pub async fn inbox(
        &self,
        actor: Uuid,
        workspace: Uuid,
        item: Option<Uuid>,
        before: Option<DateTime<Utc>>,
        key: &str,
        action: InboxAction,
    ) -> Result<Value, GovernanceError> {
        let now = self.clock.now();
        let op = match action {
            InboxAction::Read => "markInboxItemRead",
            InboxAction::ReadAll => "markAllInboxRead",
            InboxAction::Resolve => "resolveInboxItem",
        };
        let command = command(
            actor,
            op,
            key,
            &serde_json::json!({"itemId":item,"before":before}),
            now,
        )?;
        self.repository
            .mutate_inbox(InboxCommand {
                workspace_id: workspace,
                item_id: item,
                before,
                command,
                action,
            })
            .await
    }
    pub async fn get_review(
        &self,
        actor: Uuid,
        workspace: Uuid,
        review: Uuid,
    ) -> Result<Review, GovernanceError> {
        self.repository.get_review(actor, workspace, review).await
    }
    pub async fn request_review(
        &self,
        actor: Uuid,
        workspace: Uuid,
        document: Uuid,
        draft_revision: i64,
        key: &str,
    ) -> Result<Review, GovernanceError> {
        let now = self.clock.now();
        let command = command(
            actor,
            "requestReview",
            key,
            &serde_json::json!({"documentId":document,"draftRevision":draft_revision}),
            now,
        )?;
        self.repository
            .mutate_review(ReviewCommand {
                workspace_id: workspace,
                document_id: Some(document),
                review_id: self.uuid(now)?,
                expected_revision: draft_revision,
                decision: None,
                reason: None,
                command,
                action: ReviewAction::Request,
            })
            .await
    }
    pub async fn submit_review_decision(
        &self,
        actor: Uuid,
        workspace: Uuid,
        review: Uuid,
        review_revision: i64,
        key: &str,
        input: ReviewDecisionInput,
    ) -> Result<Review, GovernanceError> {
        if matches!(input.decision, ReviewDecisionInputKind::RequestChanges)
            != input.discussion_id.is_some()
        {
            return Err(GovernanceError::Validation);
        }
        let now = self.clock.now();
        let command = command(actor, "submitReviewDecision", key, &input, now)?;
        self.repository
            .mutate_review(ReviewCommand {
                workspace_id: workspace,
                document_id: None,
                review_id: review,
                expected_revision: review_revision,
                decision: Some(input),
                reason: None,
                command,
                action: ReviewAction::Decide,
            })
            .await
    }
    pub async fn cancel_review(
        &self,
        actor: Uuid,
        workspace: Uuid,
        review: Uuid,
        review_revision: i64,
        key: &str,
        input: ReviewCancelInput,
    ) -> Result<Review, GovernanceError> {
        let reason = input.reason.trim();
        if reason.is_empty() || reason.chars().count() > 1000 {
            return Err(GovernanceError::Validation);
        }
        let now = self.clock.now();
        let command = command(actor, "cancelReview", key, &input, now)?;
        self.repository
            .mutate_review(ReviewCommand {
                workspace_id: workspace,
                document_id: None,
                review_id: review,
                expected_revision: review_revision,
                decision: None,
                reason: Some(reason.to_owned()),
                command,
                action: ReviewAction::Cancel,
            })
            .await
    }
    fn uuid(&self, now: DateTime<Utc>) -> Result<Uuid, GovernanceError> {
        self.random
            .uuid_v7(now)
            .map_err(|_| GovernanceError::Internal)
    }
}
fn validate_message(input: &RichMessage) -> Result<(), GovernanceError> {
    ValidatedContent::parse(input.body.clone()).map_err(|_| GovernanceError::Validation)?;
    let mut attachments = input.attachment_ids.clone();
    attachments.sort_unstable();
    attachments.dedup();
    if attachments.len() != input.attachment_ids.len() {
        return Err(GovernanceError::Validation);
    }
    let mut ids = input.mention_user_ids.clone();
    ids.sort();
    ids.dedup();
    if ids.len() != input.mention_user_ids.len() {
        return Err(GovernanceError::Validation);
    }
    Ok(())
}
fn validate_topics(items: &[TopicInput]) -> Result<(), GovernanceError> {
    if items.is_empty() {
        return Err(GovernanceError::DiscussionTopicRequired);
    }
    for item in items {
        if item.label.trim().is_empty() || item.label.chars().count() > 500 {
            return Err(GovernanceError::DiscussionTargetInvalid);
        }
        let valid = match item.kind {
            TopicKind::Text => {
                item.text
                    .as_ref()
                    .is_some_and(|v| !v.trim().is_empty() && v.chars().count() <= 5000)
                    && item.target_id.is_none()
                    && item.region.is_none()
                    && item.url.is_none()
            }
            TopicKind::Document => {
                item.target_id.is_some()
                    && item.text.is_none()
                    && item.region.is_none()
                    && item.url.is_none()
            }
            TopicKind::Region => {
                item.target_id.is_some()
                    && item.region.is_some()
                    && item.text.is_none()
                    && item.url.is_none()
            }
            TopicKind::External => {
                item.url
                    .as_ref()
                    .is_some_and(|v| v.starts_with("https://") && v.len() <= 2048)
                    && item.text.is_none()
                    && item.target_id.is_none()
                    && item.region.is_none()
            }
        };
        if !valid {
            return Err(GovernanceError::DiscussionTargetInvalid);
        }
    }
    Ok(())
}
fn command<T: Serialize>(
    actor: Uuid,
    operation_id: &'static str,
    key: &str,
    input: &T,
    now: DateTime<Utc>,
) -> Result<Command, GovernanceError> {
    if key.trim().is_empty() || key.len() > 255 {
        return Err(GovernanceError::Validation);
    }
    let request = serde_json::to_vec(input).map_err(|_| GovernanceError::Internal)?;
    Ok(Command {
        actor_id: actor,
        operation_id,
        idempotency_key: key.to_owned(),
        request_hash: format!("{:x}", Sha256::digest(request)),
        now,
        expires_at: now + Duration::hours(24),
    })
}
