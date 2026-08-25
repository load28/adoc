use std::sync::Arc;

use adoc_application::{
    jobs::{JobExecution, JobExecutionError, JobExecutor},
    operations::{Job, JobKind},
    search::SearchProjectionService,
};
use adoc_ports::BoxFuture;
use chrono::{DateTime, Utc};
use serde_json::Value;
use uuid::Uuid;

use crate::postgres::PostgresJobRepository;

pub struct WorkerJobExecutor {
    stream: Arc<PostgresJobRepository>,
    search: SearchProjectionService,
}

impl WorkerJobExecutor {
    #[must_use]
    pub fn new(stream: Arc<PostgresJobRepository>, search: SearchProjectionService) -> Self {
        Self { stream, search }
    }
}

impl JobExecutor for WorkerJobExecutor {
    fn execute<'a>(
        &'a self,
        job: &'a Job,
        worker: &'a str,
        now: DateTime<Utc>,
    ) -> BoxFuture<'a, Result<JobExecution, JobExecutionError>> {
        Box::pin(async move {
            match job.kind {
                JobKind::OutboxToStream => self.stream.execute(job, worker, now).await,
                JobKind::OutboxToSearch => {
                    self.search
                        .execute(outbox_id(&job.payload)?, job.id, worker, job.sequence, now)
                        .await
                }
            }
        })
    }
}

fn outbox_id(payload: &Value) -> Result<Uuid, JobExecutionError> {
    payload
        .as_object()
        .filter(|object| object.len() == 1)
        .and_then(|object| object.get("outboxEventId"))
        .and_then(Value::as_str)
        .and_then(|value| Uuid::parse_str(value).ok())
        .ok_or(JobExecutionError::Permanent("JOB_PAYLOAD_INVALID"))
}
