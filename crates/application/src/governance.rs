use std::{fmt, sync::Arc};

pub use adoc_governance::{
    CreateGroupInput, CreateWorkspaceInput, Group, GroupName, Invitation, InvitationRole,
    InvitationStatus, InviteMemberInput, Membership, MembershipRole, MembershipStatus, PublishMode,
    ReasonInput, UpdateGroupInput, UpdateMemberRoleInput, UpdateWorkspaceInput, Workspace,
    WorkspaceName, WorkspaceStatus, may_change_role, normalized_email, normalized_member_ids,
    slug_base, validate_reason,
};
use adoc_ports::BoxFuture;
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::{DateTime, Duration, Utc};
use serde::Serialize;
use sha2::{Digest, Sha256};
use thiserror::Error;
use uuid::Uuid;

use crate::identity::{Clock, KeyRing, SecureRandom, TokenHash};

const INVITATION_CONTEXT: &[u8] = b"adoc-invitation-v1";

#[derive(Clone, Debug)]
pub struct Command {
    pub actor_id: Uuid,
    pub operation_id: &'static str,
    pub idempotency_key: String,
    pub request_hash: String,
    pub now: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Clone, Debug)]
pub struct NewWorkspace {
    pub id: Uuid,
    pub name: String,
    pub slug_base: String,
    pub slug_suffix: String,
    pub command: Command,
}

#[derive(Clone, Debug)]
pub struct WorkspaceChange {
    pub workspace_id: Uuid,
    pub expected_revision: i64,
    pub name: Option<String>,
    pub default_publish_mode: Option<PublishMode>,
    pub command: Command,
}

#[derive(Clone, Debug)]
pub struct WorkspaceDeletion {
    pub workspace_id: Uuid,
    pub expected_revision: i64,
    pub reason: Option<String>,
    pub delete_after: Option<DateTime<Utc>>,
    pub command: Command,
}

#[derive(Clone, Debug)]
pub struct MembershipChange {
    pub workspace_id: Uuid,
    pub user_id: Uuid,
    pub expected_revision: i64,
    pub role: Option<MembershipRole>,
    pub command: Command,
}

#[derive(Clone, Debug)]
pub struct NewInvitation {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub email: String,
    pub role: InvitationRole,
    pub token_hash: TokenHash,
    pub token_key_id: String,
    pub expires_at: DateTime<Utc>,
    pub command: Command,
}

#[derive(Clone, Debug)]
pub struct InvitationChange {
    pub workspace_id: Uuid,
    pub invitation_id: Uuid,
    pub expected_revision: i64,
    pub command: Command,
}

#[derive(Clone, Debug)]
pub struct InvitationAcceptance {
    pub invitation_id: Uuid,
    pub token_hash: TokenHash,
    pub verified_email: String,
    pub command: Command,
}

#[derive(Clone, Debug)]
pub struct NewGroup {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub name: String,
    pub member_ids: Vec<Uuid>,
    pub command: Command,
}

#[derive(Clone, Debug)]
pub struct GroupChange {
    pub workspace_id: Uuid,
    pub group_id: Uuid,
    pub expected_revision: i64,
    pub name: Option<String>,
    pub member_id: Option<Uuid>,
    pub command: Command,
}

#[derive(Clone, Debug)]
pub struct GroupMemberCommand {
    pub add: bool,
    pub actor_id: Uuid,
    pub workspace_id: Uuid,
    pub group_id: Uuid,
    pub user_id: Uuid,
    pub expected_revision: i64,
    pub idempotency_key: String,
}

struct GroupMutation<'a> {
    actor_id: Uuid,
    workspace_id: Uuid,
    group_id: Uuid,
    expected_revision: i64,
    name: Option<String>,
    member_id: Option<Uuid>,
    idempotency_key: &'a str,
    operation_id: &'static str,
    operation: GroupOperation,
}

#[derive(Clone, Debug, serde::Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InvitationPage {
    pub items: Vec<Invitation>,
    pub next_cursor: Option<String>,
}

#[derive(Clone)]
pub struct CreatedInvitation {
    pub invitation: Invitation,
    delivery_token: Arc<str>,
}

#[derive(Clone, Debug, serde::Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PersistedInvitation {
    pub invitation: Invitation,
    pub token_key_id: String,
}

impl CreatedInvitation {
    pub fn delivery_token(&self) -> &str {
        &self.delivery_token
    }
}

impl fmt::Debug for CreatedInvitation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CreatedInvitation")
            .field("invitation", &self.invitation)
            .field("delivery_token", &"[REDACTED]")
            .finish()
    }
}

pub trait GovernanceRepository: Send + Sync {
    fn list_workspaces<'a>(
        &'a self,
        actor: Uuid,
    ) -> BoxFuture<'a, Result<Vec<Workspace>, GovernanceError>>;
    fn create_workspace<'a>(
        &'a self,
        input: NewWorkspace,
    ) -> BoxFuture<'a, Result<Workspace, GovernanceError>>;
    fn get_workspace<'a>(
        &'a self,
        actor: Uuid,
        workspace: Uuid,
    ) -> BoxFuture<'a, Result<Workspace, GovernanceError>>;
    fn update_workspace<'a>(
        &'a self,
        input: WorkspaceChange,
    ) -> BoxFuture<'a, Result<Workspace, GovernanceError>>;
    fn change_workspace_deletion<'a>(
        &'a self,
        input: WorkspaceDeletion,
    ) -> BoxFuture<'a, Result<Workspace, GovernanceError>>;
    fn list_members<'a>(
        &'a self,
        actor: Uuid,
        workspace: Uuid,
    ) -> BoxFuture<'a, Result<Vec<Membership>, GovernanceError>>;
    fn change_membership<'a>(
        &'a self,
        input: MembershipChange,
    ) -> BoxFuture<'a, Result<Membership, GovernanceError>>;
    fn list_invitations<'a>(
        &'a self,
        actor: Uuid,
        workspace: Uuid,
        cursor: Option<Uuid>,
    ) -> BoxFuture<'a, Result<InvitationPage, GovernanceError>>;
    fn create_invitation<'a>(
        &'a self,
        input: NewInvitation,
    ) -> BoxFuture<'a, Result<PersistedInvitation, GovernanceError>>;
    fn revoke_invitation<'a>(
        &'a self,
        input: InvitationChange,
    ) -> BoxFuture<'a, Result<Invitation, GovernanceError>>;
    fn accept_invitation<'a>(
        &'a self,
        input: InvitationAcceptance,
    ) -> BoxFuture<'a, Result<Membership, GovernanceError>>;
    fn list_groups<'a>(
        &'a self,
        actor: Uuid,
        workspace: Uuid,
    ) -> BoxFuture<'a, Result<Vec<Group>, GovernanceError>>;
    fn get_group<'a>(
        &'a self,
        actor: Uuid,
        workspace: Uuid,
        group: Uuid,
    ) -> BoxFuture<'a, Result<Group, GovernanceError>>;
    fn create_group<'a>(&'a self, input: NewGroup)
    -> BoxFuture<'a, Result<Group, GovernanceError>>;
    fn change_group<'a>(
        &'a self,
        operation: GroupOperation,
        input: GroupChange,
    ) -> BoxFuture<'a, Result<Option<Group>, GovernanceError>>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GroupOperation {
    Rename,
    Delete,
    AddMember,
    RemoveMember,
}

#[derive(Clone)]
pub struct GovernanceService {
    repository: Arc<dyn GovernanceRepository>,
    clock: Arc<dyn Clock>,
    random: Arc<dyn SecureRandom>,
    token_keys: KeyRing,
}

impl GovernanceService {
    pub fn new(
        repository: Arc<dyn GovernanceRepository>,
        clock: Arc<dyn Clock>,
        random: Arc<dyn SecureRandom>,
        token_keys: KeyRing,
    ) -> Self {
        Self {
            repository,
            clock,
            random,
            token_keys,
        }
    }

    pub async fn list_workspaces(&self, actor: Uuid) -> Result<Vec<Workspace>, GovernanceError> {
        self.repository.list_workspaces(actor).await
    }

    pub async fn create_workspace(
        &self,
        actor: Uuid,
        input: CreateWorkspaceInput,
        key: &str,
    ) -> Result<Workspace, GovernanceError> {
        let name = WorkspaceName::parse(&input.name).map_err(|_| GovernanceError::Validation)?;
        let now = self.clock.now();
        let id = self
            .random
            .uuid_v7(now)
            .map_err(|_| GovernanceError::Internal)?;
        let suffix = id.simple().to_string()[..8].to_owned();
        self.repository
            .create_workspace(NewWorkspace {
                id,
                slug_base: slug_base(&name),
                slug_suffix: suffix,
                name: name.as_str().to_owned(),
                command: command(actor, "createWorkspace", key, &input, now)?,
            })
            .await
    }

    pub async fn get_workspace(
        &self,
        actor: Uuid,
        workspace: Uuid,
    ) -> Result<Workspace, GovernanceError> {
        self.repository.get_workspace(actor, workspace).await
    }

    pub async fn update_workspace(
        &self,
        actor: Uuid,
        workspace: Uuid,
        revision: i64,
        input: UpdateWorkspaceInput,
        key: &str,
    ) -> Result<Workspace, GovernanceError> {
        if input.name.is_none() && input.default_publish_mode.is_none() {
            return Err(GovernanceError::Validation);
        }
        let name = input
            .name
            .as_deref()
            .map(WorkspaceName::parse)
            .transpose()
            .map_err(|_| GovernanceError::Validation)?
            .map(|value| value.as_str().to_owned());
        let mode = input.default_publish_mode;
        let now = self.clock.now();
        let command = command(actor, "updateWorkspace", key, &(revision, &input), now)?;
        self.repository
            .update_workspace(WorkspaceChange {
                workspace_id: workspace,
                expected_revision: revision,
                name,
                default_publish_mode: mode,
                command,
            })
            .await
    }

    pub async fn schedule_deletion(
        &self,
        actor: Uuid,
        workspace: Uuid,
        revision: i64,
        input: ReasonInput,
        key: &str,
    ) -> Result<Workspace, GovernanceError> {
        let reason = validate_reason(&input.reason).map_err(|_| GovernanceError::Validation)?;
        let now = self.clock.now();
        self.repository
            .change_workspace_deletion(WorkspaceDeletion {
                workspace_id: workspace,
                expected_revision: revision,
                reason: Some(reason),
                delete_after: Some(
                    now + Duration::days(adoc_governance::WORKSPACE_DELETION_GRACE_DAYS),
                ),
                command: command(
                    actor,
                    "scheduleWorkspaceDeletion",
                    key,
                    &(revision, input),
                    now,
                )?,
            })
            .await
    }

    pub async fn cancel_deletion(
        &self,
        actor: Uuid,
        workspace: Uuid,
        revision: i64,
        key: &str,
    ) -> Result<Workspace, GovernanceError> {
        let now = self.clock.now();
        self.repository
            .change_workspace_deletion(WorkspaceDeletion {
                workspace_id: workspace,
                expected_revision: revision,
                reason: None,
                delete_after: None,
                command: command(actor, "cancelWorkspaceDeletion", key, &revision, now)?,
            })
            .await
    }

    pub async fn list_members(
        &self,
        actor: Uuid,
        workspace: Uuid,
    ) -> Result<Vec<Membership>, GovernanceError> {
        self.repository.list_members(actor, workspace).await
    }

    pub async fn update_member_role(
        &self,
        actor: Uuid,
        workspace: Uuid,
        user: Uuid,
        revision: i64,
        input: UpdateMemberRoleInput,
        key: &str,
    ) -> Result<Membership, GovernanceError> {
        let now = self.clock.now();
        self.repository
            .change_membership(MembershipChange {
                workspace_id: workspace,
                user_id: user,
                expected_revision: revision,
                role: Some(input.role),
                command: command(actor, "updateMemberRole", key, &(revision, input), now)?,
            })
            .await
    }

    pub async fn remove_member(
        &self,
        actor: Uuid,
        workspace: Uuid,
        user: Uuid,
        revision: i64,
        key: &str,
    ) -> Result<Membership, GovernanceError> {
        let now = self.clock.now();
        self.repository
            .change_membership(MembershipChange {
                workspace_id: workspace,
                user_id: user,
                expected_revision: revision,
                role: None,
                command: command(actor, "removeMember", key, &(revision, user), now)?,
            })
            .await
    }

    pub async fn list_invitations(
        &self,
        actor: Uuid,
        workspace: Uuid,
        cursor: Option<Uuid>,
    ) -> Result<InvitationPage, GovernanceError> {
        self.repository
            .list_invitations(actor, workspace, cursor)
            .await
    }

    pub async fn invite_member(
        &self,
        actor: Uuid,
        workspace: Uuid,
        input: adoc_governance::InviteMemberInput,
        key: &str,
    ) -> Result<CreatedInvitation, GovernanceError> {
        let email = normalized_email(&input.email).map_err(|_| GovernanceError::Validation)?;
        let now = self.clock.now();
        let id = self
            .random
            .uuid_v7(now)
            .map_err(|_| GovernanceError::Internal)?;
        let expires_at = now + Duration::days(adoc_governance::INVITATION_TTL_DAYS);
        let material = invitation_material(id, workspace, &email, expires_at);
        let signature = self.token_keys.mac_current(INVITATION_CONTEXT, &material);
        let mut bytes = Vec::with_capacity(48);
        bytes.extend_from_slice(id.as_bytes());
        bytes.extend_from_slice(&signature.hash.0);
        let token_hash = TokenHash(Sha256::digest(&bytes).into());
        let persisted = self
            .repository
            .create_invitation(NewInvitation {
                id,
                workspace_id: workspace,
                email,
                role: input.role,
                token_hash,
                token_key_id: signature.key_id,
                expires_at,
                command: command(actor, "inviteMember", key, &input, now)?,
            })
            .await?;
        let material = invitation_material(
            persisted.invitation.id,
            workspace,
            &persisted.invitation.email,
            persisted.invitation.expires_at,
        );
        let signature = self
            .token_keys
            .mac_for_key_id(&persisted.token_key_id, INVITATION_CONTEXT, &material)
            .ok_or(GovernanceError::Internal)?;
        let mut bytes = Vec::with_capacity(48);
        bytes.extend_from_slice(persisted.invitation.id.as_bytes());
        bytes.extend_from_slice(&signature.hash.0);
        Ok(CreatedInvitation {
            invitation: persisted.invitation,
            delivery_token: Arc::from(URL_SAFE_NO_PAD.encode(bytes)),
        })
    }

    pub async fn revoke_invitation(
        &self,
        actor: Uuid,
        workspace: Uuid,
        invitation: Uuid,
        revision: i64,
        key: &str,
    ) -> Result<Invitation, GovernanceError> {
        let now = self.clock.now();
        self.repository
            .revoke_invitation(InvitationChange {
                workspace_id: workspace,
                invitation_id: invitation,
                expected_revision: revision,
                command: command(actor, "revokeInvitation", key, &(revision, invitation), now)?,
            })
            .await
    }

    pub async fn accept_invitation(
        &self,
        actor: Uuid,
        verified_email: &str,
        token: &str,
        key: &str,
    ) -> Result<Membership, GovernanceError> {
        let bytes = URL_SAFE_NO_PAD
            .decode(token)
            .map_err(|_| GovernanceError::InvitationInvalid)?;
        if bytes.len() != 48 {
            return Err(GovernanceError::InvitationInvalid);
        }
        let invitation_id =
            Uuid::from_slice(&bytes[..16]).map_err(|_| GovernanceError::InvitationInvalid)?;
        let now = self.clock.now();
        self.repository
            .accept_invitation(InvitationAcceptance {
                invitation_id,
                token_hash: TokenHash(Sha256::digest(&bytes).into()),
                verified_email: normalized_email(verified_email)
                    .map_err(|_| GovernanceError::InvitationInvalid)?,
                command: command(actor, "acceptInvitation", key, &token, now)?,
            })
            .await
    }

    pub async fn list_groups(
        &self,
        actor: Uuid,
        workspace: Uuid,
    ) -> Result<Vec<Group>, GovernanceError> {
        self.repository.list_groups(actor, workspace).await
    }
    pub async fn get_group(
        &self,
        actor: Uuid,
        workspace: Uuid,
        group: Uuid,
    ) -> Result<Group, GovernanceError> {
        self.repository.get_group(actor, workspace, group).await
    }

    pub async fn create_group(
        &self,
        actor: Uuid,
        workspace: Uuid,
        input: CreateGroupInput,
        key: &str,
    ) -> Result<Group, GovernanceError> {
        let name = GroupName::parse(&input.name).map_err(|_| GovernanceError::Validation)?;
        let members = normalized_member_ids(input.member_ids.clone())
            .map_err(|_| GovernanceError::Validation)?;
        let now = self.clock.now();
        self.repository
            .create_group(NewGroup {
                id: self
                    .random
                    .uuid_v7(now)
                    .map_err(|_| GovernanceError::Internal)?,
                workspace_id: workspace,
                name: name.as_str().to_owned(),
                member_ids: members,
                command: command(actor, "createGroup", key, &input, now)?,
            })
            .await
    }

    pub async fn update_group(
        &self,
        actor: Uuid,
        workspace: Uuid,
        group: Uuid,
        revision: i64,
        input: UpdateGroupInput,
        key: &str,
    ) -> Result<Group, GovernanceError> {
        let name = GroupName::parse(&input.name).map_err(|_| GovernanceError::Validation)?;
        self.group_change(GroupMutation {
            actor_id: actor,
            workspace_id: workspace,
            group_id: group,
            expected_revision: revision,
            name: Some(name.as_str().to_owned()),
            member_id: None,
            idempotency_key: key,
            operation_id: "updateGroup",
            operation: GroupOperation::Rename,
        })
        .await?
        .ok_or(GovernanceError::Internal)
    }

    pub async fn delete_group(
        &self,
        actor: Uuid,
        workspace: Uuid,
        group: Uuid,
        revision: i64,
        key: &str,
    ) -> Result<(), GovernanceError> {
        self.group_change(GroupMutation {
            actor_id: actor,
            workspace_id: workspace,
            group_id: group,
            expected_revision: revision,
            name: None,
            member_id: None,
            idempotency_key: key,
            operation_id: "deleteGroup",
            operation: GroupOperation::Delete,
        })
        .await
        .map(|_| ())
    }

    pub async fn change_group_member(
        &self,
        input: GroupMemberCommand,
    ) -> Result<Group, GovernanceError> {
        let (operation_id, operation) = if input.add {
            ("addGroupMember", GroupOperation::AddMember)
        } else {
            ("removeGroupMember", GroupOperation::RemoveMember)
        };
        self.group_change(GroupMutation {
            actor_id: input.actor_id,
            workspace_id: input.workspace_id,
            group_id: input.group_id,
            expected_revision: input.expected_revision,
            name: None,
            member_id: Some(input.user_id),
            idempotency_key: &input.idempotency_key,
            operation_id,
            operation,
        })
        .await?
        .ok_or(GovernanceError::Internal)
    }

    async fn group_change(
        &self,
        mutation: GroupMutation<'_>,
    ) -> Result<Option<Group>, GovernanceError> {
        let now = self.clock.now();
        self.repository
            .change_group(
                mutation.operation,
                GroupChange {
                    workspace_id: mutation.workspace_id,
                    group_id: mutation.group_id,
                    expected_revision: mutation.expected_revision,
                    name: mutation.name,
                    member_id: mutation.member_id,
                    command: command(
                        mutation.actor_id,
                        mutation.operation_id,
                        mutation.idempotency_key,
                        &(
                            mutation.expected_revision,
                            mutation.group_id,
                            mutation.member_id,
                        ),
                        now,
                    )?,
                },
            )
            .await
    }
}

fn command<T: Serialize>(
    actor: Uuid,
    operation_id: &'static str,
    key: &str,
    input: &T,
    now: DateTime<Utc>,
) -> Result<Command, GovernanceError> {
    if !(16..=128).contains(&key.len()) {
        return Err(GovernanceError::Validation);
    }
    let body = serde_json::to_vec(input).map_err(|_| GovernanceError::Internal)?;
    Ok(Command {
        actor_id: actor,
        operation_id,
        idempotency_key: key.to_owned(),
        request_hash: hex::encode(Sha256::digest(body)),
        now,
        expires_at: now + Duration::hours(24),
    })
}

fn invitation_material(
    id: Uuid,
    workspace: Uuid,
    email: &str,
    expires_at: DateTime<Utc>,
) -> Vec<u8> {
    let mut material = Vec::new();
    material.extend_from_slice(id.as_bytes());
    material.extend_from_slice(workspace.as_bytes());
    material.extend_from_slice(email.as_bytes());
    material.extend_from_slice(&expires_at.timestamp().to_be_bytes());
    material
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum GovernanceError {
    #[error("governance input is invalid")]
    Validation,
    #[error("workspace was not found")]
    WorkspaceNotFound,
    #[error("workspace slug is taken")]
    WorkspaceSlugTaken,
    #[error("workspace state is invalid")]
    WorkspaceStateInvalid,
    #[error("governance operation is forbidden")]
    Forbidden,
    #[error("resource revision conflicts")]
    RevisionConflict { current_revision: i64 },
    #[error("last owner cannot be removed")]
    LastOwner,
    #[error("invitation already exists")]
    InvitationExists,
    #[error("invitation is invalid")]
    InvitationInvalid,
    #[error("invitation state is invalid")]
    InvitationStateInvalid,
    #[error("group was not found")]
    GroupNotFound,
    #[error("group name is taken")]
    GroupNameTaken,
    #[error("group is in use")]
    GroupInUse { reference_count: i64 },
    #[error("group member is invalid")]
    GroupMemberInvalid,
    #[error("group member was not found")]
    GroupMemberNotFound,
    #[error("document was not found")]
    DocumentNotFound,
    #[error("document permission is denied")]
    PermissionDenied,
    #[error("permission subject is invalid")]
    PermissionSubjectInvalid,
    #[error("manage permission requires editor access")]
    PermissionManageRequiresEditor,
    #[error("permission grant conflicts")]
    PermissionGrantConflict,
    #[error("permission change would remove the last manager")]
    PermissionLastManager,
    #[error("publish policy is invalid")]
    PublishPolicyInvalid,
    #[error("idempotency key was reused")]
    IdempotencyKeyReused,
    #[error("governance storage is unavailable")]
    StorageUnavailable,
    #[error("governance operation failed")]
    Internal,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invitation_secret_never_appears_in_debug() {
        let created = CreatedInvitation {
            invitation: Invitation {
                id: Uuid::nil(),
                email: "a@example.com".into(),
                role: InvitationRole::Member,
                status: adoc_governance::InvitationStatus::Pending,
                expires_at: Utc::now(),
                revision: 0,
            },
            delivery_token: Arc::from("secret"),
        };
        assert!(!format!("{created:?}").contains("secret"));
    }
}
