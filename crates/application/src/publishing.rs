use std::sync::Arc;

pub use adoc_document::{DocumentDiff, PublishedVersion, VersionPage, structural_diff};
use adoc_ports::BoxFuture;
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::{
    document::Draft,
    governance::{Command, GovernanceError},
    identity::{Clock, SecureRandom, TokenHash},
};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PublishDocumentInput {
    pub summary: String,
    #[serde(default)]
    pub client_instance_id: Option<Uuid>,
    #[serde(default)]
    pub lease_token: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreatePublicLinkInput {
    #[serde(default)]
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicLink {
    pub id: Uuid,
    pub expires_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub revision: i64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicLinkCreated {
    pub id: Uuid,
    pub token: String,
    pub url: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicDocument {
    pub title: String,
    pub version_number: i64,
    pub published_at: DateTime<Utc>,
    pub schema_version: i32,
    pub content: Value,
}

#[derive(Clone, Debug)]
pub struct PublishCommand {
    pub version_id: Uuid,
    pub workspace_id: Uuid,
    pub document_id: Uuid,
    pub expected_draft_revision: i64,
    pub summary: String,
    pub client_instance_id: Option<Uuid>,
    pub lease_token_hash: Option<TokenHash>,
    pub command: Command,
}

#[derive(Clone, Debug)]
pub struct RestoreVersionCommand {
    pub draft_id: Uuid,
    pub workspace_id: Uuid,
    pub document_id: Uuid,
    pub version_id: Uuid,
    pub expected_document_revision: i64,
    pub command: Command,
}

#[derive(Clone, Debug)]
pub struct CreatePublicLinkCommand {
    pub link_id: Uuid,
    pub workspace_id: Uuid,
    pub document_id: Uuid,
    pub expected_document_revision: i64,
    pub token_hash: TokenHash,
    pub expires_at: Option<DateTime<Utc>>,
    pub command: Command,
}

#[derive(Clone, Debug)]
pub struct RevokePublicLinkCommand {
    pub workspace_id: Uuid,
    pub document_id: Uuid,
    pub link_id: Uuid,
    pub expected_link_revision: i64,
    pub command: Command,
}

pub trait PublishingRepository: Send + Sync {
    fn list_versions<'a>(
        &'a self,
        actor: Uuid,
        workspace: Uuid,
        document: Uuid,
        cursor: Option<String>,
    ) -> BoxFuture<'a, Result<VersionPage, GovernanceError>>;
    fn get_version<'a>(
        &'a self,
        actor: Uuid,
        workspace: Uuid,
        document: Uuid,
        version: Uuid,
    ) -> BoxFuture<'a, Result<PublishedVersion, GovernanceError>>;
    fn compare_versions<'a>(
        &'a self,
        actor: Uuid,
        workspace: Uuid,
        document: Uuid,
        from: Uuid,
        to: Uuid,
    ) -> BoxFuture<'a, Result<DocumentDiff, GovernanceError>>;
    fn publish<'a>(
        &'a self,
        input: PublishCommand,
    ) -> BoxFuture<'a, Result<PublishedVersion, GovernanceError>>;
    fn restore_version<'a>(
        &'a self,
        input: RestoreVersionCommand,
    ) -> BoxFuture<'a, Result<Draft, GovernanceError>>;
    fn list_public_links<'a>(
        &'a self,
        actor: Uuid,
        workspace: Uuid,
        document: Uuid,
    ) -> BoxFuture<'a, Result<Vec<PublicLink>, GovernanceError>>;
    fn create_public_link<'a>(
        &'a self,
        input: CreatePublicLinkCommand,
    ) -> BoxFuture<'a, Result<Uuid, GovernanceError>>;
    fn revoke_public_link<'a>(
        &'a self,
        input: RevokePublicLinkCommand,
    ) -> BoxFuture<'a, Result<(), GovernanceError>>;
    fn public_document<'a>(
        &'a self,
        token_hash: TokenHash,
        now: DateTime<Utc>,
    ) -> BoxFuture<'a, Result<PublicDocument, GovernanceError>>;
}

#[derive(Clone)]
pub struct PublishingService {
    repository: Arc<dyn PublishingRepository>,
    clock: Arc<dyn Clock>,
    random: Arc<dyn SecureRandom>,
}

impl PublishingService {
    pub fn new(
        repository: Arc<dyn PublishingRepository>,
        clock: Arc<dyn Clock>,
        random: Arc<dyn SecureRandom>,
    ) -> Self {
        Self {
            repository,
            clock,
            random,
        }
    }

    pub async fn list_versions(
        &self,
        actor: Uuid,
        workspace: Uuid,
        document: Uuid,
        cursor: Option<String>,
    ) -> Result<VersionPage, GovernanceError> {
        self.repository
            .list_versions(actor, workspace, document, cursor)
            .await
    }
    pub async fn get_version(
        &self,
        actor: Uuid,
        workspace: Uuid,
        document: Uuid,
        version: Uuid,
    ) -> Result<PublishedVersion, GovernanceError> {
        self.repository
            .get_version(actor, workspace, document, version)
            .await
    }
    pub async fn compare_versions(
        &self,
        actor: Uuid,
        workspace: Uuid,
        document: Uuid,
        from: Uuid,
        to: Uuid,
    ) -> Result<DocumentDiff, GovernanceError> {
        self.repository
            .compare_versions(actor, workspace, document, from, to)
            .await
    }
    pub async fn publish(
        &self,
        actor: Uuid,
        workspace: Uuid,
        document: Uuid,
        expected_revision: i64,
        input: PublishDocumentInput,
        key: &str,
    ) -> Result<PublishedVersion, GovernanceError> {
        let summary = input.summary.trim();
        if summary.is_empty()
            || summary.chars().count() > 1000
            || input.client_instance_id.is_some() != input.lease_token.is_some()
        {
            return Err(GovernanceError::Validation);
        }
        let lease_token_hash = input
            .lease_token
            .as_deref()
            .map(public_token_hash)
            .transpose()
            .map_err(|_| GovernanceError::Validation)?;
        let now = self.clock.now();
        self.repository
            .publish(PublishCommand {
                version_id: self.uuid(now)?,
                workspace_id: workspace,
                document_id: document,
                expected_draft_revision: expected_revision,
                summary: summary.to_owned(),
                client_instance_id: input.client_instance_id,
                lease_token_hash,
                command: command(
                    actor,
                    "publishDocument",
                    key,
                    &(document, expected_revision, input),
                    now,
                )?,
            })
            .await
    }
    pub async fn restore_version(
        &self,
        actor: Uuid,
        workspace: Uuid,
        document: Uuid,
        version: Uuid,
        expected_revision: i64,
        key: &str,
    ) -> Result<Draft, GovernanceError> {
        let now = self.clock.now();
        self.repository
            .restore_version(RestoreVersionCommand {
                draft_id: self.uuid(now)?,
                workspace_id: workspace,
                document_id: document,
                version_id: version,
                expected_document_revision: expected_revision,
                command: command(
                    actor,
                    "restoreVersionToDraft",
                    key,
                    &(document, version, expected_revision),
                    now,
                )?,
            })
            .await
    }
    pub async fn list_public_links(
        &self,
        actor: Uuid,
        workspace: Uuid,
        document: Uuid,
    ) -> Result<Vec<PublicLink>, GovernanceError> {
        self.repository
            .list_public_links(actor, workspace, document)
            .await
    }
    pub async fn create_public_link(
        &self,
        actor: Uuid,
        workspace: Uuid,
        document: Uuid,
        expected_revision: i64,
        input: CreatePublicLinkInput,
        key: &str,
    ) -> Result<PublicLinkCreated, GovernanceError> {
        let now = self.clock.now();
        if input
            .expires_at
            .is_some_and(|expiry| expiry <= now || expiry > now + Duration::days(365))
        {
            return Err(GovernanceError::Validation);
        }
        let (token, token_hash) = self.token()?;
        let id = self.uuid(now)?;
        self.repository
            .create_public_link(CreatePublicLinkCommand {
                link_id: id,
                workspace_id: workspace,
                document_id: document,
                expected_document_revision: expected_revision,
                token_hash,
                expires_at: input.expires_at,
                command: command(
                    actor,
                    "createPublicLink",
                    key,
                    &(document, expected_revision, input),
                    now,
                )?,
            })
            .await?;
        Ok(PublicLinkCreated {
            id,
            url: format!("/public/v1/documents/{token}"),
            token,
        })
    }
    pub async fn revoke_public_link(
        &self,
        actor: Uuid,
        workspace: Uuid,
        document: Uuid,
        link: Uuid,
        expected_revision: i64,
        key: &str,
    ) -> Result<(), GovernanceError> {
        let now = self.clock.now();
        self.repository
            .revoke_public_link(RevokePublicLinkCommand {
                workspace_id: workspace,
                document_id: document,
                link_id: link,
                expected_link_revision: expected_revision,
                command: command(
                    actor,
                    "revokePublicLink",
                    key,
                    &(document, link, expected_revision),
                    now,
                )?,
            })
            .await
    }
    pub async fn public_document(&self, token: &str) -> Result<PublicDocument, GovernanceError> {
        let hash = public_token_hash(token).map_err(|_| GovernanceError::PublicLinkInvalid)?;
        self.repository
            .public_document(hash, self.clock.now())
            .await
    }
    fn uuid(&self, now: DateTime<Utc>) -> Result<Uuid, GovernanceError> {
        self.random
            .uuid_v7(now)
            .map_err(|_| GovernanceError::Internal)
    }
    fn token(&self) -> Result<(String, TokenHash), GovernanceError> {
        let mut bytes = [0_u8; 32];
        self.random
            .bytes(&mut bytes)
            .map_err(|_| GovernanceError::Internal)?;
        let token = URL_SAFE_NO_PAD.encode(bytes);
        let hash = public_token_hash(&token)?;
        Ok((token, hash))
    }
}

fn public_token_hash(token: &str) -> Result<TokenHash, GovernanceError> {
    if token.len() != 43
        || URL_SAFE_NO_PAD
            .decode(token)
            .map_err(|_| GovernanceError::PublicLinkInvalid)?
            .len()
            != 32
    {
        return Err(GovernanceError::PublicLinkInvalid);
    }
    Ok(TokenHash(Sha256::digest(token.as_bytes()).into()))
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
