use adoc_application::governance::{
    Command, GovernanceError, GovernanceRepository, Group, GroupChange, GroupOperation, Invitation,
    InvitationAcceptance, InvitationChange, InvitationPage, InvitationRole, InvitationStatus,
    Membership, MembershipChange, MembershipRole, MembershipStatus, NewGroup, NewInvitation,
    NewWorkspace, PersistedInvitation, PublishMode, Workspace, WorkspaceChange, WorkspaceDeletion,
    WorkspaceStatus, may_change_role,
};
use adoc_application::operations::{
    AuditAction, AuditEventInput, AuditTarget, AuditTargetKind, EventAudience,
};
use adoc_ports::BoxFuture;
use chrono::{DateTime, Utc};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Value, json};
use sqlx::{PgPool, Postgres, Row, Transaction, postgres::PgRow};
use subtle::ConstantTimeEq;
use uuid::Uuid;

use super::{PostgresStore, append_audit_event};

#[derive(Clone)]
pub struct PostgresGovernanceRepository {
    pool: PgPool,
}

impl PostgresGovernanceRepository {
    pub fn new(store: &PostgresStore) -> Self {
        Self {
            pool: store.pool().clone(),
        }
    }
}

impl GovernanceRepository for PostgresGovernanceRepository {
    fn list_workspaces<'a>(
        &'a self,
        actor: Uuid,
    ) -> BoxFuture<'a, Result<Vec<Workspace>, GovernanceError>> {
        Box::pin(async move {
            let rows = sqlx::query("SELECT w.id,w.name,w.slug,w.status::text,w.revision FROM workspaces w JOIN memberships m ON m.workspace_id=w.id WHERE m.user_id=$1 AND m.status='ACTIVE' AND w.status IN ('ACTIVE','DELETION_SCHEDULED') ORDER BY w.name,w.id")
                .bind(actor).fetch_all(&self.pool).await.map_err(map_store)?;
            rows.iter().map(workspace).collect()
        })
    }

    fn create_workspace<'a>(
        &'a self,
        input: NewWorkspace,
    ) -> BoxFuture<'a, Result<Workspace, GovernanceError>> {
        Box::pin(async move {
            let mut tx = self.pool.begin().await.map_err(map_store)?;
            if let Some(replay) = begin_user::<Workspace>(&mut tx, &input.command).await? {
                tx.commit().await.map_err(map_store)?;
                return Ok(replay);
            }
            let mut created = None;
            for slug in [
                &input.slug_base,
                &format!("{}-{}", input.slug_base, input.slug_suffix),
            ] {
                let row = sqlx::query("INSERT INTO workspaces(id,slug,name,created_by,created_at,updated_at) VALUES($1,$2,$3,$4,$5,$5) ON CONFLICT(slug) DO NOTHING RETURNING id,name,slug,status::text,revision")
                    .bind(input.id).bind(slug).bind(&input.name).bind(input.command.actor_id).bind(input.command.now)
                    .fetch_optional(&mut *tx).await.map_err(map_store)?;
                if let Some(row) = row {
                    created = Some(workspace(&row)?);
                    break;
                }
            }
            let created = created.ok_or(GovernanceError::WorkspaceSlugTaken)?;
            sqlx::query("INSERT INTO memberships(workspace_id,user_id,role,status,joined_at) VALUES($1,$2,'OWNER','ACTIVE',$3)")
                .bind(created.id).bind(input.command.actor_id).bind(input.command.now).execute(&mut *tx).await.map_err(map_store)?;
            append_event(
                &mut tx,
                OutboxEvent {
                    workspace_id: created.id,
                    aggregate_kind: "Workspace",
                    aggregate_id: created.id,
                    sequence: 1,
                    event_type: "WorkspaceChanged.v1",
                    payload: json!({"entityId":created.id,"revision":0,"action":"CREATED"}),
                    audience: EventAudience::workspace(),
                    occurred_at: input.command.now,
                },
            )
            .await?;
            audit_user(
                &mut tx,
                created.id,
                &input.command,
                AuditAction::WorkspaceCreated,
                AuditTargetKind::Workspace,
                created.id,
            )
            .await?;
            complete_user(&mut tx, &input.command, &created).await?;
            tx.commit().await.map_err(map_store)?;
            Ok(created)
        })
    }

    fn get_workspace<'a>(
        &'a self,
        actor: Uuid,
        workspace_id: Uuid,
    ) -> BoxFuture<'a, Result<Workspace, GovernanceError>> {
        Box::pin(async move {
            let row = sqlx::query("SELECT w.id,w.name,w.slug,w.status::text,w.revision FROM workspaces w JOIN memberships m ON m.workspace_id=w.id WHERE w.id=$1 AND m.user_id=$2 AND m.status='ACTIVE' AND w.status IN ('ACTIVE','DELETION_SCHEDULED')")
                .bind(workspace_id).bind(actor).fetch_optional(&self.pool).await.map_err(map_store)?.ok_or(GovernanceError::WorkspaceNotFound)?;
            workspace(&row)
        })
    }

    fn update_workspace<'a>(
        &'a self,
        input: WorkspaceChange,
    ) -> BoxFuture<'a, Result<Workspace, GovernanceError>> {
        Box::pin(async move {
            let mut tx = self.pool.begin().await.map_err(map_store)?;
            require_actor(
                &mut tx,
                input.workspace_id,
                input.command.actor_id,
                Capability::Admin,
            )
            .await?;
            if let Some(replay) =
                begin_workspace::<Workspace>(&mut tx, input.workspace_id, &input.command).await?
            {
                tx.commit().await.map_err(map_store)?;
                return Ok(replay);
            }
            let current = lock_workspace(&mut tx, input.workspace_id).await?;
            check_revision(current.revision, input.expected_revision)?;
            if current.status != WorkspaceStatus::Active {
                return Err(GovernanceError::WorkspaceStateInvalid);
            }
            let row = sqlx::query("UPDATE workspaces SET name=COALESCE($2,name),default_publish_mode=COALESCE($3::publish_mode,default_publish_mode),revision=revision+1,updated_at=$4 WHERE id=$1 RETURNING id,name,slug,status::text,revision")
                .bind(input.workspace_id).bind(input.name).bind(input.default_publish_mode.map(publish_mode)).bind(input.command.now).fetch_one(&mut *tx).await.map_err(map_store)?;
            let result = workspace(&row)?;
            append_event(
                &mut tx,
                OutboxEvent {
                    workspace_id: result.id,
                    aggregate_kind: "Workspace",
                    aggregate_id: result.id,
                    sequence: result.revision + 1,
                    event_type: "WorkspaceChanged.v1",
                    payload: json!({"entityId":result.id,"revision":result.revision,"action":"UPDATED"}),
                    audience: EventAudience::workspace(),
                    occurred_at: input.command.now,
                },
            )
            .await?;
            audit_user(
                &mut tx,
                result.id,
                &input.command,
                AuditAction::WorkspaceUpdated,
                AuditTargetKind::Workspace,
                result.id,
            )
            .await?;
            complete_workspace(&mut tx, input.workspace_id, &input.command, 200, &result).await?;
            tx.commit().await.map_err(map_store)?;
            Ok(result)
        })
    }

    fn change_workspace_deletion<'a>(
        &'a self,
        input: WorkspaceDeletion,
    ) -> BoxFuture<'a, Result<Workspace, GovernanceError>> {
        Box::pin(async move {
            let mut tx = self.pool.begin().await.map_err(map_store)?;
            require_actor(
                &mut tx,
                input.workspace_id,
                input.command.actor_id,
                Capability::Owner,
            )
            .await?;
            if let Some(replay) =
                begin_workspace::<Workspace>(&mut tx, input.workspace_id, &input.command).await?
            {
                tx.commit().await.map_err(map_store)?;
                return Ok(replay);
            }
            let current = lock_workspace(&mut tx, input.workspace_id).await?;
            check_revision(current.revision, input.expected_revision)?;
            let (expected, next) = if input.delete_after.is_some() {
                (WorkspaceStatus::Active, "DELETION_SCHEDULED")
            } else {
                (WorkspaceStatus::DeletionScheduled, "ACTIVE")
            };
            if current.status != expected {
                return Err(GovernanceError::WorkspaceStateInvalid);
            }
            let row = sqlx::query("UPDATE workspaces SET status=$2::workspace_status,delete_after=$3,revision=revision+1,updated_at=$4 WHERE id=$1 RETURNING id,name,slug,status::text,revision")
                .bind(input.workspace_id).bind(next).bind(input.delete_after).bind(input.command.now).fetch_one(&mut *tx).await.map_err(map_store)?;
            let result = workspace(&row)?;
            append_event(&mut tx, OutboxEvent {
                workspace_id: result.id,
                aggregate_kind: "Workspace",
                aggregate_id: result.id,
                sequence: result.revision + 1,
                event_type: "WorkspaceChanged.v1",
                payload: json!({"entityId":result.id,"revision":result.revision,"action":"UPDATED"}),
                audience: EventAudience::workspace(),
                occurred_at: input.command.now,
            })
            .await?;
            audit_user(
                &mut tx,
                result.id,
                &input.command,
                if input.delete_after.is_some() {
                    AuditAction::WorkspaceDeletionScheduled
                } else {
                    AuditAction::WorkspaceRestored
                },
                AuditTargetKind::Workspace,
                result.id,
            )
            .await?;
            complete_workspace(&mut tx, input.workspace_id, &input.command, 200, &result).await?;
            tx.commit().await.map_err(map_store)?;
            Ok(result)
        })
    }

    fn list_members<'a>(
        &'a self,
        actor: Uuid,
        workspace_id: Uuid,
    ) -> BoxFuture<'a, Result<Vec<Membership>, GovernanceError>> {
        Box::pin(async move {
            let mut tx = self.pool.begin().await.map_err(map_store)?;
            require_actor(&mut tx, workspace_id, actor, Capability::Member).await?;
            let rows = sqlx::query("SELECT user_id,role::text,status::text,revision FROM memberships WHERE workspace_id=$1 AND status<>'REMOVED' ORDER BY joined_at,user_id")
                .bind(workspace_id).fetch_all(&mut *tx).await.map_err(map_store)?;
            let result = rows.iter().map(membership).collect();
            tx.commit().await.map_err(map_store)?;
            result
        })
    }

    fn change_membership<'a>(
        &'a self,
        input: MembershipChange,
    ) -> BoxFuture<'a, Result<Membership, GovernanceError>> {
        Box::pin(async move {
            let mut tx = self.pool.begin().await.map_err(map_store)?;
            require_active_workspace(&mut tx, input.workspace_id).await?;
            let actor_role = require_actor(
                &mut tx,
                input.workspace_id,
                input.command.actor_id,
                Capability::Admin,
            )
            .await?;
            if let Some(replay) =
                begin_workspace::<Membership>(&mut tx, input.workspace_id, &input.command).await?
            {
                tx.commit().await.map_err(map_store)?;
                return Ok(replay);
            }
            let owners = sqlx::query_scalar::<_, Uuid>("SELECT user_id FROM memberships WHERE workspace_id=$1 AND role='OWNER' AND status='ACTIVE' ORDER BY user_id FOR UPDATE")
                .bind(input.workspace_id).fetch_all(&mut *tx).await.map_err(map_store)?;
            let row = sqlx::query("SELECT user_id,role::text,status::text,revision FROM memberships WHERE workspace_id=$1 AND user_id=$2 FOR UPDATE")
                .bind(input.workspace_id).bind(input.user_id).fetch_optional(&mut *tx).await.map_err(map_store)?.ok_or(GovernanceError::WorkspaceNotFound)?;
            let current = membership(&row)?;
            check_revision(current.revision, input.expected_revision)?;
            let requested = input.role.unwrap_or(current.role);
            if !may_change_role(actor_role, current.role, requested) {
                return Err(GovernanceError::Forbidden);
            }
            if current.role == MembershipRole::Owner
                && input.role != Some(MembershipRole::Owner)
                && owners.len() <= 1
            {
                return Err(GovernanceError::LastOwner);
            }
            let row = if let Some(role) = input.role {
                sqlx::query("UPDATE memberships SET role=$3::membership_role,revision=revision+1 WHERE workspace_id=$1 AND user_id=$2 RETURNING user_id,role::text,status::text,revision")
                    .bind(input.workspace_id).bind(input.user_id).bind(role_text(role)).fetch_one(&mut *tx).await.map_err(map_store)?
            } else {
                sqlx::query("UPDATE memberships SET status='REMOVED',removed_at=$3,revision=revision+1 WHERE workspace_id=$1 AND user_id=$2 RETURNING user_id,role::text,status::text,revision")
                    .bind(input.workspace_id).bind(input.user_id).bind(input.command.now).fetch_one(&mut *tx).await.map_err(map_store)?
            };
            let result = membership(&row)?;
            sqlx::query("UPDATE sessions SET revoked_at=COALESCE(revoked_at,$2) WHERE user_id=$1 AND revoked_at IS NULL")
                .bind(input.user_id).bind(input.command.now).execute(&mut *tx).await.map_err(map_store)?;
            append_event(&mut tx, OutboxEvent {
                workspace_id: input.workspace_id,
                aggregate_kind: "Membership",
                aggregate_id: input.user_id,
                sequence: result.revision + 1,
                event_type: "MembershipChanged.v1",
                payload: json!({"entityId":input.user_id,"revision":result.revision,"action":if input.role.is_some(){"UPDATED"}else{"DELETED"}}),
                audience: EventAudience::workspace(),
                occurred_at: input.command.now,
            }).await?;
            audit_user(
                &mut tx,
                input.workspace_id,
                &input.command,
                if input.role.is_some() {
                    AuditAction::MemberRoleChanged
                } else {
                    AuditAction::MemberRemoved
                },
                AuditTargetKind::Membership,
                input.user_id,
            )
            .await?;
            complete_workspace(&mut tx, input.workspace_id, &input.command, 200, &result).await?;
            tx.commit().await.map_err(map_store)?;
            Ok(result)
        })
    }

    fn list_invitations<'a>(
        &'a self,
        actor: Uuid,
        workspace_id: Uuid,
        cursor: Option<Uuid>,
    ) -> BoxFuture<'a, Result<InvitationPage, GovernanceError>> {
        Box::pin(async move {
            let mut tx = self.pool.begin().await.map_err(map_store)?;
            require_actor(&mut tx, workspace_id, actor, Capability::Admin).await?;
            let rows = sqlx::query("SELECT id,email,role::text,expires_at,accepted_at,revoked_at,revision FROM invitations WHERE workspace_id=$1 AND ($2::uuid IS NULL OR id>$2) ORDER BY id LIMIT 101")
                .bind(workspace_id).bind(cursor).fetch_all(&mut *tx).await.map_err(map_store)?;
            let next_cursor = (rows.len() > 100).then(|| rows[99].get::<Uuid, _>("id").to_string());
            let items = rows
                .iter()
                .take(100)
                .map(|row| invitation(row, Utc::now()))
                .collect::<Result<_, _>>()?;
            tx.commit().await.map_err(map_store)?;
            Ok(InvitationPage { items, next_cursor })
        })
    }

    fn create_invitation<'a>(
        &'a self,
        input: NewInvitation,
    ) -> BoxFuture<'a, Result<PersistedInvitation, GovernanceError>> {
        Box::pin(async move {
            let mut tx = self.pool.begin().await.map_err(map_store)?;
            require_active_workspace(&mut tx, input.workspace_id).await?;
            require_actor(
                &mut tx,
                input.workspace_id,
                input.command.actor_id,
                Capability::Admin,
            )
            .await?;
            if let Some(replay) =
                begin_workspace::<PersistedInvitation>(&mut tx, input.workspace_id, &input.command)
                    .await?
            {
                tx.commit().await.map_err(map_store)?;
                return Ok(replay);
            }
            let member_exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM memberships m JOIN users u ON u.id=m.user_id WHERE m.workspace_id=$1 AND m.status='ACTIVE' AND u.email_normalized=$2)")
                .bind(input.workspace_id).bind(&input.email).fetch_one(&mut *tx).await.map_err(map_store)?;
            if member_exists {
                return Err(GovernanceError::InvitationExists);
            }
            let row = sqlx::query("INSERT INTO invitations(id,workspace_id,email,role,token_hash,token_key_id,invited_by,expires_at,created_at) VALUES($1,$2,$3,$4::membership_role,$5,$6,$7,$8,$9) RETURNING id,email,role::text,expires_at,accepted_at,revoked_at,revision")
                .bind(input.id).bind(input.workspace_id).bind(&input.email).bind(invitation_role(input.role)).bind(input.token_hash.0.as_slice()).bind(&input.token_key_id).bind(input.command.actor_id).bind(input.expires_at).bind(input.command.now)
                .fetch_one(&mut *tx).await.map_err(map_invitation_store)?;
            let result = PersistedInvitation {
                invitation: invitation(&row, input.command.now)?,
                token_key_id: input.token_key_id,
            };
            append_event(
                &mut tx,
                OutboxEvent {
                    workspace_id: input.workspace_id,
                    aggregate_kind: "Invitation",
                    aggregate_id: input.id,
                    sequence: 1,
                    event_type: "InvitationDeliveryRequested.v1",
                    payload: json!({"invitationId":input.id}),
                    audience: EventAudience::internal(),
                    occurred_at: input.command.now,
                },
            )
            .await?;
            audit_user(
                &mut tx,
                input.workspace_id,
                &input.command,
                AuditAction::MemberInvited,
                AuditTargetKind::Invitation,
                input.id,
            )
            .await?;
            complete_workspace(&mut tx, input.workspace_id, &input.command, 201, &result).await?;
            tx.commit().await.map_err(map_store)?;
            Ok(result)
        })
    }

    fn revoke_invitation<'a>(
        &'a self,
        input: InvitationChange,
    ) -> BoxFuture<'a, Result<Invitation, GovernanceError>> {
        Box::pin(async move {
            let mut tx = self.pool.begin().await.map_err(map_store)?;
            require_actor(
                &mut tx,
                input.workspace_id,
                input.command.actor_id,
                Capability::Admin,
            )
            .await?;
            if let Some(replay) =
                begin_workspace::<Invitation>(&mut tx, input.workspace_id, &input.command).await?
            {
                tx.commit().await.map_err(map_store)?;
                return Ok(replay);
            }
            let row = sqlx::query("SELECT id,email,role::text,expires_at,accepted_at,revoked_at,revision FROM invitations WHERE workspace_id=$1 AND id=$2 FOR UPDATE")
                .bind(input.workspace_id).bind(input.invitation_id).fetch_optional(&mut *tx).await.map_err(map_store)?.ok_or(GovernanceError::InvitationInvalid)?;
            let current = invitation(&row, input.command.now)?;
            check_revision(current.revision, input.expected_revision)?;
            if current.status != InvitationStatus::Pending {
                return Err(GovernanceError::InvitationStateInvalid);
            }
            let row = sqlx::query("UPDATE invitations SET revoked_at=$3,revision=revision+1 WHERE workspace_id=$1 AND id=$2 RETURNING id,email,role::text,expires_at,accepted_at,revoked_at,revision")
                .bind(input.workspace_id).bind(input.invitation_id).bind(input.command.now).fetch_one(&mut *tx).await.map_err(map_store)?;
            let result = invitation(&row, input.command.now)?;
            append_event(
                &mut tx,
                OutboxEvent {
                    workspace_id: input.workspace_id,
                    aggregate_kind: "Invitation",
                    aggregate_id: input.invitation_id,
                    sequence: result.revision + 1,
                    event_type: "InvitationChanged.v1",
                    payload: json!({"entityId":input.invitation_id,"revision":result.revision,"action":"INVALIDATED"}),
                    audience: EventAudience::admin(),
                    occurred_at: input.command.now,
                },
            )
            .await?;
            complete_workspace(&mut tx, input.workspace_id, &input.command, 200, &result).await?;
            tx.commit().await.map_err(map_store)?;
            Ok(result)
        })
    }

    fn accept_invitation<'a>(
        &'a self,
        input: InvitationAcceptance,
    ) -> BoxFuture<'a, Result<Membership, GovernanceError>> {
        Box::pin(async move {
            let mut tx = self.pool.begin().await.map_err(map_store)?;
            if let Some(replay) = begin_user::<Membership>(&mut tx, &input.command).await? {
                tx.commit().await.map_err(map_store)?;
                return Ok(replay);
            }
            let row = sqlx::query("SELECT id,workspace_id,email_normalized,role::text,token_hash,expires_at,accepted_at,revoked_at,revision FROM invitations WHERE id=$1 FOR UPDATE")
                .bind(input.invitation_id).fetch_optional(&mut *tx).await.map_err(map_store)?.ok_or(GovernanceError::InvitationInvalid)?;
            let stored: Vec<u8> = row
                .try_get("token_hash")
                .map_err(|_| GovernanceError::Internal)?;
            if stored.len() != 32
                || stored.ct_eq(&input.token_hash.0).unwrap_u8() != 1
                || row.get::<String, _>("email_normalized") != input.verified_email
            {
                return Err(GovernanceError::InvitationInvalid);
            }
            let workspace_id: Uuid = row.get("workspace_id");
            require_active_workspace(&mut tx, workspace_id).await?;
            let accepted: Option<DateTime<Utc>> = row.get("accepted_at");
            let revoked: Option<DateTime<Utc>> = row.get("revoked_at");
            let expires: DateTime<Utc> = row.get("expires_at");
            if revoked.is_some() || expires <= input.command.now {
                return Err(GovernanceError::InvitationInvalid);
            }
            let role = parse_role(row.get::<String, _>("role").as_str())?;
            let result = if accepted.is_some() {
                let member = sqlx::query("SELECT user_id,role::text,status::text,revision FROM memberships WHERE workspace_id=$1 AND user_id=$2 AND status='ACTIVE'").bind(workspace_id).bind(input.command.actor_id).fetch_optional(&mut *tx).await.map_err(map_store)?.ok_or(GovernanceError::InvitationStateInvalid)?;
                membership(&member)?
            } else {
                let member = sqlx::query("INSERT INTO memberships(workspace_id,user_id,role,status,revision,joined_at,removed_at) VALUES($1,$2,$3::membership_role,'ACTIVE',0,$4,NULL) ON CONFLICT(workspace_id,user_id) DO UPDATE SET role=EXCLUDED.role,status='ACTIVE',revision=memberships.revision+1,joined_at=EXCLUDED.joined_at,removed_at=NULL RETURNING user_id,role::text,status::text,revision")
                    .bind(workspace_id).bind(input.command.actor_id).bind(role_text(role)).bind(input.command.now).fetch_one(&mut *tx).await.map_err(map_store)?;
                sqlx::query(
                    "UPDATE invitations SET accepted_at=$2,revision=revision+1 WHERE id=$1",
                )
                .bind(input.invitation_id)
                .bind(input.command.now)
                .execute(&mut *tx)
                .await
                .map_err(map_store)?;
                let result = membership(&member)?;
                append_event(&mut tx, OutboxEvent {
                    workspace_id,
                    aggregate_kind: "Membership",
                    aggregate_id: input.command.actor_id,
                    sequence: result.revision + 1,
                    event_type: "MembershipChanged.v1",
                    payload: json!({"entityId":input.command.actor_id,"revision":result.revision,"action":"CREATED"}),
                    audience: EventAudience::workspace(),
                    occurred_at: input.command.now,
                }).await?;
                audit_user(
                    &mut tx,
                    workspace_id,
                    &input.command,
                    AuditAction::MemberAdded,
                    AuditTargetKind::Membership,
                    input.command.actor_id,
                )
                .await?;
                result
            };
            complete_user(&mut tx, &input.command, &result).await?;
            tx.commit().await.map_err(map_store)?;
            Ok(result)
        })
    }

    fn list_groups<'a>(
        &'a self,
        actor: Uuid,
        workspace_id: Uuid,
    ) -> BoxFuture<'a, Result<Vec<Group>, GovernanceError>> {
        Box::pin(async move {
            let mut tx = self.pool.begin().await.map_err(map_store)?;
            require_actor(&mut tx, workspace_id, actor, Capability::Member).await?;
            let ids = sqlx::query_scalar::<_,Uuid>("SELECT id FROM groups WHERE workspace_id=$1 AND deleted_at IS NULL ORDER BY name_normalized,id").bind(workspace_id).fetch_all(&mut *tx).await.map_err(map_store)?;
            let mut output = Vec::with_capacity(ids.len());
            for id in ids {
                output.push(load_group(&mut tx, workspace_id, id, false).await?);
            }
            tx.commit().await.map_err(map_store)?;
            Ok(output)
        })
    }

    fn get_group<'a>(
        &'a self,
        actor: Uuid,
        workspace_id: Uuid,
        group_id: Uuid,
    ) -> BoxFuture<'a, Result<Group, GovernanceError>> {
        Box::pin(async move {
            let mut tx = self.pool.begin().await.map_err(map_store)?;
            require_actor(&mut tx, workspace_id, actor, Capability::Member).await?;
            let result = load_group(&mut tx, workspace_id, group_id, false).await?;
            tx.commit().await.map_err(map_store)?;
            Ok(result)
        })
    }

    fn create_group<'a>(
        &'a self,
        input: NewGroup,
    ) -> BoxFuture<'a, Result<Group, GovernanceError>> {
        Box::pin(async move {
            let mut tx = self.pool.begin().await.map_err(map_store)?;
            require_active_workspace(&mut tx, input.workspace_id).await?;
            require_actor(
                &mut tx,
                input.workspace_id,
                input.command.actor_id,
                Capability::Admin,
            )
            .await?;
            if let Some(replay) =
                begin_workspace::<Group>(&mut tx, input.workspace_id, &input.command).await?
            {
                tx.commit().await.map_err(map_store)?;
                return Ok(replay);
            }
            validate_active_members(&mut tx, input.workspace_id, &input.member_ids).await?;
            sqlx::query("INSERT INTO groups(id,workspace_id,name,created_at) VALUES($1,$2,$3,$4)")
                .bind(input.id)
                .bind(input.workspace_id)
                .bind(&input.name)
                .bind(input.command.now)
                .execute(&mut *tx)
                .await
                .map_err(map_group_store)?;
            for member in &input.member_ids {
                sqlx::query("INSERT INTO group_members(workspace_id,group_id,user_id,created_at) VALUES($1,$2,$3,$4)").bind(input.workspace_id).bind(input.id).bind(member).bind(input.command.now).execute(&mut *tx).await.map_err(map_store)?;
            }
            let result = load_group(&mut tx, input.workspace_id, input.id, false).await?;
            append_event(
                &mut tx,
                OutboxEvent {
                    workspace_id: input.workspace_id,
                    aggregate_kind: "Group",
                    aggregate_id: input.id,
                    sequence: 1,
                    event_type: "GroupChanged.v1",
                    payload: json!({"entityId":input.id,"revision":0,"action":"CREATED"}),
                    audience: EventAudience::workspace(),
                    occurred_at: input.command.now,
                },
            )
            .await?;
            audit_user(
                &mut tx,
                input.workspace_id,
                &input.command,
                AuditAction::GroupCreated,
                AuditTargetKind::Group,
                input.id,
            )
            .await?;
            complete_workspace(&mut tx, input.workspace_id, &input.command, 201, &result).await?;
            tx.commit().await.map_err(map_store)?;
            Ok(result)
        })
    }

    fn change_group<'a>(
        &'a self,
        operation: GroupOperation,
        input: GroupChange,
    ) -> BoxFuture<'a, Result<Option<Group>, GovernanceError>> {
        Box::pin(async move {
            let mut tx = self.pool.begin().await.map_err(map_store)?;
            require_active_workspace(&mut tx, input.workspace_id).await?;
            require_actor(
                &mut tx,
                input.workspace_id,
                input.command.actor_id,
                Capability::Admin,
            )
            .await?;
            if let Some(replay) =
                begin_workspace::<Option<Group>>(&mut tx, input.workspace_id, &input.command)
                    .await?
            {
                tx.commit().await.map_err(map_store)?;
                return Ok(replay);
            }
            let current = load_group(&mut tx, input.workspace_id, input.group_id, true).await?;
            check_revision(current.revision, input.expected_revision)?;
            let mut event_required = true;
            let result = match operation {
                GroupOperation::Rename => {
                    sqlx::query("UPDATE groups SET name=$3,revision=revision+1 WHERE workspace_id=$1 AND id=$2 AND deleted_at IS NULL").bind(input.workspace_id).bind(input.group_id).bind(input.name.ok_or(GovernanceError::Validation)?).execute(&mut *tx).await.map_err(map_group_store)?;
                    Some(load_group(&mut tx, input.workspace_id, input.group_id, false).await?)
                }
                GroupOperation::Delete => {
                    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM permission_grants WHERE workspace_id=$1 AND subject_kind='GROUP' AND subject_id=$2").bind(input.workspace_id).bind(input.group_id).fetch_one(&mut *tx).await.map_err(map_store)?;
                    if count > 0 {
                        return Err(GovernanceError::GroupInUse {
                            reference_count: count,
                        });
                    }
                    sqlx::query("UPDATE groups SET deleted_at=$3,revision=revision+1 WHERE workspace_id=$1 AND id=$2 AND deleted_at IS NULL").bind(input.workspace_id).bind(input.group_id).bind(input.command.now).execute(&mut *tx).await.map_err(map_store)?;
                    sqlx::query("DELETE FROM group_members WHERE workspace_id=$1 AND group_id=$2")
                        .bind(input.workspace_id)
                        .bind(input.group_id)
                        .execute(&mut *tx)
                        .await
                        .map_err(map_store)?;
                    None
                }
                GroupOperation::AddMember => {
                    let member = input.member_id.ok_or(GovernanceError::Validation)?;
                    validate_active_members(&mut tx, input.workspace_id, &[member]).await?;
                    let inserted = sqlx::query("INSERT INTO group_members(workspace_id,group_id,user_id,created_at) VALUES($1,$2,$3,$4) ON CONFLICT DO NOTHING").bind(input.workspace_id).bind(input.group_id).bind(member).bind(input.command.now).execute(&mut *tx).await.map_err(map_store)?;
                    if inserted.rows_affected() > 0 {
                        sqlx::query(
                            "UPDATE groups SET revision=revision+1 WHERE workspace_id=$1 AND id=$2",
                        )
                        .bind(input.workspace_id)
                        .bind(input.group_id)
                        .execute(&mut *tx)
                        .await
                        .map_err(map_store)?;
                    } else {
                        event_required = false;
                    }
                    Some(load_group(&mut tx, input.workspace_id, input.group_id, false).await?)
                }
                GroupOperation::RemoveMember => {
                    let member = input.member_id.ok_or(GovernanceError::Validation)?;
                    let removed = sqlx::query("DELETE FROM group_members WHERE workspace_id=$1 AND group_id=$2 AND user_id=$3").bind(input.workspace_id).bind(input.group_id).bind(member).execute(&mut *tx).await.map_err(map_store)?;
                    if removed.rows_affected() == 0 {
                        return Err(GovernanceError::GroupMemberNotFound);
                    }
                    sqlx::query(
                        "UPDATE groups SET revision=revision+1 WHERE workspace_id=$1 AND id=$2",
                    )
                    .bind(input.workspace_id)
                    .bind(input.group_id)
                    .execute(&mut *tx)
                    .await
                    .map_err(map_store)?;
                    Some(load_group(&mut tx, input.workspace_id, input.group_id, false).await?)
                }
            };
            let sequence = result
                .as_ref()
                .map_or(current.revision + 2, |group| group.revision + 1);
            if event_required {
                append_event(
                    &mut tx,
                    OutboxEvent {
                        workspace_id: input.workspace_id,
                        aggregate_kind: "Group",
                        aggregate_id: input.group_id,
                        sequence,
                        event_type: "GroupChanged.v1",
                        payload: json!({"entityId":input.group_id,"revision":sequence-1,"action":match operation {GroupOperation::Delete=>"DELETED",_=>"UPDATED"}}),
                        audience: EventAudience::workspace(),
                        occurred_at: input.command.now,
                    },
                )
                .await?;
                audit_user(
                    &mut tx,
                    input.workspace_id,
                    &input.command,
                    match operation {
                        GroupOperation::Rename => AuditAction::GroupUpdated,
                        GroupOperation::Delete => AuditAction::GroupDeleted,
                        GroupOperation::AddMember => AuditAction::GroupMemberAdded,
                        GroupOperation::RemoveMember => AuditAction::GroupMemberRemoved,
                    },
                    AuditTargetKind::Group,
                    input.group_id,
                )
                .await?;
            }
            complete_workspace(
                &mut tx,
                input.workspace_id,
                &input.command,
                if operation == GroupOperation::Delete {
                    204
                } else {
                    200
                },
                &result,
            )
            .await?;
            tx.commit().await.map_err(map_store)?;
            Ok(result)
        })
    }
}

async fn audit_user(
    tx: &mut Transaction<'_, Postgres>,
    workspace: Uuid,
    command: &Command,
    action: AuditAction,
    kind: AuditTargetKind,
    id: Uuid,
) -> Result<(), GovernanceError> {
    append_audit_event(
        tx,
        AuditEventInput::user(
            workspace,
            command.actor_id,
            action,
            AuditTarget { kind, id },
            command.now,
            &command.idempotency_key,
        ),
    )
    .await?;
    Ok(())
}

#[derive(Clone, Copy)]
enum Capability {
    Member,
    Admin,
    Owner,
}

async fn require_actor(
    tx: &mut Transaction<'_, Postgres>,
    workspace: Uuid,
    actor: Uuid,
    required: Capability,
) -> Result<MembershipRole, GovernanceError> {
    let row = sqlx::query("SELECT role::text FROM memberships WHERE workspace_id=$1 AND user_id=$2 AND status='ACTIVE'").bind(workspace).bind(actor).fetch_optional(&mut **tx).await.map_err(map_store)?.ok_or(GovernanceError::WorkspaceNotFound)?;
    let role = parse_role(row.get::<String, _>("role").as_str())?;
    let allowed = match required {
        Capability::Member => true,
        Capability::Admin => role.can_administer(),
        Capability::Owner => role.can_manage_owners(),
    };
    if allowed {
        Ok(role)
    } else {
        Err(GovernanceError::Forbidden)
    }
}

async fn require_active_workspace(
    tx: &mut Transaction<'_, Postgres>,
    workspace: Uuid,
) -> Result<(), GovernanceError> {
    let status: Option<String> =
        sqlx::query_scalar("SELECT status::text FROM workspaces WHERE id=$1 FOR UPDATE")
            .bind(workspace)
            .fetch_optional(&mut **tx)
            .await
            .map_err(map_store)?;
    match status.as_deref() {
        Some("ACTIVE") => Ok(()),
        Some(_) => Err(GovernanceError::WorkspaceStateInvalid),
        None => Err(GovernanceError::WorkspaceNotFound),
    }
}

async fn lock_workspace(
    tx: &mut Transaction<'_, Postgres>,
    id: Uuid,
) -> Result<Workspace, GovernanceError> {
    let row = sqlx::query(
        "SELECT id,name,slug,status::text,revision FROM workspaces WHERE id=$1 FOR UPDATE",
    )
    .bind(id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(map_store)?
    .ok_or(GovernanceError::WorkspaceNotFound)?;
    workspace(&row)
}
pub(super) fn check_revision(current: i64, expected: i64) -> Result<(), GovernanceError> {
    if current == expected {
        Ok(())
    } else {
        Err(GovernanceError::RevisionConflict {
            current_revision: current,
        })
    }
}

async fn validate_active_members(
    tx: &mut Transaction<'_, Postgres>,
    workspace: Uuid,
    members: &[Uuid],
) -> Result<(), GovernanceError> {
    if members.is_empty() {
        return Ok(());
    }
    let count:i64=sqlx::query_scalar("SELECT count(*) FROM memberships WHERE workspace_id=$1 AND user_id=ANY($2) AND status='ACTIVE'").bind(workspace).bind(members).fetch_one(&mut **tx).await.map_err(map_store)?;
    if count == members.len() as i64 {
        Ok(())
    } else {
        Err(GovernanceError::GroupMemberInvalid)
    }
}

async fn load_group(
    tx: &mut Transaction<'_, Postgres>,
    workspace: Uuid,
    id: Uuid,
    lock: bool,
) -> Result<Group, GovernanceError> {
    let query = if lock {
        "SELECT id,name,revision FROM groups WHERE workspace_id=$1 AND id=$2 AND deleted_at IS NULL FOR UPDATE"
    } else {
        "SELECT id,name,revision FROM groups WHERE workspace_id=$1 AND id=$2 AND deleted_at IS NULL"
    };
    let row = sqlx::query(query)
        .bind(workspace)
        .bind(id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(map_store)?
        .ok_or(GovernanceError::GroupNotFound)?;
    let member_ids = sqlx::query_scalar(
        "SELECT user_id FROM group_members WHERE workspace_id=$1 AND group_id=$2 ORDER BY user_id",
    )
    .bind(workspace)
    .bind(id)
    .fetch_all(&mut **tx)
    .await
    .map_err(map_store)?;
    Ok(Group {
        id: row.get("id"),
        name: row.get("name"),
        member_ids,
        revision: row.get("revision"),
    })
}

pub(super) async fn begin_workspace<T: DeserializeOwned>(
    tx: &mut Transaction<'_, Postgres>,
    workspace: Uuid,
    command: &adoc_application::governance::Command,
) -> Result<Option<T>, GovernanceError> {
    let inserted=sqlx::query("INSERT INTO idempotency_keys(workspace_id,actor_id,operation_id,key,request_hash,locked_until,expires_at,created_at) VALUES($1,$2,$3,$4,$5,$6,$7,$8) ON CONFLICT DO NOTHING").bind(workspace).bind(command.actor_id).bind(command.operation_id).bind(&command.idempotency_key).bind(&command.request_hash).bind(command.now+chrono::Duration::seconds(30)).bind(command.expires_at).bind(command.now).execute(&mut **tx).await.map_err(map_store)?;
    if inserted.rows_affected() == 1 {
        return Ok(None);
    }
    let row=sqlx::query("SELECT request_hash,response_json FROM idempotency_keys WHERE workspace_id=$1 AND actor_id=$2 AND operation_id=$3 AND key=$4 FOR UPDATE").bind(workspace).bind(command.actor_id).bind(command.operation_id).bind(&command.idempotency_key).fetch_one(&mut **tx).await.map_err(map_store)?;
    if row.get::<String, _>("request_hash") != command.request_hash {
        return Err(GovernanceError::IdempotencyKeyReused);
    }
    let value: Option<Value> = row.get("response_json");
    value
        .map(|value| serde_json::from_value(value).map_err(|_| GovernanceError::Internal))
        .transpose()
}
pub(super) async fn complete_workspace<T: Serialize>(
    tx: &mut Transaction<'_, Postgres>,
    workspace: Uuid,
    command: &adoc_application::governance::Command,
    status: i32,
    value: &T,
) -> Result<(), GovernanceError> {
    let value = serde_json::to_value(value).map_err(|_| GovernanceError::Internal)?;
    let completed = sqlx::query("UPDATE idempotency_keys SET response_status=$5,response_json=$6 WHERE workspace_id=$1 AND actor_id=$2 AND operation_id=$3 AND key=$4 AND response_json IS NULL").bind(workspace).bind(command.actor_id).bind(command.operation_id).bind(&command.idempotency_key).bind(status).bind(value).execute(&mut **tx).await.map_err(map_store)?;
    if completed.rows_affected() == 1 {
        Ok(())
    } else {
        Err(GovernanceError::Internal)
    }
}
async fn begin_user<T: DeserializeOwned>(
    tx: &mut Transaction<'_, Postgres>,
    command: &adoc_application::governance::Command,
) -> Result<Option<T>, GovernanceError> {
    let inserted=sqlx::query("INSERT INTO user_command_receipts(user_id,operation_id,key,request_hash,created_at,expires_at) VALUES($1,$2,$3,$4,$5,$6) ON CONFLICT DO NOTHING").bind(command.actor_id).bind(command.operation_id).bind(&command.idempotency_key).bind(&command.request_hash).bind(command.now).bind(command.expires_at).execute(&mut **tx).await.map_err(map_store)?;
    if inserted.rows_affected() == 1 {
        return Ok(None);
    }
    let row=sqlx::query("SELECT request_hash,response_json FROM user_command_receipts WHERE user_id=$1 AND operation_id=$2 AND key=$3 FOR UPDATE").bind(command.actor_id).bind(command.operation_id).bind(&command.idempotency_key).fetch_one(&mut **tx).await.map_err(map_store)?;
    if row.get::<String, _>("request_hash") != command.request_hash {
        return Err(GovernanceError::IdempotencyKeyReused);
    }
    let value: Option<Value> = row.get("response_json");
    value
        .map(|value| serde_json::from_value(value).map_err(|_| GovernanceError::Internal))
        .transpose()
}
async fn complete_user<T: Serialize>(
    tx: &mut Transaction<'_, Postgres>,
    command: &adoc_application::governance::Command,
    value: &T,
) -> Result<(), GovernanceError> {
    let completed = sqlx::query("UPDATE user_command_receipts SET response_json=$4 WHERE user_id=$1 AND operation_id=$2 AND key=$3 AND response_json IS NULL").bind(command.actor_id).bind(command.operation_id).bind(&command.idempotency_key).bind(serde_json::to_value(value).map_err(|_|GovernanceError::Internal)?).execute(&mut **tx).await.map_err(map_store)?;
    if completed.rows_affected() == 1 {
        Ok(())
    } else {
        Err(GovernanceError::Internal)
    }
}

pub(super) struct OutboxEvent<'a> {
    pub(super) workspace_id: Uuid,
    pub(super) aggregate_kind: &'a str,
    pub(super) aggregate_id: Uuid,
    pub(super) sequence: i64,
    pub(super) event_type: &'a str,
    pub(super) payload: Value,
    pub(super) audience: EventAudience,
    pub(super) occurred_at: DateTime<Utc>,
}

pub(super) async fn append_event(
    tx: &mut Transaction<'_, Postgres>,
    event: OutboxEvent<'_>,
) -> Result<(), GovernanceError> {
    if !event.audience.is_valid() || event.payload.to_string().len() > 65_536 {
        return Err(GovernanceError::Internal);
    }
    let event_id = Uuid::now_v7();
    let correlation_id = event_id.to_string();
    sqlx::query("INSERT INTO workspace_sequences(workspace_id) VALUES($1) ON CONFLICT(workspace_id) DO NOTHING")
        .bind(event.workspace_id).execute(&mut **tx).await.map_err(map_store)?;
    let projection_sequence:i64=sqlx::query_scalar("UPDATE workspace_sequences SET next_projection_sequence=next_projection_sequence+1 WHERE workspace_id=$1 RETURNING next_projection_sequence-1")
        .bind(event.workspace_id).fetch_one(&mut **tx).await.map_err(map_store)?;
    sqlx::query("INSERT INTO outbox_events(id,workspace_id,aggregate_kind,aggregate_id,sequence,event_type,event_version,projection_sequence,payload_json,audience_kind,audience_id,minimum_access,correlation_id,occurred_at) VALUES($1,$2,$3,$4,$5,$6,1,$7,$8,$9::event_audience_kind,$10,$11::document_access,$12,$13)")
        .bind(event_id).bind(event.workspace_id).bind(event.aggregate_kind).bind(event.aggregate_id)
        .bind(event.sequence).bind(event.event_type).bind(projection_sequence).bind(event.payload)
        .bind(super::outbox::audience_kind(event.audience.kind)).bind(event.audience.id)
        .bind(event.audience.minimum_access.map(super::outbox::access_text)).bind(&correlation_id)
        .bind(event.occurred_at).execute(&mut **tx).await.map_err(map_store)?;
    sqlx::query("INSERT INTO jobs(id,workspace_id,kind,payload_json,dedupe_key,status,priority,sequence,attempt,max_attempts,run_after,correlation_id,created_at,updated_at) VALUES($1,$2,'OUTBOX_TO_STREAM',$3,$4,'QUEUED',50,1,0,5,$5,$6,$5,$5) ON CONFLICT(kind,dedupe_key) DO NOTHING")
        .bind(Uuid::now_v7()).bind(event.workspace_id).bind(json!({"outboxEventId":event_id}))
        .bind(format!("workspace-stream:{event_id}")).bind(event.occurred_at).bind(&correlation_id)
        .execute(&mut **tx).await.map_err(map_store)?;
    if super::outbox::is_search_projection_event(event.event_type) {
        sqlx::query("INSERT INTO jobs(id,workspace_id,kind,payload_json,dedupe_key,status,priority,sequence,attempt,max_attempts,run_after,correlation_id,created_at,updated_at) VALUES($1,$2,'OUTBOX_TO_SEARCH',$3,$4,'QUEUED',25,1,0,5,$5,$6,$5,$5) ON CONFLICT(kind,dedupe_key) DO NOTHING")
            .bind(Uuid::now_v7()).bind(event.workspace_id).bind(json!({"outboxEventId":event_id}))
            .bind(format!("search-projection:{event_id}")).bind(event.occurred_at).bind(&correlation_id)
            .execute(&mut **tx).await.map_err(map_store)?;
    }
    Ok(())
}

fn workspace(row: &PgRow) -> Result<Workspace, GovernanceError> {
    Ok(Workspace {
        id: row.try_get("id").map_err(|_| GovernanceError::Internal)?,
        name: row.try_get("name").map_err(|_| GovernanceError::Internal)?,
        slug: row.try_get("slug").map_err(|_| GovernanceError::Internal)?,
        status: match row
            .try_get::<String, _>("status")
            .map_err(|_| GovernanceError::Internal)?
            .as_str()
        {
            "ACTIVE" => WorkspaceStatus::Active,
            "DELETION_SCHEDULED" => WorkspaceStatus::DeletionScheduled,
            "PURGING" => WorkspaceStatus::Purging,
            "DELETED" => WorkspaceStatus::Deleted,
            _ => return Err(GovernanceError::Internal),
        },
        revision: row
            .try_get("revision")
            .map_err(|_| GovernanceError::Internal)?,
    })
}
fn membership(row: &PgRow) -> Result<Membership, GovernanceError> {
    Ok(Membership {
        user_id: row
            .try_get("user_id")
            .map_err(|_| GovernanceError::Internal)?,
        role: parse_role(
            &row.try_get::<String, _>("role")
                .map_err(|_| GovernanceError::Internal)?,
        )?,
        status: match row
            .try_get::<String, _>("status")
            .map_err(|_| GovernanceError::Internal)?
            .as_str()
        {
            "ACTIVE" => MembershipStatus::Active,
            "SUSPENDED" => MembershipStatus::Suspended,
            "REMOVED" => MembershipStatus::Removed,
            _ => return Err(GovernanceError::Internal),
        },
        revision: row
            .try_get("revision")
            .map_err(|_| GovernanceError::Internal)?,
    })
}
fn invitation(row: &PgRow, now: DateTime<Utc>) -> Result<Invitation, GovernanceError> {
    let accepted: Option<DateTime<Utc>> = row
        .try_get("accepted_at")
        .map_err(|_| GovernanceError::Internal)?;
    let revoked: Option<DateTime<Utc>> = row
        .try_get("revoked_at")
        .map_err(|_| GovernanceError::Internal)?;
    let expires_at = row
        .try_get("expires_at")
        .map_err(|_| GovernanceError::Internal)?;
    let status = if accepted.is_some() {
        InvitationStatus::Accepted
    } else if revoked.is_some() {
        InvitationStatus::Revoked
    } else if expires_at <= now {
        InvitationStatus::Expired
    } else {
        InvitationStatus::Pending
    };
    Ok(Invitation {
        id: row.try_get("id").map_err(|_| GovernanceError::Internal)?,
        email: row
            .try_get("email")
            .map_err(|_| GovernanceError::Internal)?,
        role: match row
            .try_get::<String, _>("role")
            .map_err(|_| GovernanceError::Internal)?
            .as_str()
        {
            "MEMBER" => InvitationRole::Member,
            "ADMIN" => InvitationRole::Admin,
            _ => return Err(GovernanceError::Internal),
        },
        status,
        expires_at,
        revision: row
            .try_get("revision")
            .map_err(|_| GovernanceError::Internal)?,
    })
}
fn parse_role(value: &str) -> Result<MembershipRole, GovernanceError> {
    match value {
        "MEMBER" => Ok(MembershipRole::Member),
        "ADMIN" => Ok(MembershipRole::Admin),
        "OWNER" => Ok(MembershipRole::Owner),
        _ => Err(GovernanceError::Internal),
    }
}
fn role_text(role: MembershipRole) -> &'static str {
    match role {
        MembershipRole::Member => "MEMBER",
        MembershipRole::Admin => "ADMIN",
        MembershipRole::Owner => "OWNER",
    }
}
fn invitation_role(role: InvitationRole) -> &'static str {
    match role {
        InvitationRole::Member => "MEMBER",
        InvitationRole::Admin => "ADMIN",
    }
}
fn publish_mode(mode: PublishMode) -> &'static str {
    match mode {
        PublishMode::Direct => "DIRECT",
        PublishMode::ReviewRequired => "REVIEW_REQUIRED",
    }
}
pub(super) fn map_store(_: sqlx::Error) -> GovernanceError {
    GovernanceError::StorageUnavailable
}
fn constraint(error: &sqlx::Error) -> Option<&str> {
    match error {
        sqlx::Error::Database(db) => db.constraint(),
        _ => None,
    }
}
fn map_invitation_store(error: sqlx::Error) -> GovernanceError {
    if constraint(&error) == Some("invitations_active_email_idx") {
        GovernanceError::InvitationExists
    } else {
        map_store(error)
    }
}
fn map_group_store(error: sqlx::Error) -> GovernanceError {
    if constraint(&error) == Some("groups_active_name_idx") {
        GovernanceError::GroupNameTaken
    } else {
        map_store(error)
    }
}
