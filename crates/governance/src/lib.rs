#![forbid(unsafe_code)]

//! Workspace governance bounded context.

use std::collections::{BTreeMap, BTreeSet};

use chrono::{DateTime, Utc};
use email_address::EmailAddress;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use unicode_normalization::UnicodeNormalization;
use uuid::Uuid;

pub const INVITATION_TTL_DAYS: i64 = 7;
pub const WORKSPACE_DELETION_GRACE_DAYS: i64 = 30;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceName(String);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GroupName(String);

impl WorkspaceName {
    pub fn parse(value: &str) -> Result<Self, GovernanceValidationError> {
        normalized_name(value).map(Self)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl GroupName {
    pub fn parse(value: &str) -> Result<Self, GovernanceValidationError> {
        normalized_name(value).map(Self)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn normalized(&self) -> String {
        self.0.to_lowercase()
    }
}

fn normalized_name(value: &str) -> Result<String, GovernanceValidationError> {
    let value = value.nfc().collect::<String>();
    let value = value.trim();
    let length = value.chars().count();
    if !(1..=200).contains(&length) || value.chars().any(char::is_control) {
        return Err(GovernanceValidationError::Name);
    }
    Ok(value.to_owned())
}

pub fn slug_base(name: &WorkspaceName) -> String {
    let mut output = String::new();
    let mut hyphen = false;
    for character in name.as_str().chars().flat_map(char::to_lowercase) {
        if character.is_ascii_alphanumeric() {
            output.push(character);
            hyphen = false;
        } else if !output.is_empty() && !hyphen {
            output.push('-');
            hyphen = true;
        }
    }
    while output.ends_with('-') {
        output.pop();
    }
    if output.len() < 2 {
        output = "workspace".to_owned();
    }
    output.truncate(54);
    while output.ends_with('-') {
        output.pop();
    }
    output
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WorkspaceStatus {
    Active,
    DeletionScheduled,
    Purging,
    Deleted,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PublishMode {
    Direct,
    ReviewRequired,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Ord, PartialOrd, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MembershipRole {
    Member,
    Admin,
    Owner,
}

impl MembershipRole {
    pub fn can_administer(self) -> bool {
        matches!(self, Self::Admin | Self::Owner)
    }

    pub fn can_manage_owners(self) -> bool {
        self == Self::Owner
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MembershipStatus {
    Active,
    Suspended,
    Removed,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum InvitationRole {
    Member,
    Admin,
}

impl From<InvitationRole> for MembershipRole {
    fn from(role: InvitationRole) -> Self {
        match role {
            InvitationRole::Member => Self::Member,
            InvitationRole::Admin => Self::Admin,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum InvitationStatus {
    Pending,
    Accepted,
    Revoked,
    Expired,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Workspace {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub status: WorkspaceStatus,
    pub revision: i64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Membership {
    pub user_id: Uuid,
    pub role: MembershipRole,
    pub status: MembershipStatus,
    pub revision: i64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Invitation {
    pub id: Uuid,
    pub email: String,
    pub role: InvitationRole,
    pub status: InvitationStatus,
    pub expires_at: DateTime<Utc>,
    pub revision: i64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Group {
    pub id: Uuid,
    pub name: String,
    pub member_ids: Vec<Uuid>,
    pub revision: i64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateWorkspaceInput {
    pub name: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UpdateWorkspaceInput {
    pub name: Option<String>,
    pub default_publish_mode: Option<PublishMode>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReasonInput {
    pub reason: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InviteMemberInput {
    pub email: String,
    pub role: InvitationRole,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UpdateMemberRoleInput {
    pub role: MembershipRole,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateGroupInput {
    pub name: String,
    #[serde(default)]
    pub member_ids: Vec<Uuid>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UpdateGroupInput {
    pub name: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Access {
    NoAccess,
    Viewer,
    Contributor,
    Editor,
}

impl Access {
    pub fn can_view(self) -> bool {
        self >= Self::Viewer
    }

    pub fn can_contribute(self) -> bool {
        self >= Self::Contributor
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SubjectKind {
    User,
    Group,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionGrant {
    pub id: Uuid,
    pub subject_kind: SubjectKind,
    pub subject_id: Uuid,
    pub access: Access,
    pub manage: bool,
    pub revision: i64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PermissionGrantInput {
    pub subject_kind: SubjectKind,
    pub subject_id: Uuid,
    pub access: Access,
    pub manage: bool,
}

impl PermissionGrantInput {
    pub fn validate(&self) -> Result<(), GovernanceValidationError> {
        if self.manage && self.access != Access::Editor {
            Err(GovernanceValidationError::PermissionManage)
        } else {
            Ok(())
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EffectivePermission {
    pub access: Access,
    pub manage: bool,
    pub source_document_id: Option<Uuid>,
    pub evidence_grant_ids: Vec<Uuid>,
}

impl EffectivePermission {
    pub fn denied() -> Self {
        Self {
            access: Access::NoAccess,
            manage: false,
            source_document_id: None,
            evidence_grant_ids: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PermissionNode {
    pub document_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub user_grant: Option<PermissionGrant>,
    pub group_grants: Vec<PermissionGrant>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PermissionDecision {
    NoGrant,
    UserGrant,
    GroupDeny,
    GroupMax,
    Inherited,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionStep {
    pub document_id: Uuid,
    pub decision: PermissionDecision,
}

pub fn resolve_permission_path(
    leaf_to_root: &[PermissionNode],
) -> Result<(EffectivePermission, Vec<PermissionStep>), GovernanceValidationError> {
    let mut result = None;
    let mut steps = Vec::with_capacity(leaf_to_root.len());
    for node in leaf_to_root {
        if result.is_some() {
            steps.push(PermissionStep {
                document_id: node.document_id,
                decision: PermissionDecision::Inherited,
            });
            continue;
        }
        if let Some((effective, decision)) = resolve_local(node) {
            result = Some(effective);
            steps.push(PermissionStep {
                document_id: node.document_id,
                decision,
            });
        } else {
            steps.push(PermissionStep {
                document_id: node.document_id,
                decision: PermissionDecision::NoGrant,
            });
        }
    }
    Ok((result.unwrap_or_else(EffectivePermission::denied), steps))
}

pub fn compile_permission_scope(
    parent_before_child: &[PermissionNode],
) -> Result<BTreeMap<Uuid, EffectivePermission>, GovernanceValidationError> {
    let mut resolved = BTreeMap::new();
    for node in parent_before_child {
        let effective = if let Some((local, _)) = resolve_local(node) {
            local
        } else if let Some(parent_id) = node.parent_id {
            resolved
                .get(&parent_id)
                .cloned()
                .ok_or(GovernanceValidationError::PermissionTree)?
        } else {
            EffectivePermission::denied()
        };
        if resolved.insert(node.document_id, effective).is_some() {
            return Err(GovernanceValidationError::PermissionTree);
        }
    }
    Ok(resolved)
}

fn resolve_local(node: &PermissionNode) -> Option<(EffectivePermission, PermissionDecision)> {
    if let Some(grant) = &node.user_grant {
        return Some((
            EffectivePermission {
                access: grant.access,
                manage: grant.manage,
                source_document_id: Some(node.document_id),
                evidence_grant_ids: vec![grant.id],
            },
            PermissionDecision::UserGrant,
        ));
    }
    let mut denies = node
        .group_grants
        .iter()
        .filter(|grant| grant.access == Access::NoAccess)
        .collect::<Vec<_>>();
    if !denies.is_empty() {
        denies.sort_by_key(|grant| grant.id);
        return Some((
            EffectivePermission {
                access: Access::NoAccess,
                manage: false,
                source_document_id: Some(node.document_id),
                evidence_grant_ids: denies.into_iter().map(|grant| grant.id).collect(),
            },
            PermissionDecision::GroupDeny,
        ));
    }
    let access = node.group_grants.iter().map(|grant| grant.access).max()?;
    let mut selected = node
        .group_grants
        .iter()
        .filter(|grant| grant.access == access)
        .collect::<Vec<_>>();
    selected.sort_by_key(|grant| grant.id);
    let manage = access == Access::Editor && selected.iter().any(|grant| grant.manage);
    Some((
        EffectivePermission {
            access,
            manage,
            source_document_id: Some(node.document_id),
            evidence_grant_ids: selected.into_iter().map(|grant| grant.id).collect(),
        },
        PermissionDecision::GroupMax,
    ))
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(
    tag = "kind",
    rename_all = "SCREAMING_SNAKE_CASE",
    rename_all_fields = "camelCase"
)]
pub enum ReviewerRule {
    AnyEditor,
    Users { user_ids: Vec<Uuid> },
    Groups { group_ids: Vec<Uuid> },
}

impl ReviewerRule {
    pub fn normalize(self) -> Result<Self, GovernanceValidationError> {
        match self {
            Self::AnyEditor => Ok(Self::AnyEditor),
            Self::Users { user_ids } => Ok(Self::Users {
                user_ids: normalized_policy_ids(user_ids)?,
            }),
            Self::Groups { group_ids } => Ok(Self::Groups {
                group_ids: normalized_policy_ids(group_ids)?,
            }),
        }
    }
}

fn normalized_policy_ids(values: Vec<Uuid>) -> Result<Vec<Uuid>, GovernanceValidationError> {
    let values = values.into_iter().collect::<BTreeSet<_>>();
    if values.is_empty() || values.len() > 100 {
        return Err(GovernanceValidationError::PublishPolicy);
    }
    Ok(values.into_iter().collect())
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublishPolicy {
    pub document_id: Uuid,
    pub mode: PublishMode,
    pub required_approvals: i16,
    pub reviewer_rule: ReviewerRule,
    pub inherited_from_document_id: Option<Uuid>,
    pub revision: i64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SetPublishPolicyInput {
    pub mode: PublishMode,
    pub required_approvals: i16,
    pub reviewer_rule: ReviewerRule,
}

impl SetPublishPolicyInput {
    pub fn normalize(self) -> Result<Self, GovernanceValidationError> {
        let reviewer_rule = self.reviewer_rule.normalize()?;
        let valid = match (&self.mode, &reviewer_rule) {
            (PublishMode::Direct, ReviewerRule::AnyEditor) => self.required_approvals == 0,
            (PublishMode::ReviewRequired, _) => (1..=20).contains(&self.required_approvals),
            _ => false,
        };
        if !valid {
            return Err(GovernanceValidationError::PublishPolicy);
        }
        Ok(Self {
            mode: self.mode,
            required_approvals: self.required_approvals,
            reviewer_rule,
        })
    }
}

pub fn normalized_email(value: &str) -> Result<String, GovernanceValidationError> {
    let value = value.trim();
    let address = value
        .parse::<EmailAddress>()
        .map_err(|_| GovernanceValidationError::Email)?;
    if value.len() > 320 {
        return Err(GovernanceValidationError::Email);
    }
    Ok(address.to_string().to_ascii_lowercase())
}

pub fn normalized_member_ids(values: Vec<Uuid>) -> Result<Vec<Uuid>, GovernanceValidationError> {
    if values.len() > 1000 {
        return Err(GovernanceValidationError::Members);
    }
    Ok(values
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect())
}

pub fn validate_reason(value: &str) -> Result<String, GovernanceValidationError> {
    let value = value.trim();
    if value.is_empty() || value.chars().count() > 500 || value.chars().any(char::is_control) {
        return Err(GovernanceValidationError::Reason);
    }
    Ok(value.to_owned())
}

pub fn may_change_role(
    actor: MembershipRole,
    target: MembershipRole,
    requested: MembershipRole,
) -> bool {
    actor.can_manage_owners()
        || (actor.can_administer()
            && target != MembershipRole::Owner
            && requested != MembershipRole::Owner)
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum GovernanceValidationError {
    #[error("name is invalid")]
    Name,
    #[error("email is invalid")]
    Email,
    #[error("member list is invalid")]
    Members,
    #[error("reason is invalid")]
    Reason,
    #[error("manage requires editor access")]
    PermissionManage,
    #[error("permission tree is invalid")]
    PermissionTree,
    #[error("publish policy is invalid")]
    PublishPolicy,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_are_normalized_and_slug_has_a_safe_fallback() {
        let name = WorkspaceName::parse("  인증  팀  ").unwrap();
        assert_eq!(name.as_str(), "인증  팀");
        assert_eq!(slug_base(&name), "workspace");
        assert_eq!(
            slug_base(&WorkspaceName::parse("API & Platform").unwrap()),
            "api-platform"
        );
    }

    #[test]
    fn role_capability_never_lets_admin_manage_owner() {
        assert!(!may_change_role(
            MembershipRole::Admin,
            MembershipRole::Owner,
            MembershipRole::Member
        ));
        assert!(!may_change_role(
            MembershipRole::Admin,
            MembershipRole::Member,
            MembershipRole::Owner
        ));
        assert!(may_change_role(
            MembershipRole::Owner,
            MembershipRole::Owner,
            MembershipRole::Member
        ));
    }

    #[test]
    fn member_ids_are_deterministic_and_bounded() {
        let id = Uuid::nil();
        assert_eq!(normalized_member_ids(vec![id, id]).unwrap(), vec![id]);
        assert!(normalized_member_ids(vec![Uuid::nil(); 1001]).is_err());
    }

    fn grant(id: u128, access: Access, manage: bool) -> PermissionGrant {
        PermissionGrant {
            id: Uuid::from_u128(id),
            subject_kind: SubjectKind::Group,
            subject_id: Uuid::from_u128(id + 100),
            access,
            manage,
            revision: 0,
        }
    }

    #[test]
    fn permission_precedence_is_nearest_user_deny_then_group_max() {
        let leaf = Uuid::from_u128(1);
        let root = Uuid::from_u128(2);
        let path = vec![
            PermissionNode {
                document_id: leaf,
                parent_id: Some(root),
                user_grant: None,
                group_grants: vec![
                    grant(11, Access::Editor, true),
                    grant(10, Access::NoAccess, false),
                ],
            },
            PermissionNode {
                document_id: root,
                parent_id: None,
                user_grant: Some(grant(12, Access::Editor, true)),
                group_grants: Vec::new(),
            },
        ];
        let (effective, steps) = resolve_permission_path(&path).unwrap();
        assert_eq!(effective.access, Access::NoAccess);
        assert!(!effective.manage);
        assert_eq!(effective.evidence_grant_ids, vec![Uuid::from_u128(10)]);
        assert_eq!(steps[0].decision, PermissionDecision::GroupDeny);
        assert_eq!(steps[1].decision, PermissionDecision::Inherited);

        let mut user_wins = path;
        user_wins[0].user_grant = Some(grant(13, Access::Viewer, false));
        assert_eq!(
            resolve_permission_path(&user_wins).unwrap().0.access,
            Access::Viewer
        );
    }

    #[test]
    fn point_and_scope_use_the_same_inheritance_result() {
        let root = Uuid::from_u128(20);
        let child = Uuid::from_u128(21);
        let grandchild = Uuid::from_u128(22);
        let nodes = vec![
            PermissionNode {
                document_id: root,
                parent_id: None,
                user_grant: None,
                group_grants: vec![grant(30, Access::Viewer, false)],
            },
            PermissionNode {
                document_id: child,
                parent_id: Some(root),
                user_grant: Some(grant(31, Access::Editor, true)),
                group_grants: Vec::new(),
            },
            PermissionNode {
                document_id: grandchild,
                parent_id: Some(child),
                user_grant: None,
                group_grants: Vec::new(),
            },
        ];
        let scope = compile_permission_scope(&nodes).unwrap();
        for (index, node) in nodes.iter().enumerate() {
            let mut path = vec![node.clone()];
            let mut parent = node.parent_id;
            while let Some(parent_id) = parent {
                let ancestor = nodes[..index]
                    .iter()
                    .find(|candidate| candidate.document_id == parent_id)
                    .unwrap();
                path.push(ancestor.clone());
                parent = ancestor.parent_id;
            }
            assert_eq!(
                &resolve_permission_path(&path).unwrap().0,
                scope.get(&node.document_id).unwrap()
            );
        }
    }

    #[test]
    fn publish_policy_normalizes_subjects_and_rejects_invalid_direct_mode() {
        let id = Uuid::from_u128(40);
        let policy = SetPublishPolicyInput {
            mode: PublishMode::ReviewRequired,
            required_approvals: 1,
            reviewer_rule: ReviewerRule::Users {
                user_ids: vec![id, id],
            },
        }
        .normalize()
        .unwrap();
        assert_eq!(
            policy.reviewer_rule,
            ReviewerRule::Users { user_ids: vec![id] }
        );
        assert!(
            SetPublishPolicyInput {
                mode: PublishMode::Direct,
                required_approvals: 1,
                reviewer_rule: ReviewerRule::AnyEditor,
            }
            .normalize()
            .is_err()
        );
    }
}
