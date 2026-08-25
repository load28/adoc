use adoc_application::{
    governance::GovernanceError,
    jobs::{JobExecution, JobExecutionError, JobExecutor, JobRepository},
    operations::{
        EventAudience, EventAudienceKind, Job, JobKind, JobSignal, JobStatus, StreamAccess,
        StreamWake,
    },
};
use adoc_ports::BoxFuture;
use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::{PgPool, Postgres, Row, Transaction, postgres::PgRow};
use uuid::Uuid;

use super::{PostgresStore, governance::map_store, outbox::priority_bucket};

#[derive(Clone)]
pub struct PostgresJobRepository {
    pool: PgPool,
}

impl PostgresJobRepository {
    #[must_use]
    pub fn new(store: &PostgresStore) -> Self {
        Self {
            pool: store.pool().clone(),
        }
    }
}

impl PostgresJobRepository {
    fn reconcile_core<'a>(
        &'a self,
        now: DateTime<Utc>,
        limit: i64,
    ) -> BoxFuture<'a, Result<Vec<JobSignal>, GovernanceError>> {
        Box::pin(async move {
            if !(1..=1000).contains(&limit) {
                return Err(GovernanceError::Validation);
            }
            let mut tx = self.pool.begin().await.map_err(map_store)?;
            sqlx::query(
                "UPDATE jobs SET status='CANCELLED',sequence=sequence+1,completed_at=$1,updated_at=$1,lease_owner=NULL,lease_until=NULL,last_error_code=NULL \
                 WHERE status='CANCEL_REQUESTED' AND (lease_until IS NULL OR lease_until<=$1)",
            )
            .bind(now)
            .execute(&mut *tx)
            .await
            .map_err(map_store)?;
            sqlx::query(
                "UPDATE jobs SET status=CASE WHEN attempt>=max_attempts THEN 'DEAD_LETTER'::job_status ELSE 'QUEUED'::job_status END, \
                 sequence=sequence+1,run_after=$1,completed_at=CASE WHEN attempt>=max_attempts THEN $1 ELSE NULL END,updated_at=$1, \
                 lease_owner=NULL,lease_until=NULL,last_error_code='LEASE_EXPIRED' \
                 WHERE status='RUNNING' AND lease_until<=$1",
            )
            .bind(now)
            .execute(&mut *tx)
            .await
            .map_err(map_store)?;
            let rows = sqlx::query(
                "SELECT id,priority FROM jobs WHERE status='QUEUED' AND run_after<=$1 \
                 ORDER BY priority DESC,run_after,id FOR UPDATE SKIP LOCKED LIMIT $2",
            )
            .bind(now)
            .bind(limit)
            .fetch_all(&mut *tx)
            .await
            .map_err(map_store)?;
            let signals = rows
                .iter()
                .map(|row| JobSignal {
                    id: row.get("id"),
                    bucket: priority_bucket(row.get("priority")),
                })
                .collect();
            tx.commit().await.map_err(map_store)?;
            Ok(signals)
        })
    }

    fn claim_core<'a>(
        &'a self,
        id: Uuid,
        worker: &'a str,
        now: DateTime<Utc>,
        lease_until: DateTime<Utc>,
    ) -> BoxFuture<'a, Result<Option<Job>, GovernanceError>> {
        Box::pin(async move {
            if worker.is_empty() || worker.len() > 100 || lease_until <= now {
                return Err(GovernanceError::Validation);
            }
            let row = sqlx::query(
                "UPDATE jobs SET status='RUNNING',sequence=sequence+1,attempt=attempt+1,lease_owner=$2,lease_until=$3,updated_at=$4,last_error_code=NULL \
                 WHERE id=$1 AND status='QUEUED' AND run_after<=$4 AND attempt<max_attempts \
                 RETURNING id,workspace_id,kind,payload_json,status::text,priority,sequence,attempt,max_attempts,correlation_id",
            )
            .bind(id)
            .bind(worker)
            .bind(lease_until)
            .bind(now)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_store)?;
            row.as_ref().map(job).transpose()
        })
    }
}

impl JobExecutor for PostgresJobRepository {
    fn execute<'a>(
        &'a self,
        job: &'a Job,
        worker: &'a str,
        now: DateTime<Utc>,
    ) -> BoxFuture<'a, Result<JobExecution, JobExecutionError>> {
        Box::pin(async move {
            if job.kind != JobKind::OutboxToStream {
                return Err(JobExecutionError::Permanent("JOB_KIND_UNSUPPORTED"));
            }
            let mut tx = self.pool.begin().await.map_err(transient)?;
            let status = sqlx::query_scalar::<_, String>(
                "SELECT status::text FROM jobs WHERE id=$1 AND sequence=$2 AND lease_owner=$3 FOR UPDATE",
            )
            .bind(job.id)
            .bind(job.sequence)
            .bind(worker)
            .fetch_optional(&mut *tx)
            .await
            .map_err(transient)?
            .ok_or(JobExecutionError::Transient("JOB_LEASE_LOST"))?;
            if status == "CANCEL_REQUESTED" {
                finish_cancelled(&mut tx, job.id, now).await?;
                tx.commit().await.map_err(transient)?;
                return Ok(JobExecution::Cancelled);
            }
            if status != "RUNNING" {
                return Err(JobExecutionError::Transient("JOB_LEASE_LOST"));
            }
            let outbox_id = outbox_id(&job.payload)?;
            let row = sqlx::query(
                "SELECT id,workspace_id,aggregate_id,event_type,event_version,payload_json,audience_kind::text,audience_id,minimum_access::text,correlation_id,occurred_at, \
                 EXISTS(SELECT 1 FROM consumer_receipts r WHERE r.consumer='workspace-stream' AND r.event_id=o.id) AS received \
                 FROM outbox_events o WHERE id=$1 FOR UPDATE",
            )
            .bind(outbox_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(transient)?
            .ok_or(JobExecutionError::Permanent("OUTBOX_EVENT_NOT_FOUND"))?;
            if row.get::<bool, _>("received") {
                finish_succeeded(&mut tx, job.id, now).await?;
                tx.commit().await.map_err(transient)?;
                return Ok(JobExecution::Delivered(None));
            }
            let audience = audience(&row)?;
            if audience.kind == EventAudienceKind::Internal {
                record_receipt(&mut tx, outbox_id, now).await?;
                mark_published(&mut tx, outbox_id, now).await?;
                finish_succeeded(&mut tx, job.id, now).await?;
                tx.commit().await.map_err(transient)?;
                return Ok(JobExecution::Delivered(None));
            }
            let event_type = normalize_event_type(row.get("event_type"))?;
            let payload: Value = row.get("payload_json");
            validate_payload(event_type, &payload)?;
            let workspace: Uuid = row.get("workspace_id");
            sqlx::query(
                "INSERT INTO workspace_sequences(workspace_id,next_audit_sequence,next_stream_sequence) VALUES($1,1,1) ON CONFLICT(workspace_id) DO NOTHING",
            )
            .bind(workspace)
            .execute(&mut *tx)
            .await
            .map_err(transient)?;
            let sequence: i64 = sqlx::query_scalar(
                "UPDATE workspace_sequences SET next_stream_sequence=next_stream_sequence+1 WHERE workspace_id=$1 RETURNING next_stream_sequence-1",
            )
            .bind(workspace)
            .fetch_one(&mut *tx)
            .await
            .map_err(transient)?;
            let stream_id = Uuid::now_v7();
            sqlx::query(
                "INSERT INTO workspace_stream_events(id,workspace_id,sequence,outbox_event_id,aggregate_id,event_type,event_version,payload_json,audience_kind,audience_id,minimum_access,correlation_id,occurred_at,created_at,expires_at) \
                 VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9::event_audience_kind,$10,$11::document_access,$12,$13,$14,$14+interval '24 hours')",
            )
            .bind(stream_id)
            .bind(workspace)
            .bind(sequence)
            .bind(outbox_id)
            .bind(row.get::<Uuid, _>("aggregate_id"))
            .bind(event_type)
            .bind(row.get::<i32, _>("event_version"))
            .bind(payload)
            .bind(row.get::<String, _>("audience_kind"))
            .bind(audience.id)
            .bind(audience.minimum_access.map(super::outbox::access_text))
            .bind(row.get::<String, _>("correlation_id"))
            .bind(row.get::<DateTime<Utc>, _>("occurred_at"))
            .bind(now)
            .execute(&mut *tx)
            .await
            .map_err(transient)?;
            record_receipt(&mut tx, outbox_id, now).await?;
            mark_published(&mut tx, outbox_id, now).await?;
            finish_succeeded(&mut tx, job.id, now).await?;
            tx.commit().await.map_err(transient)?;
            Ok(JobExecution::Delivered(Some(StreamWake {
                workspace_id: workspace,
                sequence,
            })))
        })
    }
}

impl JobRepository for PostgresJobRepository {
    fn reconcile<'a>(
        &'a self,
        now: DateTime<Utc>,
        limit: i64,
    ) -> BoxFuture<'a, Result<Vec<JobSignal>, GovernanceError>> {
        self.reconcile_core(now, limit)
    }

    fn claim<'a>(
        &'a self,
        id: Uuid,
        worker: &'a str,
        now: DateTime<Utc>,
        lease_until: DateTime<Utc>,
    ) -> BoxFuture<'a, Result<Option<Job>, GovernanceError>> {
        self.claim_core(id, worker, now, lease_until)
    }

    fn transition_failure<'a>(
        &'a self,
        job: &'a Job,
        worker: &'a str,
        code: &'a str,
        transient_error: bool,
        run_after: DateTime<Utc>,
        now: DateTime<Utc>,
    ) -> BoxFuture<'a, Result<(), GovernanceError>> {
        Box::pin(async move {
            let changed = sqlx::query(
                "UPDATE jobs SET status=CASE WHEN $4 AND attempt<max_attempts THEN 'QUEUED'::job_status WHEN $4 THEN 'DEAD_LETTER'::job_status ELSE 'FAILED'::job_status END, \
                 sequence=sequence+1,run_after=CASE WHEN $4 AND attempt<max_attempts THEN $5 ELSE run_after END, \
                 completed_at=CASE WHEN $4 AND attempt<max_attempts THEN NULL ELSE $6 END,lease_owner=NULL,lease_until=NULL,last_error_code=$7,updated_at=$6 \
                 WHERE id=$1 AND sequence=$2 AND status='RUNNING' AND lease_owner=$3",
            )
            .bind(job.id)
            .bind(job.sequence)
            .bind(worker)
            .bind(transient_error)
            .bind(run_after)
            .bind(now)
            .bind(code)
            .execute(&self.pool)
            .await
            .map_err(map_store)?;
            if changed.rows_affected() == 1 {
                Ok(())
            } else {
                Err(GovernanceError::JobStateInvalid)
            }
        })
    }

    fn request_cancel<'a>(
        &'a self,
        id: Uuid,
        expected_sequence: i64,
        now: DateTime<Utc>,
    ) -> BoxFuture<'a, Result<(), GovernanceError>> {
        Box::pin(async move {
            let changed = sqlx::query(
                "UPDATE jobs SET status='CANCEL_REQUESTED',sequence=sequence+1,cancel_requested_at=$3,updated_at=$3 \
                 WHERE id=$1 AND sequence=$2 AND status IN ('QUEUED','RUNNING')",
            )
            .bind(id)
            .bind(expected_sequence)
            .bind(now)
            .execute(&self.pool)
            .await
            .map_err(map_store)?;
            if changed.rows_affected() == 1 {
                return Ok(());
            }
            let current: Option<i64> = sqlx::query_scalar("SELECT sequence FROM jobs WHERE id=$1")
                .bind(id)
                .fetch_optional(&self.pool)
                .await
                .map_err(map_store)?;
            match current {
                None => Err(GovernanceError::JobNotFound),
                Some(sequence) if sequence != expected_sequence => {
                    Err(GovernanceError::RevisionConflict {
                        current_revision: sequence,
                    })
                }
                Some(_) => Err(GovernanceError::JobStateInvalid),
            }
        })
    }

    fn cleanup_stream<'a>(
        &'a self,
        now: DateTime<Utc>,
        limit: i64,
    ) -> BoxFuture<'a, Result<u64, GovernanceError>> {
        Box::pin(async move {
            if !(1..=10_000).contains(&limit) {
                return Err(GovernanceError::Validation);
            }
            let result = sqlx::query(
                "DELETE FROM workspace_stream_events WHERE id IN (SELECT id FROM workspace_stream_events WHERE expires_at<=$1 ORDER BY expires_at,id LIMIT $2)",
            )
            .bind(now)
            .bind(limit)
            .execute(&self.pool)
            .await
            .map_err(map_store)?;
            Ok(result.rows_affected())
        })
    }
}

async fn finish_succeeded(
    tx: &mut Transaction<'_, Postgres>,
    id: Uuid,
    now: DateTime<Utc>,
) -> Result<(), JobExecutionError> {
    sqlx::query("UPDATE jobs SET status='SUCCEEDED',sequence=sequence+1,completed_at=$2,updated_at=$2,lease_owner=NULL,lease_until=NULL,last_error_code=NULL WHERE id=$1 AND status='RUNNING'")
        .bind(id).bind(now).execute(&mut **tx).await.map_err(transient)?;
    Ok(())
}

async fn finish_cancelled(
    tx: &mut Transaction<'_, Postgres>,
    id: Uuid,
    now: DateTime<Utc>,
) -> Result<(), JobExecutionError> {
    sqlx::query("UPDATE jobs SET status='CANCELLED',sequence=sequence+1,completed_at=$2,updated_at=$2,lease_owner=NULL,lease_until=NULL WHERE id=$1 AND status='CANCEL_REQUESTED'")
        .bind(id).bind(now).execute(&mut **tx).await.map_err(transient)?;
    Ok(())
}

async fn record_receipt(
    tx: &mut Transaction<'_, Postgres>,
    event: Uuid,
    now: DateTime<Utc>,
) -> Result<(), JobExecutionError> {
    sqlx::query("INSERT INTO consumer_receipts(consumer,event_id,processed_at) VALUES('workspace-stream',$1,$2) ON CONFLICT DO NOTHING")
        .bind(event).bind(now).execute(&mut **tx).await.map_err(transient)?;
    Ok(())
}

async fn mark_published(
    tx: &mut Transaction<'_, Postgres>,
    event: Uuid,
    now: DateTime<Utc>,
) -> Result<(), JobExecutionError> {
    sqlx::query("UPDATE outbox_events SET published_at=COALESCE(published_at,$2) WHERE id=$1")
        .bind(event)
        .bind(now)
        .execute(&mut **tx)
        .await
        .map_err(transient)?;
    Ok(())
}

fn job(row: &PgRow) -> Result<Job, GovernanceError> {
    Ok(Job {
        id: row.get("id"),
        workspace_id: row.get("workspace_id"),
        kind: match row.get::<String, _>("kind").as_str() {
            "OUTBOX_TO_STREAM" => JobKind::OutboxToStream,
            "OUTBOX_TO_SEARCH" => JobKind::OutboxToSearch,
            _ => return Err(GovernanceError::Internal),
        },
        payload: row.get("payload_json"),
        status: match row.get::<String, _>("status").as_str() {
            "QUEUED" => JobStatus::Queued,
            "RUNNING" => JobStatus::Running,
            "CANCEL_REQUESTED" => JobStatus::CancelRequested,
            "SUCCEEDED" => JobStatus::Succeeded,
            "FAILED" => JobStatus::Failed,
            "CANCELLED" => JobStatus::Cancelled,
            "TIMED_OUT" => JobStatus::TimedOut,
            "DEAD_LETTER" => JobStatus::DeadLetter,
            _ => return Err(GovernanceError::Internal),
        },
        priority: row.get("priority"),
        sequence: row.get("sequence"),
        attempt: row.get("attempt"),
        max_attempts: row.get("max_attempts"),
        correlation_id: row.get("correlation_id"),
    })
}

fn outbox_id(payload: &Value) -> Result<Uuid, JobExecutionError> {
    let object = payload
        .as_object()
        .filter(|object| object.len() == 1)
        .ok_or(JobExecutionError::Permanent("JOB_PAYLOAD_INVALID"))?;
    object
        .get("outboxEventId")
        .and_then(Value::as_str)
        .and_then(|value| Uuid::parse_str(value).ok())
        .ok_or(JobExecutionError::Permanent("JOB_PAYLOAD_INVALID"))
}

fn audience(row: &PgRow) -> Result<EventAudience, JobExecutionError> {
    let kind = match row.get::<String, _>("audience_kind").as_str() {
        "INTERNAL" => EventAudienceKind::Internal,
        "WORKSPACE" => EventAudienceKind::Workspace,
        "ADMIN" => EventAudienceKind::Admin,
        "USER" => EventAudienceKind::User,
        "DOCUMENT" => EventAudienceKind::Document,
        _ => return Err(JobExecutionError::Permanent("EVENT_AUDIENCE_INVALID")),
    };
    let minimum_access = row
        .get::<Option<String>, _>("minimum_access")
        .map(|value| match value.as_str() {
            "VIEWER" => Ok(StreamAccess::Viewer),
            "CONTRIBUTOR" => Ok(StreamAccess::Contributor),
            "EDITOR" => Ok(StreamAccess::Editor),
            _ => Err(JobExecutionError::Permanent("EVENT_AUDIENCE_INVALID")),
        })
        .transpose()?;
    let audience = EventAudience {
        kind,
        id: row.get("audience_id"),
        minimum_access,
    };
    if audience.is_valid() {
        Ok(audience)
    } else {
        Err(JobExecutionError::Permanent("EVENT_AUDIENCE_INVALID"))
    }
}

fn normalize_event_type(value: String) -> Result<&'static str, JobExecutionError> {
    match value.as_str() {
        "WorkspaceChanged.v1" => Ok("WORKSPACE_CHANGED"),
        "MembershipChanged.v1" => Ok("MEMBERSHIP_CHANGED"),
        "InvitationChanged.v1" => Ok("INVITATION_CHANGED"),
        "GroupChanged.v1" => Ok("GROUP_CHANGED"),
        "PermissionChanged.v1" => Ok("PERMISSION_CHANGED"),
        "PublishPolicyChanged.v1" => Ok("PUBLISH_POLICY_CHANGED"),
        "DocumentChanged.v1" => Ok("DOCUMENT_CHANGED"),
        "DocumentMoved.v1" => Ok("DOCUMENT_MOVED"),
        "DraftChanged.v1" => Ok("DRAFT_CHANGED"),
        "LeaseChanged.v1" => Ok("LEASE_CHANGED"),
        "VersionPublished.v1" => Ok("VERSION_PUBLISHED"),
        "DiscussionChanged.v1" => Ok("DISCUSSION_CHANGED"),
        "MessageChanged.v1" => Ok("MESSAGE_CHANGED"),
        "ReviewChanged.v1" => Ok("REVIEW_CHANGED"),
        "InboxChanged.v1" => Ok("INBOX_CHANGED"),
        "ReferenceChanged.v1" => Ok("REFERENCE_CHANGED"),
        "VocabularyChanged.v1" => Ok("VOCABULARY_CHANGED"),
        "AIJobChanged.v1" => Ok("AI_JOB_CHANGED"),
        "ProposalApplied.v1" => Ok("PROPOSAL_APPLIED"),
        "FileChanged.v1" => Ok("FILE_CHANGED"),
        "PublicLinkChanged.v1" => Ok("PUBLIC_LINK_CHANGED"),
        "PurgeChanged.v1" => Ok("PURGE_CHANGED"),
        _ => Err(JobExecutionError::Permanent("EVENT_TYPE_INVALID")),
    }
}

fn validate_payload(event_type: &str, payload: &Value) -> Result<(), JobExecutionError> {
    let object = payload
        .as_object()
        .ok_or(JobExecutionError::Permanent("EVENT_CONTRACT_INVALID"))?;
    let required: &[&str] = match event_type {
        "DOCUMENT_CHANGED" => &["documentId", "revision", "treeRevision", "action"],
        "DOCUMENT_MOVED" => &[
            "documentId",
            "beforeParentId",
            "afterParentId",
            "revision",
            "treeRevision",
        ],
        "DRAFT_CHANGED" => &["documentId", "draftId", "revision", "operationIds"],
        "LEASE_CHANGED" => &["documentId", "holderUserId", "expiresAt", "revision"],
        "VERSION_PUBLISHED" => &["documentId", "versionId", "number", "sourceDraftRevision"],
        "AI_JOB_CHANGED" => &["jobId", "status", "jobSequence"],
        "PROPOSAL_APPLIED" => &[
            "proposalId",
            "documentId",
            "appliedOperationIds",
            "resultRevision",
        ],
        "PURGE_CHANGED" => &["targetKind", "targetId", "step", "status"],
        _ => &["entityId", "revision", "action"],
    };
    let maximum = if event_type == "AI_JOB_CHANGED" {
        5
    } else {
        required.len()
    };
    if object.len() < required.len()
        || object.len() > maximum
        || !required.iter().all(|key| object.contains_key(*key))
        || (event_type == "AI_JOB_CHANGED"
            && object.keys().any(|key| {
                !matches!(
                    key.as_str(),
                    "jobId" | "status" | "jobSequence" | "phase" | "queuePosition"
                )
            }))
    {
        return Err(JobExecutionError::Permanent("EVENT_CONTRACT_INVALID"));
    }
    Ok(())
}

fn transient(_: sqlx::Error) -> JobExecutionError {
    JobExecutionError::Transient("DATABASE_UNAVAILABLE")
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{normalize_event_type, validate_payload};

    #[test]
    fn event_registry_accepts_every_stream_type_and_exact_payload_shape() {
        let events = [
            ("WorkspaceChanged.v1", "WORKSPACE_CHANGED"),
            ("MembershipChanged.v1", "MEMBERSHIP_CHANGED"),
            ("InvitationChanged.v1", "INVITATION_CHANGED"),
            ("GroupChanged.v1", "GROUP_CHANGED"),
            ("PermissionChanged.v1", "PERMISSION_CHANGED"),
            ("PublishPolicyChanged.v1", "PUBLISH_POLICY_CHANGED"),
            ("DocumentChanged.v1", "DOCUMENT_CHANGED"),
            ("DocumentMoved.v1", "DOCUMENT_MOVED"),
            ("DraftChanged.v1", "DRAFT_CHANGED"),
            ("LeaseChanged.v1", "LEASE_CHANGED"),
            ("VersionPublished.v1", "VERSION_PUBLISHED"),
            ("DiscussionChanged.v1", "DISCUSSION_CHANGED"),
            ("MessageChanged.v1", "MESSAGE_CHANGED"),
            ("ReviewChanged.v1", "REVIEW_CHANGED"),
            ("InboxChanged.v1", "INBOX_CHANGED"),
            ("ReferenceChanged.v1", "REFERENCE_CHANGED"),
            ("VocabularyChanged.v1", "VOCABULARY_CHANGED"),
            ("AIJobChanged.v1", "AI_JOB_CHANGED"),
            ("ProposalApplied.v1", "PROPOSAL_APPLIED"),
            ("FileChanged.v1", "FILE_CHANGED"),
            ("PublicLinkChanged.v1", "PUBLIC_LINK_CHANGED"),
            ("PurgeChanged.v1", "PURGE_CHANGED"),
        ];
        for (wire, canonical) in events {
            let normalized = normalize_event_type(wire.to_owned()).expect("registered event");
            assert_eq!(normalized, canonical, "{wire}");
            let payload = match normalized {
                "DOCUMENT_CHANGED" => {
                    json!({"documentId":"id","revision":1,"treeRevision":2,"action":"UPDATED"})
                }
                "DOCUMENT_MOVED" => {
                    json!({"documentId":"id","beforeParentId":null,"afterParentId":"parent","revision":1,"treeRevision":2})
                }
                "DRAFT_CHANGED" => {
                    json!({"documentId":"id","draftId":"draft","revision":1,"operationIds":[]})
                }
                "LEASE_CHANGED" => {
                    json!({"documentId":"id","holderUserId":null,"expiresAt":null,"revision":1})
                }
                "VERSION_PUBLISHED" => {
                    json!({"documentId":"id","versionId":"version","number":1,"sourceDraftRevision":1})
                }
                "AI_JOB_CHANGED" => json!({"jobId":"id","status":"RUNNING","jobSequence":1}),
                "PROPOSAL_APPLIED" => {
                    json!({"proposalId":"id","documentId":"document","appliedOperationIds":[],"resultRevision":1})
                }
                "PURGE_CHANGED" => {
                    json!({"targetKind":"DOCUMENT","targetId":"id","step":"COMPLETED","status":"COMPLETED"})
                }
                _ => json!({"entityId":"id","revision":1,"action":"UPDATED"}),
            };
            validate_payload(normalized, &payload).expect("canonical payload");
        }
    }

    #[test]
    fn event_registry_rejects_unknown_type_and_payload_drift() {
        assert!(normalize_event_type("Unknown.v1".to_owned()).is_err());
        assert!(
            validate_payload(
                "DOCUMENT_CHANGED",
                &json!({"documentId":"id","revision":1,"treeRevision":2})
            )
            .is_err()
        );
        assert!(
            validate_payload(
                "AI_JOB_CHANGED",
                &json!({"jobId":"id","status":"RUNNING","jobSequence":1,"secret":"forbidden"})
            )
            .is_err()
        );
    }
}
