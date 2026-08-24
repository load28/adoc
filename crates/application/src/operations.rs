use std::{pin::Pin, sync::Arc};

pub use adoc_operations::*;
use adoc_ports::BoxFuture;
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use bytes::Bytes;
use chrono::{DateTime, Duration, Utc};
use futures_util::{Stream, StreamExt};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::{
    governance::{Command, GovernanceError},
    identity::{Clock, KeyRing, TokenHash},
};

#[derive(Clone, Debug)]
pub struct AuditEventInput {
    pub workspace_id: Uuid,
    pub actor: AuditActor,
    pub action: AuditAction,
    pub target: AuditTarget,
    pub before: Option<AuditFields>,
    pub after: Option<AuditFields>,
    pub metadata: AuditFields,
    pub occurred_at: DateTime<Utc>,
    pub correlation_id: String,
}

impl AuditEventInput {
    #[must_use]
    pub fn user(
        workspace_id: Uuid,
        actor_id: Uuid,
        action: AuditAction,
        target: AuditTarget,
        occurred_at: DateTime<Utc>,
        correlation_id: impl Into<String>,
    ) -> Self {
        Self {
            workspace_id,
            actor: AuditActor::user(actor_id),
            action,
            target,
            before: None,
            after: None,
            metadata: AuditFields::new(),
            occurred_at,
            correlation_id: correlation_id.into(),
        }
    }

    #[must_use]
    pub fn system(
        workspace_id: Uuid,
        action: AuditAction,
        target: AuditTarget,
        occurred_at: DateTime<Utc>,
        correlation_id: impl Into<String>,
    ) -> Self {
        let mut value = Self::user(
            workspace_id,
            Uuid::nil(),
            action,
            target,
            occurred_at,
            correlation_id,
        );
        value.actor = AuditActor::system();
        value
    }

    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.actor.is_valid()
            && (8..=128).contains(&self.correlation_id.len())
            && fields_valid(self.before.as_ref())
            && fields_valid(self.after.as_ref())
            && fields_valid(Some(&self.metadata))
    }
}

fn fields_valid(fields: Option<&AuditFields>) -> bool {
    fields.is_none_or(|values| {
        values.len() <= 32
            && values.iter().all(|(key, value)| {
                !key.is_empty()
                    && key.len() <= 64
                    && !matches!(
                        key.as_str(),
                        "title"
                            | "content"
                            | "email"
                            | "filename"
                            | "fileName"
                            | "token"
                            | "checksum"
                    )
                    && match value {
                        AuditValue::String(value) => value.len() <= 500,
                        AuditValue::Integer(_) | AuditValue::Boolean(_) | AuditValue::Null => true,
                    }
            })
    })
}

pub trait AuditRepository: Send + Sync {
    fn list<'a>(
        &'a self,
        actor: Uuid,
        workspace: Uuid,
        cursor: Option<String>,
    ) -> BoxFuture<'a, Result<AuditPage, GovernanceError>>;
}

pub struct AuditService {
    repository: Arc<dyn AuditRepository>,
}

impl AuditService {
    pub fn new(repository: Arc<dyn AuditRepository>) -> Self {
        Self { repository }
    }

    pub async fn list(
        &self,
        actor: Uuid,
        workspace: Uuid,
        cursor: Option<String>,
    ) -> Result<AuditPage, GovernanceError> {
        self.repository.list(actor, workspace, cursor).await
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PurgeJobReference {
    pub job_id: Uuid,
    pub status: String,
}

#[derive(Clone, Debug)]
pub struct DocumentPurgeCommand {
    pub workspace_id: Uuid,
    pub document_id: Uuid,
    pub expected_revision: i64,
    pub reason: String,
    pub command: Command,
}

#[derive(Clone, Debug)]
pub enum PurgeAdvance {
    Continue(PurgeRun),
    DeleteObjects(Vec<PurgeObject>),
    Completed,
}

pub trait RetentionRepository: Send + Sync {
    fn request_document<'a>(
        &'a self,
        input: DocumentPurgeCommand,
    ) -> BoxFuture<'a, Result<PurgeJobReference, GovernanceError>>;
    fn claim_due<'a>(
        &'a self,
        now: DateTime<Utc>,
        worker: &'a str,
        limit: i64,
    ) -> BoxFuture<'a, Result<Vec<PurgeRun>, GovernanceError>>;
    fn advance<'a>(
        &'a self,
        run: &'a PurgeRun,
        worker: &'a str,
        now: DateTime<Utc>,
    ) -> BoxFuture<'a, Result<PurgeAdvance, GovernanceError>>;
    fn finish_object<'a>(
        &'a self,
        object: &'a PurgeObject,
        success: bool,
        error_code: Option<&'a str>,
        now: DateTime<Utc>,
    ) -> BoxFuture<'a, Result<(), GovernanceError>>;
    fn fail<'a>(
        &'a self,
        run_id: Uuid,
        worker: &'a str,
        error_code: &'a str,
        now: DateTime<Utc>,
    ) -> BoxFuture<'a, Result<(), GovernanceError>>;
}

pub struct RetentionService {
    repository: Arc<dyn RetentionRepository>,
    storage: Arc<dyn ObjectStorage>,
    clock: Arc<dyn Clock>,
    worker_id: Arc<str>,
}

impl RetentionService {
    pub fn new(
        repository: Arc<dyn RetentionRepository>,
        storage: Arc<dyn ObjectStorage>,
        clock: Arc<dyn Clock>,
        worker_id: Arc<str>,
    ) -> Self {
        Self {
            repository,
            storage,
            clock,
            worker_id,
        }
    }

    pub async fn request_document(
        &self,
        input: DocumentPurgeCommand,
    ) -> Result<PurgeJobReference, GovernanceError> {
        if input.reason.trim().is_empty() || input.reason.len() > 1000 {
            return Err(GovernanceError::Validation);
        }
        self.repository.request_document(input).await
    }

    pub async fn request_document_purge(
        &self,
        actor: Uuid,
        workspace: Uuid,
        document: Uuid,
        expected_revision: i64,
        reason: String,
        idempotency_key: &str,
    ) -> Result<PurgeJobReference, GovernanceError> {
        if !(16..=128).contains(&idempotency_key.len()) {
            return Err(GovernanceError::Validation);
        }
        let now = self.clock.now();
        let mut digest = Sha256::new();
        digest.update(workspace.as_bytes());
        digest.update(document.as_bytes());
        digest.update(expected_revision.to_be_bytes());
        digest.update(reason.as_bytes());
        let command = Command {
            actor_id: actor,
            operation_id: "purgeDocument",
            idempotency_key: idempotency_key.to_owned(),
            request_hash: hex::encode(digest.finalize()),
            now,
            expires_at: now + Duration::hours(24),
        };
        self.request_document(DocumentPurgeCommand {
            workspace_id: workspace,
            document_id: document,
            expected_revision,
            reason,
            command,
        })
        .await
    }

    pub async fn run_once(&self, limit: i64) -> Result<usize, GovernanceError> {
        let now = self.clock.now();
        let runs = self
            .repository
            .claim_due(now, &self.worker_id, limit)
            .await?;
        let mut completed = 0;
        for mut run in runs {
            let result = async {
                for _ in 0..8 {
                    match self.repository.advance(&run, &self.worker_id, now).await? {
                        PurgeAdvance::Continue(next) => run = next,
                        PurgeAdvance::DeleteObjects(objects) => {
                            for object in objects {
                                let result = self.storage.delete(&object.storage_key).await;
                                self.repository
                                    .finish_object(
                                        &object,
                                        result.is_ok(),
                                        result.as_ref().err().map(|_| "OBJECT_DELETE_FAILED"),
                                        now,
                                    )
                                    .await?;
                                if result.is_err() {
                                    return Err(GovernanceError::DependencyUnavailable);
                                }
                            }
                        }
                        PurgeAdvance::Completed => return Ok(()),
                    }
                }
                Err(GovernanceError::Internal)
            }
            .await;
            match result {
                Ok(()) => completed += 1,
                Err(error) => {
                    self.repository
                        .fail(run.id, &self.worker_id, purge_error_code(&error), now)
                        .await?;
                }
            }
        }
        Ok(completed)
    }
}

fn purge_error_code(error: &GovernanceError) -> &'static str {
    match error {
        GovernanceError::DependencyUnavailable => "DEPENDENCY_UNAVAILABLE",
        GovernanceError::DocumentStateInvalid => "DOCUMENT_STATE_INVALID",
        GovernanceError::WorkspaceStateInvalid => "WORKSPACE_STATE_INVALID",
        _ => "PURGE_STEP_FAILED",
    }
}

pub type ByteStream = Pin<Box<dyn Stream<Item = Result<Bytes, StorageError>> + Send>>;
#[derive(Clone, Debug)]
pub struct ObjectMetadata {
    pub size: u64,
}
#[derive(Clone, Debug, thiserror::Error)]
pub enum StorageError {
    #[error("object not found")]
    NotFound,
    #[error("object already exists")]
    AlreadyExists,
    #[error("object storage unavailable")]
    Unavailable,
}
pub trait ObjectStorage: Send + Sync {
    fn write<'a>(
        &'a self,
        key: &'a str,
        stream: ByteStream,
        max_bytes: u64,
    ) -> BoxFuture<'a, Result<ObjectMetadata, StorageError>>;
    fn stat<'a>(&'a self, key: &'a str) -> BoxFuture<'a, Result<ObjectMetadata, StorageError>>;
    fn read<'a>(
        &'a self,
        key: &'a str,
        range: Option<ByteRange>,
    ) -> BoxFuture<'a, Result<ByteStream, StorageError>>;
    fn delete<'a>(&'a self, key: &'a str) -> BoxFuture<'a, Result<(), StorageError>>;
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScanVerdict {
    Clean,
    Malware,
}
pub trait MalwareScanner: Send + Sync {
    fn scan<'a>(&'a self, stream: ByteStream) -> BoxFuture<'a, Result<ScanVerdict, StorageError>>;
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateFileUploadInput {
    pub name: String,
    pub mime_type: String,
    pub size: u64,
    pub checksum: String,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CompleteFileUploadInput {
    pub checksum_sha256: String,
    pub size_bytes: u64,
}
#[derive(Clone, Debug)]
pub struct CreateFileCommand {
    pub workspace_id: Uuid,
    pub asset_id: Uuid,
    pub storage_key: String,
    pub original_name: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub checksum_sha256: String,
    pub token_hash: TokenHash,
    pub token_key_id: String,
    pub expires_at: DateTime<Utc>,
    pub command: Command,
}
#[derive(Clone, Debug)]
pub struct UploadAuthorization {
    pub storage_key: String,
    pub expected_size: u64,
    pub uploaded: bool,
}
#[derive(Clone, Debug)]
pub struct FileAccess {
    pub asset: FileAsset,
    pub storage_key: String,
}
#[derive(Clone, Debug)]
pub struct FileMutation {
    pub actor_id: Uuid,
    pub workspace_id: Uuid,
    pub asset_id: Uuid,
    pub expected_revision: i64,
    pub success: bool,
    pub failure_code: Option<String>,
    pub detected_mime: Option<String>,
    pub command: Command,
}
#[derive(Clone, Debug)]
pub struct GcCandidate {
    pub asset_id: Uuid,
    pub storage_key: String,
}

pub trait FileRepository: Send + Sync {
    fn upload_key_id<'a>(
        &'a self,
        actor: Uuid,
        workspace: Uuid,
        asset: Uuid,
    ) -> BoxFuture<'a, Result<Option<String>, GovernanceError>>;
    fn create<'a>(
        &'a self,
        input: CreateFileCommand,
    ) -> BoxFuture<'a, Result<FileAsset, GovernanceError>>;
    fn authorize_upload<'a>(
        &'a self,
        actor: Uuid,
        workspace: Uuid,
        asset: Uuid,
        token_hash: TokenHash,
        now: DateTime<Utc>,
    ) -> BoxFuture<'a, Result<UploadAuthorization, GovernanceError>>;
    fn mark_uploaded<'a>(
        &'a self,
        actor: Uuid,
        workspace: Uuid,
        asset: Uuid,
        now: DateTime<Utc>,
    ) -> BoxFuture<'a, Result<(), GovernanceError>>;
    fn begin_validation<'a>(
        &'a self,
        workspace: Uuid,
        asset: Uuid,
        expected_revision: i64,
        command: &'a Command,
    ) -> BoxFuture<'a, Result<FileAccess, GovernanceError>>;
    fn access<'a>(
        &'a self,
        actor: Uuid,
        workspace: Uuid,
        asset: Uuid,
    ) -> BoxFuture<'a, Result<FileAccess, GovernanceError>>;
    fn public_access<'a>(
        &'a self,
        token_hash: TokenHash,
        asset: Uuid,
        now: DateTime<Utc>,
    ) -> BoxFuture<'a, Result<FileAccess, GovernanceError>>;
    fn mutate<'a>(
        &'a self,
        input: FileMutation,
    ) -> BoxFuture<'a, Result<FileAsset, GovernanceError>>;
    fn delete<'a>(
        &'a self,
        actor: Uuid,
        workspace: Uuid,
        asset: Uuid,
        revision: i64,
        key: &'a str,
        now: DateTime<Utc>,
    ) -> BoxFuture<'a, Result<(), GovernanceError>>;
    fn claim_gc<'a>(
        &'a self,
        now: DateTime<Utc>,
        limit: i64,
    ) -> BoxFuture<'a, Result<Vec<GcCandidate>, GovernanceError>>;
    fn finish_gc<'a>(
        &'a self,
        asset: Uuid,
        success: bool,
        now: DateTime<Utc>,
    ) -> BoxFuture<'a, Result<(), GovernanceError>>;
}

pub struct FileGarbageCollector {
    repository: Arc<dyn FileRepository>,
    storage: Arc<dyn ObjectStorage>,
    clock: Arc<dyn Clock>,
}
impl FileGarbageCollector {
    pub fn new(
        repository: Arc<dyn FileRepository>,
        storage: Arc<dyn ObjectStorage>,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            repository,
            storage,
            clock,
        }
    }
    pub async fn run_once(&self, limit: i64) -> Result<usize, GovernanceError> {
        let now = self.clock.now();
        let candidates = self.repository.claim_gc(now, limit).await?;
        let mut deleted = 0;
        for candidate in candidates {
            let success = self.storage.delete(&candidate.storage_key).await.is_ok();
            self.repository
                .finish_gc(candidate.asset_id, success, now)
                .await?;
            deleted += usize::from(success);
        }
        Ok(deleted)
    }
}

pub struct FileService {
    repository: Arc<dyn FileRepository>,
    storage: Arc<dyn ObjectStorage>,
    scanner: Arc<dyn MalwareScanner>,
    clock: Arc<dyn Clock>,
    token_keys: Arc<KeyRing>,
    upload_max: u64,
    upload_base: Arc<str>,
}
impl FileService {
    pub fn new(
        repository: Arc<dyn FileRepository>,
        storage: Arc<dyn ObjectStorage>,
        scanner: Arc<dyn MalwareScanner>,
        clock: Arc<dyn Clock>,
        token_keys: Arc<KeyRing>,
        upload_max: u64,
        upload_base: Arc<str>,
    ) -> Self {
        Self {
            repository,
            storage,
            scanner,
            clock,
            token_keys,
            upload_max,
            upload_base,
        }
    }
    pub async fn create_upload(
        &self,
        actor: Uuid,
        workspace: Uuid,
        key: &str,
        input: CreateFileUploadInput,
    ) -> Result<FileUpload, GovernanceError> {
        let name = sanitize_filename(&input.name).ok_or(GovernanceError::Validation)?;
        if input.size == 0
            || input.size > self.upload_max
            || !is_sha(&input.checksum)
            || !allowed_declared_mime(&input.mime_type)
        {
            return Err(GovernanceError::Validation);
        }
        let now = self.clock.now();
        let material = format!("{actor}:{workspace}:{key}");
        let asset_digest = Sha256::digest(format!("file-asset:{material}").as_bytes());
        let asset_id = Uuid::from_bytes(
            asset_digest[..16]
                .try_into()
                .map_err(|_| GovernanceError::Internal)?,
        );
        let token_key_id = self
            .repository
            .upload_key_id(actor, workspace, asset_id)
            .await?
            .unwrap_or_else(|| self.token_keys.current_id().to_owned());
        let token = self
            .token_keys
            .mac_for_key_id(&token_key_id, b"adoc:file-upload:v1", material.as_bytes())
            .ok_or(GovernanceError::UploadExpired)?;
        let storage = self
            .token_keys
            .mac_for_key_id(&token_key_id, b"adoc:file-storage:v1", material.as_bytes())
            .ok_or(GovernanceError::Internal)?;
        let upload_token = URL_SAFE_NO_PAD.encode(token.hash.0);
        let token_hash = TokenHash(Sha256::digest(upload_token.as_bytes()).into());
        let expires_at = now + Duration::hours(1);
        let command = command(actor, "createFileUpload", key, &input, now)?;
        self.repository
            .create(CreateFileCommand {
                workspace_id: workspace,
                asset_id,
                storage_key: hex::encode(storage.hash.0),
                original_name: name,
                mime_type: input.mime_type,
                size_bytes: i64::try_from(input.size).map_err(|_| GovernanceError::Validation)?,
                checksum_sha256: input.checksum,
                token_hash,
                token_key_id,
                expires_at,
                command,
            })
            .await?;
        Ok(FileUpload {
            asset_id,
            upload_url: format!(
                "{}/workspaces/{workspace}/files/{asset_id}/content",
                self.upload_base.trim_end_matches('/')
            ),
            upload_token,
            expires_at,
        })
    }
    pub async fn upload(
        &self,
        actor: Uuid,
        workspace: Uuid,
        asset: Uuid,
        token: &str,
        content_length: u64,
        stream: ByteStream,
    ) -> Result<(), GovernanceError> {
        let auth = self
            .repository
            .authorize_upload(
                actor,
                workspace,
                asset,
                TokenHash(Sha256::digest(token.as_bytes()).into()),
                self.clock.now(),
            )
            .await?;
        if content_length != auth.expected_size {
            return Err(GovernanceError::FileSizeMismatch);
        }
        if auth.uploaded {
            return Ok(());
        }
        let meta = self
            .storage
            .write(&auth.storage_key, stream, auth.expected_size)
            .await
            .map_err(map_storage)?;
        if meta.size != auth.expected_size {
            return Err(GovernanceError::FileSizeMismatch);
        }
        self.repository
            .mark_uploaded(actor, workspace, asset, self.clock.now())
            .await
    }
    pub async fn complete(
        &self,
        actor: Uuid,
        workspace: Uuid,
        asset: Uuid,
        revision: i64,
        key: &str,
        input: CompleteFileUploadInput,
    ) -> Result<FileAsset, GovernanceError> {
        if !is_sha(&input.checksum_sha256) {
            return Err(GovernanceError::Validation);
        }
        let now = self.clock.now();
        let command = command(actor, "completeFileUpload", key, &input, now)?;
        let access = self
            .repository
            .begin_validation(workspace, asset, revision, &command)
            .await?;
        if !matches!(
            access.asset.status,
            FileStatus::Validating | FileStatus::Ready | FileStatus::Failed
        ) {
            return Err(GovernanceError::FileStateInvalid);
        }
        let meta = self
            .storage
            .stat(&access.storage_key)
            .await
            .map_err(map_storage)?;
        let (success, failure) = if meta.size != input.size_bytes
            || input.size_bytes != u64::try_from(access.asset.size_bytes).unwrap_or(0)
        {
            (false, Some("SIZE_MISMATCH"))
        } else {
            let (hash, prefix) = hash_stream(
                self.storage
                    .read(&access.storage_key, None)
                    .await
                    .map_err(map_storage)?,
            )
            .await?;
            if hash != input.checksum_sha256 || hash != access.asset.checksum_sha256 {
                (false, Some("CHECKSUM_MISMATCH"))
            } else if detected_mime(&access.asset.mime_type, &prefix).is_none() {
                (false, Some("MIME_REJECTED"))
            } else {
                match self
                    .scanner
                    .scan(
                        self.storage
                            .read(&access.storage_key, None)
                            .await
                            .map_err(map_storage)?,
                    )
                    .await
                    .map_err(map_storage)?
                {
                    ScanVerdict::Clean => (true, None),
                    ScanVerdict::Malware => (false, Some("MALWARE_DETECTED")),
                }
            }
        };
        let detected_mime = if success {
            Some(access.asset.mime_type.clone())
        } else {
            None
        };
        self.repository
            .mutate(FileMutation {
                actor_id: actor,
                workspace_id: workspace,
                asset_id: asset,
                expected_revision: access.asset.revision,
                success,
                failure_code: failure.map(str::to_owned),
                detected_mime,
                command,
            })
            .await
    }
    pub async fn metadata(
        &self,
        actor: Uuid,
        workspace: Uuid,
        asset: Uuid,
    ) -> Result<FileAsset, GovernanceError> {
        Ok(self.repository.access(actor, workspace, asset).await?.asset)
    }
    pub async fn download(
        &self,
        actor: Uuid,
        workspace: Uuid,
        asset: Uuid,
        range: Option<ByteRange>,
    ) -> Result<(FileAsset, ByteStream), GovernanceError> {
        let access = self.repository.access(actor, workspace, asset).await?;
        if access.asset.status != FileStatus::Ready {
            return Err(GovernanceError::FileStateInvalid);
        }
        let stream = self
            .storage
            .read(&access.storage_key, range)
            .await
            .map_err(map_storage)?;
        Ok((access.asset, stream))
    }
    pub async fn public_download(
        &self,
        token: &str,
        asset: Uuid,
        range_value: Option<&str>,
    ) -> Result<(FileAsset, ByteStream, Option<ByteRange>), GovernanceError> {
        if token.len() != 43
            || URL_SAFE_NO_PAD
                .decode(token)
                .map_err(|_| GovernanceError::PublicLinkInvalid)?
                .len()
                != 32
        {
            return Err(GovernanceError::PublicLinkInvalid);
        }
        let access = self
            .repository
            .public_access(
                TokenHash(Sha256::digest(token.as_bytes()).into()),
                asset,
                self.clock.now(),
            )
            .await?;
        let size = u64::try_from(access.asset.size_bytes).map_err(|_| GovernanceError::Internal)?;
        let range = range_value
            .map(|value| ByteRange::parse(value, size).ok_or(GovernanceError::Validation))
            .transpose()?;
        let stream = self
            .storage
            .read(&access.storage_key, range)
            .await
            .map_err(map_storage)?;
        Ok((access.asset, stream, range))
    }
    pub async fn delete(
        &self,
        actor: Uuid,
        workspace: Uuid,
        asset: Uuid,
        revision: i64,
        key: &str,
    ) -> Result<(), GovernanceError> {
        self.repository
            .delete(actor, workspace, asset, revision, key, self.clock.now())
            .await
    }
}
async fn hash_stream(mut stream: ByteStream) -> Result<(String, Vec<u8>), GovernanceError> {
    let mut hash = Sha256::new();
    let mut prefix = Vec::with_capacity(8192);
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(map_storage)?;
        if prefix.len() < 8192 {
            let remaining = 8192 - prefix.len();
            prefix.extend_from_slice(&chunk[..chunk.len().min(remaining)]);
        }
        hash.update(chunk)
    }
    Ok((hex::encode(hash.finalize()), prefix))
}
fn detected_mime<'a>(declared: &'a str, prefix: &[u8]) -> Option<&'a str> {
    if let Some(kind) = infer::get(prefix) {
        return (kind.mime_type() == declared).then_some(declared);
    }
    matches!(
        declared,
        "text/plain" | "text/markdown" | "application/json"
    )
    .then(|| std::str::from_utf8(prefix).ok())
    .flatten()
    .filter(|value| !value.contains('\0'))
    .map(|_| declared)
}
fn is_sha(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|v| v.is_ascii_hexdigit() && !v.is_ascii_uppercase())
}
fn allowed_declared_mime(value: &str) -> bool {
    matches!(
        value,
        "image/png"
            | "image/jpeg"
            | "image/gif"
            | "image/webp"
            | "application/pdf"
            | "text/plain"
            | "text/markdown"
            | "application/json"
            | "application/zip"
    )
}
fn map_storage(error: StorageError) -> GovernanceError {
    match error {
        StorageError::NotFound => GovernanceError::FileNotFound,
        StorageError::AlreadyExists => GovernanceError::FileStateInvalid,
        StorageError::Unavailable => GovernanceError::StorageUnavailable,
    }
}
fn command<T: Serialize>(
    actor: Uuid,
    operation_id: &'static str,
    key: &str,
    input: &T,
    now: DateTime<Utc>,
) -> Result<Command, GovernanceError> {
    if !(8..=128).contains(&key.len()) {
        return Err(GovernanceError::Validation);
    }
    let bytes = serde_json::to_vec(input).map_err(|_| GovernanceError::Internal)?;
    Ok(Command {
        actor_id: actor,
        operation_id,
        idempotency_key: key.into(),
        request_hash: hex::encode(Sha256::digest(bytes)),
        now,
        expires_at: now + Duration::hours(24),
    })
}
