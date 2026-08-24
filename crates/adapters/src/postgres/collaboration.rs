use adoc_application::{
    collaboration::{
        CollaborationRepository, Discussion, DiscussionAction, DiscussionCommand, DiscussionDetail,
        DiscussionPage, DiscussionStatus, InboxAction, InboxCommand, InboxFilter, InboxItem,
        InboxKind, InboxPage, Message, MessageAction, MessageCommand, Topic, TopicInput, TopicKind,
        may_edit_message,
    },
    governance::GovernanceError,
    permission::Access,
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
