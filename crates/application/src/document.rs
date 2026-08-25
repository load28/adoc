use std::sync::Arc;

pub use adoc_document::{
    Document, DocumentOperation, DocumentStatus, Draft, OperationBase, OperationError,
    OperationErrorCode, OperationPrecondition, OperationScope, ReducerInput, ReferenceEffect,
    ReferenceSnapshot, ReferenceTarget, RegionResolutionStatus, TreeRank, ValidatedContent,
    apply_operations, canonical_hash, reanchor_region,
};
use adoc_ports::BoxFuture;
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::{
    governance::{Command, GovernanceError, ReasonInput},
    identity::{Clock, SecureRandom, TokenHash},
};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateDocumentInput {
    pub title: String,
    #[serde(default)]
    pub parent_id: Option<Uuid>,
    #[serde(default)]
    pub after_document_id: Option<Uuid>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UpdateDocumentMetadataInput {
    pub title: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MoveDocumentInput {
    pub new_parent_id: Option<Uuid>,
    pub after_document_id: Option<Uuid>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MoveDocumentCommitInput {
    pub new_parent_id: Option<Uuid>,
    pub after_document_id: Option<Uuid>,
    pub preview_token: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RestoreDocumentInput {
    pub parent_id: Option<Uuid>,
    pub after_document_id: Option<Uuid>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AcquireLeaseInput {
    pub client_instance_id: Uuid,
    #[serde(default)]
    pub force: bool,
    #[serde(default)]
    pub reason: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ApplyOperationsInput {
    pub operations: Vec<DocumentOperation>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentTreeNode {
    pub document: Document,
    pub children: Vec<DocumentTreeNode>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentTree {
    pub nodes: Vec<DocumentTreeNode>,
    pub watermark: i64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentPage {
    pub items: Vec<Document>,
    pub next_cursor: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentDetail {
    #[serde(flatten)]
    pub document: Document,
    pub draft: Option<Draft>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImpactPreview {
    pub preview_token: String,
    pub permission_changes: i64,
    pub policy_changes: i64,
    pub expires_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EditLeaseView {
    pub holder_user_id: Uuid,
    pub client_instance_id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub revision: i64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MutationResult {
    pub revision: i64,
    pub content_fingerprint: String,
    pub applied_operation_ids: Vec<Uuid>,
    pub inverse_operations: Vec<DocumentOperation>,
}

#[derive(Clone, Debug)]
pub struct NewDocument {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub title: String,
    pub parent_id: Option<Uuid>,
    pub after_document_id: Option<Uuid>,
    pub command: Command,
}

#[derive(Clone, Debug)]
pub struct DocumentChange {
    pub workspace_id: Uuid,
    pub document_id: Uuid,
    pub expected_revision: i64,
    pub title: Option<String>,
    pub reason: Option<String>,
    pub restore: Option<RestoreDocumentInput>,
    pub command: Command,
}

#[derive(Clone, Debug)]
pub struct MovePreviewRequest {
    pub workspace_id: Uuid,
    pub document_id: Uuid,
    pub actor_id: Uuid,
    pub expected_revision: i64,
    pub input: MoveDocumentInput,
    pub token_hash: TokenHash,
    pub expires_at: DateTime<Utc>,
}

#[derive(Clone, Debug)]
pub struct StoredImpactPreview {
    pub permission_changes: i64,
    pub policy_changes: i64,
    pub expires_at: DateTime<Utc>,
}

#[derive(Clone, Debug)]
pub struct MoveCommit {
    pub workspace_id: Uuid,
    pub document_id: Uuid,
    pub expected_revision: i64,
    pub input: MoveDocumentInput,
    pub preview_token_hash: TokenHash,
    pub command: Command,
}

#[derive(Clone, Debug)]
pub struct DraftCreate {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub document_id: Uuid,
    pub command: Command,
}

#[derive(Clone, Debug)]
pub struct LeaseAcquire {
    pub workspace_id: Uuid,
    pub document_id: Uuid,
    pub actor_id: Uuid,
    pub expected_document_revision: i64,
    pub input: AcquireLeaseInput,
    pub token_hash: TokenHash,
    pub command: Command,
}

#[derive(Clone, Debug)]
pub struct LeaseMutation {
    pub workspace_id: Uuid,
    pub document_id: Uuid,
    pub actor_id: Uuid,
    pub client_instance_id: Uuid,
    pub expected_lease_revision: i64,
    pub token_hash: TokenHash,
    pub release: bool,
    pub command: Command,
}

#[derive(Clone, Debug)]
pub struct DraftMutation {
    pub workspace_id: Uuid,
    pub document_id: Uuid,
    pub actor_id: Uuid,
    pub client_instance_id: Uuid,
    pub expected_draft_revision: i64,
    pub token_hash: TokenHash,
    pub operations: Vec<DocumentOperation>,
    pub command: Command,
}

pub struct LeaseCommandRequest<'a> {
    pub actor_id: Uuid,
    pub workspace_id: Uuid,
    pub document_id: Uuid,
    pub client_instance_id: Uuid,
    pub expected_revision: i64,
    pub token: &'a str,
    pub release: bool,
    pub idempotency_key: &'a str,
}

pub struct ApplyOperationsRequest<'a> {
    pub actor_id: Uuid,
    pub workspace_id: Uuid,
    pub document_id: Uuid,
    pub client_instance_id: Uuid,
    pub expected_revision: i64,
    pub token: &'a str,
    pub input: ApplyOperationsInput,
    pub idempotency_key: &'a str,
}

pub trait DocumentRepository: Send + Sync {
    fn tree<'a>(
        &'a self,
        actor: Uuid,
        workspace: Uuid,
    ) -> BoxFuture<'a, Result<DocumentTree, GovernanceError>>;
    fn trash<'a>(
        &'a self,
        actor: Uuid,
        workspace: Uuid,
        cursor: Option<String>,
    ) -> BoxFuture<'a, Result<DocumentPage, GovernanceError>>;
    fn detail<'a>(
        &'a self,
        actor: Uuid,
        workspace: Uuid,
        document: Uuid,
    ) -> BoxFuture<'a, Result<DocumentDetail, GovernanceError>>;
    fn create<'a>(&'a self, input: NewDocument)
    -> BoxFuture<'a, Result<Document, GovernanceError>>;
    fn change<'a>(
        &'a self,
        input: DocumentChange,
    ) -> BoxFuture<'a, Result<Document, GovernanceError>>;
    fn preview_move<'a>(
        &'a self,
        input: MovePreviewRequest,
    ) -> BoxFuture<'a, Result<StoredImpactPreview, GovernanceError>>;
    fn move_document<'a>(
        &'a self,
        input: MoveCommit,
    ) -> BoxFuture<'a, Result<Document, GovernanceError>>;
    fn draft<'a>(
        &'a self,
        actor: Uuid,
        workspace: Uuid,
        document: Uuid,
    ) -> BoxFuture<'a, Result<Draft, GovernanceError>>;
    fn create_draft<'a>(
        &'a self,
        input: DraftCreate,
    ) -> BoxFuture<'a, Result<Draft, GovernanceError>>;
    fn acquire_lease<'a>(
        &'a self,
        input: LeaseAcquire,
    ) -> BoxFuture<'a, Result<EditLeaseView, GovernanceError>>;
    fn mutate_lease<'a>(
        &'a self,
        input: LeaseMutation,
    ) -> BoxFuture<'a, Result<Option<EditLeaseView>, GovernanceError>>;
    fn apply_operations<'a>(
        &'a self,
        input: DraftMutation,
    ) -> BoxFuture<'a, Result<MutationResult, GovernanceError>>;
}

#[derive(Clone)]
pub struct DocumentService {
    repository: Arc<dyn DocumentRepository>,
    clock: Arc<dyn Clock>,
    random: Arc<dyn SecureRandom>,
}

impl DocumentService {
    pub fn new(
        repository: Arc<dyn DocumentRepository>,
        clock: Arc<dyn Clock>,
        random: Arc<dyn SecureRandom>,
    ) -> Self {
        Self {
            repository,
            clock,
            random,
        }
    }

    pub async fn tree(
        &self,
        actor: Uuid,
        workspace: Uuid,
    ) -> Result<DocumentTree, GovernanceError> {
        self.repository.tree(actor, workspace).await
    }
    pub async fn trash(
        &self,
        actor: Uuid,
        workspace: Uuid,
        cursor: Option<String>,
    ) -> Result<DocumentPage, GovernanceError> {
        self.repository.trash(actor, workspace, cursor).await
    }
    pub async fn detail(
        &self,
        actor: Uuid,
        workspace: Uuid,
        document: Uuid,
    ) -> Result<DocumentDetail, GovernanceError> {
        self.repository.detail(actor, workspace, document).await
    }
    pub async fn draft(
        &self,
        actor: Uuid,
        workspace: Uuid,
        document: Uuid,
    ) -> Result<Draft, GovernanceError> {
        self.repository.draft(actor, workspace, document).await
    }

    pub async fn create(
        &self,
        actor: Uuid,
        workspace: Uuid,
        input: CreateDocumentInput,
        key: &str,
    ) -> Result<Document, GovernanceError> {
        let title = adoc_document::DocumentTitle::parse(&input.title)
            .map_err(|_| GovernanceError::Validation)?;
        let now = self.clock.now();
        self.repository
            .create(NewDocument {
                id: self
                    .random
                    .uuid_v7(now)
                    .map_err(|_| GovernanceError::Internal)?,
                workspace_id: workspace,
                title: title.as_str().to_owned(),
                parent_id: input.parent_id,
                after_document_id: input.after_document_id,
                command: command(actor, "createDocument", key, &input, now)?,
            })
            .await
    }

    pub async fn rename(
        &self,
        actor: Uuid,
        workspace: Uuid,
        document: Uuid,
        revision: i64,
        input: UpdateDocumentMetadataInput,
        key: &str,
    ) -> Result<Document, GovernanceError> {
        let title = adoc_document::DocumentTitle::parse(&input.title)
            .map_err(|_| GovernanceError::Validation)?;
        let now = self.clock.now();
        self.repository
            .change(DocumentChange {
                workspace_id: workspace,
                document_id: document,
                expected_revision: revision,
                title: Some(title.as_str().to_owned()),
                reason: None,
                restore: None,
                command: command(
                    actor,
                    "updateDocumentMetadata",
                    key,
                    &(document, revision, input),
                    now,
                )?,
            })
            .await
    }

    pub async fn trash_document(
        &self,
        actor: Uuid,
        workspace: Uuid,
        document: Uuid,
        revision: i64,
        input: ReasonInput,
        key: &str,
    ) -> Result<Document, GovernanceError> {
        let reason = input.reason.trim();
        if reason.is_empty() || reason.chars().count() > 500 {
            return Err(GovernanceError::Validation);
        }
        let now = self.clock.now();
        self.repository
            .change(DocumentChange {
                workspace_id: workspace,
                document_id: document,
                expected_revision: revision,
                title: None,
                reason: Some(reason.to_owned()),
                restore: None,
                command: command(
                    actor,
                    "trashDocument",
                    key,
                    &(document, revision, input),
                    now,
                )?,
            })
            .await
    }

    pub async fn restore_document(
        &self,
        actor: Uuid,
        workspace: Uuid,
        document: Uuid,
        revision: i64,
        input: RestoreDocumentInput,
        key: &str,
    ) -> Result<Document, GovernanceError> {
        let now = self.clock.now();
        self.repository
            .change(DocumentChange {
                workspace_id: workspace,
                document_id: document,
                expected_revision: revision,
                title: None,
                reason: None,
                restore: Some(input.clone()),
                command: command(
                    actor,
                    "restoreDocument",
                    key,
                    &(document, revision, input),
                    now,
                )?,
            })
            .await
    }

    pub async fn preview_move(
        &self,
        actor: Uuid,
        workspace: Uuid,
        document: Uuid,
        revision: i64,
        input: MoveDocumentInput,
    ) -> Result<ImpactPreview, GovernanceError> {
        let (token, hash) = self.token()?;
        let expires_at = self.clock.now() + Duration::minutes(5);
        let stored = self
            .repository
            .preview_move(MovePreviewRequest {
                workspace_id: workspace,
                document_id: document,
                actor_id: actor,
                expected_revision: revision,
                input,
                token_hash: hash,
                expires_at,
            })
            .await?;
        Ok(ImpactPreview {
            preview_token: token,
            permission_changes: stored.permission_changes,
            policy_changes: stored.policy_changes,
            expires_at: stored.expires_at,
        })
    }

    pub async fn move_document(
        &self,
        actor: Uuid,
        workspace: Uuid,
        document: Uuid,
        revision: i64,
        input: MoveDocumentCommitInput,
        key: &str,
    ) -> Result<Document, GovernanceError> {
        let now = self.clock.now();
        let token_hash = token_hash(&input.preview_token).map_err(|error| match error {
            GovernanceError::EditLeaseInvalid => GovernanceError::MovePreviewStale,
            other => other,
        })?;
        let destination = MoveDocumentInput {
            new_parent_id: input.new_parent_id,
            after_document_id: input.after_document_id,
        };
        self.repository
            .move_document(MoveCommit {
                workspace_id: workspace,
                document_id: document,
                expected_revision: revision,
                input: destination,
                preview_token_hash: token_hash,
                command: command(
                    actor,
                    "moveDocument",
                    key,
                    &(document, revision, input),
                    now,
                )?,
            })
            .await
    }

    pub async fn create_draft(
        &self,
        actor: Uuid,
        workspace: Uuid,
        document: Uuid,
        key: &str,
    ) -> Result<Draft, GovernanceError> {
        let now = self.clock.now();
        self.repository
            .create_draft(DraftCreate {
                id: self
                    .random
                    .uuid_v7(now)
                    .map_err(|_| GovernanceError::Internal)?,
                workspace_id: workspace,
                document_id: document,
                command: command(actor, "createOrGetDraft", key, &document, now)?,
            })
            .await
    }

    pub async fn acquire_lease(
        &self,
        actor: Uuid,
        workspace: Uuid,
        document: Uuid,
        revision: i64,
        input: AcquireLeaseInput,
        key: &str,
    ) -> Result<EditLeaseView, GovernanceError> {
        let (token, token_hash) = self.token()?;
        let now = self.clock.now();
        let mut lease = self
            .repository
            .acquire_lease(LeaseAcquire {
                workspace_id: workspace,
                document_id: document,
                actor_id: actor,
                expected_document_revision: revision,
                input: input.clone(),
                token_hash,
                command: command(
                    actor,
                    "acquireEditLease",
                    key,
                    &(document, revision, input),
                    now,
                )?,
            })
            .await?;
        lease.token = Some(token);
        Ok(lease)
    }

    pub async fn mutate_lease(
        &self,
        request: LeaseCommandRequest<'_>,
    ) -> Result<Option<EditLeaseView>, GovernanceError> {
        let now = self.clock.now();
        self.repository
            .mutate_lease(LeaseMutation {
                workspace_id: request.workspace_id,
                document_id: request.document_id,
                actor_id: request.actor_id,
                client_instance_id: request.client_instance_id,
                expected_lease_revision: request.expected_revision,
                token_hash: token_hash(request.token)?,
                release: request.release,
                command: command(
                    request.actor_id,
                    if request.release {
                        "releaseEditLease"
                    } else {
                        "renewEditLease"
                    },
                    request.idempotency_key,
                    &(
                        request.document_id,
                        request.client_instance_id,
                        request.expected_revision,
                        request.release,
                    ),
                    now,
                )?,
            })
            .await
    }

    pub async fn apply_operations(
        &self,
        request: ApplyOperationsRequest<'_>,
    ) -> Result<MutationResult, GovernanceError> {
        let now = self.clock.now();
        self.repository
            .apply_operations(DraftMutation {
                workspace_id: request.workspace_id,
                document_id: request.document_id,
                actor_id: request.actor_id,
                client_instance_id: request.client_instance_id,
                expected_draft_revision: request.expected_revision,
                token_hash: token_hash(request.token)?,
                operations: request.input.operations.clone(),
                command: command(
                    request.actor_id,
                    "applyDraftOperations",
                    request.idempotency_key,
                    &(
                        request.document_id,
                        request.client_instance_id,
                        request.expected_revision,
                        request.input,
                    ),
                    now,
                )?,
            })
            .await
    }

    fn token(&self) -> Result<(String, TokenHash), GovernanceError> {
        let mut bytes = [0_u8; 32];
        self.random
            .bytes(&mut bytes)
            .map_err(|_| GovernanceError::Internal)?;
        let token = URL_SAFE_NO_PAD.encode(bytes);
        Ok((token.clone(), token_hash(&token)?))
    }
}

pub(crate) fn token_hash(token: &str) -> Result<TokenHash, GovernanceError> {
    if token.len() != 43 {
        return Err(GovernanceError::EditLeaseInvalid);
    }
    Ok(TokenHash(Sha256::digest(token.as_bytes()).into()))
}

pub(crate) fn command<T: Serialize>(
    actor_id: Uuid,
    operation_id: &'static str,
    key: &str,
    input: &T,
    now: DateTime<Utc>,
) -> Result<Command, GovernanceError> {
    if !(16..=128).contains(&key.len()) {
        return Err(GovernanceError::Validation);
    }
    let request = serde_json::to_vec(input).map_err(|_| GovernanceError::Internal)?;
    Ok(Command {
        actor_id,
        operation_id,
        idempotency_key: key.to_owned(),
        request_hash: hex::encode(Sha256::digest(request)),
        now,
        expires_at: now + Duration::hours(24),
    })
}
