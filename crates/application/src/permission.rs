use std::sync::Arc;

pub use adoc_governance::{
    Access, EffectivePermission, PermissionDecision, PermissionGrant, PermissionGrantInput,
    PermissionNode, PermissionStep, PublishMode, PublishPolicy, ReviewerRule,
    SetPublishPolicyInput, SubjectKind, compile_permission_scope, resolve_permission_path,
};
use adoc_ports::BoxFuture;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::{
    governance::{Command, GovernanceError},
    identity::Clock,
};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessStamp {
    pub permission_revision: i64,
    pub policy_revision: i64,
    pub membership_revision: i64,
}

#[derive(Clone, Debug)]
pub struct PointSnapshot {
    pub workspace_id: Uuid,
    pub user_id: Uuid,
    pub document_id: Uuid,
    pub stamp: AccessStamp,
    pub nodes: Vec<PermissionNode>,
}

#[derive(Clone, Debug)]
pub struct ScopeSnapshot {
    pub workspace_id: Uuid,
    pub user_id: Uuid,
    pub stamp: AccessStamp,
    pub nodes: Vec<PermissionNode>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionView {
    pub effective: EffectivePermission,
    pub explicit_grants: Vec<PermissionGrant>,
    pub revision: i64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionExplanation {
    pub effective: EffectivePermission,
    pub steps: Vec<PermissionStep>,
    pub fingerprint: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionScope {
    pub workspace_id: Uuid,
    pub user_id: Uuid,
    pub accessible_document_ids: Vec<Uuid>,
    pub fingerprint: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CachedPoint {
    pub effective: EffectivePermission,
    pub fingerprint: String,
}

#[derive(Clone, Debug)]
pub struct PermissionMutation {
    pub workspace_id: Uuid,
    pub document_id: Uuid,
    pub grant_id: Uuid,
    pub expected_revision: i64,
    pub input: Option<PermissionGrantInput>,
    pub command: Command,
}

#[derive(Clone, Debug)]
pub struct SetPermissionCommand {
    pub actor_id: Uuid,
    pub workspace_id: Uuid,
    pub document_id: Uuid,
    pub grant_id: Uuid,
    pub expected_revision: i64,
    pub input: PermissionGrantInput,
    pub idempotency_key: String,
}

#[derive(Clone, Debug)]
pub struct PolicyMutation {
    pub workspace_id: Uuid,
    pub document_id: Uuid,
    pub expected_revision: i64,
    pub input: SetPublishPolicyInput,
    pub command: Command,
}

pub trait PermissionRepository: Send + Sync {
    fn access_stamp<'a>(
        &'a self,
        user_id: Uuid,
        workspace_id: Uuid,
        document_id: Uuid,
    ) -> BoxFuture<'a, Result<AccessStamp, GovernanceError>>;
    fn point_snapshot<'a>(
        &'a self,
        user_id: Uuid,
        workspace_id: Uuid,
        document_id: Uuid,
    ) -> BoxFuture<'a, Result<PointSnapshot, GovernanceError>>;
    fn scope_snapshot<'a>(
        &'a self,
        user_id: Uuid,
        workspace_id: Uuid,
    ) -> BoxFuture<'a, Result<ScopeSnapshot, GovernanceError>>;
    fn permission_metadata<'a>(
        &'a self,
        workspace_id: Uuid,
        document_id: Uuid,
    ) -> BoxFuture<'a, Result<(Vec<PermissionGrant>, i64), GovernanceError>>;
    fn subject_snapshot<'a>(
        &'a self,
        workspace_id: Uuid,
        document_id: Uuid,
        kind: SubjectKind,
        subject_id: Uuid,
    ) -> BoxFuture<'a, Result<PointSnapshot, GovernanceError>>;
    fn set_permission<'a>(
        &'a self,
        input: PermissionMutation,
    ) -> BoxFuture<'a, Result<PermissionGrant, GovernanceError>>;
    fn delete_permission<'a>(
        &'a self,
        input: PermissionMutation,
    ) -> BoxFuture<'a, Result<(), GovernanceError>>;
    fn effective_policy<'a>(
        &'a self,
        workspace_id: Uuid,
        document_id: Uuid,
    ) -> BoxFuture<'a, Result<PublishPolicy, GovernanceError>>;
    fn set_policy<'a>(
        &'a self,
        input: PolicyMutation,
    ) -> BoxFuture<'a, Result<PublishPolicy, GovernanceError>>;
}

pub trait PermissionCache: Send + Sync {
    fn get<'a>(&'a self, key: &'a str) -> BoxFuture<'a, Result<Option<CachedPoint>, ()>>;
    fn put<'a>(&'a self, key: &'a str, value: &'a CachedPoint) -> BoxFuture<'a, Result<(), ()>>;
}

#[derive(Clone)]
pub struct PermissionService {
    repository: Arc<dyn PermissionRepository>,
    cache: Arc<dyn PermissionCache>,
    clock: Arc<dyn Clock>,
}

impl PermissionService {
    pub fn new(
        repository: Arc<dyn PermissionRepository>,
        cache: Arc<dyn PermissionCache>,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            repository,
            cache,
            clock,
        }
    }

    pub async fn point(
        &self,
        user_id: Uuid,
        workspace_id: Uuid,
        document_id: Uuid,
    ) -> Result<EffectivePermission, GovernanceError> {
        let stamp = self
            .repository
            .access_stamp(user_id, workspace_id, document_id)
            .await?;
        let key = cache_key(workspace_id, user_id, document_id, stamp);
        if let Ok(Some(cached)) = self.cache.get(&key).await
            && valid_cached_point(&cached)
        {
            return Ok(cached.effective);
        }
        let snapshot = self
            .repository
            .point_snapshot(user_id, workspace_id, document_id)
            .await?;
        let (effective, _) =
            resolve_permission_path(&snapshot.nodes).map_err(|_| GovernanceError::Internal)?;
        let cached = CachedPoint {
            fingerprint: point_fingerprint(&snapshot, &effective)?,
            effective: effective.clone(),
        };
        let key = cache_key(
            snapshot.workspace_id,
            snapshot.user_id,
            snapshot.document_id,
            snapshot.stamp,
        );
        let _ = self.cache.put(&key, &cached).await;
        Ok(effective)
    }

    pub async fn scope(
        &self,
        user_id: Uuid,
        workspace_id: Uuid,
    ) -> Result<PermissionScope, GovernanceError> {
        let snapshot = self
            .repository
            .scope_snapshot(user_id, workspace_id)
            .await?;
        let resolved =
            compile_permission_scope(&snapshot.nodes).map_err(|_| GovernanceError::Internal)?;
        let accessible_document_ids = resolved
            .iter()
            .filter_map(|(id, permission)| permission.access.can_view().then_some(*id))
            .collect::<Vec<_>>();
        let fingerprint = scope_fingerprint(&snapshot, &resolved)?;
        Ok(PermissionScope {
            workspace_id,
            user_id,
            accessible_document_ids,
            fingerprint,
        })
    }

    pub async fn get_permissions(
        &self,
        actor_id: Uuid,
        workspace_id: Uuid,
        document_id: Uuid,
    ) -> Result<PermissionView, GovernanceError> {
        let snapshot = self
            .repository
            .point_snapshot(actor_id, workspace_id, document_id)
            .await?;
        let (effective, _) =
            resolve_permission_path(&snapshot.nodes).map_err(|_| GovernanceError::Internal)?;
        if !effective.manage {
            return Err(GovernanceError::DocumentNotFound);
        }
        let (explicit_grants, revision) = self
            .repository
            .permission_metadata(workspace_id, document_id)
            .await?;
        Ok(PermissionView {
            effective,
            explicit_grants,
            revision,
        })
    }

    pub async fn explain(
        &self,
        actor_id: Uuid,
        workspace_id: Uuid,
        document_id: Uuid,
        kind: SubjectKind,
        subject_id: Uuid,
    ) -> Result<PermissionExplanation, GovernanceError> {
        let actor = self
            .repository
            .point_snapshot(actor_id, workspace_id, document_id)
            .await?;
        let (actor_effective, _) =
            resolve_permission_path(&actor.nodes).map_err(|_| GovernanceError::Internal)?;
        let self_query = kind == SubjectKind::User && subject_id == actor_id;
        if !(actor_effective.manage || (self_query && actor_effective.access.can_view())) {
            return Err(GovernanceError::DocumentNotFound);
        }
        let subject = self
            .repository
            .subject_snapshot(workspace_id, document_id, kind, subject_id)
            .await?;
        let (effective, steps) =
            resolve_permission_path(&subject.nodes).map_err(|_| GovernanceError::Internal)?;
        let fingerprint = point_fingerprint(&subject, &effective)?;
        Ok(PermissionExplanation {
            effective,
            steps,
            fingerprint,
        })
    }

    pub async fn set_permission(
        &self,
        request: SetPermissionCommand,
    ) -> Result<PermissionGrant, GovernanceError> {
        request.input.validate().map_err(|error| match error {
            adoc_governance::GovernanceValidationError::PermissionManage => {
                GovernanceError::PermissionManageRequiresEditor
            }
            _ => GovernanceError::Validation,
        })?;
        let now = self.clock.now();
        self.repository
            .set_permission(PermissionMutation {
                workspace_id: request.workspace_id,
                document_id: request.document_id,
                grant_id: request.grant_id,
                expected_revision: request.expected_revision,
                command: command(
                    request.actor_id,
                    "setDocumentPermission",
                    &request.idempotency_key,
                    &(
                        request.document_id,
                        request.grant_id,
                        request.expected_revision,
                        &request.input,
                    ),
                    now,
                )?,
                input: Some(request.input),
            })
            .await
    }

    pub async fn delete_permission(
        &self,
        actor_id: Uuid,
        workspace_id: Uuid,
        document_id: Uuid,
        grant_id: Uuid,
        expected_revision: i64,
        idempotency_key: &str,
    ) -> Result<(), GovernanceError> {
        let now = self.clock.now();
        self.repository
            .delete_permission(PermissionMutation {
                workspace_id,
                document_id,
                grant_id,
                expected_revision,
                command: command(
                    actor_id,
                    "deleteDocumentPermission",
                    idempotency_key,
                    &(document_id, grant_id, expected_revision),
                    now,
                )?,
                input: None,
            })
            .await
    }

    pub async fn get_policy(
        &self,
        actor_id: Uuid,
        workspace_id: Uuid,
        document_id: Uuid,
    ) -> Result<PublishPolicy, GovernanceError> {
        let permission = self.point(actor_id, workspace_id, document_id).await?;
        if !permission.access.can_view() {
            return Err(GovernanceError::DocumentNotFound);
        }
        self.repository
            .effective_policy(workspace_id, document_id)
            .await
    }

    pub async fn set_policy(
        &self,
        actor_id: Uuid,
        workspace_id: Uuid,
        document_id: Uuid,
        expected_revision: i64,
        input: SetPublishPolicyInput,
        idempotency_key: &str,
    ) -> Result<PublishPolicy, GovernanceError> {
        let input = input
            .normalize()
            .map_err(|_| GovernanceError::PublishPolicyInvalid)?;
        let now = self.clock.now();
        self.repository
            .set_policy(PolicyMutation {
                workspace_id,
                document_id,
                expected_revision,
                command: command(
                    actor_id,
                    "setPublishPolicy",
                    idempotency_key,
                    &(document_id, expected_revision, &input),
                    now,
                )?,
                input,
            })
            .await
    }
}

fn command<T: Serialize>(
    actor_id: Uuid,
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
        actor_id,
        operation_id,
        idempotency_key: key.to_owned(),
        request_hash: hex::encode(Sha256::digest(body)),
        now,
        expires_at: now + chrono::Duration::hours(24),
    })
}

fn cache_key(workspace: Uuid, user: Uuid, document: Uuid, stamp: AccessStamp) -> String {
    format!(
        "adoc:permission:v1:{workspace}:{user}:{document}:{}:{}:{}",
        stamp.permission_revision, stamp.policy_revision, stamp.membership_revision
    )
}

fn valid_cached_point(cached: &CachedPoint) -> bool {
    let fingerprint_valid = cached.fingerprint.len() == 64
        && cached
            .fingerprint
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'));
    let effective = &cached.effective;
    let evidence_sorted = effective
        .evidence_grant_ids
        .windows(2)
        .all(|pair| pair[0] < pair[1]);
    let permission_valid = (!effective.manage || effective.access == Access::Editor)
        && effective.source_document_id.is_some() != effective.evidence_grant_ids.is_empty()
        && (effective.source_document_id.is_some()
            || (effective.access == Access::NoAccess && !effective.manage));
    fingerprint_valid && evidence_sorted && permission_valid
}

fn point_fingerprint(
    snapshot: &PointSnapshot,
    effective: &EffectivePermission,
) -> Result<String, GovernanceError> {
    digest(&(
        1_u8,
        snapshot.stamp,
        snapshot.user_id,
        snapshot.document_id,
        effective,
    ))
}

fn scope_fingerprint(
    snapshot: &ScopeSnapshot,
    resolved: &std::collections::BTreeMap<Uuid, EffectivePermission>,
) -> Result<String, GovernanceError> {
    digest(&(1_u8, snapshot.stamp, snapshot.user_id, resolved))
}

fn digest<T: Serialize>(value: &T) -> Result<String, GovernanceError> {
    let bytes = serde_json::to_vec(value).map_err(|_| GovernanceError::Internal)?;
    Ok(hex::encode(Sha256::digest(bytes)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_value_requires_canonical_fingerprint_and_permission() {
        let document = Uuid::now_v7();
        let grant = Uuid::now_v7();
        let valid = CachedPoint {
            effective: EffectivePermission {
                access: Access::Editor,
                manage: true,
                source_document_id: Some(document),
                evidence_grant_ids: vec![grant],
            },
            fingerprint: "a".repeat(64),
        };
        assert!(valid_cached_point(&valid));
        assert!(!valid_cached_point(&CachedPoint {
            fingerprint: "corrupt".to_owned(),
            ..valid.clone()
        }));
        assert!(!valid_cached_point(&CachedPoint {
            effective: EffectivePermission {
                access: Access::Viewer,
                manage: true,
                ..valid.effective
            },
            ..valid
        }));
    }
}
