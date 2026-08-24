use std::sync::Arc;

use adoc_document::{
    DocumentOperation, OperationBase, OperationPrecondition, OperationScope, ReferenceTarget,
};
pub use adoc_knowledge::*;
use adoc_ports::BoxFuture;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::{
    document::{ApplyOperationsInput, ApplyOperationsRequest, DocumentService},
    governance::{Command, GovernanceError},
    identity::{Clock, SecureRandom},
};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateReferenceInput {
    pub source_region: OperationScope,
    pub target: ReferenceTarget,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WriteVocabularyConceptInput {
    pub canonical_term: String,
    pub definition: String,
    pub terms: Vec<VocabularyTerm>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DeprecateVocabularyConceptInput {
    pub reason: String,
    #[serde(default)]
    pub replacement_concept_id: Option<Uuid>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NormalizedVocabularyInput {
    pub canonical_term: String,
    pub definition: String,
    pub terms: Vec<NormalizedVocabularyTerm>,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NormalizedVocabularyTerm {
    pub term: String,
    pub normalized_term: String,
    pub kind: VocabularyTermKind,
}

#[derive(Clone, Copy, Debug)]
pub enum VocabularyAction {
    Create,
    Update,
    Deprecate,
}
#[derive(Clone, Debug)]
pub struct VocabularyCommand {
    pub workspace_id: Uuid,
    pub concept_id: Uuid,
    pub expected_revision: Option<i64>,
    pub input: Option<NormalizedVocabularyInput>,
    pub replacement_concept_id: Option<Uuid>,
    pub reason: Option<String>,
    pub action: VocabularyAction,
    pub command: Command,
}

pub trait KnowledgeRepository: Send + Sync {
    fn get_reference<'a>(
        &'a self,
        actor: Uuid,
        workspace: Uuid,
        document: Uuid,
        id: Uuid,
    ) -> BoxFuture<'a, Result<Reference, GovernanceError>>;
    fn list_backlinks<'a>(
        &'a self,
        actor: Uuid,
        workspace: Uuid,
        target: Uuid,
        cursor: Option<String>,
    ) -> BoxFuture<'a, Result<ReferencePage, GovernanceError>>;
    fn list_vocabulary<'a>(
        &'a self,
        actor: Uuid,
        workspace: Uuid,
        cursor: Option<String>,
    ) -> BoxFuture<'a, Result<VocabularyPage, GovernanceError>>;
    fn get_vocabulary<'a>(
        &'a self,
        actor: Uuid,
        workspace: Uuid,
        id: Uuid,
    ) -> BoxFuture<'a, Result<VocabularyConcept, GovernanceError>>;
    fn mutate_vocabulary<'a>(
        &'a self,
        input: VocabularyCommand,
    ) -> BoxFuture<'a, Result<VocabularyConcept, GovernanceError>>;
}

pub struct KnowledgeService {
    repository: Arc<dyn KnowledgeRepository>,
    documents: Arc<DocumentService>,
    clock: Arc<dyn Clock>,
    random: Arc<dyn SecureRandom>,
}
impl KnowledgeService {
    pub fn new(
        repository: Arc<dyn KnowledgeRepository>,
        documents: Arc<DocumentService>,
        clock: Arc<dyn Clock>,
        random: Arc<dyn SecureRandom>,
    ) -> Self {
        Self {
            repository,
            documents,
            clock,
            random,
        }
    }
    #[allow(clippy::too_many_arguments)]
    pub async fn create_reference(
        &self,
        actor: Uuid,
        workspace: Uuid,
        document: Uuid,
        client: Uuid,
        revision: i64,
        token: &str,
        key: &str,
        input: CreateReferenceInput,
    ) -> Result<Reference, GovernanceError> {
        let target = normalize_target(input.target)?;
        let id = stable_uuid(&[
            "reference",
            &actor.to_string(),
            &workspace.to_string(),
            &document.to_string(),
            key,
        ]);
        let operation = DocumentOperation::AddReference {
            base: operation_base(
                stable_uuid(&[
                    "reference-operation",
                    &actor.to_string(),
                    &workspace.to_string(),
                    &document.to_string(),
                    key,
                ]),
                revision,
                input.source_region.clone(),
            ),
            reference_id: id,
            source_region: input.source_region,
            target,
        };
        self.documents
            .apply_operations(ApplyOperationsRequest {
                actor_id: actor,
                workspace_id: workspace,
                document_id: document,
                client_instance_id: client,
                expected_revision: revision,
                token,
                input: ApplyOperationsInput {
                    operations: vec![operation],
                },
                idempotency_key: key,
            })
            .await?;
        self.repository
            .get_reference(actor, workspace, document, id)
            .await
    }
    #[allow(clippy::too_many_arguments)]
    pub async fn delete_reference(
        &self,
        actor: Uuid,
        workspace: Uuid,
        document: Uuid,
        id: Uuid,
        client: Uuid,
        revision: i64,
        token: &str,
        key: &str,
    ) -> Result<(), GovernanceError> {
        let current = self
            .repository
            .get_reference(actor, workspace, document, id)
            .await?;
        let source_region: OperationScope =
            serde_json::from_value(current.source_region).map_err(|_| GovernanceError::Internal)?;
        let target: ReferenceTarget =
            serde_json::from_value(current.target).map_err(|_| GovernanceError::Internal)?;
        let operation = DocumentOperation::RemoveReference {
            base: operation_base(
                stable_uuid(&[
                    "reference-operation",
                    &actor.to_string(),
                    &workspace.to_string(),
                    &document.to_string(),
                    key,
                ]),
                revision,
                source_region.clone(),
            ),
            reference_id: id,
            source_region,
            target,
        };
        self.documents
            .apply_operations(ApplyOperationsRequest {
                actor_id: actor,
                workspace_id: workspace,
                document_id: document,
                client_instance_id: client,
                expected_revision: revision,
                token,
                input: ApplyOperationsInput {
                    operations: vec![operation],
                },
                idempotency_key: key,
            })
            .await?;
        Ok(())
    }
    pub async fn list_backlinks(
        &self,
        actor: Uuid,
        workspace: Uuid,
        target: Uuid,
        cursor: Option<String>,
    ) -> Result<ReferencePage, GovernanceError> {
        self.repository
            .list_backlinks(actor, workspace, target, cursor)
            .await
    }
    pub async fn list_vocabulary(
        &self,
        actor: Uuid,
        workspace: Uuid,
        cursor: Option<String>,
    ) -> Result<VocabularyPage, GovernanceError> {
        self.repository
            .list_vocabulary(actor, workspace, cursor)
            .await
    }
    pub async fn get_vocabulary(
        &self,
        actor: Uuid,
        workspace: Uuid,
        id: Uuid,
    ) -> Result<VocabularyConcept, GovernanceError> {
        self.repository.get_vocabulary(actor, workspace, id).await
    }
    pub async fn create_vocabulary(
        &self,
        actor: Uuid,
        workspace: Uuid,
        key: &str,
        input: WriteVocabularyConceptInput,
    ) -> Result<VocabularyConcept, GovernanceError> {
        let now = self.clock.now();
        let normalized = normalized_input(input)?;
        let command = command(actor, "createVocabularyConcept", key, &normalized, now)?;
        self.repository
            .mutate_vocabulary(VocabularyCommand {
                workspace_id: workspace,
                concept_id: self.uuid(now)?,
                expected_revision: None,
                input: Some(normalized),
                replacement_concept_id: None,
                reason: None,
                action: VocabularyAction::Create,
                command,
            })
            .await
    }
    pub async fn update_vocabulary(
        &self,
        actor: Uuid,
        workspace: Uuid,
        id: Uuid,
        revision: i64,
        key: &str,
        input: WriteVocabularyConceptInput,
    ) -> Result<VocabularyConcept, GovernanceError> {
        let now = self.clock.now();
        let normalized = normalized_input(input)?;
        let command = command(actor, "updateVocabularyConcept", key, &normalized, now)?;
        self.repository
            .mutate_vocabulary(VocabularyCommand {
                workspace_id: workspace,
                concept_id: id,
                expected_revision: Some(revision),
                input: Some(normalized),
                replacement_concept_id: None,
                reason: None,
                action: VocabularyAction::Update,
                command,
            })
            .await
    }
    pub async fn deprecate_vocabulary(
        &self,
        actor: Uuid,
        workspace: Uuid,
        id: Uuid,
        revision: i64,
        key: &str,
        input: DeprecateVocabularyConceptInput,
    ) -> Result<VocabularyConcept, GovernanceError> {
        let reason = input.reason.trim();
        if reason.is_empty() || reason.chars().count() > 1000 {
            return Err(GovernanceError::Validation);
        }
        let now = self.clock.now();
        let command = command(actor, "deprecateVocabularyConcept", key, &input, now)?;
        self.repository
            .mutate_vocabulary(VocabularyCommand {
                workspace_id: workspace,
                concept_id: id,
                expected_revision: Some(revision),
                input: None,
                replacement_concept_id: input.replacement_concept_id,
                reason: Some(reason.to_owned()),
                action: VocabularyAction::Deprecate,
                command,
            })
            .await
    }
    fn uuid(&self, now: DateTime<Utc>) -> Result<Uuid, GovernanceError> {
        self.random
            .uuid_v7(now)
            .map_err(|_| GovernanceError::Internal)
    }
}

fn operation_base(op_id: Uuid, revision: i64, scope: OperationScope) -> OperationBase {
    OperationBase {
        op_id,
        scope,
        precondition: OperationPrecondition {
            draft_revision: revision,
            target_hash: None,
        },
        depends_on: Vec::new(),
    }
}
fn stable_uuid(parts: &[&str]) -> Uuid {
    let digest = Sha256::digest(parts.join("\u{1f}"));
    let mut bytes = [0_u8; 16];
    bytes.copy_from_slice(&digest[..16]);
    bytes[6] = (bytes[6] & 0x0f) | 0x50;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    Uuid::from_bytes(bytes)
}
fn normalize_target(mut target: ReferenceTarget) -> Result<ReferenceTarget, GovernanceError> {
    let kind_valid = matches!(
        target.kind.as_str(),
        "DOCUMENT" | "REGION" | "DISCUSSION" | "VOCABULARY" | "EXTERNAL"
    );
    let region_valid = (target.kind == "REGION") == target.region.is_some();
    if target.id.trim().is_empty() || target.id.len() > 2048 || !kind_valid || !region_valid {
        return Err(GovernanceError::ReferenceTargetInvalid);
    }
    target.id = if target.kind == "EXTERNAL" {
        let url =
            url::Url::parse(&target.id).map_err(|_| GovernanceError::ReferenceTargetInvalid)?;
        if url.scheme() != "https"
            || !url.username().is_empty()
            || url.password().is_some()
            || url.fragment().is_some()
            || url.host_str().is_none()
        {
            return Err(GovernanceError::ReferenceTargetInvalid);
        }
        url.to_string()
    } else {
        Uuid::parse_str(&target.id)
            .map_err(|_| GovernanceError::ReferenceTargetInvalid)?
            .to_string()
    };
    Ok(target)
}
fn normalized_input(
    input: WriteVocabularyConceptInput,
) -> Result<NormalizedVocabularyInput, GovernanceError> {
    let definition = input.definition.trim();
    if !(1..=5000).contains(&definition.chars().count()) {
        return Err(GovernanceError::Validation);
    }
    let (canonical, terms) =
        normalize_terms(&input.canonical_term, input.terms).ok_or(GovernanceError::Validation)?;
    Ok(NormalizedVocabularyInput {
        canonical_term: canonical,
        definition: definition.to_owned(),
        terms: terms
            .into_iter()
            .map(|(term, normalized_term)| NormalizedVocabularyTerm {
                term: term.term,
                kind: term.kind,
                normalized_term,
            })
            .collect(),
    })
}
fn command<T: Serialize>(
    actor: Uuid,
    operation_id: &'static str,
    key: &str,
    input: &T,
    now: DateTime<Utc>,
) -> Result<Command, GovernanceError> {
    if !(8..=200).contains(&key.len()) {
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
