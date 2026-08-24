use std::sync::Arc;

use adoc_adapters::{
    identity::{SystemClock, SystemSecureRandom},
    postgres::{PostgresGovernanceRepository, PostgresStore},
};
use adoc_application::governance::{
    CreateGroupInput, CreateWorkspaceInput, GovernanceError, GovernanceService, InviteMemberInput,
    ReasonInput, UpdateGroupInput, UpdateMemberRoleInput, UpdateWorkspaceInput,
};
use adoc_configuration::AppConfig;
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{delete, get, post, put},
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    HealthState,
    identity_http::{
        Authenticated, Problem, PublishConflict, expected_revision, idempotency_key, key_ring,
        validate_command,
    },
};

#[derive(Clone)]
pub(crate) struct GovernanceRuntime {
    pub(crate) service: Arc<GovernanceService>,
}

impl GovernanceRuntime {
    pub(crate) fn new(config: &AppConfig, store: &PostgresStore) -> Result<Self, GovernanceError> {
        let keys =
            key_ring(&config.auth.token_hash_pepper).map_err(|_| GovernanceError::Internal)?;
        Ok(Self {
            service: Arc::new(GovernanceService::new(
                Arc::new(PostgresGovernanceRepository::new(store)),
                Arc::new(SystemClock),
                Arc::new(SystemSecureRandom),
                keys,
            )),
        })
    }
}

pub(crate) fn governance_routes() -> Router<HealthState> {
    Router::new()
        .route("/workspaces", get(list_workspaces).post(create_workspace))
        .route(
            "/workspaces/{workspace_id}",
            get(get_workspace).put(update_workspace),
        )
        .route(
            "/workspaces/{workspace_id}/deletion",
            post(schedule_deletion).delete(cancel_deletion),
        )
        .route("/workspaces/{workspace_id}/members", get(list_members))
        .route(
            "/workspaces/{workspace_id}/members/{user_id}/role",
            put(update_member_role),
        )
        .route(
            "/workspaces/{workspace_id}/members/{user_id}",
            delete(remove_member),
        )
        .route(
            "/workspaces/{workspace_id}/invitations",
            get(list_invitations).post(invite_member),
        )
        .route(
            "/workspaces/{workspace_id}/invitations/{invitation_id}",
            delete(revoke_invitation),
        )
        .route("/invitations/{token}/accept", post(accept_invitation))
        .route(
            "/workspaces/{workspace_id}/groups",
            get(list_groups).post(create_group),
        )
        .route(
            "/workspaces/{workspace_id}/groups/{group_id}",
            get(get_group).put(update_group).delete(delete_group),
        )
        .route(
            "/workspaces/{workspace_id}/groups/{group_id}/members/{user_id}",
            put(add_group_member).delete(remove_group_member),
        )
}

async fn list_workspaces(
    State(state): State<HealthState>,
    auth: Authenticated,
) -> Result<Json<serde_json::Value>, Problem> {
    let value = state
        .governance
        .service
        .list_workspaces(auth.principal.user.id)
        .await
        .map_err(Problem::from)?;
    Ok(Json(
        serde_json::to_value(value).map_err(|_| Problem::internal())?,
    ))
}

async fn create_workspace(
    State(state): State<HealthState>,
    headers: HeaderMap,
    auth: Authenticated,
    Json(input): Json<CreateWorkspaceInput>,
) -> Result<Response, Problem> {
    command_headers(&state, &headers, &auth)?;
    let value = state
        .governance
        .service
        .create_workspace(auth.principal.user.id, input, idempotency_key(&headers)?)
        .await
        .map_err(Problem::from)?;
    Ok((StatusCode::CREATED, Json(value)).into_response())
}

async fn get_workspace(
    State(state): State<HealthState>,
    auth: Authenticated,
    Path(workspace): Path<Uuid>,
) -> Result<Json<serde_json::Value>, Problem> {
    let value = state
        .governance
        .service
        .get_workspace(auth.principal.user.id, workspace)
        .await
        .map_err(Problem::from)?;
    Ok(Json(
        serde_json::to_value(value).map_err(|_| Problem::internal())?,
    ))
}

async fn update_workspace(
    State(state): State<HealthState>,
    headers: HeaderMap,
    auth: Authenticated,
    Path(workspace): Path<Uuid>,
    Json(input): Json<UpdateWorkspaceInput>,
) -> Result<Json<serde_json::Value>, Problem> {
    command_headers(&state, &headers, &auth)?;
    let value = state
        .governance
        .service
        .update_workspace(
            auth.principal.user.id,
            workspace,
            expected_revision(&headers)?,
            input,
            idempotency_key(&headers)?,
        )
        .await
        .map_err(Problem::from)?;
    json_value(value)
}

async fn schedule_deletion(
    State(state): State<HealthState>,
    headers: HeaderMap,
    auth: Authenticated,
    Path(workspace): Path<Uuid>,
    Json(input): Json<ReasonInput>,
) -> Result<Json<serde_json::Value>, Problem> {
    command_headers(&state, &headers, &auth)?;
    json_value(
        state
            .governance
            .service
            .schedule_deletion(
                auth.principal.user.id,
                workspace,
                expected_revision(&headers)?,
                input,
                idempotency_key(&headers)?,
            )
            .await
            .map_err(Problem::from)?,
    )
}
async fn cancel_deletion(
    State(state): State<HealthState>,
    headers: HeaderMap,
    auth: Authenticated,
    Path(workspace): Path<Uuid>,
) -> Result<Json<serde_json::Value>, Problem> {
    command_headers(&state, &headers, &auth)?;
    json_value(
        state
            .governance
            .service
            .cancel_deletion(
                auth.principal.user.id,
                workspace,
                expected_revision(&headers)?,
                idempotency_key(&headers)?,
            )
            .await
            .map_err(Problem::from)?,
    )
}
async fn list_members(
    State(state): State<HealthState>,
    auth: Authenticated,
    Path(workspace): Path<Uuid>,
) -> Result<Json<serde_json::Value>, Problem> {
    json_value(
        state
            .governance
            .service
            .list_members(auth.principal.user.id, workspace)
            .await
            .map_err(Problem::from)?,
    )
}
async fn update_member_role(
    State(state): State<HealthState>,
    headers: HeaderMap,
    auth: Authenticated,
    Path((workspace, user)): Path<(Uuid, Uuid)>,
    Json(input): Json<UpdateMemberRoleInput>,
) -> Result<Json<serde_json::Value>, Problem> {
    command_headers(&state, &headers, &auth)?;
    json_value(
        state
            .governance
            .service
            .update_member_role(
                auth.principal.user.id,
                workspace,
                user,
                expected_revision(&headers)?,
                input,
                idempotency_key(&headers)?,
            )
            .await
            .map_err(Problem::from)?,
    )
}
async fn remove_member(
    State(state): State<HealthState>,
    headers: HeaderMap,
    auth: Authenticated,
    Path((workspace, user)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, Problem> {
    command_headers(&state, &headers, &auth)?;
    json_value(
        state
            .governance
            .service
            .remove_member(
                auth.principal.user.id,
                workspace,
                user,
                expected_revision(&headers)?,
                idempotency_key(&headers)?,
            )
            .await
            .map_err(Problem::from)?,
    )
}

#[derive(Deserialize)]
struct CursorQuery {
    cursor: Option<Uuid>,
}
async fn list_invitations(
    State(state): State<HealthState>,
    auth: Authenticated,
    Path(workspace): Path<Uuid>,
    Query(query): Query<CursorQuery>,
) -> Result<Json<serde_json::Value>, Problem> {
    json_value(
        state
            .governance
            .service
            .list_invitations(auth.principal.user.id, workspace, query.cursor)
            .await
            .map_err(Problem::from)?,
    )
}
async fn invite_member(
    State(state): State<HealthState>,
    headers: HeaderMap,
    auth: Authenticated,
    Path(workspace): Path<Uuid>,
    Json(input): Json<InviteMemberInput>,
) -> Result<Response, Problem> {
    command_headers(&state, &headers, &auth)?;
    let created = state
        .governance
        .service
        .invite_member(
            auth.principal.user.id,
            workspace,
            input,
            idempotency_key(&headers)?,
        )
        .await
        .map_err(Problem::from)?;
    Ok((StatusCode::CREATED, Json(created.invitation)).into_response())
}
async fn revoke_invitation(
    State(state): State<HealthState>,
    headers: HeaderMap,
    auth: Authenticated,
    Path((workspace, invitation)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, Problem> {
    command_headers(&state, &headers, &auth)?;
    json_value(
        state
            .governance
            .service
            .revoke_invitation(
                auth.principal.user.id,
                workspace,
                invitation,
                expected_revision(&headers)?,
                idempotency_key(&headers)?,
            )
            .await
            .map_err(Problem::from)?,
    )
}
async fn accept_invitation(
    State(state): State<HealthState>,
    headers: HeaderMap,
    auth: Authenticated,
    Path(token): Path<String>,
) -> Result<Response, Problem> {
    command_headers(&state, &headers, &auth)?;
    let value = state
        .governance
        .service
        .accept_invitation(
            auth.principal.user.id,
            &auth.principal.user.email,
            &token,
            idempotency_key(&headers)?,
        )
        .await
        .map_err(Problem::from)?;
    Ok((StatusCode::CREATED, Json(value)).into_response())
}
async fn list_groups(
    State(state): State<HealthState>,
    auth: Authenticated,
    Path(workspace): Path<Uuid>,
) -> Result<Json<serde_json::Value>, Problem> {
    json_value(
        state
            .governance
            .service
            .list_groups(auth.principal.user.id, workspace)
            .await
            .map_err(Problem::from)?,
    )
}
async fn get_group(
    State(state): State<HealthState>,
    auth: Authenticated,
    Path((workspace, group)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, Problem> {
    json_value(
        state
            .governance
            .service
            .get_group(auth.principal.user.id, workspace, group)
            .await
            .map_err(Problem::from)?,
    )
}
async fn create_group(
    State(state): State<HealthState>,
    headers: HeaderMap,
    auth: Authenticated,
    Path(workspace): Path<Uuid>,
    Json(input): Json<CreateGroupInput>,
) -> Result<Response, Problem> {
    command_headers(&state, &headers, &auth)?;
    let value = state
        .governance
        .service
        .create_group(
            auth.principal.user.id,
            workspace,
            input,
            idempotency_key(&headers)?,
        )
        .await
        .map_err(Problem::from)?;
    Ok((StatusCode::CREATED, Json(value)).into_response())
}
async fn update_group(
    State(state): State<HealthState>,
    headers: HeaderMap,
    auth: Authenticated,
    Path((workspace, group)): Path<(Uuid, Uuid)>,
    Json(input): Json<UpdateGroupInput>,
) -> Result<Json<serde_json::Value>, Problem> {
    command_headers(&state, &headers, &auth)?;
    json_value(
        state
            .governance
            .service
            .update_group(
                auth.principal.user.id,
                workspace,
                group,
                expected_revision(&headers)?,
                input,
                idempotency_key(&headers)?,
            )
            .await
            .map_err(Problem::from)?,
    )
}
async fn delete_group(
    State(state): State<HealthState>,
    headers: HeaderMap,
    auth: Authenticated,
    Path((workspace, group)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, Problem> {
    command_headers(&state, &headers, &auth)?;
    state
        .governance
        .service
        .delete_group(
            auth.principal.user.id,
            workspace,
            group,
            expected_revision(&headers)?,
            idempotency_key(&headers)?,
        )
        .await
        .map_err(Problem::from)?;
    Ok(StatusCode::NO_CONTENT)
}
async fn add_group_member(
    State(state): State<HealthState>,
    headers: HeaderMap,
    auth: Authenticated,
    Path((workspace, group, user)): Path<(Uuid, Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, Problem> {
    change_group_member(true, state, headers, auth, workspace, group, user).await
}
async fn remove_group_member(
    State(state): State<HealthState>,
    headers: HeaderMap,
    auth: Authenticated,
    Path((workspace, group, user)): Path<(Uuid, Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, Problem> {
    change_group_member(false, state, headers, auth, workspace, group, user).await
}
async fn change_group_member(
    add: bool,
    state: HealthState,
    headers: HeaderMap,
    auth: Authenticated,
    workspace: Uuid,
    group: Uuid,
    user: Uuid,
) -> Result<Json<serde_json::Value>, Problem> {
    command_headers(&state, &headers, &auth)?;
    json_value(
        state
            .governance
            .service
            .change_group_member(adoc_application::governance::GroupMemberCommand {
                add,
                actor_id: auth.principal.user.id,
                workspace_id: workspace,
                group_id: group,
                user_id: user,
                expected_revision: expected_revision(&headers)?,
                idempotency_key: idempotency_key(&headers)?.to_owned(),
            })
            .await
            .map_err(Problem::from)?,
    )
}

fn command_headers(
    state: &HealthState,
    headers: &HeaderMap,
    auth: &Authenticated,
) -> Result<(), Problem> {
    validate_command(&state.identity, headers, auth)
}
fn json_value<T: serde::Serialize>(value: T) -> Result<Json<serde_json::Value>, Problem> {
    Ok(Json(
        serde_json::to_value(value).map_err(|_| Problem::internal())?,
    ))
}

impl From<GovernanceError> for Problem {
    fn from(error: GovernanceError) -> Self {
        let publish_conflict = match &error {
            GovernanceError::PublishBaseStale {
                base_version_id,
                current_version_id,
                draft_id,
            } => Some(PublishConflict {
                base_version_id: *base_version_id,
                current_version_id: *current_version_id,
                draft_id: *draft_id,
            }),
            _ => None,
        };
        let (status, code, retryable, current_revision, reference_count) = match error {
            GovernanceError::Validation => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "VALIDATION_FAILED",
                false,
                None,
                None,
            ),
            GovernanceError::WorkspaceNotFound => (
                StatusCode::NOT_FOUND,
                "WORKSPACE_NOT_FOUND",
                false,
                None,
                None,
            ),
            GovernanceError::WorkspaceSlugTaken => (
                StatusCode::CONFLICT,
                "WORKSPACE_SLUG_TAKEN",
                false,
                None,
                None,
            ),
            GovernanceError::WorkspaceStateInvalid => (
                StatusCode::CONFLICT,
                "WORKSPACE_STATE_INVALID",
                false,
                None,
                None,
            ),
            GovernanceError::Forbidden => (StatusCode::FORBIDDEN, "FORBIDDEN", false, None, None),
            GovernanceError::RevisionConflict { current_revision } => (
                StatusCode::CONFLICT,
                "REVISION_CONFLICT",
                false,
                Some(current_revision),
                None,
            ),
            GovernanceError::LastOwner => (StatusCode::CONFLICT, "LAST_OWNER", false, None, None),
            GovernanceError::InvitationExists => {
                (StatusCode::CONFLICT, "INVITATION_EXISTS", false, None, None)
            }
            GovernanceError::InvitationInvalid => (
                StatusCode::NOT_FOUND,
                "INVITATION_INVALID",
                false,
                None,
                None,
            ),
            GovernanceError::InvitationStateInvalid => (
                StatusCode::CONFLICT,
                "INVITATION_STATE_INVALID",
                false,
                None,
                None,
            ),
            GovernanceError::GroupNotFound => {
                (StatusCode::NOT_FOUND, "GROUP_NOT_FOUND", false, None, None)
            }
            GovernanceError::GroupNameTaken => {
                (StatusCode::CONFLICT, "GROUP_NAME_TAKEN", false, None, None)
            }
            GovernanceError::GroupInUse { reference_count } => (
                StatusCode::CONFLICT,
                "GROUP_IN_USE",
                false,
                None,
                Some(reference_count),
            ),
            GovernanceError::GroupMemberInvalid => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "GROUP_MEMBER_INVALID",
                false,
                None,
                None,
            ),
            GovernanceError::GroupMemberNotFound => (
                StatusCode::NOT_FOUND,
                "GROUP_MEMBER_NOT_FOUND",
                false,
                None,
                None,
            ),
            GovernanceError::DocumentNotFound => (
                StatusCode::NOT_FOUND,
                "DOCUMENT_NOT_FOUND",
                false,
                None,
                None,
            ),
            GovernanceError::DocumentParentInvalid => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "DOCUMENT_PARENT_INVALID",
                false,
                None,
                None,
            ),
            GovernanceError::DocumentTreeCycle => (
                StatusCode::CONFLICT,
                "DOCUMENT_TREE_CYCLE",
                false,
                None,
                None,
            ),
            GovernanceError::DocumentRankConflict => (
                StatusCode::CONFLICT,
                "DOCUMENT_RANK_CONFLICT",
                false,
                None,
                None,
            ),
            GovernanceError::DocumentStateInvalid => (
                StatusCode::CONFLICT,
                "DOCUMENT_STATE_INVALID",
                false,
                None,
                None,
            ),
            GovernanceError::DocumentEffectivelyTrashed => (
                StatusCode::CONFLICT,
                "DOCUMENT_EFFECTIVELY_TRASHED",
                false,
                None,
                None,
            ),
            GovernanceError::MovePreviewStale => (
                StatusCode::CONFLICT,
                "MOVE_PREVIEW_STALE",
                false,
                None,
                None,
            ),
            GovernanceError::DraftNotFound => {
                (StatusCode::NOT_FOUND, "DRAFT_NOT_FOUND", false, None, None)
            }
            GovernanceError::DraftExists => {
                (StatusCode::CONFLICT, "DRAFT_EXISTS", false, None, None)
            }
            GovernanceError::OperationPreconditionFailed => (
                StatusCode::CONFLICT,
                "OPERATION_PRECONDITION_FAILED",
                false,
                None,
                None,
            ),
            GovernanceError::EditLeaseHeld { .. } => {
                (StatusCode::LOCKED, "EDIT_LEASE_HELD", false, None, None)
            }
            GovernanceError::EditLeaseInvalid => (
                StatusCode::CONFLICT,
                "EDIT_LEASE_INVALID",
                false,
                None,
                None,
            ),
            GovernanceError::EditLeaseExpired => (
                StatusCode::CONFLICT,
                "EDIT_LEASE_EXPIRED",
                false,
                None,
                None,
            ),
            GovernanceError::NoEffect => (StatusCode::CONFLICT, "NO_EFFECT", false, None, None),
            GovernanceError::DependencyUnavailable => (
                StatusCode::SERVICE_UNAVAILABLE,
                "DEPENDENCY_UNAVAILABLE",
                true,
                None,
                None,
            ),
            GovernanceError::PermissionDenied => (
                StatusCode::FORBIDDEN,
                "PERMISSION_DENIED",
                false,
                None,
                None,
            ),
            GovernanceError::PermissionSubjectInvalid => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "PERMISSION_SUBJECT_INVALID",
                false,
                None,
                None,
            ),
            GovernanceError::PermissionManageRequiresEditor => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "PERMISSION_MANAGE_REQUIRES_EDITOR",
                false,
                None,
                None,
            ),
            GovernanceError::PermissionGrantConflict => (
                StatusCode::CONFLICT,
                "PERMISSION_GRANT_CONFLICT",
                false,
                None,
                None,
            ),
            GovernanceError::PermissionLastManager => (
                StatusCode::CONFLICT,
                "PERMISSION_LAST_MANAGER",
                false,
                None,
                None,
            ),
            GovernanceError::PublishPolicyInvalid => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "PUBLISH_POLICY_INVALID",
                false,
                None,
                None,
            ),
            GovernanceError::VersionNotFound => (
                StatusCode::NOT_FOUND,
                "VERSION_NOT_FOUND",
                false,
                None,
                None,
            ),
            GovernanceError::PublishBaseStale { .. } => (
                StatusCode::CONFLICT,
                "PUBLISH_BASE_STALE",
                false,
                None,
                None,
            ),
            GovernanceError::PublishReviewRequired => (
                StatusCode::CONFLICT,
                "PUBLISH_REVIEW_REQUIRED",
                false,
                None,
                None,
            ),
            GovernanceError::PublishLeaseConflict => (
                StatusCode::LOCKED,
                "PUBLISH_LEASE_CONFLICT",
                false,
                None,
                None,
            ),
            GovernanceError::DocumentUnpublished => (
                StatusCode::CONFLICT,
                "DOCUMENT_UNPUBLISHED",
                false,
                None,
                None,
            ),
            GovernanceError::PublicLinkInvalid => (
                StatusCode::NOT_FOUND,
                "PUBLIC_LINK_INVALID",
                false,
                None,
                None,
            ),
            GovernanceError::PublicLinkStateInvalid => (
                StatusCode::CONFLICT,
                "PUBLIC_LINK_STATE_INVALID",
                false,
                None,
                None,
            ),
            GovernanceError::PublicLinkTokenAlreadyIssued => (
                StatusCode::CONFLICT,
                "PUBLIC_LINK_TOKEN_ALREADY_ISSUED",
                false,
                None,
                None,
            ),
            GovernanceError::DiscussionNotFound => (
                StatusCode::NOT_FOUND,
                "DISCUSSION_NOT_FOUND",
                false,
                None,
                None,
            ),
            GovernanceError::DiscussionTargetInvalid => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "DISCUSSION_TARGET_INVALID",
                false,
                None,
                None,
            ),
            GovernanceError::DiscussionStateInvalid => (
                StatusCode::CONFLICT,
                "DISCUSSION_STATE_INVALID",
                false,
                None,
                None,
            ),
            GovernanceError::DiscussionClosed => {
                (StatusCode::CONFLICT, "DISCUSSION_CLOSED", false, None, None)
            }
            GovernanceError::DiscussionTopicRequired => (
                StatusCode::CONFLICT,
                "DISCUSSION_TOPIC_REQUIRED",
                false,
                None,
                None,
            ),
            GovernanceError::MessageNotFound => (
                StatusCode::NOT_FOUND,
                "MESSAGE_NOT_FOUND",
                false,
                None,
                None,
            ),
            GovernanceError::MessageEditWindowExpired => (
                StatusCode::CONFLICT,
                "MESSAGE_EDIT_WINDOW_EXPIRED",
                false,
                None,
                None,
            ),
            GovernanceError::MessageStateInvalid => (
                StatusCode::CONFLICT,
                "MESSAGE_STATE_INVALID",
                false,
                None,
                None,
            ),
            GovernanceError::InboxItemNotFound => (
                StatusCode::NOT_FOUND,
                "INBOX_ITEM_NOT_FOUND",
                false,
                None,
                None,
            ),
            GovernanceError::ReviewNotFound => {
                (StatusCode::NOT_FOUND, "REVIEW_NOT_FOUND", false, None, None)
            }
            GovernanceError::ReviewStateInvalid => (
                StatusCode::CONFLICT,
                "REVIEW_STATE_INVALID",
                false,
                None,
                None,
            ),
            GovernanceError::ReviewNotEligible => (
                StatusCode::FORBIDDEN,
                "REVIEW_NOT_ELIGIBLE",
                false,
                None,
                None,
            ),
            GovernanceError::ReferenceNotFound => (
                StatusCode::NOT_FOUND,
                "REFERENCE_NOT_FOUND",
                false,
                None,
                None,
            ),
            GovernanceError::ReferenceTargetInvalid => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "REFERENCE_TARGET_INVALID",
                false,
                None,
                None,
            ),
            GovernanceError::VocabularyNotFound => (
                StatusCode::NOT_FOUND,
                "VOCABULARY_NOT_FOUND",
                false,
                None,
                None,
            ),
            GovernanceError::VocabularyTermConflict => (
                StatusCode::CONFLICT,
                "VOCABULARY_TERM_CONFLICT",
                false,
                None,
                None,
            ),
            GovernanceError::VocabularyStateInvalid => (
                StatusCode::CONFLICT,
                "VOCABULARY_STATE_INVALID",
                false,
                None,
                None,
            ),
            GovernanceError::IdempotencyKeyReused => (
                StatusCode::CONFLICT,
                "IDEMPOTENCY_KEY_REUSED",
                false,
                None,
                None,
            ),
            GovernanceError::StorageUnavailable => (
                StatusCode::SERVICE_UNAVAILABLE,
                "DEPENDENCY_UNAVAILABLE",
                true,
                None,
                None,
            ),
            GovernanceError::Internal => return Problem::internal(),
        };
        Self {
            status,
            code,
            retryable,
            current_revision,
            reference_count,
            publish_conflict,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn governance_problem_mapping_preserves_safe_conflict_metadata() {
        let revision = Problem::from(GovernanceError::RevisionConflict {
            current_revision: 7,
        });
        assert_eq!(revision.status, StatusCode::CONFLICT);
        assert_eq!(revision.code, "REVISION_CONFLICT");
        assert_eq!(revision.current_revision, Some(7));
        assert!(revision.publish_conflict.is_none());

        let group = Problem::from(GovernanceError::GroupInUse { reference_count: 3 });
        assert_eq!(group.status, StatusCode::CONFLICT);
        assert_eq!(group.code, "GROUP_IN_USE");
        assert_eq!(group.reference_count, Some(3));

        let base = Uuid::now_v7();
        let current = Uuid::now_v7();
        let draft = Uuid::now_v7();
        let stale = Problem::from(GovernanceError::PublishBaseStale {
            base_version_id: Some(base),
            current_version_id: Some(current),
            draft_id: draft,
        });
        let conflict = stale.publish_conflict.unwrap();
        assert_eq!(conflict.base_version_id, Some(base));
        assert_eq!(conflict.current_version_id, Some(current));
        assert_eq!(conflict.draft_id, draft);
    }

    #[test]
    fn governance_problem_mapping_hides_storage_details() {
        let problem = Problem::from(GovernanceError::StorageUnavailable);
        assert_eq!(problem.status, StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(problem.code, "DEPENDENCY_UNAVAILABLE");
        assert!(problem.retryable);
    }
}
