#![forbid(unsafe_code)]

//! Workspace governance bounded context.

use std::collections::BTreeSet;

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
}
