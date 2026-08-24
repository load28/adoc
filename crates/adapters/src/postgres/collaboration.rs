use adoc_application::{
    collaboration::{
        CollaborationRepository, Discussion, DiscussionAction, DiscussionCommand, DiscussionDetail,
        DiscussionPage, DiscussionStatus, InboxAction, InboxCommand, InboxFilter, InboxItem,
        InboxKind, InboxPage, Message, MessageAction, MessageCommand, Review, ReviewAction,
        ReviewAssignment, ReviewCommand, ReviewDecision, ReviewDecisionInputKind, ReviewStatus,
        Topic, TopicInput, TopicKind, may_edit_message, review_status,
    },
    governance::GovernanceError,
    permission::{Access, PublishMode, ReviewerRule},
};
use adoc_ports::BoxFuture;
use chrono::{DateTime, Utc};
use serde_json::{Value, json};
use sqlx::{PgPool, Postgres, Row, Transaction, postgres::PgRow};
use uuid::Uuid;

use super::{
    PostgresStore,
    document::{require_access, require_effective_active},
    governance::{
        OutboxEvent, append_event, begin_workspace, check_revision, complete_workspace, map_store,
    },
    permission::load_effective_policy,
};

#[derive(Clone)]
pub struct PostgresCollaborationRepository {
    pool: PgPool,
}
impl PostgresCollaborationRepository {
    pub fn new(store: &PostgresStore) -> Self {
        Self {
            pool: store.pool().clone(),
        }
    }
}

impl CollaborationRepository for PostgresCollaborationRepository {
    fn list_discussions<'a>(
        &'a self,
        actor: Uuid,
        workspace: Uuid,
        document: Uuid,
        cursor: Option<String>,
    ) -> BoxFuture<'a, Result<DiscussionPage, GovernanceError>> {
        Box::pin(async move {
            let cursor = parse_cursor(cursor)?;
            let mut tx = self.pool.begin().await.map_err(map_store)?;
            require_access(&mut tx, actor, workspace, document, Access::Viewer, false).await?;
            require_effective_active(&mut tx, workspace, document).await?;
            let rows=sqlx::query("SELECT id,document_id,title,status::text,revision,created_at FROM discussions WHERE workspace_id=$1 AND document_id=$2 AND ($3::uuid IS NULL OR (created_at,id)<(SELECT created_at,id FROM discussions WHERE workspace_id=$1 AND id=$3)) ORDER BY created_at DESC,id DESC LIMIT 51").bind(workspace).bind(document).bind(cursor).fetch_all(&mut *tx).await.map_err(map_store)?;
            let mut items = Vec::new();
            for row in rows.iter().take(50) {
                items.push(load_discussion_row(&mut tx, workspace, row).await?)
            }
            let next_cursor =
                (rows.len() > 50).then(|| items.last().expect("nonempty").id.to_string());
            tx.commit().await.map_err(map_store)?;
            Ok(DiscussionPage { items, next_cursor })
        })
    }
    fn get_discussion<'a>(
        &'a self,
        actor: Uuid,
        workspace: Uuid,
        id: Uuid,
        cursor: Option<String>,
    ) -> BoxFuture<'a, Result<DiscussionDetail, GovernanceError>> {
        Box::pin(async move {
            let cursor = parse_cursor(cursor)?;
            let mut tx = self.pool.begin().await.map_err(map_store)?;
            let row = discussion_row(&mut tx, workspace, id, false).await?;
            let document: Uuid = row.get("document_id");
            require_access(&mut tx, actor, workspace, document, Access::Viewer, false)
                .await
                .map_err(|_| GovernanceError::DiscussionNotFound)?;
            require_effective_active(&mut tx, workspace, document)
                .await
                .map_err(|_| GovernanceError::DiscussionNotFound)?;
            let discussion = load_discussion_row(&mut tx, workspace, &row).await?;
            let rows=sqlx::query("SELECT id,author_id,body_json,mention_user_ids,revision,created_at,edited_at,deleted_at FROM messages WHERE workspace_id=$1 AND discussion_id=$2 AND ($3::uuid IS NULL OR (created_at,id)>(SELECT created_at,id FROM messages WHERE workspace_id=$1 AND id=$3)) ORDER BY created_at,id LIMIT 51").bind(workspace).bind(id).bind(cursor).fetch_all(&mut *tx).await.map_err(map_store)?;
            let messages = rows
                .iter()
                .take(50)
                .map(message)
                .collect::<Result<Vec<_>, _>>()?;
            let next_cursor =
                (rows.len() > 50).then(|| messages.last().expect("nonempty").id.to_string());
            tx.commit().await.map_err(map_store)?;
            Ok(DiscussionDetail {
                discussion,
                messages,
                next_cursor,
            })
        })
    }
    fn mutate_discussion<'a>(
        &'a self,
        input: DiscussionCommand,
    ) -> BoxFuture<'a, Result<Discussion, GovernanceError>> {
        Box::pin(async move {
            let mut tx = self.pool.begin().await.map_err(map_store)?;
            let document = if matches!(input.action, DiscussionAction::Create) {
                input.document_id.ok_or(GovernanceError::Validation)?
            } else {
                discussion_row(&mut tx, input.workspace_id, input.discussion_id, false)
                    .await?
                    .get("document_id")
            };
            let access = require_access(
                &mut tx,
                input.command.actor_id,
                input.workspace_id,
                document,
                Access::Contributor,
                false,
            )
            .await;
            if matches!(input.action, DiscussionAction::Create) {
                access?;
            } else {
                access.map_err(|_| GovernanceError::DiscussionNotFound)?;
            }
            if let Some(replay) =
                begin_workspace::<Discussion>(&mut tx, input.workspace_id, &input.command).await?
            {
                tx.commit().await.map_err(map_store)?;
                return Ok(replay);
            }
            let active = require_effective_active(&mut tx, input.workspace_id, document).await;
            if matches!(input.action, DiscussionAction::Create) {
                active?;
            } else {
                active.map_err(|_| GovernanceError::DiscussionNotFound)?;
            }
            match input.action {
                DiscussionAction::Create => {
                    sqlx::query("INSERT INTO discussions(id,workspace_id,document_id,title,created_by,created_at) VALUES($1,$2,$3,$4,$5,$6)").bind(input.discussion_id).bind(input.workspace_id).bind(document).bind(input.title.as_deref()).bind(input.command.actor_id).bind(input.command.now).execute(&mut *tx).await.map_err(map_store)?;
                    for (index, (id, topic)) in input.topics.iter().enumerate() {
                        validate_topic_target(
                            &mut tx,
                            input.command.actor_id,
                            input.workspace_id,
                            topic,
                        )
                        .await?;
                        insert_topic(
                            &mut tx,
                            input.workspace_id,
                            input.discussion_id,
                            *id,
                            topic,
                            index as i64 + 1,
                        )
                        .await?
                    }
                    let (id, msg) = input
                        .first_message
                        .as_ref()
                        .ok_or(GovernanceError::Validation)?;
                    insert_message(
                        &mut tx,
                        input.workspace_id,
                        input.discussion_id,
                        *id,
                        input.command.actor_id,
                        msg,
                        input.command.now,
                    )
                    .await?;
                    sync_mentions(
                        &mut tx,
                        input.workspace_id,
                        document,
                        input.discussion_id,
                        *id,
                        &[],
                        &msg.mention_user_ids,
                        input.command.now,
                    )
                    .await?;
                }
                action => {
                    let row =
                        discussion_row(&mut tx, input.workspace_id, input.discussion_id, true)
                            .await?;
                    let status: String = row.get("status");
                    check_revision(
                        row.get("revision"),
                        input.expected_revision.ok_or(GovernanceError::Validation)?,
                    )?;
                    if status == "CLOSED" && !matches!(action, DiscussionAction::Reopen) {
                        return Err(GovernanceError::DiscussionClosed);
                    }
                    if matches!(
                        action,
                        DiscussionAction::Update | DiscussionAction::RemoveTopic
                    ) {
                        require_creator_or_editor(
                            &mut tx,
                            &row,
                            input.command.actor_id,
                            input.workspace_id,
                            document,
                        )
                        .await?;
                    }
                    match action {
                        DiscussionAction::Update => {
                            sqlx::query("UPDATE discussions SET title=$3,revision=revision+1 WHERE workspace_id=$1 AND id=$2").bind(input.workspace_id).bind(input.discussion_id).bind(input.title.as_deref()).execute(&mut *tx).await.map_err(map_store)?;
                        }
                        DiscussionAction::Close => {
                            sqlx::query("UPDATE discussions SET status='CLOSED',closed_at=$3,revision=revision+1 WHERE workspace_id=$1 AND id=$2").bind(input.workspace_id).bind(input.discussion_id).bind(input.command.now).execute(&mut *tx).await.map_err(map_store)?;
                        }
                        DiscussionAction::Reopen => {
                            if status != "CLOSED" {
                                return Err(GovernanceError::DiscussionStateInvalid);
                            }
                            sqlx::query("UPDATE discussions SET status='OPEN',closed_at=NULL,revision=revision+1 WHERE workspace_id=$1 AND id=$2").bind(input.workspace_id).bind(input.discussion_id).execute(&mut *tx).await.map_err(map_store)?;
                        }
                        DiscussionAction::AddTopic => {
                            let (id, topic) =
                                input.topics.first().ok_or(GovernanceError::Validation)?;
                            validate_topic_target(
                                &mut tx,
                                input.command.actor_id,
                                input.workspace_id,
                                topic,
                            )
                            .await?;
                            let rank: i64 = sqlx::query_scalar(
                                "SELECT count(*)+1 FROM discussion_topics WHERE discussion_id=$1",
                            )
                            .bind(input.discussion_id)
                            .fetch_one(&mut *tx)
                            .await
                            .map_err(map_store)?;
                            insert_topic(
                                &mut tx,
                                input.workspace_id,
                                input.discussion_id,
                                *id,
                                topic,
                                rank,
                            )
                            .await?;
                            bump_discussion(&mut tx, input.workspace_id, input.discussion_id)
                                .await?;
                        }
                        DiscussionAction::RemoveTopic => {
                            let count: i64 = sqlx::query_scalar(
                                "SELECT count(*) FROM discussion_topics WHERE discussion_id=$1",
                            )
                            .bind(input.discussion_id)
                            .fetch_one(&mut *tx)
                            .await
                            .map_err(map_store)?;
                            if count <= 1 {
                                return Err(GovernanceError::DiscussionTopicRequired);
                            }
                            let result=sqlx::query("DELETE FROM discussion_topics WHERE workspace_id=$1 AND discussion_id=$2 AND id=$3").bind(input.workspace_id).bind(input.discussion_id).bind(input.topic_id).execute(&mut *tx).await.map_err(map_store)?;
                            if result.rows_affected() != 1 {
                                return Err(GovernanceError::DiscussionTargetInvalid);
                            }
                            bump_discussion(&mut tx, input.workspace_id, input.discussion_id)
                                .await?;
                        }
                        DiscussionAction::Create => unreachable!(),
                    }
                }
            }
            let row =
                discussion_row(&mut tx, input.workspace_id, input.discussion_id, false).await?;
            let result = load_discussion_row(&mut tx, input.workspace_id, &row).await?;
            append_event(&mut tx,OutboxEvent{workspace_id:input.workspace_id,aggregate_kind:"Discussion",aggregate_id:input.discussion_id,sequence:result.revision+1,event_type:"DiscussionChanged.v1",payload:json!({"discussionId":result.id,"documentId":result.document_id,"revision":result.revision,"action":format!("{:?}",input.action)}),occurred_at:input.command.now}).await?;
            complete_workspace(&mut tx, input.workspace_id, &input.command, 200, &result).await?;
            tx.commit().await.map_err(map_store)?;
            Ok(result)
        })
    }
    fn mutate_message<'a>(
        &'a self,
        input: MessageCommand,
    ) -> BoxFuture<'a, Result<Message, GovernanceError>> {
        Box::pin(async move {
            let mut tx = self.pool.begin().await.map_err(map_store)?;
            let discussion =
                discussion_row(&mut tx, input.workspace_id, input.discussion_id, false).await?;
            let document: Uuid = discussion.get("document_id");
            require_access(
                &mut tx,
                input.command.actor_id,
                input.workspace_id,
                document,
                Access::Contributor,
                false,
            )
            .await
            .map_err(|_| GovernanceError::DiscussionNotFound)?;
            if let Some(replay) =
                begin_workspace::<Message>(&mut tx, input.workspace_id, &input.command).await?
            {
                tx.commit().await.map_err(map_store)?;
                return Ok(replay);
            }
            require_effective_active(&mut tx, input.workspace_id, document)
                .await
                .map_err(|_| GovernanceError::DiscussionNotFound)?;
            let discussion =
                discussion_row(&mut tx, input.workspace_id, input.discussion_id, true).await?;
            if discussion.get::<String, _>("status") != "OPEN" {
                return Err(GovernanceError::DiscussionClosed);
            };
            match input.action {
                MessageAction::Create => {
                    let msg = input.message.as_ref().ok_or(GovernanceError::Validation)?;
                    insert_message(
                        &mut tx,
                        input.workspace_id,
                        input.discussion_id,
                        input.message_id,
                        input.command.actor_id,
                        msg,
                        input.command.now,
                    )
                    .await?;
                    sync_mentions(
                        &mut tx,
                        input.workspace_id,
                        document,
                        input.discussion_id,
                        input.message_id,
                        &[],
                        &msg.mention_user_ids,
                        input.command.now,
                    )
                    .await?
                }
                MessageAction::Update | MessageAction::Redact => {
                    let row=sqlx::query("SELECT id,author_id,body_json,mention_user_ids,revision,created_at,edited_at,deleted_at FROM messages WHERE workspace_id=$1 AND discussion_id=$2 AND id=$3 FOR UPDATE").bind(input.workspace_id).bind(input.discussion_id).bind(input.message_id).fetch_optional(&mut *tx).await.map_err(map_store)?.ok_or(GovernanceError::MessageNotFound)?;
                    if row.get::<Option<DateTime<Utc>>, _>("deleted_at").is_some() {
                        return Err(GovernanceError::MessageStateInvalid);
                    }
                    let revision: i64 = row.get("revision");
                    check_revision(
                        revision,
                        input.expected_revision.ok_or(GovernanceError::Validation)?,
                    )?;
                    let author: Uuid = row.get("author_id");
                    let in_window = may_edit_message(
                        author,
                        input.command.actor_id,
                        row.get("created_at"),
                        input.command.now,
                    );
                    if !in_window {
                        if matches!(input.action, MessageAction::Update) {
                            return Err(GovernanceError::MessageEditWindowExpired);
                        }
                        require_access(
                            &mut tx,
                            input.command.actor_id,
                            input.workspace_id,
                            document,
                            Access::Editor,
                            false,
                        )
                        .await?;
                    }
                    let old_mentions: Vec<Uuid> = row.get("mention_user_ids");
                    sqlx::query("INSERT INTO message_revisions(message_id,revision,body_json,mention_user_ids,deleted_at,edited_by,edited_at) VALUES($1,$2,$3,$4,$5,$6,$7)").bind(input.message_id).bind(revision+1).bind(row.get::<Value,_>("body_json")).bind(&old_mentions).bind(row.get::<Option<DateTime<Utc>>,_>("deleted_at")).bind(input.command.actor_id).bind(input.command.now).execute(&mut *tx).await.map_err(map_store)?;
                    let (body, mentions, deleted) = match &input.message {
                        Some(msg) => (msg.body.clone(), msg.mention_user_ids.clone(), None),
                        None => (
                            json!({"schemaVersion":1,"root":{"type":"doc","children":[]}}),
                            Vec::new(),
                            Some(input.command.now),
                        ),
                    };
                    sqlx::query("UPDATE messages SET body_json=$2,mention_user_ids=$3,revision=revision+1,edited_at=$4,deleted_at=$5 WHERE id=$1").bind(input.message_id).bind(body).bind(&mentions).bind(input.command.now).bind(deleted).execute(&mut *tx).await.map_err(map_store)?;
                    sync_mentions(
                        &mut tx,
                        input.workspace_id,
                        document,
                        input.discussion_id,
                        input.message_id,
                        &old_mentions,
                        &mentions,
                        input.command.now,
                    )
                    .await?
                }
            }
            let row=sqlx::query("SELECT id,author_id,body_json,mention_user_ids,revision,created_at,edited_at,deleted_at FROM messages WHERE id=$1").bind(input.message_id).fetch_one(&mut *tx).await.map_err(map_store)?;
            let result = message(&row)?;
            append_event(&mut tx,OutboxEvent{workspace_id:input.workspace_id,aggregate_kind:"Message",aggregate_id:input.message_id,sequence:result.revision+1,event_type:"MessageChanged.v1",payload:json!({"messageId":result.id,"discussionId":input.discussion_id,"revision":result.revision,"action":format!("{:?}",input.action)}),occurred_at:input.command.now}).await?;
            complete_workspace(&mut tx, input.workspace_id, &input.command, 200, &result).await?;
            tx.commit().await.map_err(map_store)?;
            Ok(result)
        })
    }
    fn list_inbox<'a>(
        &'a self,
        actor: Uuid,
        workspace: Uuid,
        cursor: Option<String>,
        filter: InboxFilter,
    ) -> BoxFuture<'a, Result<InboxPage, GovernanceError>> {
        Box::pin(async move {
            let cursor = parse_cursor(cursor)?;
            let active: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM memberships m JOIN workspaces w ON w.id=m.workspace_id WHERE m.workspace_id=$1 AND m.user_id=$2 AND m.status='ACTIVE' AND w.status='ACTIVE')")
                .bind(workspace)
                .bind(actor)
                .fetch_one(&self.pool)
                .await
                .map_err(map_store)?;
            if !active {
                return Err(GovernanceError::WorkspaceNotFound);
            }
            let condition = match filter {
                InboxFilter::All => "TRUE",
                InboxFilter::Unread => "read_at IS NULL",
                InboxFilter::Actionable => "resolved_at IS NULL",
                InboxFilter::Resolved => "resolved_at IS NOT NULL",
            };
            let sql = format!(
                "SELECT id,kind,target_json,revision,created_at,read_at,resolved_at FROM inbox_items WHERE workspace_id=$1 AND user_id=$2 AND {condition} AND ($3::uuid IS NULL OR (created_at,id)<(SELECT created_at,id FROM inbox_items WHERE workspace_id=$1 AND user_id=$2 AND id=$3)) ORDER BY created_at DESC,id DESC LIMIT 51"
            );
            let rows = sqlx::query(&sql)
                .bind(workspace)
                .bind(actor)
                .bind(cursor)
                .fetch_all(&self.pool)
                .await
                .map_err(map_store)?;
            let items = rows
                .iter()
                .take(50)
                .map(inbox)
                .collect::<Result<Vec<_>, _>>()?;
            let next_cursor =
                (rows.len() > 50).then(|| items.last().expect("nonempty").id.to_string());
            Ok(InboxPage { items, next_cursor })
        })
    }
    fn mutate_inbox<'a>(
        &'a self,
        input: InboxCommand,
    ) -> BoxFuture<'a, Result<Value, GovernanceError>> {
        Box::pin(async move {
            let mut tx = self.pool.begin().await.map_err(map_store)?;
            let active: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM memberships m JOIN workspaces w ON w.id=m.workspace_id WHERE m.workspace_id=$1 AND m.user_id=$2 AND m.status='ACTIVE' AND w.status='ACTIVE')")
                .bind(input.workspace_id)
                .bind(input.command.actor_id)
                .fetch_one(&mut *tx)
                .await
                .map_err(map_store)?;
            if !active {
                return Err(GovernanceError::WorkspaceNotFound);
            }
            if let Some(replay) =
                begin_workspace::<Value>(&mut tx, input.workspace_id, &input.command).await?
            {
                tx.commit().await.map_err(map_store)?;
                return Ok(replay);
            }
            let result = match input.action {
                InboxAction::Read | InboxAction::Resolve => {
                    let column = if matches!(input.action, InboxAction::Read) {
                        "read_at"
                    } else {
                        "resolved_at"
                    };
                    let sql = format!(
                        "UPDATE inbox_items SET {column}=$4,revision=revision+1 WHERE workspace_id=$1 AND user_id=$2 AND id=$3 AND {column} IS NULL RETURNING id,kind,target_json,revision,created_at,read_at,resolved_at"
                    );
                    let changed = sqlx::query(&sql)
                        .bind(input.workspace_id)
                        .bind(input.command.actor_id)
                        .bind(input.item_id)
                        .bind(input.command.now)
                        .fetch_optional(&mut *tx)
                        .await
                        .map_err(map_store)?;
                    let row = if let Some(row) = changed {
                        let item = inbox(&row)?;
                        append_inbox_event(
                            &mut tx,
                            input.workspace_id,
                            &item,
                            if matches!(input.action, InboxAction::Read) {
                                "READ"
                            } else {
                                "RESOLVED"
                            },
                            input.command.now,
                        )
                        .await?;
                        row
                    } else {
                        sqlx::query("SELECT id,kind,target_json,revision,created_at,read_at,resolved_at FROM inbox_items WHERE workspace_id=$1 AND user_id=$2 AND id=$3")
                            .bind(input.workspace_id).bind(input.command.actor_id).bind(input.item_id)
                            .fetch_optional(&mut *tx).await.map_err(map_store)?.ok_or(GovernanceError::InboxItemNotFound)?
                    };
                    serde_json::to_value(inbox(&row)?).map_err(|_| GovernanceError::Internal)?
                }
                InboxAction::ReadAll => {
                    let rows=sqlx::query("UPDATE inbox_items SET read_at=$3,revision=revision+1 WHERE workspace_id=$1 AND user_id=$2 AND read_at IS NULL AND created_at<=$4 RETURNING id,kind,target_json,revision,created_at,read_at,resolved_at").bind(input.workspace_id).bind(input.command.actor_id).bind(input.command.now).bind(input.before.ok_or(GovernanceError::Validation)?).fetch_all(&mut *tx).await.map_err(map_store)?;
                    for row in &rows {
                        append_inbox_event(
                            &mut tx,
                            input.workspace_id,
                            &inbox(row)?,
                            "READ",
                            input.command.now,
                        )
                        .await?;
                    }
                    json!({"count":rows.len()})
                }
            };
            complete_workspace(&mut tx, input.workspace_id, &input.command, 200, &result).await?;
            tx.commit().await.map_err(map_store)?;
            Ok(result)
        })
    }
    fn get_review<'a>(
        &'a self,
        actor: Uuid,
        workspace: Uuid,
        id: Uuid,
    ) -> BoxFuture<'a, Result<Review, GovernanceError>> {
        Box::pin(async move {
            let mut tx = self.pool.begin().await.map_err(map_store)?;
            let row = review_row(&mut tx, workspace, id, false).await?;
            let document: Uuid = row.get("document_id");
            require_access(&mut tx, actor, workspace, document, Access::Viewer, false)
                .await
                .map_err(|_| GovernanceError::ReviewNotFound)?;
            let result = load_review(&mut tx, workspace, &row).await?;
            tx.commit().await.map_err(map_store)?;
            Ok(result)
        })
    }
    fn mutate_review<'a>(
        &'a self,
        input: ReviewCommand,
    ) -> BoxFuture<'a, Result<Review, GovernanceError>> {
        Box::pin(async move {
            let mut tx = self.pool.begin().await.map_err(map_store)?;
            let document = if matches!(input.action, ReviewAction::Request) {
                input.document_id.ok_or(GovernanceError::Validation)?
            } else {
                review_row(&mut tx, input.workspace_id, input.review_id, false)
                    .await?
                    .get("document_id")
            };
            let minimum = if matches!(input.action, ReviewAction::Request) {
                Access::Contributor
            } else {
                Access::Viewer
            };
            require_access(
                &mut tx,
                input.command.actor_id,
                input.workspace_id,
                document,
                minimum,
                false,
            )
            .await
            .map_err(|_| {
                if matches!(input.action, ReviewAction::Request) {
                    GovernanceError::DocumentNotFound
                } else {
                    GovernanceError::ReviewNotFound
                }
            })?;
            if let Some(replay) =
                begin_workspace::<Review>(&mut tx, input.workspace_id, &input.command).await?
            {
                tx.commit().await.map_err(map_store)?;
                return Ok(replay);
            }
            match input.action {
                ReviewAction::Request => request_review(&mut tx, &input, document).await?,
                ReviewAction::Decide => decide_review(&mut tx, &input, document).await?,
                ReviewAction::Cancel => cancel_review(&mut tx, &input, document).await?,
            }
            let row = review_row(&mut tx, input.workspace_id, input.review_id, false).await?;
            let result = load_review(&mut tx, input.workspace_id, &row).await?;
            let event_action = if result.status == ReviewStatus::Invalidated {
                "INVALIDATED".to_owned()
            } else {
                format!("{:?}", input.action)
            };
            append_event(&mut tx,OutboxEvent{workspace_id:input.workspace_id,aggregate_kind:"Review",aggregate_id:input.review_id,sequence:result.revision+1,event_type:"ReviewChanged.v1",payload:json!({"reviewId":result.id,"documentId":result.document_id,"draftRevision":result.draft_revision,"revision":result.revision,"action":event_action}),occurred_at:input.command.now}).await?;
            complete_workspace(&mut tx, input.workspace_id, &input.command, 200, &result).await?;
            tx.commit().await.map_err(map_store)?;
            Ok(result)
        })
    }
}

async fn request_review(
    tx: &mut Transaction<'_, Postgres>,
    input: &ReviewCommand,
    document: Uuid,
) -> Result<(), GovernanceError> {
    require_effective_active(tx, input.workspace_id, document).await?;
    let draft=sqlx::query("SELECT id,revision,content_json FROM drafts WHERE workspace_id=$1 AND document_id=$2 FOR UPDATE").bind(input.workspace_id).bind(document).fetch_optional(&mut **tx).await.map_err(map_store)?.ok_or(GovernanceError::DraftNotFound)?;
    check_revision(draft.get("revision"), input.expected_revision)?;
    let policy = load_effective_policy(&mut **tx, input.workspace_id, document).await?;
    if policy.mode != PublishMode::ReviewRequired {
        return Err(GovernanceError::PublishPolicyInvalid);
    }
    let active=sqlx::query("SELECT id,status::text,policy_snapshot_json FROM reviews WHERE workspace_id=$1 AND document_id=$2 AND (status='REQUESTED' OR (status='APPROVED' AND draft_id=$3)) FOR UPDATE").bind(input.workspace_id).bind(document).bind(draft.get::<Uuid,_>("id")).fetch_optional(&mut **tx).await.map_err(map_store)?;
    if let Some(active) = active {
        let snapshot: Value = active.get("policy_snapshot_json");
        let source = serde_json::to_value(policy.inherited_from_document_id)
            .map_err(|_| GovernanceError::Internal)?;
        let outdated = snapshot.get("policyRevision").and_then(Value::as_i64)
            != Some(policy.revision)
            || snapshot.get("sourceDocumentId") != Some(&source);
        if active.get::<String, _>("status") == "APPROVED" && outdated {
            invalidate_reviews(tx, input.workspace_id, &[document], input.command.now).await?;
        } else {
            return Err(GovernanceError::ReviewStateInvalid);
        }
    }
    let reviewers = eligible_reviewers(
        tx,
        input.workspace_id,
        document,
        input.command.actor_id,
        &policy.reviewer_rule,
    )
    .await?;
    if reviewers.len() < policy.required_approvals as usize {
        return Err(GovernanceError::ReviewNotEligible);
    }
    let snapshot = json!({"sourceDocumentId":policy.inherited_from_document_id,"policyRevision":policy.revision,"requiredApprovals":policy.required_approvals,"reviewerRule":policy.reviewer_rule,"reviewerIds":reviewers});
    sqlx::query("INSERT INTO reviews(id,workspace_id,document_id,draft_id,draft_revision,policy_snapshot_json,status,requested_by,requested_at) VALUES($1,$2,$3,$4,$5,$6,'REQUESTED',$7,$8)").bind(input.review_id).bind(input.workspace_id).bind(document).bind(draft.get::<Uuid,_>("id")).bind(input.expected_revision).bind(&snapshot).bind(input.command.actor_id).bind(input.command.now).execute(&mut **tx).await.map_err(map_store)?;
    for reviewer in reviewers {
        sqlx::query(
            "INSERT INTO review_assignments(workspace_id,review_id,reviewer_id) VALUES($1,$2,$3)",
        )
        .bind(input.workspace_id)
        .bind(input.review_id)
        .bind(reviewer)
        .execute(&mut **tx)
        .await
        .map_err(map_store)?;
        let source = format!("review:{}:{}", input.review_id, reviewer);
        let row=sqlx::query("INSERT INTO inbox_items(id,workspace_id,user_id,kind,source_key,target_json,created_at) VALUES($1,$2,$3,'REVIEW_REQUESTED',$4,$5,$6) ON CONFLICT(workspace_id,user_id,source_key) DO UPDATE SET resolved_at=NULL,revision=inbox_items.revision+1 RETURNING id,kind,target_json,revision,created_at,read_at,resolved_at").bind(Uuid::now_v7()).bind(input.workspace_id).bind(reviewer).bind(source).bind(json!({"kind":"REVIEW","id":input.review_id})).bind(input.command.now).fetch_one(&mut **tx).await.map_err(map_store)?;
        append_inbox_event(
            tx,
            input.workspace_id,
            &inbox(&row)?,
            "REVIEW_REQUESTED",
            input.command.now,
        )
        .await?;
    }
    Ok(())
}

async fn decide_review(
    tx: &mut Transaction<'_, Postgres>,
    input: &ReviewCommand,
    document: Uuid,
) -> Result<(), GovernanceError> {
    let review = review_row(tx, input.workspace_id, input.review_id, true).await?;
    check_revision(review.get("revision"), input.expected_revision)?;
    let status: String = review.get("status");
    if !matches!(status.as_str(), "REQUESTED" | "APPROVED") {
        return Err(GovernanceError::ReviewStateInvalid);
    }
    let decision = input.decision.as_ref().ok_or(GovernanceError::Validation)?;
    let assignment=sqlx::query("SELECT decision::text,revision FROM review_assignments WHERE workspace_id=$1 AND review_id=$2 AND reviewer_id=$3 FOR UPDATE").bind(input.workspace_id).bind(input.review_id).bind(input.command.actor_id).fetch_optional(&mut **tx).await.map_err(map_store)?.ok_or(GovernanceError::ReviewNotEligible)?;
    let reviewers=sqlx::query_scalar::<_,Uuid>("SELECT reviewer_id FROM review_assignments WHERE workspace_id=$1 AND review_id=$2 ORDER BY reviewer_id").bind(input.workspace_id).bind(input.review_id).fetch_all(&mut **tx).await.map_err(map_store)?;
    for reviewer in reviewers {
        if require_access(
            tx,
            reviewer,
            input.workspace_id,
            document,
            Access::Viewer,
            false,
        )
        .await
        .is_err()
        {
            sqlx::query("UPDATE reviews SET status='INVALIDATED',resolved_at=$3,revision=revision+1 WHERE workspace_id=$1 AND id=$2").bind(input.workspace_id).bind(input.review_id).bind(input.command.now).execute(&mut **tx).await.map_err(map_store)?;
            resolve_review_inbox(tx, input.workspace_id, input.review_id, input.command.now)
                .await?;
            return Ok(());
        }
    }
    if let Some(discussion) = decision.discussion_id {
        let row = discussion_row(tx, input.workspace_id, discussion, false)
            .await
            .map_err(|_| GovernanceError::DiscussionTargetInvalid)?;
        if row.get::<Uuid, _>("document_id") != document {
            return Err(GovernanceError::DiscussionTargetInvalid);
        }
    }
    let next = assignment.get::<i64, _>("revision") + 1;
    let value = match decision.decision {
        ReviewDecisionInputKind::Approve => "APPROVED",
        ReviewDecisionInputKind::RequestChanges => "CHANGES_REQUESTED",
    };
    sqlx::query("INSERT INTO review_decision_revisions(workspace_id,review_id,reviewer_id,revision,decision,discussion_id,decided_at) VALUES($1,$2,$3,$4,$5::review_decision,$6,$7)").bind(input.workspace_id).bind(input.review_id).bind(input.command.actor_id).bind(next).bind(value).bind(decision.discussion_id).bind(input.command.now).execute(&mut **tx).await.map_err(map_store)?;
    sqlx::query("UPDATE review_assignments SET decision=$4::review_decision,discussion_id=$5,decided_at=$6,revision=$7 WHERE workspace_id=$1 AND review_id=$2 AND reviewer_id=$3").bind(input.workspace_id).bind(input.review_id).bind(input.command.actor_id).bind(value).bind(decision.discussion_id).bind(input.command.now).bind(next).execute(&mut **tx).await.map_err(map_store)?;
    let assignments = load_assignments(tx, input.workspace_id, input.review_id).await?;
    let required = review
        .get::<Value, _>("policy_snapshot_json")
        .get("requiredApprovals")
        .and_then(Value::as_u64)
        .ok_or(GovernanceError::Internal)? as usize;
    let next_status = review_status(&assignments, required);
    let resolved = (!matches!(next_status, ReviewStatus::Requested)).then_some(input.command.now);
    sqlx::query("UPDATE reviews SET status=$3::review_status,resolved_at=$4,revision=revision+1 WHERE workspace_id=$1 AND id=$2").bind(input.workspace_id).bind(input.review_id).bind(review_status_text(next_status)).bind(resolved).execute(&mut **tx).await.map_err(map_store)?;
    if !matches!(next_status, ReviewStatus::Requested) {
        resolve_review_inbox(tx, input.workspace_id, input.review_id, input.command.now).await?;
    }
    let source = format!(
        "review-decision:{}:{}:{}",
        input.review_id, input.command.actor_id, next
    );
    let row=sqlx::query("INSERT INTO inbox_items(id,workspace_id,user_id,kind,source_key,target_json,created_at) VALUES($1,$2,$3,'REVIEW_DECIDED',$4,$5,$6) RETURNING id,kind,target_json,revision,created_at,read_at,resolved_at").bind(Uuid::now_v7()).bind(input.workspace_id).bind(review.get::<Uuid,_>("requested_by")).bind(source).bind(json!({"kind":"REVIEW","id":input.review_id})).bind(input.command.now).fetch_one(&mut **tx).await.map_err(map_store)?;
    append_inbox_event(
        tx,
        input.workspace_id,
        &inbox(&row)?,
        "REVIEW_DECIDED",
        input.command.now,
    )
    .await?;
    Ok(())
}

async fn cancel_review(
    tx: &mut Transaction<'_, Postgres>,
    input: &ReviewCommand,
    document: Uuid,
) -> Result<(), GovernanceError> {
    let review = review_row(tx, input.workspace_id, input.review_id, true).await?;
    check_revision(review.get("revision"), input.expected_revision)?;
    if review.get::<String, _>("status") != "REQUESTED" {
        return Err(GovernanceError::ReviewStateInvalid);
    }
    if review.get::<Uuid, _>("requested_by") != input.command.actor_id {
        require_access(
            tx,
            input.command.actor_id,
            input.workspace_id,
            document,
            Access::Editor,
            false,
        )
        .await
        .map_err(|_| GovernanceError::ReviewNotEligible)?;
    }
    let _reason = input.reason.as_deref().ok_or(GovernanceError::Validation)?;
    sqlx::query("UPDATE reviews SET status='CANCELLED',resolved_at=$3,revision=revision+1 WHERE workspace_id=$1 AND id=$2").bind(input.workspace_id).bind(input.review_id).bind(input.command.now).execute(&mut **tx).await.map_err(map_store)?;
    resolve_review_inbox(tx, input.workspace_id, input.review_id, input.command.now).await
}

async fn eligible_reviewers(
    tx: &mut Transaction<'_, Postgres>,
    workspace: Uuid,
    document: Uuid,
    requester: Uuid,
    rule: &ReviewerRule,
) -> Result<Vec<Uuid>, GovernanceError> {
    let candidates = match rule {
        ReviewerRule::AnyEditor => sqlx::query_scalar::<_, Uuid>(
            "SELECT user_id FROM memberships WHERE workspace_id=$1 AND status='ACTIVE' ORDER BY user_id",
        )
        .bind(workspace)
        .fetch_all(&mut **tx)
        .await
        .map_err(map_store)?,
        ReviewerRule::Users { user_ids } => user_ids.clone(),
        ReviewerRule::Groups { group_ids } => sqlx::query_scalar::<_, Uuid>("SELECT DISTINCT gm.user_id FROM group_members gm JOIN memberships m ON m.workspace_id=gm.workspace_id AND m.user_id=gm.user_id AND m.status='ACTIVE' WHERE gm.workspace_id=$1 AND gm.group_id=ANY($2) ORDER BY gm.user_id")
            .bind(workspace).bind(group_ids).fetch_all(&mut **tx).await.map_err(map_store)?,
    };
    let minimum = if matches!(rule, ReviewerRule::AnyEditor) {
        Access::Editor
    } else {
        Access::Viewer
    };
    let mut eligible = Vec::new();
    for candidate in candidates {
        if candidate != requester
            && require_access(tx, candidate, workspace, document, minimum, false)
                .await
                .is_ok()
        {
            eligible.push(candidate);
        }
    }
    eligible.sort();
    eligible.dedup();
    Ok(eligible)
}

async fn review_row<'a>(
    tx: &mut Transaction<'a, Postgres>,
    workspace: Uuid,
    id: Uuid,
    lock: bool,
) -> Result<PgRow, GovernanceError> {
    let sql = if lock {
        "SELECT id,document_id,draft_id,draft_revision,policy_snapshot_json,status::text,requested_by,requested_at,resolved_at,revision FROM reviews WHERE workspace_id=$1 AND id=$2 FOR UPDATE"
    } else {
        "SELECT id,document_id,draft_id,draft_revision,policy_snapshot_json,status::text,requested_by,requested_at,resolved_at,revision FROM reviews WHERE workspace_id=$1 AND id=$2"
    };
    sqlx::query(sql)
        .bind(workspace)
        .bind(id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(map_store)?
        .ok_or(GovernanceError::ReviewNotFound)
}

async fn load_assignments(
    tx: &mut Transaction<'_, Postgres>,
    workspace: Uuid,
    review: Uuid,
) -> Result<Vec<ReviewAssignment>, GovernanceError> {
    sqlx::query("SELECT reviewer_id,decision::text,discussion_id,decided_at,revision FROM review_assignments WHERE workspace_id=$1 AND review_id=$2 ORDER BY reviewer_id")
        .bind(workspace).bind(review).fetch_all(&mut **tx).await.map_err(map_store)?
        .iter().map(|row| Ok(ReviewAssignment { reviewer_id:row.get("reviewer_id"), decision:parse_review_decision(row.get("decision"))?, discussion_id:row.get("discussion_id"), decided_at:row.get("decided_at"), revision:row.get("revision") })).collect()
}

async fn load_review(
    tx: &mut Transaction<'_, Postgres>,
    workspace: Uuid,
    row: &PgRow,
) -> Result<Review, GovernanceError> {
    let document = row.get("document_id");
    let snapshot: Value = row.get("policy_snapshot_json");
    let current = load_effective_policy(&mut **tx, workspace, document).await?;
    let outdated = snapshot.get("policyRevision").and_then(Value::as_i64) != Some(current.revision)
        || snapshot.get("sourceDocumentId")
            != Some(
                &serde_json::to_value(current.inherited_from_document_id)
                    .map_err(|_| GovernanceError::Internal)?,
            );
    Ok(Review {
        id: row.get("id"),
        document_id: document,
        draft_id: row.get("draft_id"),
        draft_revision: row.get("draft_revision"),
        requested_by: row.get("requested_by"),
        policy_snapshot: snapshot,
        policy_outdated: outdated,
        status: parse_review_status(row.get("status"))?,
        assignments: load_assignments(tx, workspace, row.get("id")).await?,
        requested_at: row.get("requested_at"),
        resolved_at: row.get("resolved_at"),
        revision: row.get("revision"),
    })
}

async fn resolve_review_inbox(
    tx: &mut Transaction<'_, Postgres>,
    workspace: Uuid,
    review: Uuid,
    now: DateTime<Utc>,
) -> Result<(), GovernanceError> {
    let rows=sqlx::query("UPDATE inbox_items SET resolved_at=$3,revision=revision+1 WHERE workspace_id=$1 AND source_key LIKE $2 AND resolved_at IS NULL RETURNING id,kind,target_json,revision,created_at,read_at,resolved_at").bind(workspace).bind(format!("review:{review}:%")).bind(now).fetch_all(&mut **tx).await.map_err(map_store)?;
    for row in rows {
        append_inbox_event(tx, workspace, &inbox(&row)?, "RESOLVED", now).await?;
    }
    Ok(())
}

fn parse_review_status(value: String) -> Result<ReviewStatus, GovernanceError> {
    match value.as_str() {
        "REQUESTED" => Ok(ReviewStatus::Requested),
        "APPROVED" => Ok(ReviewStatus::Approved),
        "CHANGES_REQUESTED" => Ok(ReviewStatus::ChangesRequested),
        "CANCELLED" => Ok(ReviewStatus::Cancelled),
        "INVALIDATED" => Ok(ReviewStatus::Invalidated),
        _ => Err(GovernanceError::Internal),
    }
}
fn review_status_text(value: ReviewStatus) -> &'static str {
    match value {
        ReviewStatus::Requested => "REQUESTED",
        ReviewStatus::Approved => "APPROVED",
        ReviewStatus::ChangesRequested => "CHANGES_REQUESTED",
        ReviewStatus::Cancelled => "CANCELLED",
        ReviewStatus::Invalidated => "INVALIDATED",
    }
}
fn parse_review_decision(value: String) -> Result<ReviewDecision, GovernanceError> {
    match value.as_str() {
        "PENDING" => Ok(ReviewDecision::Pending),
        "APPROVED" => Ok(ReviewDecision::Approved),
        "CHANGES_REQUESTED" => Ok(ReviewDecision::ChangesRequested),
        _ => Err(GovernanceError::Internal),
    }
}

pub(super) async fn invalidate_reviews(
    tx: &mut Transaction<'_, Postgres>,
    workspace: Uuid,
    documents: &[Uuid],
    now: DateTime<Utc>,
) -> Result<(), GovernanceError> {
    let rows=sqlx::query("SELECT id,document_id,revision FROM reviews WHERE workspace_id=$1 AND document_id=ANY($2) AND status IN ('REQUESTED','APPROVED') ORDER BY id FOR UPDATE").bind(workspace).bind(documents).fetch_all(&mut **tx).await.map_err(map_store)?;
    for row in rows {
        let review: Uuid = row.get("id");
        let revision:i64=sqlx::query_scalar("UPDATE reviews SET status='INVALIDATED',resolved_at=$3,revision=revision+1 WHERE workspace_id=$1 AND id=$2 RETURNING revision").bind(workspace).bind(review).bind(now).fetch_one(&mut **tx).await.map_err(map_store)?;
        resolve_review_inbox(tx, workspace, review, now).await?;
        append_event(tx,OutboxEvent{workspace_id:workspace,aggregate_kind:"Review",aggregate_id:review,sequence:revision+1,event_type:"ReviewChanged.v1",payload:json!({"reviewId":review,"documentId":row.get::<Uuid,_>("document_id"),"revision":revision,"action":"INVALIDATED"}),occurred_at:now}).await?;
    }
    Ok(())
}

async fn discussion_row<'a>(
    tx: &mut Transaction<'a, Postgres>,
    workspace: Uuid,
    id: Uuid,
    lock: bool,
) -> Result<PgRow, GovernanceError> {
    let sql = if lock {
        "SELECT id,document_id,title,status::text,revision,created_by,created_at FROM discussions WHERE workspace_id=$1 AND id=$2 FOR UPDATE"
    } else {
        "SELECT id,document_id,title,status::text,revision,created_by,created_at FROM discussions WHERE workspace_id=$1 AND id=$2"
    };
    sqlx::query(sql)
        .bind(workspace)
        .bind(id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(map_store)?
        .ok_or(GovernanceError::DiscussionNotFound)
}
async fn load_discussion_row(
    tx: &mut Transaction<'_, Postgres>,
    workspace: Uuid,
    row: &PgRow,
) -> Result<Discussion, GovernanceError> {
    let id: Uuid = row.get("id");
    let topics=sqlx::query("SELECT id,kind,target_json,label,rank FROM discussion_topics WHERE workspace_id=$1 AND discussion_id=$2 ORDER BY rank,id").bind(workspace).bind(id).fetch_all(&mut **tx).await.map_err(map_store)?.iter().map(topic).collect::<Result<Vec<_>,_>>()?;
    Ok(Discussion {
        id,
        document_id: row.get("document_id"),
        title: row.get("title"),
        status: if row.get::<String, _>("status") == "OPEN" {
            DiscussionStatus::Open
        } else {
            DiscussionStatus::Closed
        },
        topics,
        revision: row.get("revision"),
    })
}
fn topic(row: &PgRow) -> Result<Topic, GovernanceError> {
    Ok(Topic {
        id: row.get("id"),
        kind: parse_topic(row.get("kind"))?,
        target: row.get("target_json"),
        label: row.get("label"),
        rank: row.get("rank"),
    })
}
fn message(row: &PgRow) -> Result<Message, GovernanceError> {
    Ok(Message {
        id: row.get("id"),
        author_id: row.get("author_id"),
        body: row.get("body_json"),
        mention_user_ids: row.get("mention_user_ids"),
        revision: row.get("revision"),
        created_at: row.get("created_at"),
        edited_at: row.get("edited_at"),
        deleted_at: row.get("deleted_at"),
    })
}
fn inbox(row: &PgRow) -> Result<InboxItem, GovernanceError> {
    let kind = match row.get::<String, _>("kind").as_str() {
        "REVIEW_REQUESTED" => InboxKind::ReviewRequested,
        "REVIEW_DECIDED" => InboxKind::ReviewDecided,
        "MENTIONED" => InboxKind::Mentioned,
        "DISCUSSION_CHANGED" => InboxKind::DiscussionChanged,
        "PERMISSION_CHANGED" => InboxKind::PermissionChanged,
        "AI_JOB_COMPLETED" => InboxKind::AiJobCompleted,
        _ => return Err(GovernanceError::Internal),
    };
    Ok(InboxItem {
        id: row.get("id"),
        kind,
        target: row.get("target_json"),
        revision: row.get("revision"),
        created_at: row.get("created_at"),
        read_at: row.get("read_at"),
        resolved_at: row.get("resolved_at"),
    })
}
fn parse_topic(value: String) -> Result<TopicKind, GovernanceError> {
    match value.as_str() {
        "TEXT" => Ok(TopicKind::Text),
        "DOCUMENT" => Ok(TopicKind::Document),
        "REGION" => Ok(TopicKind::Region),
        "EXTERNAL" => Ok(TopicKind::External),
        _ => Err(GovernanceError::Internal),
    }
}
fn topic_kind(value: TopicKind) -> &'static str {
    match value {
        TopicKind::Text => "TEXT",
        TopicKind::Document => "DOCUMENT",
        TopicKind::Region => "REGION",
        TopicKind::External => "EXTERNAL",
    }
}
fn topic_target(input: &TopicInput) -> Value {
    match input.kind {
        TopicKind::Text => json!({"text":input.text}),
        TopicKind::Document => json!({"targetId":input.target_id}),
        TopicKind::Region => json!({"targetId":input.target_id,"region":input.region}),
        TopicKind::External => json!({"url":input.url}),
    }
}
async fn validate_topic_target(
    tx: &mut Transaction<'_, Postgres>,
    actor: Uuid,
    workspace: Uuid,
    input: &TopicInput,
) -> Result<(), GovernanceError> {
    if let Some(document) = input.target_id {
        require_access(tx, actor, workspace, document, Access::Viewer, false)
            .await
            .map_err(|_| GovernanceError::DiscussionTargetInvalid)?;
        require_effective_active(tx, workspace, document)
            .await
            .map_err(|_| GovernanceError::DiscussionTargetInvalid)?
    }
    Ok(())
}
async fn insert_topic(
    tx: &mut Transaction<'_, Postgres>,
    workspace: Uuid,
    discussion: Uuid,
    id: Uuid,
    input: &TopicInput,
    rank: i64,
) -> Result<(), GovernanceError> {
    sqlx::query("INSERT INTO discussion_topics(id,workspace_id,discussion_id,kind,target_json,label,rank) VALUES($1,$2,$3,$4,$5,$6,$7)").bind(id).bind(workspace).bind(discussion).bind(topic_kind(input.kind)).bind(topic_target(input)).bind(input.label.trim()).bind(format!("{rank:032}")).execute(&mut **tx).await.map_err(map_store)?;
    Ok(())
}
async fn insert_message(
    tx: &mut Transaction<'_, Postgres>,
    workspace: Uuid,
    discussion: Uuid,
    id: Uuid,
    author: Uuid,
    input: &adoc_application::collaboration::RichMessage,
    now: DateTime<Utc>,
) -> Result<(), GovernanceError> {
    sqlx::query("INSERT INTO messages(id,workspace_id,discussion_id,author_id,body_json,mention_user_ids,created_at) VALUES($1,$2,$3,$4,$5,$6,$7)").bind(id).bind(workspace).bind(discussion).bind(author).bind(&input.body).bind(&input.mention_user_ids).bind(now).execute(&mut **tx).await.map_err(map_store)?;
    Ok(())
}
#[allow(clippy::too_many_arguments)]
async fn sync_mentions(
    tx: &mut Transaction<'_, Postgres>,
    workspace: Uuid,
    document: Uuid,
    discussion: Uuid,
    message: Uuid,
    old: &[Uuid],
    new: &[Uuid],
    now: DateTime<Utc>,
) -> Result<(), GovernanceError> {
    let mut recipients = new.to_vec();
    recipients.sort();
    for user in &recipients {
        let active:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM memberships WHERE workspace_id=$1 AND user_id=$2 AND status='ACTIVE')").bind(workspace).bind(user).fetch_one(&mut **tx).await.map_err(map_store)?;
        if !active
            || require_access(tx, *user, workspace, document, Access::Contributor, false)
                .await
                .is_err()
        {
            return Err(GovernanceError::DiscussionTargetInvalid);
        }
        let source = format!("mention:{message}:{user}");
        let row=sqlx::query("INSERT INTO inbox_items(id,workspace_id,user_id,kind,source_key,target_json,created_at) VALUES($1,$2,$3,'MENTIONED',$4,$5,$6) ON CONFLICT(workspace_id,user_id,source_key) DO UPDATE SET resolved_at=NULL,revision=inbox_items.revision+1 WHERE inbox_items.resolved_at IS NOT NULL RETURNING id,kind,target_json,revision,created_at,read_at,resolved_at").bind(Uuid::now_v7()).bind(workspace).bind(user).bind(source).bind(json!({"kind":"DISCUSSION","id":discussion})).bind(now).fetch_optional(&mut **tx).await.map_err(map_store)?;
        if let Some(row) = row {
            append_inbox_event(tx, workspace, &inbox(&row)?, "MENTIONED", now).await?;
        }
    }
    for user in old.iter().filter(|id| !new.contains(id)) {
        let source = format!("mention:{message}:{user}");
        let row=sqlx::query("UPDATE inbox_items SET resolved_at=$4,revision=revision+1 WHERE workspace_id=$1 AND user_id=$2 AND source_key=$3 AND resolved_at IS NULL RETURNING id,kind,target_json,revision,created_at,read_at,resolved_at").bind(workspace).bind(user).bind(source).bind(now).fetch_optional(&mut **tx).await.map_err(map_store)?;
        if let Some(row) = row {
            append_inbox_event(tx, workspace, &inbox(&row)?, "RESOLVED", now).await?;
        }
    }
    Ok(())
}
async fn append_inbox_event(
    tx: &mut Transaction<'_, Postgres>,
    workspace: Uuid,
    item: &InboxItem,
    action: &'static str,
    now: DateTime<Utc>,
) -> Result<(), GovernanceError> {
    append_event(
        tx,
        OutboxEvent {
            workspace_id: workspace,
            aggregate_kind: "InboxItem",
            aggregate_id: item.id,
            sequence: item.revision + 1,
            event_type: "InboxChanged.v1",
            payload: json!({"itemId":item.id,"revision":item.revision,"action":action}),
            occurred_at: now,
        },
    )
    .await
}
async fn require_creator_or_editor(
    tx: &mut Transaction<'_, Postgres>,
    row: &PgRow,
    actor: Uuid,
    workspace: Uuid,
    document: Uuid,
) -> Result<(), GovernanceError> {
    if row.get::<Uuid, _>("created_by") == actor {
        return Ok(());
    }
    require_access(tx, actor, workspace, document, Access::Editor, false)
        .await
        .map(|_| ())
}
async fn bump_discussion(
    tx: &mut Transaction<'_, Postgres>,
    workspace: Uuid,
    id: Uuid,
) -> Result<(), GovernanceError> {
    sqlx::query("UPDATE discussions SET revision=revision+1 WHERE workspace_id=$1 AND id=$2")
        .bind(workspace)
        .bind(id)
        .execute(&mut **tx)
        .await
        .map_err(map_store)?;
    Ok(())
}
fn parse_cursor(value: Option<String>) -> Result<Option<Uuid>, GovernanceError> {
    value
        .map(|v| Uuid::parse_str(&v).map_err(|_| GovernanceError::Validation))
        .transpose()
}
