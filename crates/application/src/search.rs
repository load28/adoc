use std::sync::Arc;

pub use adoc_knowledge::{
    PermissionPathNode, SEARCH_PROJECTION_SCHEMA, SearchProjection, SearchSourceKind,
    TOMBSTONE_REGION_ID, extract_search_regions, permission_fingerprint, permission_scope_token,
    projection_id, snapshot_hash,
};
use adoc_ports::BoxFuture;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::jobs::{JobExecution, JobExecutionError};

#[derive(Clone, Debug)]
pub enum ProjectionMutation {
    Replace {
        workspace_id: Uuid,
        document_id: Uuid,
        source_kind: SearchSourceKind,
        sequence: i64,
        regions: Vec<SearchProjection>,
    },
    DeleteTree {
        workspace_id: Uuid,
        root_document_id: Uuid,
        sequence: i64,
    },
    DeleteWorkspace {
        workspace_id: Uuid,
        sequence: i64,
    },
}

#[derive(Clone, Debug)]
pub struct ProjectionWork {
    pub outbox_event_id: Uuid,
    pub already_completed: bool,
    pub mutations: Vec<ProjectionMutation>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum SearchProjectionError {
    #[error("transient search projection failure: {0}")]
    Transient(&'static str),
    #[error("permanent search projection failure: {0}")]
    Permanent(&'static str),
}

pub trait SearchProjectionRepository: Send + Sync {
    fn prepare<'a>(
        &'a self,
        outbox_event_id: Uuid,
    ) -> BoxFuture<'a, Result<ProjectionWork, SearchProjectionError>>;
    fn complete<'a>(
        &'a self,
        outbox_event_id: Uuid,
        job_id: Uuid,
        worker: &'a str,
        job_sequence: i64,
        now: DateTime<Utc>,
    ) -> BoxFuture<'a, Result<JobExecution, SearchProjectionError>>;
}

pub trait SearchIndex: Send + Sync {
    fn apply<'a>(
        &'a self,
        mutations: &'a [ProjectionMutation],
    ) -> BoxFuture<'a, Result<(), SearchProjectionError>>;
}

pub struct SearchProjectionService {
    repository: Arc<dyn SearchProjectionRepository>,
    index: Arc<dyn SearchIndex>,
}

impl SearchProjectionService {
    pub fn new(
        repository: Arc<dyn SearchProjectionRepository>,
        index: Arc<dyn SearchIndex>,
    ) -> Self {
        Self { repository, index }
    }

    pub async fn execute(
        &self,
        outbox_event_id: Uuid,
        job_id: Uuid,
        worker: &str,
        job_sequence: i64,
        now: DateTime<Utc>,
    ) -> Result<JobExecution, JobExecutionError> {
        let work = self
            .repository
            .prepare(outbox_event_id)
            .await
            .map_err(job_error)?;
        if !work.already_completed {
            self.index.apply(&work.mutations).await.map_err(job_error)?;
        }
        self.repository
            .complete(outbox_event_id, job_id, worker, job_sequence, now)
            .await
            .map_err(job_error)
    }
}

fn job_error(error: SearchProjectionError) -> JobExecutionError {
    match error {
        SearchProjectionError::Transient(code) => JobExecutionError::Transient(code),
        SearchProjectionError::Permanent(code) => JobExecutionError::Permanent(code),
    }
}
