use std::{collections::BTreeSet, sync::Arc};

pub use crate::ai_result::{
    AiClaim, AiConflict, AiFinding, AiResult, AiResultStatus, AiResultValidationError,
    FindingSeverity, ResultApplication, ResultValidationSummary, prohibited_term_in_content,
    validate_dependency_selection, validate_result,
};
use adoc_knowledge::SearchSource;
use adoc_ports::BoxFuture;
pub use adoc_writing_intelligence::{
    AiTarget, AiTask, AiTaskKind, ContextArtifact, ContextSource, ContextSourceKind, IncludeReason,
    MAX_OUTPUT_BYTES, RESULT_VALIDATOR_VERSION, RuntimeEvent, RuntimePhase, RuntimeRequest,
    RuntimeResult, RuntimeUsage, SourceAuthority, TASK_DEFINITION_VERSION,
    WRITING_RULE_BASELINE_VERSION, runtime_output_schema, task_definition,
};
use chrono::{DateTime, Duration, Utc};
use serde::Serialize;
use uuid::Uuid;

use crate::{
    document::{DocumentOperation, MutationResult, command, token_hash},
    governance::{Command, GovernanceError},
    identity::{Clock, TokenHash},
    jobs::{JobExecution, JobExecutionError},
    operations::{Job, JobSignal},
    search::{KnowledgeRetrievalService, SearchRetrievalError},
};

#[derive(Clone, Copy, Debug, serde::Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProposalStatus {
    Open,
    Applied,
    Rejected,
    Stale,
    Cancelled,
}

#[derive(Clone, Debug, serde::Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProposalView {
    pub proposal_id: Uuid,
    pub job_id: Uuid,
    pub document_id: Uuid,
    pub base_revision: i64,
    pub operations: Vec<DocumentOperation>,
    pub status: ProposalStatus,
    pub revision: i64,
    pub applied_revision: Option<i64>,
    pub applied_operation_ids: Vec<Uuid>,
    pub created_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, serde::Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ApplyProposalInput {
    #[serde(default)]
    pub operation_ids: Option<Vec<Uuid>>,
}

#[derive(Clone, Debug, serde::Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WritingRuleOverride {
    pub rule_id: String,
    pub enabled: bool,
    pub severity: FindingSeverity,
    pub values: Vec<String>,
}

#[derive(Clone, Debug, serde::Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WritingConfigurationInput {
    pub baseline_version: String,
    pub overrides: Vec<WritingRuleOverride>,
}

#[derive(Clone, Debug, serde::Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WritingConfigurationView {
    pub baseline_version: String,
    pub overrides: Vec<WritingRuleOverride>,
    pub revision: i64,
}

#[derive(Clone, Debug)]
pub struct ProposalApply {
    pub actor_id: Uuid,
    pub workspace_id: Uuid,
    pub proposal_id: Uuid,
    pub client_instance_id: Uuid,
    pub expected_draft_revision: i64,
    pub token_hash: TokenHash,
    pub operation_ids: Option<Vec<Uuid>>,
    pub command: Command,
}

#[derive(Clone, Debug)]
pub struct ProposalReject {
    pub actor_id: Uuid,
    pub workspace_id: Uuid,
    pub proposal_id: Uuid,
    pub expected_proposal_revision: i64,
    pub reason: String,
    pub command: Command,
}

#[derive(Clone, Debug)]
pub struct WritingConfigurationUpdate {
    pub actor_id: Uuid,
    pub workspace_id: Uuid,
    pub expected_revision: i64,
    pub input: WritingConfigurationInput,
    pub command: Command,
}

pub trait WritingIntelligenceRepository: Send + Sync {
    fn get_proposal<'a>(
        &'a self,
        actor_id: Uuid,
        workspace_id: Uuid,
        proposal_id: Uuid,
    ) -> BoxFuture<'a, Result<ProposalView, GovernanceError>>;

    fn apply_proposal<'a>(
        &'a self,
        input: ProposalApply,
    ) -> BoxFuture<'a, Result<MutationResult, GovernanceError>>;

    fn reject_proposal<'a>(
        &'a self,
        input: ProposalReject,
    ) -> BoxFuture<'a, Result<ProposalView, GovernanceError>>;

    fn get_writing_configuration<'a>(
        &'a self,
        actor_id: Uuid,
        workspace_id: Uuid,
    ) -> BoxFuture<'a, Result<WritingConfigurationView, GovernanceError>>;

    fn update_writing_configuration<'a>(
        &'a self,
        input: WritingConfigurationUpdate,
    ) -> BoxFuture<'a, Result<WritingConfigurationView, GovernanceError>>;
}

pub struct WritingIntelligenceService {
    repository: Arc<dyn WritingIntelligenceRepository>,
    clock: Arc<dyn Clock>,
}

pub struct ApplyProposalRequest<'a> {
    pub actor_id: Uuid,
    pub workspace_id: Uuid,
    pub proposal_id: Uuid,
    pub client_instance_id: Uuid,
    pub expected_revision: i64,
    pub token: &'a str,
    pub input: ApplyProposalInput,
    pub idempotency_key: &'a str,
}

impl WritingIntelligenceService {
    pub fn new(repository: Arc<dyn WritingIntelligenceRepository>, clock: Arc<dyn Clock>) -> Self {
        Self { repository, clock }
    }

    pub async fn get_proposal(
        &self,
        actor: Uuid,
        workspace: Uuid,
        proposal: Uuid,
    ) -> Result<ProposalView, GovernanceError> {
        self.repository
            .get_proposal(actor, workspace, proposal)
            .await
    }

    pub async fn apply_proposal(
        &self,
        request: ApplyProposalRequest<'_>,
    ) -> Result<MutationResult, GovernanceError> {
        let now = self.clock.now();
        self.repository
            .apply_proposal(ProposalApply {
                actor_id: request.actor_id,
                workspace_id: request.workspace_id,
                proposal_id: request.proposal_id,
                client_instance_id: request.client_instance_id,
                expected_draft_revision: request.expected_revision,
                token_hash: token_hash(request.token)?,
                operation_ids: request.input.operation_ids.clone(),
                command: command(
                    request.actor_id,
                    "applyProposal",
                    request.idempotency_key,
                    &(
                        request.proposal_id,
                        request.client_instance_id,
                        request.expected_revision,
                        request.input,
                    ),
                    now,
                )?,
            })
            .await
    }

    pub async fn reject_proposal(
        &self,
        actor: Uuid,
        workspace: Uuid,
        proposal: Uuid,
        revision: i64,
        reason: String,
        key: &str,
    ) -> Result<ProposalView, GovernanceError> {
        let reason = reason.trim().to_owned();
        if reason.is_empty() || reason.chars().count() > 500 {
            return Err(GovernanceError::Validation);
        }
        let now = self.clock.now();
        self.repository
            .reject_proposal(ProposalReject {
                actor_id: actor,
                workspace_id: workspace,
                proposal_id: proposal,
                expected_proposal_revision: revision,
                reason: reason.clone(),
                command: command(
                    actor,
                    "rejectProposal",
                    key,
                    &(proposal, revision, reason),
                    now,
                )?,
            })
            .await
    }

    pub async fn get_writing_configuration(
        &self,
        actor: Uuid,
        workspace: Uuid,
    ) -> Result<WritingConfigurationView, GovernanceError> {
        self.repository
            .get_writing_configuration(actor, workspace)
            .await
    }

    pub async fn update_writing_configuration(
        &self,
        actor: Uuid,
        workspace: Uuid,
        revision: i64,
        input: WritingConfigurationInput,
        key: &str,
    ) -> Result<WritingConfigurationView, GovernanceError> {
        if input.baseline_version != WRITING_RULE_BASELINE_VERSION || !input.overrides.is_empty() {
            return Err(GovernanceError::WritingConfigurationInvalid);
        }
        let now = self.clock.now();
        self.repository
            .update_writing_configuration(WritingConfigurationUpdate {
                actor_id: actor,
                workspace_id: workspace,
                expected_revision: revision,
                input: input.clone(),
                command: command(
                    actor,
                    "updateWritingConfiguration",
                    key,
                    &(workspace, revision, input),
                    now,
                )?,
            })
            .await
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeErrorKind {
    Cancelled,
    TimedOut,
    Transient,
    Permanent,
    Refused,
    OutputLimit,
    Contract,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RuntimeError {
    pub kind: RuntimeErrorKind,
    pub code: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProviderHealthStatus {
    Healthy,
    Degraded,
    Unavailable,
    Unconfigured,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderHealth {
    pub status: ProviderHealthStatus,
    pub code: Option<&'static str>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RuntimeCapabilities {
    pub structured_output: bool,
    pub embedding: bool,
}

pub trait Cancellation: Send + Sync {
    fn is_cancelled<'a>(&'a self) -> BoxFuture<'a, bool>;
}

pub trait RuntimeEventSink: Send + Sync {
    fn emit<'a>(&'a self, event: RuntimeEvent) -> BoxFuture<'a, Result<(), RuntimeError>>;
}

pub trait AiRuntime: Send + Sync {
    fn execute<'a>(
        &'a self,
        request: &'a RuntimeRequest,
        events: &'a dyn RuntimeEventSink,
        cancellation: &'a dyn Cancellation,
    ) -> BoxFuture<'a, Result<RuntimeResult, RuntimeError>>;

    fn health<'a>(&'a self) -> BoxFuture<'a, ProviderHealth>;

    fn capabilities(&self) -> RuntimeCapabilities;
}

#[derive(Clone, Debug, PartialEq)]
pub struct EmbeddingResult {
    pub vector: Vec<f32>,
    pub input_units: u64,
    pub provider_request_id: Option<String>,
}

pub trait EmbeddingRuntime: Send + Sync {
    fn embed<'a>(
        &'a self,
        text: &'a str,
        dimensions: usize,
        cancellation: &'a dyn Cancellation,
    ) -> BoxFuture<'a, Result<EmbeddingResult, RuntimeError>>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContextSelection {
    pub include_source_ids: BTreeSet<Uuid>,
    pub exclude_source_ids: BTreeSet<Uuid>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreparedContext {
    pub stamp: String,
    pub retrieval_query: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ContextOmission {
    SourceUnavailable,
    SourceExcluded,
    ContextBudget,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextSourcePreview {
    pub source_id: Uuid,
    pub kind: ContextSourceKind,
    pub stable_id: String,
    pub authority: SourceAuthority,
    pub include_reason: IncludeReason,
    pub snapshot_hash: String,
    pub included: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextPreview {
    pub artifact_fingerprint: String,
    pub expires_at: DateTime<Utc>,
    pub sources: Vec<ContextSourcePreview>,
    pub omissions: Vec<ContextOmission>,
    pub estimated_input_units: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContextError {
    Validation,
    NotFound,
    PermissionDenied,
    Stale,
    IdempotencyConflict,
    Limit,
    Quota,
    RetrievalUnavailable,
    Storage,
    StorageAt(&'static str),
}

pub trait AiContextRepository: Send + Sync {
    fn prepare<'a>(
        &'a self,
        task: &'a AiTask,
    ) -> BoxFuture<'a, Result<PreparedContext, ContextError>>;

    fn materialize<'a>(
        &'a self,
        task: &'a AiTask,
        prepared: &'a PreparedContext,
        retrieved: &'a [SearchSource],
    ) -> BoxFuture<'a, Result<ContextArtifact, ContextError>>;
}

pub trait AiKnowledgeRetrieval: Send + Sync {
    fn retrieve<'a>(
        &'a self,
        task: &'a AiTask,
        query: &'a str,
        vector: Option<Vec<f32>>,
        now: DateTime<Utc>,
    ) -> BoxFuture<'a, Result<Vec<SearchSource>, ContextError>>;
}

impl AiKnowledgeRetrieval for KnowledgeRetrievalService {
    fn retrieve<'a>(
        &'a self,
        task: &'a AiTask,
        query: &'a str,
        vector: Option<Vec<f32>>,
        now: DateTime<Utc>,
    ) -> BoxFuture<'a, Result<Vec<SearchSource>, ContextError>> {
        Box::pin(async move {
            self.search(
                task.actor_id,
                task.workspace_id,
                query,
                vector,
                true,
                30,
                None,
                now,
            )
            .await
            .map(|page| page.items.into_iter().map(|item| item.source).collect())
            .map_err(map_retrieval_error)
        })
    }
}

fn map_retrieval_error(error: SearchRetrievalError) -> ContextError {
    match error {
        SearchRetrievalError::Validation | SearchRetrievalError::CursorExpired => {
            ContextError::Validation
        }
        SearchRetrievalError::WorkspaceNotFound => ContextError::NotFound,
        SearchRetrievalError::Unavailable => ContextError::RetrievalUnavailable,
        SearchRetrievalError::Internal => ContextError::Storage,
    }
}

pub struct AiContextService {
    repository: Arc<dyn AiContextRepository>,
    retrieval: Arc<dyn AiKnowledgeRetrieval>,
    embedding: Option<Arc<dyn EmbeddingRuntime>>,
    embedding_dimensions: usize,
    maximum_input_units: u64,
}

#[derive(Clone, Copy, Debug, serde::Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AiJobStatus {
    Queued,
    Running,
    CancelRequested,
    Succeeded,
    Failed,
    Cancelled,
    TimedOut,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiJobView {
    pub id: Uuid,
    pub kind: AiTaskKind,
    pub status: AiJobStatus,
    pub sequence: i64,
    pub revision: i64,
    pub result: Option<serde_json::Value>,
    pub proposal_id: Option<Uuid>,
    pub error_code: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiJobPage {
    pub items: Vec<AiJobView>,
    pub next_cursor: Option<String>,
}

pub struct AiAdmission {
    pub view: AiJobView,
    pub signal: JobSignal,
}

pub enum AiExecutionStart {
    Execute(RuntimeRequest),
    Completed,
    Cancelled,
}

pub trait AiJobRepository: Send + Sync {
    fn admit<'a>(
        &'a self,
        task: &'a AiTask,
        artifact: &'a ContextArtifact,
        fingerprint: &'a str,
        request_key: &'a str,
        now: DateTime<Utc>,
    ) -> BoxFuture<'a, Result<AiAdmission, ContextError>>;

    fn list<'a>(
        &'a self,
        actor_id: Uuid,
        workspace_id: Uuid,
        cursor: Option<&'a str>,
    ) -> BoxFuture<'a, Result<AiJobPage, ContextError>>;

    fn get<'a>(
        &'a self,
        actor_id: Uuid,
        workspace_id: Uuid,
        job_id: Uuid,
    ) -> BoxFuture<'a, Result<AiJobView, ContextError>>;

    fn cancel<'a>(
        &'a self,
        actor_id: Uuid,
        workspace_id: Uuid,
        job_id: Uuid,
        expected_revision: i64,
        request_key: &'a str,
        now: DateTime<Utc>,
    ) -> BoxFuture<'a, Result<(), ContextError>>;

    fn start<'a>(
        &'a self,
        job: &'a Job,
        worker: &'a str,
        now: DateTime<Utc>,
        timeout_millis: u64,
    ) -> BoxFuture<'a, Result<AiExecutionStart, JobExecutionError>>;

    fn is_cancelled<'a>(&'a self, generic_job_id: Uuid) -> BoxFuture<'a, bool>;

    fn finish_success<'a>(
        &'a self,
        job: &'a Job,
        worker: &'a str,
        result: &'a RuntimeResult,
        now: DateTime<Utc>,
    ) -> BoxFuture<'a, Result<JobExecution, JobExecutionError>>;

    fn finish_terminal<'a>(
        &'a self,
        job: &'a Job,
        worker: &'a str,
        status: AiJobStatus,
        code: &'a str,
        now: DateTime<Utc>,
    ) -> BoxFuture<'a, Result<JobExecution, JobExecutionError>>;
}

pub struct AiJobService {
    context: Arc<AiContextService>,
    repository: Arc<dyn AiJobRepository>,
}

impl AiJobService {
    pub fn new(context: Arc<AiContextService>, repository: Arc<dyn AiJobRepository>) -> Self {
        Self {
            context,
            repository,
        }
    }

    pub async fn create(
        &self,
        task: &AiTask,
        selection: &ContextSelection,
        expected_fingerprint: &str,
        request_key: &str,
        now: DateTime<Utc>,
        cancellation: &dyn Cancellation,
    ) -> Result<AiAdmission, ContextError> {
        if expected_fingerprint.len() != 64 || !(8..=200).contains(&request_key.len()) {
            return Err(ContextError::Validation);
        }
        let (preview, artifact) = self
            .context
            .preview(task, selection, now, cancellation)
            .await?;
        if preview.artifact_fingerprint != expected_fingerprint {
            return Err(ContextError::Stale);
        }
        self.repository
            .admit(task, &artifact, expected_fingerprint, request_key, now)
            .await
    }

    pub async fn list(
        &self,
        actor_id: Uuid,
        workspace_id: Uuid,
        cursor: Option<&str>,
    ) -> Result<AiJobPage, ContextError> {
        self.repository.list(actor_id, workspace_id, cursor).await
    }

    pub async fn get(
        &self,
        actor_id: Uuid,
        workspace_id: Uuid,
        job_id: Uuid,
    ) -> Result<AiJobView, ContextError> {
        self.repository.get(actor_id, workspace_id, job_id).await
    }

    pub async fn cancel(
        &self,
        actor_id: Uuid,
        workspace_id: Uuid,
        job_id: Uuid,
        expected_revision: i64,
        request_key: &str,
        now: DateTime<Utc>,
    ) -> Result<(), ContextError> {
        self.repository
            .cancel(
                actor_id,
                workspace_id,
                job_id,
                expected_revision,
                request_key,
                now,
            )
            .await
    }
}

pub struct AiJobExecutionService {
    repository: Arc<dyn AiJobRepository>,
    runtime: Arc<dyn AiRuntime>,
    timeout_millis: u64,
}

impl AiJobExecutionService {
    pub fn new(
        repository: Arc<dyn AiJobRepository>,
        runtime: Arc<dyn AiRuntime>,
        timeout_millis: u64,
    ) -> Result<Self, ContextError> {
        if timeout_millis == 0 {
            return Err(ContextError::Validation);
        }
        Ok(Self {
            repository,
            runtime,
            timeout_millis,
        })
    }

    pub async fn execute(
        &self,
        job: &Job,
        worker: &str,
        now: DateTime<Utc>,
    ) -> Result<JobExecution, JobExecutionError> {
        let request = match self
            .repository
            .start(job, worker, now, self.timeout_millis)
            .await?
        {
            AiExecutionStart::Execute(request) => request,
            AiExecutionStart::Completed => return Ok(JobExecution::Delivered(None)),
            AiExecutionStart::Cancelled => return Ok(JobExecution::Cancelled),
        };
        let cancellation = RepositoryCancellation {
            repository: self.repository.clone(),
            job_id: job.id,
        };
        let result = self
            .runtime
            .execute(&request, &IgnoreRuntimeEvents, &cancellation)
            .await;
        match result {
            Ok(result) => {
                if cancellation.is_cancelled().await {
                    self.repository
                        .finish_terminal(job, worker, AiJobStatus::Cancelled, "AI_CANCELLED", now)
                        .await
                } else {
                    self.repository
                        .finish_success(job, worker, &result, now)
                        .await
                }
            }
            Err(error) if error.kind == RuntimeErrorKind::Transient => {
                Err(JobExecutionError::Transient(error.code))
            }
            Err(error) => {
                let status = match error.kind {
                    RuntimeErrorKind::Cancelled => AiJobStatus::Cancelled,
                    RuntimeErrorKind::TimedOut => AiJobStatus::TimedOut,
                    _ => AiJobStatus::Failed,
                };
                self.repository
                    .finish_terminal(job, worker, status, error.code, now)
                    .await
            }
        }
    }
}

struct RepositoryCancellation {
    repository: Arc<dyn AiJobRepository>,
    job_id: Uuid,
}

impl Cancellation for RepositoryCancellation {
    fn is_cancelled<'a>(&'a self) -> BoxFuture<'a, bool> {
        self.repository.is_cancelled(self.job_id)
    }
}

impl AiContextService {
    pub fn new(
        repository: Arc<dyn AiContextRepository>,
        retrieval: Arc<dyn AiKnowledgeRetrieval>,
        embedding: Option<Arc<dyn EmbeddingRuntime>>,
        embedding_dimensions: usize,
        maximum_input_units: u64,
    ) -> Result<Self, ContextError> {
        if embedding_dimensions == 0 || maximum_input_units == 0 {
            return Err(ContextError::Validation);
        }
        Ok(Self {
            repository,
            retrieval,
            embedding,
            embedding_dimensions,
            maximum_input_units,
        })
    }

    pub async fn preview(
        &self,
        task: &AiTask,
        selection: &ContextSelection,
        now: DateTime<Utc>,
        cancellation: &dyn Cancellation,
    ) -> Result<(ContextPreview, ContextArtifact), ContextError> {
        if !task.is_valid()
            || !selection
                .include_source_ids
                .is_disjoint(&selection.exclude_source_ids)
        {
            return Err(ContextError::Validation);
        }
        let mut last_error = ContextError::Stale;
        for _ in 0..2 {
            let prepared = self.repository.prepare(task).await?;
            let vector = if let Some(embedding) = &self.embedding {
                Some(
                    embedding
                        .embed(
                            &prepared.retrieval_query,
                            self.embedding_dimensions,
                            cancellation,
                        )
                        .await
                        .map_err(|_| ContextError::RetrievalUnavailable)?
                        .vector,
                )
            } else {
                None
            };
            let retrieved = self
                .retrieval
                .retrieve(task, &prepared.retrieval_query, vector, now)
                .await?;
            match self
                .repository
                .materialize(task, &prepared, &retrieved)
                .await
            {
                Ok(mut artifact) => {
                    let mut omissions = apply_selection(&mut artifact.sources, selection)?;
                    let mut preview_sources = artifact
                        .sources
                        .iter()
                        .map(source_preview)
                        .collect::<Vec<_>>();
                    artifact.sources.retain(|source| source.included);
                    let fingerprint = loop {
                        if let Some(fingerprint) =
                            artifact.normalize_and_fingerprint(self.maximum_input_units)
                        {
                            break fingerprint;
                        }
                        let Some(index) = artifact.sources.iter().rposition(|source| {
                            !matches!(
                                source.include_reason,
                                IncludeReason::CurrentTarget | IncludeReason::UserProvided
                            )
                        }) else {
                            return Err(ContextError::Limit);
                        };
                        let removed = artifact.sources.remove(index);
                        if let Some(preview) = preview_sources
                            .iter_mut()
                            .find(|source| source.source_id == removed.source_id)
                        {
                            preview.included = false;
                        }
                        omissions.push(ContextOmission::ContextBudget);
                    };
                    let preview = ContextPreview {
                        artifact_fingerprint: fingerprint,
                        expires_at: now + Duration::minutes(5),
                        sources: preview_sources,
                        omissions,
                        estimated_input_units: artifact.estimated_input_units,
                    };
                    return Ok((preview, artifact));
                }
                Err(ContextError::Stale) => last_error = ContextError::Stale,
                Err(error) => return Err(error),
            }
        }
        Err(last_error)
    }
}

fn apply_selection(
    sources: &mut [ContextSource],
    selection: &ContextSelection,
) -> Result<Vec<ContextOmission>, ContextError> {
    let available = sources
        .iter()
        .map(|source| source.source_id)
        .collect::<BTreeSet<_>>();
    if !selection.include_source_ids.is_subset(&available)
        || !selection.exclude_source_ids.is_subset(&available)
    {
        return Err(ContextError::Validation);
    }
    let mut omissions = Vec::new();
    for source in sources {
        if selection.exclude_source_ids.contains(&source.source_id) {
            if matches!(
                source.include_reason,
                IncludeReason::CurrentTarget | IncludeReason::UserProvided
            ) {
                return Err(ContextError::Validation);
            }
            source.included = false;
            omissions.push(ContextOmission::SourceExcluded);
        } else if selection.include_source_ids.contains(&source.source_id) {
            source.included = true;
        }
    }
    Ok(omissions)
}

fn source_preview(source: &ContextSource) -> ContextSourcePreview {
    ContextSourcePreview {
        source_id: source.source_id,
        kind: source.kind,
        stable_id: source.stable_id.clone(),
        authority: source.authority,
        include_reason: source.include_reason,
        snapshot_hash: source.snapshot_hash.clone(),
        included: source.included,
    }
}

#[derive(Default)]
pub struct NeverCancelled;

impl Cancellation for NeverCancelled {
    fn is_cancelled<'a>(&'a self) -> BoxFuture<'a, bool> {
        Box::pin(async { false })
    }
}

#[derive(Default)]
pub struct IgnoreRuntimeEvents;

impl RuntimeEventSink for IgnoreRuntimeEvents {
    fn emit<'a>(&'a self, _event: RuntimeEvent) -> BoxFuture<'a, Result<(), RuntimeError>> {
        Box::pin(async { Ok(()) })
    }
}
