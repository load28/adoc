use std::sync::Arc;

pub use adoc_operations::{Job, JobKind, JobSignal, StreamWake};
use adoc_ports::BoxFuture;
use chrono::{DateTime, Duration, Utc};
use uuid::Uuid;

use crate::{governance::GovernanceError, identity::Clock};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JobExecutionError {
    Transient(&'static str),
    Permanent(&'static str),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JobExecution {
    Delivered(Option<StreamWake>),
    Cancelled,
}

pub trait JobRepository: Send + Sync {
    fn reconcile<'a>(
        &'a self,
        now: DateTime<Utc>,
        limit: i64,
    ) -> BoxFuture<'a, Result<Vec<JobSignal>, GovernanceError>>;
    fn claim<'a>(
        &'a self,
        id: Uuid,
        worker: &'a str,
        now: DateTime<Utc>,
        lease_until: DateTime<Utc>,
    ) -> BoxFuture<'a, Result<Option<Job>, GovernanceError>>;
    fn execute_outbox_to_stream<'a>(
        &'a self,
        job: &'a Job,
        worker: &'a str,
        now: DateTime<Utc>,
    ) -> BoxFuture<'a, Result<JobExecution, JobExecutionError>>;
    fn transition_failure<'a>(
        &'a self,
        job: &'a Job,
        worker: &'a str,
        code: &'a str,
        transient: bool,
        run_after: DateTime<Utc>,
        now: DateTime<Utc>,
    ) -> BoxFuture<'a, Result<(), GovernanceError>>;
    fn request_cancel<'a>(
        &'a self,
        id: Uuid,
        expected_sequence: i64,
        now: DateTime<Utc>,
    ) -> BoxFuture<'a, Result<(), GovernanceError>>;
    fn cleanup_stream<'a>(
        &'a self,
        now: DateTime<Utc>,
        limit: i64,
    ) -> BoxFuture<'a, Result<u64, GovernanceError>>;
}

pub trait JobSignalQueue: Send + Sync {
    fn signal<'a>(&'a self, jobs: &'a [JobSignal]) -> BoxFuture<'a, Result<(), GovernanceError>>;
    fn drain<'a>(&'a self, limit: usize) -> BoxFuture<'a, Result<Vec<Uuid>, GovernanceError>>;
    fn publish_stream_wake<'a>(
        &'a self,
        wake: StreamWake,
    ) -> BoxFuture<'a, Result<(), GovernanceError>>;
}

pub struct JobRuntime {
    repository: Arc<dyn JobRepository>,
    queue: Arc<dyn JobSignalQueue>,
    clock: Arc<dyn Clock>,
    worker_id: Arc<str>,
    lease: Duration,
}

impl JobRuntime {
    pub fn new(
        repository: Arc<dyn JobRepository>,
        queue: Arc<dyn JobSignalQueue>,
        clock: Arc<dyn Clock>,
        worker_id: Arc<str>,
        lease: Duration,
    ) -> Self {
        Self {
            repository,
            queue,
            clock,
            worker_id,
            lease,
        }
    }

    pub async fn run_once(&self, limit: i64, reconcile: bool) -> Result<usize, GovernanceError> {
        if !(1..=1000).contains(&limit) {
            return Err(GovernanceError::Validation);
        }
        let now = self.clock.now();
        let discovered = if reconcile {
            self.repository.reconcile(now, limit).await?
        } else {
            Vec::new()
        };
        let signalled = self.queue.signal(&discovered).await.is_ok();
        let mut ids = self.queue.drain(limit as usize).await.unwrap_or_default();
        if !signalled || ids.is_empty() {
            ids.extend(discovered.into_iter().map(|signal| signal.id));
        }
        ids.sort_unstable();
        ids.dedup();
        ids.truncate(limit as usize);

        let mut completed = 0;
        for id in ids {
            let Some(job) = self
                .repository
                .claim(id, &self.worker_id, now, now + self.lease)
                .await?
            else {
                continue;
            };
            let result = match job.kind {
                JobKind::OutboxToStream => {
                    self.repository
                        .execute_outbox_to_stream(&job, &self.worker_id, now)
                        .await
                }
            };
            match result {
                Ok(JobExecution::Delivered(wake)) => {
                    completed += 1;
                    if let Some(wake) = wake {
                        let _ = self.queue.publish_stream_wake(wake).await;
                    }
                }
                Ok(JobExecution::Cancelled) => completed += 1,
                Err(JobExecutionError::Transient(code)) => {
                    self.repository
                        .transition_failure(
                            &job,
                            &self.worker_id,
                            code,
                            true,
                            retry_at(job.id, job.attempt, now),
                            now,
                        )
                        .await?;
                }
                Err(JobExecutionError::Permanent(code)) => {
                    self.repository
                        .transition_failure(&job, &self.worker_id, code, false, now, now)
                        .await?;
                }
            }
        }
        Ok(completed)
    }

    pub async fn cleanup_stream(&self, limit: i64) -> Result<u64, GovernanceError> {
        self.repository
            .cleanup_stream(self.clock.now(), limit)
            .await
    }
}

fn retry_at(id: Uuid, attempt: i32, now: DateTime<Utc>) -> DateTime<Utc> {
    let exponent = attempt.saturating_sub(1).clamp(0, 6) as u32;
    let base = 5_i64.saturating_mul(2_i64.pow(exponent)).min(300);
    let jitter = i64::from(id.as_bytes()[15] % 6);
    now + Duration::seconds(base + jitter)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retry_is_bounded_and_deterministic() {
        let now = Utc::now();
        let id = Uuid::from_u128(5);
        assert_eq!(retry_at(id, 1, now), retry_at(id, 1, now));
        assert!(retry_at(id, 100, now) <= now + Duration::seconds(305));
    }
}
