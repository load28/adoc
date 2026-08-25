use std::collections::BTreeSet;

use adoc_application::{
    ai::{
        AiAdmission, AiContextRepository, AiExecutionStart, AiJobPage, AiJobRepository,
        AiJobStatus, AiJobView, AiTarget, AiTask, ContextArtifact, ContextError, ContextSource,
        ContextSourceKind, IncludeReason, MAX_OUTPUT_BYTES, PreparedContext, RuntimeRequest,
        RuntimeResult, SourceAuthority, TASK_DEFINITION_VERSION, runtime_output_schema,
        task_definition,
    },
    governance::GovernanceError,
    jobs::{JobExecution, JobExecutionError},
    operations::{Job, JobPriorityBucket, JobSignal},
    search::{
        CompiledSearchScope, SearchScopeCompiler, SearchSource, SearchSourceKind,
        extract_search_regions, projection_id, snapshot_hash,
    },
};
use adoc_ports::BoxFuture;
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use super::{
    PostgresSearchRetrievalRepository, PostgresStore, document::dry_run_draft_operations_tx,
    retrieval::compile_scope_tx,
};

#[derive(Clone)]
pub struct PostgresAiContextRepository {
    pool: PgPool,
    scope: PostgresSearchRetrievalRepository,
}

impl PostgresAiContextRepository {
    #[must_use]
    pub fn new(store: &PostgresStore) -> Self {
        Self {
            pool: store.pool().clone(),
            scope: PostgresSearchRetrievalRepository::new(store),
        }
    }

    async fn capture(&self, task: &AiTask) -> Result<CapturedContext, ContextError> {
        if !task.is_valid() {
            return Err(ContextError::Validation);
        }
        let before = self
            .scope
            .compile(task.actor_id, task.workspace_id, true)
            .await
            .map_err(scope_error)?;
        let mut tx = self.pool.begin().await.map_err(|_| ContextError::Storage)?;
        sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ READ ONLY")
            .execute(&mut *tx)
            .await
            .map_err(|_| ContextError::Storage)?;
        let target = capture_target(&mut tx, task, &before).await?;
        let writing_rule_version = sqlx::query(
            "SELECT baseline_version,revision FROM writing_configurations WHERE workspace_id=$1",
        )
        .bind(task.workspace_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|_| ContextError::Storage)?
        .map_or_else(
            || format!("{}:0", adoc_application::ai::WRITING_RULE_BASELINE_VERSION),
            |row| {
                format!(
                    "{}:{}",
                    row.get::<String, _>("baseline_version"),
                    row.get::<i64, _>("revision")
                )
            },
        );
        let vocabulary_revision: i64 = sqlx::query_scalar(
            "SELECT COALESCE(max(revision),0) FROM vocabulary_concepts WHERE workspace_id=$1",
        )
        .bind(task.workspace_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(|_| ContextError::Storage)?;
        tx.commit().await.map_err(|_| ContextError::Storage)?;
        let after = self
            .scope
            .compile(task.actor_id, task.workspace_id, true)
            .await
            .map_err(scope_error)?;
        if before.fingerprint != after.fingerprint {
            return Err(ContextError::Stale);
        }
        let stamp = stamp(&CaptureStamp {
            task,
            target_revision: target.revision,
            target_snapshot_hash: &target.snapshot_hash,
            permission_scope_fingerprint: &before.fingerprint,
            writing_rule_version: &writing_rule_version,
            vocabulary_revision,
        })?;
        Ok(CapturedContext {
            stamp,
            retrieval_query: target.query.clone(),
            target,
            scope: before,
            writing_rule_version,
            vocabulary_revision,
        })
    }

    async fn sources(
        &self,
        task: &AiTask,
        capture: &CapturedContext,
        retrieved: &[SearchSource],
    ) -> Result<Vec<ContextSource>, ContextError> {
        let mut tx = self.pool.begin().await.map_err(|_| ContextError::Storage)?;
        sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ READ ONLY")
            .execute(&mut *tx)
            .await
            .map_err(|_| ContextError::Storage)?;
        let mut sources = direct_sources(&capture.target, task, &capture.scope)?;
        if let Some(document_id) = capture.target.document_id {
            append_discussions(
                &mut tx,
                task.workspace_id,
                document_id,
                &capture.scope,
                &mut sources,
            )
            .await?;
            append_references(
                &mut tx,
                task.workspace_id,
                document_id,
                &capture.scope,
                &mut sources,
            )
            .await?;
        }
        append_vocabulary(
            &mut tx,
            task.workspace_id,
            capture.vocabulary_revision,
            &mut sources,
        )
        .await?;
        for source in retrieved {
            if let Some(materialized) =
                materialize_search_source(&mut tx, task.workspace_id, &capture.scope, source)
                    .await?
            {
                sources.push(materialized);
            }
        }
        tx.commit().await.map_err(|_| ContextError::Storage)?;
        deduplicate_sources(&mut sources);
        Ok(sources)
    }
}

impl AiContextRepository for PostgresAiContextRepository {
    fn prepare<'a>(
        &'a self,
        task: &'a AiTask,
    ) -> BoxFuture<'a, Result<PreparedContext, ContextError>> {
        Box::pin(async move {
            let capture = self.capture(task).await?;
            Ok(PreparedContext {
                stamp: capture.stamp,
                retrieval_query: capture.retrieval_query,
            })
        })
    }

    fn materialize<'a>(
        &'a self,
        task: &'a AiTask,
        prepared: &'a PreparedContext,
        retrieved: &'a [SearchSource],
    ) -> BoxFuture<'a, Result<ContextArtifact, ContextError>> {
        Box::pin(async move {
            let capture = self.capture(task).await?;
            if capture.stamp != prepared.stamp
                || capture.retrieval_query != prepared.retrieval_query
            {
                return Err(ContextError::Stale);
            }
            let sources = self.sources(task, &capture, retrieved).await?;
            let after = self.capture(task).await?;
            if after.stamp != capture.stamp {
                return Err(ContextError::Stale);
            }
            Ok(ContextArtifact {
                schema_version: 1,
                task: task.clone(),
                task_definition_version: TASK_DEFINITION_VERSION.to_owned(),
                sources,
                writing_rule_version: capture.writing_rule_version,
                vocabulary_revision: capture.vocabulary_revision,
                permission_scope_fingerprint: capture.scope.fingerprint,
                estimated_input_units: 0,
            })
        })
    }
}

impl AiJobRepository for PostgresAiContextRepository {
    fn admit<'a>(
        &'a self,
        task: &'a AiTask,
        artifact: &'a ContextArtifact,
        fingerprint: &'a str,
        request_key: &'a str,
        now: chrono::DateTime<chrono::Utc>,
    ) -> BoxFuture<'a, Result<AiAdmission, ContextError>> {
        Box::pin(async move {
            if !task.is_valid()
                || artifact.task != *task
                || !(8..=200).contains(&request_key.len())
                || fingerprint.len() != 64
            {
                return Err(ContextError::Validation);
            }
            let mut tx = self
                .pool
                .begin()
                .await
                .map_err(|_| ContextError::StorageAt("admission_begin"))?;
            sqlx::query("SET TRANSACTION ISOLATION LEVEL SERIALIZABLE")
                .execute(&mut *tx)
                .await
                .map_err(|_| ContextError::StorageAt("admission_isolation"))?;
            if let Some(row) = sqlx::query("SELECT j.id,j.kind,j.target_json,j.status::text,j.revision,j.error_code,r.result_json,p.id AS proposal_id,g.id AS runtime_job_id,g.priority FROM ai_jobs j LEFT JOIN ai_results r ON r.job_id=j.id LEFT JOIN proposals p ON p.job_id=j.id JOIN jobs g ON g.id=j.runtime_job_id WHERE j.workspace_id=$1 AND j.user_id=$2 AND j.request_key=$3")
                .bind(task.workspace_id).bind(task.actor_id).bind(request_key).fetch_optional(&mut *tx).await.map_err(|_| ContextError::StorageAt("admission_replay_read"))?
            {
                let view = ai_job_view(&row)?;
                let signal = JobSignal { id: row.get("runtime_job_id"), bucket: priority_bucket(row.get("priority")) };
                tx.commit().await.map_err(|_| ContextError::StorageAt("admission_replay_commit"))?;
                return Ok(AiAdmission { view, signal });
            }
            let scope = compile_scope_tx(&mut tx, task.actor_id, task.workspace_id, true)
                .await
                .map_err(scope_error)?;
            if scope.fingerprint != artifact.permission_scope_fingerprint {
                return Err(ContextError::Stale);
            }
            validate_artifact_tx(&mut tx, task, artifact, &scope).await?;
            let config = sqlx::query("SELECT provider,model,user_concurrency_limit,workspace_concurrency_limit,monthly_budget_microunits FROM ai_configurations WHERE workspace_id=$1 FOR UPDATE")
                .bind(task.workspace_id).fetch_optional(&mut *tx).await.map_err(|_| ContextError::StorageAt("ai_configuration_read"))?.ok_or(ContextError::RetrievalUnavailable)?;
            let user_active: i64 = sqlx::query_scalar("SELECT count(*) FROM ai_jobs WHERE workspace_id=$1 AND user_id=$2 AND status IN ('QUEUED','RUNNING','CANCEL_REQUESTED')")
                .bind(task.workspace_id).bind(task.actor_id).fetch_one(&mut *tx).await.map_err(|_| ContextError::StorageAt("ai_user_concurrency_read"))?;
            let workspace_active: i64 = sqlx::query_scalar("SELECT count(*) FROM ai_jobs WHERE workspace_id=$1 AND status IN ('QUEUED','RUNNING','CANCEL_REQUESTED')")
                .bind(task.workspace_id).fetch_one(&mut *tx).await.map_err(|_| ContextError::StorageAt("ai_workspace_concurrency_read"))?;
            let month_usage: i64 = sqlx::query_scalar("SELECT COALESCE(sum(estimated_microunits),0)::bigint FROM ai_usage_daily WHERE workspace_id=$1 AND usage_date>=date_trunc('month',$2::timestamptz)::date")
                .bind(task.workspace_id).bind(now).fetch_one(&mut *tx).await.map_err(|_| ContextError::StorageAt("ai_usage_read"))?;
            if user_active >= i64::from(config.get::<i16, _>("user_concurrency_limit"))
                || workspace_active
                    >= i64::from(config.get::<i16, _>("workspace_concurrency_limit"))
                || month_usage >= config.get::<i64, _>("monthly_budget_microunits")
            {
                return Err(ContextError::Quota);
            }
            let ai_job_id = Uuid::now_v7();
            let runtime_job_id = Uuid::now_v7();
            let correlation_id = ai_job_id.to_string();
            let priority = if task_definition(task.kind).timeout_class == "INTERACTIVE" {
                75_i16
            } else {
                50_i16
            };
            let metadata = serde_json::json!({
                "schemaVersion": artifact.schema_version,
                "taskDefinitionVersion": artifact.task_definition_version,
                "writingRuleVersion": artifact.writing_rule_version,
                "vocabularyRevision": artifact.vocabulary_revision,
                "permissionScopeFingerprint": artifact.permission_scope_fingerprint,
                "estimatedInputUnits": artifact.estimated_input_units
            });
            sqlx::query("INSERT INTO jobs(id,workspace_id,kind,payload_json,dedupe_key,status,priority,sequence,attempt,max_attempts,run_after,correlation_id,created_at,updated_at) VALUES($1,$2,'AI_RUNTIME',$3,$4,'QUEUED',$5,1,0,3,$6,$7,$6,$6)")
                .bind(runtime_job_id).bind(task.workspace_id).bind(serde_json::json!({"aiJobId":ai_job_id})).bind(format!("ai-runtime:{ai_job_id}")).bind(priority).bind(now).bind(&correlation_id)
                .execute(&mut *tx).await.map_err(|_| ContextError::StorageAt("runtime_job_insert"))?;
            sqlx::query("INSERT INTO ai_jobs(id,workspace_id,user_id,kind,target_json,expected_revision,context_fingerprint,context_metadata_json,request_key,runtime_job_id,status,priority,provider,model,created_at) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,'QUEUED',$11,$12,$13,$14)")
                .bind(ai_job_id).bind(task.workspace_id).bind(task.actor_id).bind(enum_text(&task.kind)?).bind(serde_json::to_value(task).map_err(|_| ContextError::Storage)?)
                .bind(task.expected_revision).bind(fingerprint).bind(metadata).bind(request_key).bind(runtime_job_id).bind(priority).bind(config.get::<String,_>("provider")).bind(config.get::<String,_>("model")).bind(now)
                .execute(&mut *tx).await.map_err(|_| ContextError::StorageAt("ai_job_insert"))?;
            for (ordinal, source) in artifact.sources.iter().enumerate() {
                sqlx::query("INSERT INTO ai_context_sources(job_id,workspace_id,ordinal,source_kind,source_id,stable_id,authority,include_reason,snapshot_hash,snapshot_text,source_revision,permission_key,included,metadata_json) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14)")
                    .bind(ai_job_id).bind(task.workspace_id).bind(i32::try_from(ordinal).map_err(|_| ContextError::Limit)?)
                    .bind(enum_text(&source.kind)?).bind(source.source_id.to_string()).bind(&source.stable_id).bind(enum_text(&source.authority)?).bind(enum_text(&source.include_reason)?)
                    .bind(&source.snapshot_hash).bind(&source.snapshot_text).bind(source.source_revision).bind(&source.permission_key).bind(source.included)
                    .bind(serde_json::json!({"documentId":source.document_id,"regionId":source.region_id,"version":source.version,"draftRevision":source.draft_revision,"retrievedAt":source.retrieved_at}))
                    .execute(&mut *tx).await.map_err(|_| ContextError::StorageAt("context_source_insert"))?;
            }
            append_ai_job_event_tx(
                &mut tx,
                task.workspace_id,
                task.actor_id,
                ai_job_id,
                0,
                "QUEUED",
                now,
            )
            .await
            .map_err(|_| ContextError::StorageAt("ai_job_event_insert"))?;
            tx.commit()
                .await
                .map_err(|_| ContextError::StorageAt("admission_commit"))?;
            Ok(AiAdmission {
                view: AiJobView {
                    id: ai_job_id,
                    kind: task.kind,
                    target: task.target.clone(),
                    status: AiJobStatus::Queued,
                    sequence: 1,
                    revision: 0,
                    result: None,
                    proposal_id: None,
                    error_code: None,
                },
                signal: JobSignal {
                    id: runtime_job_id,
                    bucket: priority_bucket(priority),
                },
            })
        })
    }

    fn list<'a>(
        &'a self,
        actor_id: Uuid,
        workspace_id: Uuid,
        cursor: Option<&'a str>,
    ) -> BoxFuture<'a, Result<AiJobPage, ContextError>> {
        Box::pin(async move {
            require_membership(&self.pool, actor_id, workspace_id).await?;
            let cursor = cursor
                .map(Uuid::parse_str)
                .transpose()
                .map_err(|_| ContextError::Validation)?;
            let rows = sqlx::query("SELECT j.id,j.kind,j.target_json,j.status::text,j.revision,j.error_code,r.result_json,p.id AS proposal_id FROM ai_jobs j LEFT JOIN ai_results r ON r.job_id=j.id LEFT JOIN proposals p ON p.job_id=j.id WHERE j.workspace_id=$1 AND j.user_id=$2 AND ($3::uuid IS NULL OR j.id<$3) ORDER BY j.id DESC LIMIT 51")
                .bind(workspace_id).bind(actor_id).bind(cursor).fetch_all(&self.pool).await.map_err(|_| ContextError::Storage)?;
            let mut items = rows
                .iter()
                .take(50)
                .map(ai_job_view)
                .collect::<Result<Vec<_>, _>>()?;
            let next_cursor = (rows.len() > 50)
                .then(|| items.last().map(|item| item.id.to_string()))
                .flatten();
            items.truncate(50);
            Ok(AiJobPage { items, next_cursor })
        })
    }

    fn get<'a>(
        &'a self,
        actor_id: Uuid,
        workspace_id: Uuid,
        job_id: Uuid,
    ) -> BoxFuture<'a, Result<AiJobView, ContextError>> {
        Box::pin(async move {
            require_membership(&self.pool, actor_id, workspace_id).await?;
            let row = sqlx::query("SELECT j.id,j.kind,j.target_json,j.status::text,j.revision,j.error_code,r.result_json,p.id AS proposal_id FROM ai_jobs j LEFT JOIN ai_results r ON r.job_id=j.id LEFT JOIN proposals p ON p.job_id=j.id WHERE j.workspace_id=$1 AND j.user_id=$2 AND j.id=$3")
                .bind(workspace_id).bind(actor_id).bind(job_id).fetch_optional(&self.pool).await.map_err(|_| ContextError::Storage)?.ok_or(ContextError::NotFound)?;
            ai_job_view(&row)
        })
    }

    fn cancel<'a>(
        &'a self,
        actor_id: Uuid,
        workspace_id: Uuid,
        job_id: Uuid,
        expected_revision: i64,
        request_key: &'a str,
        now: chrono::DateTime<chrono::Utc>,
    ) -> BoxFuture<'a, Result<(), ContextError>> {
        Box::pin(async move {
            if !(16..=128).contains(&request_key.len()) {
                return Err(ContextError::Validation);
            }
            let mut tx = self.pool.begin().await.map_err(|_| ContextError::Storage)?;
            let request_hash = hex::encode(Sha256::digest(
                format!("{workspace_id}:{actor_id}:{job_id}:{expected_revision}").as_bytes(),
            ));
            let inserted = sqlx::query("INSERT INTO idempotency_keys(workspace_id,actor_id,operation_id,key,request_hash,locked_until,expires_at,created_at) VALUES($1,$2,'cancelAIJob',$3,$4,$5,$6,$5) ON CONFLICT DO NOTHING")
                .bind(workspace_id).bind(actor_id).bind(request_key).bind(&request_hash).bind(now).bind(now + chrono::Duration::hours(24))
                .execute(&mut *tx).await.map_err(|_| ContextError::Storage)?;
            if inserted.rows_affected() == 0 {
                let receipt = sqlx::query("SELECT request_hash,response_json FROM idempotency_keys WHERE workspace_id=$1 AND actor_id=$2 AND operation_id='cancelAIJob' AND key=$3 FOR UPDATE")
                    .bind(workspace_id).bind(actor_id).bind(request_key).fetch_one(&mut *tx).await.map_err(|_| ContextError::Storage)?;
                if receipt.get::<String, _>("request_hash") != request_hash {
                    return Err(ContextError::IdempotencyConflict);
                }
                if receipt.get::<Option<Value>, _>("response_json").is_some() {
                    tx.commit().await.map_err(|_| ContextError::Storage)?;
                    return Ok(());
                }
            }
            let row = sqlx::query("SELECT j.status::text,j.revision,j.runtime_job_id,g.status::text AS generic_status FROM ai_jobs j JOIN jobs g ON g.id=j.runtime_job_id WHERE j.workspace_id=$1 AND j.user_id=$2 AND j.id=$3 FOR UPDATE OF j,g")
                .bind(workspace_id).bind(actor_id).bind(job_id).fetch_optional(&mut *tx).await.map_err(|_| ContextError::Storage)?.ok_or(ContextError::NotFound)?;
            if row.get::<i64, _>("revision") != expected_revision {
                return Err(ContextError::Stale);
            }
            let runtime_job_id: Uuid = row.get("runtime_job_id");
            let status = match row.get::<String, _>("generic_status").as_str() {
                "QUEUED" => {
                    sqlx::query("UPDATE jobs SET status='CANCELLED',sequence=sequence+1,cancel_requested_at=$2,completed_at=$2,updated_at=$2 WHERE id=$1 AND status='QUEUED'").bind(runtime_job_id).bind(now).execute(&mut *tx).await.map_err(|_| ContextError::Storage)?;
                    sqlx::query("UPDATE ai_jobs SET status='CANCELLED',revision=revision+1,error_code='AI_CANCELLED',completed_at=$2 WHERE id=$1 AND status='QUEUED'").bind(job_id).bind(now).execute(&mut *tx).await.map_err(|_| ContextError::Storage)?;
                    "CANCELLED"
                }
                "RUNNING" => {
                    sqlx::query("UPDATE jobs SET status='CANCEL_REQUESTED',sequence=sequence+1,cancel_requested_at=$2,updated_at=$2 WHERE id=$1 AND status='RUNNING'").bind(runtime_job_id).bind(now).execute(&mut *tx).await.map_err(|_| ContextError::Storage)?;
                    sqlx::query("UPDATE ai_jobs SET status='CANCEL_REQUESTED',revision=revision+1 WHERE id=$1 AND status='RUNNING'").bind(job_id).execute(&mut *tx).await.map_err(|_| ContextError::Storage)?;
                    "CANCEL_REQUESTED"
                }
                _ => return Err(ContextError::Validation),
            };
            append_ai_job_event_tx(
                &mut tx,
                workspace_id,
                actor_id,
                job_id,
                expected_revision + 1,
                status,
                now,
            )
            .await
            .map_err(|_| ContextError::Storage)?;
            sqlx::query("UPDATE idempotency_keys SET response_status=202,response_json='{}'::jsonb WHERE workspace_id=$1 AND actor_id=$2 AND operation_id='cancelAIJob' AND key=$3 AND response_json IS NULL")
                .bind(workspace_id).bind(actor_id).bind(request_key).execute(&mut *tx).await.map_err(|_| ContextError::Storage)?;
            tx.commit().await.map_err(|_| ContextError::Storage)
        })
    }

    fn start<'a>(
        &'a self,
        job: &'a Job,
        worker: &'a str,
        now: chrono::DateTime<chrono::Utc>,
        timeout_millis: u64,
    ) -> BoxFuture<'a, Result<AiExecutionStart, JobExecutionError>> {
        Box::pin(async move {
            let ai_job_id = ai_job_id(&job.payload)?;
            let mut tx = self.pool.begin().await.map_err(transient_job)?;
            sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ")
                .execute(&mut *tx)
                .await
                .map_err(transient_job)?;
            let generic_status = sqlx::query_scalar::<_,String>("SELECT status::text FROM jobs WHERE id=$1 AND sequence=$2 AND lease_owner=$3 FOR UPDATE")
                .bind(job.id).bind(job.sequence).bind(worker).fetch_optional(&mut *tx).await.map_err(transient_job)?.ok_or(JobExecutionError::Transient("JOB_LEASE_LOST"))?;
            let row = sqlx::query("SELECT j.workspace_id,j.user_id,j.kind,j.target_json,j.context_fingerprint,j.context_metadata_json,j.status::text,j.model,j.provider,EXISTS(SELECT 1 FROM ai_results r WHERE r.job_id=j.id) AS completed FROM ai_jobs j WHERE j.id=$1 AND j.runtime_job_id=$2 FOR UPDATE")
                .bind(ai_job_id).bind(job.id).fetch_optional(&mut *tx).await.map_err(transient_job)?.ok_or(JobExecutionError::Permanent("AI_JOB_NOT_FOUND"))?;
            if generic_status == "CANCEL_REQUESTED" {
                finish_cancelled_tx(&mut tx, ai_job_id, job.id, now).await?;
                tx.commit().await.map_err(transient_job)?;
                return Ok(AiExecutionStart::Cancelled);
            }
            if row.get::<bool, _>("completed") {
                finish_generic_success_tx(&mut tx, job.id, now).await?;
                tx.commit().await.map_err(transient_job)?;
                return Ok(AiExecutionStart::Completed);
            }
            if generic_status != "RUNNING" {
                return Err(JobExecutionError::Transient("JOB_LEASE_LOST"));
            }
            let task: AiTask = match serde_json::from_value(row.get("target_json")) {
                Ok(task) => task,
                Err(_) => {
                    finish_pre_runtime_failure_tx(
                        &mut tx,
                        ai_job_id,
                        job.id,
                        "AI_CONTEXT_INVALID",
                        now,
                    )
                    .await?;
                    tx.commit().await.map_err(transient_job)?;
                    return Ok(AiExecutionStart::Completed);
                }
            };
            let scope =
                match compile_scope_tx(&mut tx, task.actor_id, task.workspace_id, true).await {
                    Ok(scope) => scope,
                    Err(
                        adoc_application::search::SearchRetrievalError::Internal
                        | adoc_application::search::SearchRetrievalError::Unavailable,
                    ) => return Err(JobExecutionError::Transient("AI_STORAGE_UNAVAILABLE")),
                    Err(_) => {
                        finish_pre_runtime_failure_tx(
                            &mut tx,
                            ai_job_id,
                            job.id,
                            "AI_PERMISSION_CHANGED",
                            now,
                        )
                        .await?;
                        tx.commit().await.map_err(transient_job)?;
                        return Ok(AiExecutionStart::Completed);
                    }
                };
            let metadata: Value = row.get("context_metadata_json");
            if metadata
                .get("permissionScopeFingerprint")
                .and_then(Value::as_str)
                != Some(scope.fingerprint.as_str())
            {
                finish_pre_runtime_failure_tx(
                    &mut tx,
                    ai_job_id,
                    job.id,
                    "AI_PERMISSION_CHANGED",
                    now,
                )
                .await?;
                tx.commit().await.map_err(transient_job)?;
                return Ok(AiExecutionStart::Completed);
            }
            let sources = match load_context_sources_tx(&mut tx, ai_job_id, &scope).await {
                Ok(sources) => sources,
                Err(JobExecutionError::Transient(code)) => {
                    return Err(JobExecutionError::Transient(code));
                }
                Err(JobExecutionError::Permanent(code)) => {
                    finish_pre_runtime_failure_tx(&mut tx, ai_job_id, job.id, code, now).await?;
                    tx.commit().await.map_err(transient_job)?;
                    return Ok(AiExecutionStart::Completed);
                }
            };
            let artifact = (|| {
                Ok::<_, JobExecutionError>(ContextArtifact {
                    schema_version: metadata
                        .get("schemaVersion")
                        .and_then(Value::as_u64)
                        .and_then(|value| u8::try_from(value).ok())
                        .ok_or(JobExecutionError::Permanent("AI_CONTEXT_INVALID"))?,
                    task: task.clone(),
                    task_definition_version: metadata
                        .get("taskDefinitionVersion")
                        .and_then(Value::as_str)
                        .ok_or(JobExecutionError::Permanent("AI_CONTEXT_INVALID"))?
                        .to_owned(),
                    sources,
                    writing_rule_version: metadata
                        .get("writingRuleVersion")
                        .and_then(Value::as_str)
                        .ok_or(JobExecutionError::Permanent("AI_CONTEXT_INVALID"))?
                        .to_owned(),
                    vocabulary_revision: metadata
                        .get("vocabularyRevision")
                        .and_then(Value::as_i64)
                        .ok_or(JobExecutionError::Permanent("AI_CONTEXT_INVALID"))?,
                    permission_scope_fingerprint: scope.fingerprint,
                    estimated_input_units: metadata
                        .get("estimatedInputUnits")
                        .and_then(Value::as_u64)
                        .unwrap_or(0),
                })
            })();
            let artifact = match artifact {
                Ok(artifact) => artifact,
                Err(JobExecutionError::Permanent(code)) => {
                    finish_pre_runtime_failure_tx(&mut tx, ai_job_id, job.id, code, now).await?;
                    tx.commit().await.map_err(transient_job)?;
                    return Ok(AiExecutionStart::Completed);
                }
                Err(error) => return Err(error),
            };
            let revision: i64 = sqlx::query_scalar("UPDATE ai_jobs SET status='RUNNING',attempt=attempt+1,revision=revision+1,started_at=COALESCE(started_at,$2),error_code=NULL WHERE id=$1 AND status IN ('QUEUED','RUNNING') RETURNING revision")
                .bind(ai_job_id).bind(now).fetch_one(&mut *tx).await.map_err(transient_job)?;
            append_ai_job_event_tx(
                &mut tx,
                task.workspace_id,
                task.actor_id,
                ai_job_id,
                revision,
                "RUNNING",
                now,
            )
            .await
            .map_err(transient_job)?;
            tx.commit().await.map_err(transient_job)?;
            Ok(AiExecutionStart::Execute(RuntimeRequest {
                job_id: ai_job_id,
                task_kind: task.kind,
                model: row.get("model"),
                policy_artifact: serde_json::json!({"taskDefinitionVersion":TASK_DEFINITION_VERSION,"applicationPolicy":task_definition(task.kind).application_policy,"instruction":task.instruction}),
                context_artifact: serde_json::to_value(artifact)
                    .map_err(|_| JobExecutionError::Permanent("AI_CONTEXT_INVALID"))?,
                output_schema: runtime_output_schema(task.kind),
                timeout_millis,
                max_output_bytes: MAX_OUTPUT_BYTES,
            }))
        })
    }

    fn is_cancelled<'a>(&'a self, generic_job_id: Uuid) -> BoxFuture<'a, bool> {
        Box::pin(async move {
            sqlx::query_scalar::<_, String>("SELECT status::text FROM jobs WHERE id=$1")
                .bind(generic_job_id)
                .fetch_optional(&self.pool)
                .await
                .ok()
                .flatten()
                .is_none_or(|status| matches!(status.as_str(), "CANCEL_REQUESTED" | "CANCELLED"))
        })
    }

    fn finish_success<'a>(
        &'a self,
        job: &'a Job,
        worker: &'a str,
        result: &'a RuntimeResult,
        now: chrono::DateTime<chrono::Utc>,
    ) -> BoxFuture<'a, Result<JobExecution, JobExecutionError>> {
        Box::pin(async move {
            let ai_job_id = ai_job_id(&job.payload)?;
            let mut tx = self.pool.begin().await.map_err(transient_job)?;
            let status = lock_generic_tx(&mut tx, job, worker).await?;
            if status == "CANCEL_REQUESTED" {
                finish_cancelled_tx(&mut tx, ai_job_id, job.id, now).await?;
                tx.commit().await.map_err(transient_job)?;
                return Ok(JobExecution::Cancelled);
            }
            if status != "RUNNING" {
                return Err(JobExecutionError::Transient("JOB_LEASE_LOST"));
            }
            let result_context = sqlx::query("SELECT workspace_id,user_id,target_json,expected_revision,context_metadata_json FROM ai_jobs WHERE id=$1 AND status='RUNNING' FOR UPDATE")
                .bind(ai_job_id).fetch_optional(&mut *tx).await.map_err(transient_job)?.ok_or(JobExecutionError::Transient("AI_JOB_STATE_CHANGED"))?;
            let task: AiTask = serde_json::from_value(result_context.get("target_json"))
                .map_err(|_| JobExecutionError::Permanent("AI_RESULT_INVALID"))?;
            let included_rows = sqlx::query_scalar::<_, String>(
                "SELECT source_id FROM ai_context_sources WHERE job_id=$1 AND included=true",
            )
            .bind(ai_job_id)
            .fetch_all(&mut *tx)
            .await
            .map_err(transient_job)?;
            let included_source_ids: BTreeSet<Uuid> = match included_rows
                .into_iter()
                .map(|value| Uuid::parse_str(&value))
                .collect()
            {
                Ok(ids) => ids,
                Err(_) => {
                    finish_pre_runtime_failure_tx(
                        &mut tx,
                        ai_job_id,
                        job.id,
                        "AI_RESULT_INVALID",
                        now,
                    )
                    .await?;
                    tx.commit().await.map_err(transient_job)?;
                    return Ok(JobExecution::Delivered(None));
                }
            };
            let (validated, application) = match adoc_application::ai::validate_result(
                result.output_json.clone(),
                &task,
                &included_source_ids,
            ) {
                Ok(value) => value,
                Err(error) => {
                    let code = match error {
                        adoc_application::ai::AiResultValidationError::Revision => {
                            "AI_RESULT_STALE"
                        }
                        _ => "AI_RESULT_INVALID",
                    };
                    finish_pre_runtime_failure_tx(&mut tx, ai_job_id, job.id, code, now).await?;
                    tx.commit().await.map_err(transient_job)?;
                    return Ok(JobExecution::Delivered(None));
                }
            };
            let document_id = result_document_id(&mut tx, &task).await?;
            if !validated.operations.is_empty() {
                let Some(document_id) = document_id else {
                    finish_pre_runtime_failure_tx(
                        &mut tx,
                        ai_job_id,
                        job.id,
                        "AI_RESULT_INVALID",
                        now,
                    )
                    .await?;
                    tx.commit().await.map_err(transient_job)?;
                    return Ok(JobExecution::Delivered(None));
                };
                let resulting_content = match dry_run_draft_operations_tx(
                    &mut tx,
                    task.workspace_id,
                    document_id,
                    task.expected_revision,
                    validated.operations.clone(),
                )
                .await
                {
                    Ok(content) => content,
                    Err(GovernanceError::RevisionConflict { .. }) => {
                        finish_pre_runtime_failure_tx(
                            &mut tx,
                            ai_job_id,
                            job.id,
                            "AI_RESULT_STALE",
                            now,
                        )
                        .await?;
                        tx.commit().await.map_err(transient_job)?;
                        return Ok(JobExecution::Delivered(None));
                    }
                    Err(_) => {
                        finish_pre_runtime_failure_tx(
                            &mut tx,
                            ai_job_id,
                            job.id,
                            "AI_RESULT_INVALID",
                            now,
                        )
                        .await?;
                        tx.commit().await.map_err(transient_job)?;
                        return Ok(JobExecution::Delivered(None));
                    }
                };
                let prohibited: Vec<String> = sqlx::query_scalar("SELECT vt.term FROM vocabulary_terms vt JOIN vocabulary_concepts vc ON vc.workspace_id=vt.workspace_id AND vc.id=vt.concept_id WHERE vt.workspace_id=$1 AND vt.kind='PROHIBITED' AND vc.status='ACTIVE' ORDER BY vt.normalized_term")
                    .bind(task.workspace_id).fetch_all(&mut *tx).await.map_err(transient_job)?;
                if adoc_application::ai::prohibited_term_in_content(&resulting_content, &prohibited)
                    .is_some()
                {
                    finish_pre_runtime_failure_tx(
                        &mut tx,
                        ai_job_id,
                        job.id,
                        "AI_RESULT_RULE_BLOCKED",
                        now,
                    )
                    .await?;
                    tx.commit().await.map_err(transient_job)?;
                    return Ok(JobExecution::Delivered(None));
                }
            }
            let metadata: Value = result_context.get("context_metadata_json");
            let writing_rule_version = metadata
                .get("writingRuleVersion")
                .and_then(Value::as_str)
                .ok_or(JobExecutionError::Permanent("AI_RESULT_INVALID"))?;
            let vocabulary_revision = metadata
                .get("vocabularyRevision")
                .and_then(Value::as_i64)
                .ok_or(JobExecutionError::Permanent("AI_RESULT_INVALID"))?;
            let application_name = match application {
                adoc_application::ai::ResultApplication::None => "NONE",
                adoc_application::ai::ResultApplication::BoundedRewrite => "BOUNDED_REWRITE",
                adoc_application::ai::ResultApplication::Proposal => "PROPOSAL",
            };
            let validation = serde_json::json!({"validatorVersion":adoc_application::ai::RESULT_VALIDATOR_VERSION,"writingRuleVersion":writing_rule_version,"vocabularyRevision":vocabulary_revision,"status":"VALIDATED","application":application_name});
            let canonical_result = serde_json::to_value(&validated)
                .map_err(|_| JobExecutionError::Permanent("AI_RESULT_INVALID"))?;
            sqlx::query("INSERT INTO ai_results(job_id,schema_version,result_json,validation_json,completed_at) VALUES($1,1,$2,$3,$4) ON CONFLICT(job_id) DO NOTHING")
                .bind(ai_job_id).bind(canonical_result).bind(&validation).bind(now).execute(&mut *tx).await.map_err(transient_job)?;
            if application == adoc_application::ai::ResultApplication::Proposal {
                let document_id =
                    document_id.ok_or(JobExecutionError::Permanent("AI_RESULT_INVALID"))?;
                sqlx::query("INSERT INTO proposals(id,workspace_id,job_id,document_id,owner_user_id,base_revision,operations_json,writing_rule_version,vocabulary_revision,validation_json,status,revision,created_at) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,'OPEN',0,$11) ON CONFLICT(job_id) DO NOTHING")
                    .bind(Uuid::now_v7()).bind(task.workspace_id).bind(ai_job_id).bind(document_id).bind(task.actor_id).bind(task.expected_revision)
                    .bind(serde_json::to_value(&validated.operations).map_err(|_| JobExecutionError::Permanent("AI_RESULT_INVALID"))?).bind(writing_rule_version).bind(vocabulary_revision).bind(validation).bind(now)
                    .execute(&mut *tx).await.map_err(transient_job)?;
            }
            let row = sqlx::query("UPDATE ai_jobs SET status='SUCCEEDED',usage_json=$2,error_code=NULL,revision=revision+1,completed_at=$3 WHERE id=$1 AND status='RUNNING' RETURNING workspace_id,user_id,provider,model,revision")
                .bind(ai_job_id).bind(serde_json::to_value(&result.usage).map_err(|_| JobExecutionError::Permanent("AI_USAGE_INVALID"))?).bind(now).fetch_optional(&mut *tx).await.map_err(transient_job)?.ok_or(JobExecutionError::Transient("AI_JOB_STATE_CHANGED"))?;
            sqlx::query("INSERT INTO ai_usage_daily(workspace_id,usage_date,provider,model,input_tokens,output_tokens,estimated_microunits,job_count) VALUES($1,$2::date,$3,$4,$5,$6,$7,1) ON CONFLICT(workspace_id,usage_date,provider,model) DO UPDATE SET input_tokens=ai_usage_daily.input_tokens+EXCLUDED.input_tokens,output_tokens=ai_usage_daily.output_tokens+EXCLUDED.output_tokens,estimated_microunits=ai_usage_daily.estimated_microunits+EXCLUDED.estimated_microunits,job_count=ai_usage_daily.job_count+1")
                .bind(row.get::<Uuid,_>("workspace_id")).bind(now.date_naive()).bind(row.get::<String,_>("provider")).bind(row.get::<String,_>("model"))
                .bind(i64::try_from(result.usage.input_units).unwrap_or(i64::MAX)).bind(i64::try_from(result.usage.output_units).unwrap_or(i64::MAX)).bind(i64::try_from(result.usage.estimated_microunits.unwrap_or(0)).unwrap_or(i64::MAX))
                .execute(&mut *tx).await.map_err(transient_job)?;
            append_ai_job_event_tx(
                &mut tx,
                row.get("workspace_id"),
                row.get("user_id"),
                ai_job_id,
                row.get("revision"),
                "SUCCEEDED",
                now,
            )
            .await
            .map_err(transient_job)?;
            finish_generic_success_tx(&mut tx, job.id, now).await?;
            tx.commit().await.map_err(transient_job)?;
            Ok(JobExecution::Delivered(None))
        })
    }

    fn finish_terminal<'a>(
        &'a self,
        job: &'a Job,
        worker: &'a str,
        status: AiJobStatus,
        code: &'a str,
        now: chrono::DateTime<chrono::Utc>,
    ) -> BoxFuture<'a, Result<JobExecution, JobExecutionError>> {
        Box::pin(async move {
            let ai_job_id = ai_job_id(&job.payload)?;
            let mut tx = self.pool.begin().await.map_err(transient_job)?;
            let current = lock_generic_tx(&mut tx, job, worker).await?;
            if current == "CANCEL_REQUESTED" || status == AiJobStatus::Cancelled {
                finish_cancelled_tx(&mut tx, ai_job_id, job.id, now).await?;
                tx.commit().await.map_err(transient_job)?;
                return Ok(JobExecution::Cancelled);
            }
            if current != "RUNNING" {
                return Err(JobExecutionError::Transient("JOB_LEASE_LOST"));
            }
            let status_text = match status {
                AiJobStatus::TimedOut => "TIMED_OUT",
                _ => "FAILED",
            };
            let row = sqlx::query("UPDATE ai_jobs SET status=$2::ai_job_status,error_code=$3,revision=revision+1,completed_at=$4 WHERE id=$1 AND status='RUNNING' RETURNING workspace_id,user_id,revision")
                .bind(ai_job_id).bind(status_text).bind(code).bind(now).fetch_one(&mut *tx).await.map_err(transient_job)?;
            append_ai_job_event_tx(
                &mut tx,
                row.get("workspace_id"),
                row.get("user_id"),
                ai_job_id,
                row.get("revision"),
                status_text,
                now,
            )
            .await
            .map_err(transient_job)?;
            finish_generic_success_tx(&mut tx, job.id, now).await?;
            tx.commit().await.map_err(transient_job)?;
            Ok(JobExecution::Delivered(None))
        })
    }
}

struct CapturedContext {
    stamp: String,
    retrieval_query: String,
    target: TargetSnapshot,
    scope: CompiledSearchScope,
    writing_rule_version: String,
    vocabulary_revision: i64,
}

struct TargetSnapshot {
    revision: i64,
    snapshot_hash: String,
    query: String,
    document_id: Option<Uuid>,
    source: ContextSource,
}

async fn result_document_id(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    task: &AiTask,
) -> Result<Option<Uuid>, JobExecutionError> {
    match &task.target {
        AiTarget::Document { document_id } | AiTarget::Region { document_id, .. } => {
            Ok(Some(*document_id))
        }
        AiTarget::Discussion { discussion_id } => sqlx::query_scalar(
            "SELECT document_id FROM discussions WHERE workspace_id=$1 AND id=$2",
        )
        .bind(task.workspace_id)
        .bind(discussion_id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(transient_job),
        AiTarget::WorkspaceQuery { .. } => Ok(None),
    }
}

#[derive(Serialize)]
struct CaptureStamp<'a> {
    task: &'a AiTask,
    target_revision: i64,
    target_snapshot_hash: &'a str,
    permission_scope_fingerprint: &'a str,
    writing_rule_version: &'a str,
    vocabulary_revision: i64,
}

async fn capture_target(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    task: &AiTask,
    scope: &CompiledSearchScope,
) -> Result<TargetSnapshot, ContextError> {
    match &task.target {
        AiTarget::Document { document_id } | AiTarget::Region { document_id, .. } => {
            let permission_key =
                draft_permission(scope, *document_id).ok_or(ContextError::PermissionDenied)?;
            let row = sqlx::query(
                "SELECT d.title,dr.revision,dr.content_json FROM documents d JOIN drafts dr ON dr.workspace_id=d.workspace_id AND dr.document_id=d.id WHERE d.workspace_id=$1 AND d.id=$2 AND d.status='ACTIVE'",
            )
            .bind(task.workspace_id)
            .bind(document_id)
            .fetch_optional(&mut **tx)
            .await
            .map_err(|_| ContextError::Storage)?
            .ok_or(ContextError::NotFound)?;
            let revision = row.get::<i64, _>("revision");
            if revision != task.expected_revision {
                return Err(ContextError::Stale);
            }
            let content: Value = row.get("content_json");
            let regions = extract_search_regions(&content).ok_or(ContextError::Storage)?;
            let selected = match &task.target {
                AiTarget::Region { region, .. } => region_uuid(region)
                    .and_then(|id| regions.iter().find(|item| item.id == id))
                    .map(|region| (Some(region.id), region.body.clone()))
                    .ok_or(ContextError::NotFound)?,
                _ => (
                    None,
                    regions
                        .iter()
                        .map(|region| region.body.as_str())
                        .collect::<Vec<_>>()
                        .join("\n"),
                ),
            };
            let hash = snapshot_hash(&content);
            let stable_id = selected.0.map_or_else(
                || format!("draft:{document_id}"),
                |region| format!("draft:{document_id}:{region}"),
            );
            let source = source(ContextSourceInput {
                kind: ContextSourceKind::Draft,
                stable_id,
                document_id: Some(*document_id),
                region_id: selected.0,
                version: None,
                draft_revision: Some(revision),
                authority: SourceAuthority::UserExplicit,
                include_reason: IncludeReason::CurrentTarget,
                snapshot_hash: hash.clone(),
                snapshot_text: selected.1,
                permission_key: Some(permission_key),
                source_revision: revision,
            })?;
            Ok(TargetSnapshot {
                revision,
                snapshot_hash: hash,
                query: task
                    .instruction
                    .clone()
                    .unwrap_or_else(|| row.get::<String, _>("title")),
                document_id: Some(*document_id),
                source,
            })
        }
        AiTarget::Discussion { discussion_id } => {
            let row = sqlx::query("SELECT d.id AS document_id,x.title,x.revision FROM discussions x JOIN documents d ON d.workspace_id=x.workspace_id AND d.id=x.document_id WHERE x.workspace_id=$1 AND x.id=$2 AND d.status='ACTIVE'")
                .bind(task.workspace_id).bind(discussion_id).fetch_optional(&mut **tx).await.map_err(|_| ContextError::Storage)?.ok_or(ContextError::NotFound)?;
            let document_id = row.get::<Uuid, _>("document_id");
            let permission_key =
                draft_permission(scope, document_id).ok_or(ContextError::PermissionDenied)?;
            let revision = row.get::<i64, _>("revision");
            if revision != task.expected_revision {
                return Err(ContextError::Stale);
            }
            let text = discussion_text(tx, task.workspace_id, *discussion_id).await?;
            let hash = digest(text.as_bytes());
            let source = source(ContextSourceInput {
                kind: ContextSourceKind::Discussion,
                stable_id: format!("discussion:{discussion_id}"),
                document_id: Some(document_id),
                region_id: None,
                version: None,
                draft_revision: None,
                authority: SourceAuthority::DiscussionConfirmed,
                include_reason: IncludeReason::CurrentTarget,
                snapshot_hash: hash.clone(),
                snapshot_text: text,
                permission_key: Some(permission_key),
                source_revision: revision,
            })?;
            Ok(TargetSnapshot {
                revision,
                snapshot_hash: hash,
                query: task.instruction.clone().unwrap_or_else(|| row.get("title")),
                document_id: Some(document_id),
                source,
            })
        }
        AiTarget::WorkspaceQuery { question } => {
            let hash = digest(question.as_bytes());
            let source = source(ContextSourceInput {
                kind: ContextSourceKind::UserInput,
                stable_id: format!("query:{hash}"),
                document_id: None,
                region_id: None,
                version: None,
                draft_revision: None,
                authority: SourceAuthority::UserExplicit,
                include_reason: IncludeReason::UserProvided,
                snapshot_hash: hash.clone(),
                snapshot_text: question.clone(),
                permission_key: None,
                source_revision: 0,
            })?;
            Ok(TargetSnapshot {
                revision: 0,
                snapshot_hash: hash,
                query: question.clone(),
                document_id: None,
                source,
            })
        }
    }
}

fn direct_sources(
    target: &TargetSnapshot,
    _task: &AiTask,
    _scope: &CompiledSearchScope,
) -> Result<Vec<ContextSource>, ContextError> {
    Ok(vec![target.source.clone()])
}

async fn append_discussions(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    workspace: Uuid,
    document: Uuid,
    scope: &CompiledSearchScope,
    sources: &mut Vec<ContextSource>,
) -> Result<(), ContextError> {
    let permission_key = draft_permission(scope, document).ok_or(ContextError::PermissionDenied)?;
    let rows = sqlx::query("SELECT id,revision FROM discussions WHERE workspace_id=$1 AND document_id=$2 AND status='OPEN' ORDER BY created_at,id LIMIT 20")
        .bind(workspace).bind(document).fetch_all(&mut **tx).await.map_err(|_| ContextError::Storage)?;
    for row in rows {
        let id = row.get::<Uuid, _>("id");
        let text = discussion_text(tx, workspace, id).await?;
        if text.is_empty() {
            continue;
        }
        sources.push(source(ContextSourceInput {
            kind: ContextSourceKind::Discussion,
            stable_id: format!("discussion:{id}"),
            document_id: Some(document),
            region_id: None,
            version: None,
            draft_revision: None,
            authority: SourceAuthority::DiscussionConfirmed,
            include_reason: IncludeReason::DiscussionContext,
            snapshot_hash: digest(text.as_bytes()),
            snapshot_text: text,
            permission_key: Some(permission_key.clone()),
            source_revision: row.get("revision"),
        })?);
    }
    Ok(())
}

async fn discussion_text(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    workspace: Uuid,
    discussion: Uuid,
) -> Result<String, ContextError> {
    let rows = sqlx::query("SELECT body_json FROM messages WHERE workspace_id=$1 AND discussion_id=$2 AND deleted_at IS NULL ORDER BY created_at,id LIMIT 200")
        .bind(workspace).bind(discussion).fetch_all(&mut **tx).await.map_err(|_| ContextError::Storage)?;
    Ok(rows
        .iter()
        .map(|row| plain_text(&row.get::<Value, _>("body_json")))
        .filter(|text| !text.is_empty())
        .collect::<Vec<_>>()
        .join("\n"))
}

async fn append_references(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    workspace: Uuid,
    document: Uuid,
    scope: &CompiledSearchScope,
    sources: &mut Vec<ContextSource>,
) -> Result<(), ContextError> {
    let rows = sqlx::query("SELECT target_id FROM references_graph WHERE workspace_id=$1 AND source_kind='DOCUMENT' AND source_id=$2 AND target_kind='DOCUMENT' ORDER BY target_id LIMIT 50")
        .bind(workspace).bind(document).fetch_all(&mut **tx).await.map_err(|_| ContextError::Storage)?;
    for row in rows {
        let Some(target) = row.get::<String, _>("target_id").parse::<Uuid>().ok() else {
            continue;
        };
        let Some(key) = published_permission(scope, target) else {
            continue;
        };
        let row = sqlx::query("SELECT pv.number,pv.content_json FROM documents d JOIN published_versions pv ON pv.workspace_id=d.workspace_id AND pv.id=d.current_version_id WHERE d.workspace_id=$1 AND d.id=$2 AND d.status='ACTIVE'")
            .bind(workspace).bind(target).fetch_optional(&mut **tx).await.map_err(|_| ContextError::Storage)?;
        let Some(row) = row else { continue };
        let content: Value = row.get("content_json");
        let hash = snapshot_hash(&content);
        for region in extract_search_regions(&content).ok_or(ContextError::Storage)? {
            sources.push(source(ContextSourceInput {
                kind: ContextSourceKind::PublishedRegion,
                stable_id: projection_id(workspace, SearchSourceKind::Published, target, region.id),
                document_id: Some(target),
                region_id: Some(region.id),
                version: Some(row.get("number")),
                draft_revision: None,
                authority: SourceAuthority::Official,
                include_reason: IncludeReason::ExplicitReference,
                snapshot_hash: hash.clone(),
                snapshot_text: region.body,
                permission_key: Some(key.clone()),
                source_revision: row.get("number"),
            })?);
        }
    }
    Ok(())
}

async fn append_vocabulary(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    workspace: Uuid,
    revision: i64,
    sources: &mut Vec<ContextSource>,
) -> Result<(), ContextError> {
    let rows = sqlx::query("SELECT c.id,c.canonical_term,c.definition,c.revision,COALESCE(jsonb_agg(jsonb_build_object('term',t.term,'kind',t.kind) ORDER BY t.kind,t.normalized_term) FILTER (WHERE t.term IS NOT NULL),'[]'::jsonb) AS terms FROM vocabulary_concepts c LEFT JOIN vocabulary_terms t ON t.workspace_id=c.workspace_id AND t.concept_id=c.id WHERE c.workspace_id=$1 AND c.status='ACTIVE' GROUP BY c.id ORDER BY c.canonical_term,c.id LIMIT 200")
        .bind(workspace).fetch_all(&mut **tx).await.map_err(|_| ContextError::Storage)?;
    if rows.is_empty() {
        return Ok(());
    }
    let values = rows.iter().map(|row| serde_json::json!({"id":row.get::<Uuid,_>("id"),"term":row.get::<String,_>("canonical_term"),"definition":row.get::<String,_>("definition"),"revision":row.get::<i64,_>("revision"),"terms":row.get::<Value,_>("terms")})).collect::<Vec<_>>();
    let text = serde_json::to_string(&values).map_err(|_| ContextError::Storage)?;
    sources.push(source(ContextSourceInput {
        kind: ContextSourceKind::Vocabulary,
        stable_id: format!("vocabulary:{workspace}:{revision}"),
        document_id: None,
        region_id: None,
        version: None,
        draft_revision: None,
        authority: SourceAuthority::Vocabulary,
        include_reason: IncludeReason::VocabularyPolicy,
        snapshot_hash: digest(text.as_bytes()),
        snapshot_text: text,
        permission_key: None,
        source_revision: revision,
    })?);
    Ok(())
}

async fn materialize_search_source(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    workspace: Uuid,
    scope: &CompiledSearchScope,
    candidate: &SearchSource,
) -> Result<Option<ContextSource>, ContextError> {
    let key = match candidate.kind {
        SearchSourceKind::Published => published_permission(scope, candidate.document_id),
        SearchSourceKind::Draft => draft_permission(scope, candidate.document_id),
    };
    let Some(key) = key else { return Ok(None) };
    let row = match candidate.kind {
        SearchSourceKind::Published => sqlx::query("SELECT pv.number AS revision,pv.number,pv.content_json FROM documents d JOIN published_versions pv ON pv.workspace_id=d.workspace_id AND pv.id=d.current_version_id WHERE d.workspace_id=$1 AND d.id=$2 AND d.status='ACTIVE' AND pv.number=$3")
            .bind(workspace).bind(candidate.document_id).bind(candidate.version).fetch_optional(&mut **tx).await,
        SearchSourceKind::Draft => sqlx::query("SELECT dr.revision,NULL::bigint AS number,dr.content_json FROM documents d JOIN drafts dr ON dr.workspace_id=d.workspace_id AND dr.document_id=d.id WHERE d.workspace_id=$1 AND d.id=$2 AND d.status='ACTIVE' AND dr.revision=$3")
            .bind(workspace).bind(candidate.document_id).bind(candidate.draft_revision).fetch_optional(&mut **tx).await,
    }.map_err(|_| ContextError::Storage)?;
    let Some(row) = row else { return Ok(None) };
    let content: Value = row.get("content_json");
    if snapshot_hash(&content) != candidate.snapshot_hash
        || projection_id(
            workspace,
            candidate.kind,
            candidate.document_id,
            candidate.region_id,
        ) != candidate.stable_id
    {
        return Ok(None);
    }
    let Some(region) = extract_search_regions(&content)
        .ok_or(ContextError::Storage)?
        .into_iter()
        .find(|region| region.id == candidate.region_id)
    else {
        return Ok(None);
    };
    let revision = row.get::<i64, _>("revision");
    source(ContextSourceInput {
        kind: match candidate.kind {
            SearchSourceKind::Published => ContextSourceKind::PublishedRegion,
            SearchSourceKind::Draft => ContextSourceKind::Draft,
        },
        stable_id: candidate.stable_id.clone(),
        document_id: Some(candidate.document_id),
        region_id: Some(candidate.region_id),
        version: candidate.version,
        draft_revision: candidate.draft_revision,
        authority: if candidate.kind == SearchSourceKind::Published {
            SourceAuthority::Official
        } else {
            SourceAuthority::RelatedInternal
        },
        include_reason: IncludeReason::RetrievedRelated,
        snapshot_hash: candidate.snapshot_hash.clone(),
        snapshot_text: region.body,
        permission_key: Some(key),
        source_revision: revision,
    })
    .map(Some)
}

struct ContextSourceInput {
    kind: ContextSourceKind,
    stable_id: String,
    document_id: Option<Uuid>,
    region_id: Option<Uuid>,
    version: Option<i64>,
    draft_revision: Option<i64>,
    authority: SourceAuthority,
    include_reason: IncludeReason,
    snapshot_hash: String,
    snapshot_text: String,
    permission_key: Option<String>,
    source_revision: i64,
}

fn source(input: ContextSourceInput) -> Result<ContextSource, ContextError> {
    let mut value = ContextSource {
        source_id: Uuid::nil(),
        kind: input.kind,
        stable_id: input.stable_id,
        document_id: input.document_id,
        region_id: input.region_id,
        version: input.version,
        draft_revision: input.draft_revision,
        authority: input.authority,
        include_reason: input.include_reason,
        snapshot_hash: input.snapshot_hash,
        snapshot_text: input.snapshot_text,
        permission_key: input.permission_key,
        source_revision: input.source_revision,
        retrieved_at: None,
        included: true,
    };
    value.assign_id();
    value.is_valid().then_some(value).ok_or(ContextError::Limit)
}

fn draft_permission(scope: &CompiledSearchScope, document: Uuid) -> Option<String> {
    scope
        .draft_keys
        .iter()
        .find(|key| key.document_id == document)
        .map(|key| key.composite_key.clone())
}

fn published_permission(scope: &CompiledSearchScope, document: Uuid) -> Option<String> {
    scope
        .published_keys
        .iter()
        .find(|key| key.document_id == document)
        .map(|key| key.composite_key.clone())
}

fn region_uuid(value: &Value) -> Option<Uuid> {
    ["regionId", "blockId", "id"].into_iter().find_map(|key| {
        value
            .get(key)
            .and_then(Value::as_str)
            .and_then(|value| Uuid::parse_str(value).ok())
    })
}

fn plain_text(value: &Value) -> String {
    let mut values = Vec::new();
    collect_text(value, &mut values);
    values
        .join(" ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn collect_text(value: &Value, output: &mut Vec<String>) {
    match value {
        Value::Array(values) => values.iter().for_each(|value| collect_text(value, output)),
        Value::Object(object) => {
            if object.get("type").and_then(Value::as_str) == Some("text")
                && let Some(text) = object.get("text").and_then(Value::as_str)
            {
                output.push(text.to_owned());
            }
            for key in ["summary", "children", "root"] {
                if let Some(value) = object.get(key) {
                    collect_text(value, output);
                }
            }
        }
        _ => {}
    }
}

fn deduplicate_sources(sources: &mut Vec<ContextSource>) {
    let mut seen = BTreeSet::new();
    sources.retain(|source| seen.insert(source.source_id));
}

fn stamp(value: &impl Serialize) -> Result<String, ContextError> {
    serde_json::to_vec(value)
        .map(digest)
        .map_err(|_| ContextError::Storage)
}

fn digest(value: impl AsRef<[u8]>) -> String {
    hex::encode(Sha256::digest(value.as_ref()))
}

fn scope_error(error: adoc_application::search::SearchRetrievalError) -> ContextError {
    use adoc_application::search::SearchRetrievalError;
    match error {
        SearchRetrievalError::Validation | SearchRetrievalError::CursorExpired => {
            ContextError::Validation
        }
        SearchRetrievalError::WorkspaceNotFound => ContextError::NotFound,
        SearchRetrievalError::Unavailable => ContextError::RetrievalUnavailable,
        SearchRetrievalError::Internal => ContextError::StorageAt("permission_scope"),
    }
}

async fn validate_artifact_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    task: &AiTask,
    artifact: &ContextArtifact,
    scope: &CompiledSearchScope,
) -> Result<(), ContextError> {
    if artifact.sources.iter().any(|source| {
        source.included
            && source.document_id.is_some_and(|document| {
                let expected = match source.kind {
                    ContextSourceKind::Draft => draft_permission(scope, document),
                    ContextSourceKind::PublishedRegion => published_permission(scope, document),
                    ContextSourceKind::Discussion => draft_permission(scope, document),
                    ContextSourceKind::Vocabulary | ContextSourceKind::UserInput => None,
                };
                source.permission_key != expected
            })
    }) {
        return Err(ContextError::Stale);
    }
    match task.target {
        AiTarget::Document { document_id } | AiTarget::Region { document_id, .. } => {
            let current: Option<i64> = sqlx::query_scalar("SELECT dr.revision FROM drafts dr JOIN documents d ON d.workspace_id=dr.workspace_id AND d.id=dr.document_id WHERE dr.workspace_id=$1 AND dr.document_id=$2 AND d.status='ACTIVE'")
                .bind(task.workspace_id).bind(document_id).fetch_optional(&mut **tx).await.map_err(|_| ContextError::StorageAt("artifact_target_revision"))?;
            if current != Some(task.expected_revision)
                || draft_permission(scope, document_id).is_none()
            {
                return Err(ContextError::Stale);
            }
        }
        AiTarget::Discussion { discussion_id } => {
            let row = sqlx::query(
                "SELECT revision,document_id FROM discussions WHERE workspace_id=$1 AND id=$2",
            )
            .bind(task.workspace_id)
            .bind(discussion_id)
            .fetch_optional(&mut **tx)
            .await
            .map_err(|_| ContextError::StorageAt("artifact_discussion_revision"))?
            .ok_or(ContextError::Stale)?;
            if row.get::<i64, _>("revision") != task.expected_revision
                || draft_permission(scope, row.get("document_id")).is_none()
            {
                return Err(ContextError::Stale);
            }
        }
        AiTarget::WorkspaceQuery { .. } => {}
    }
    Ok(())
}

async fn load_context_sources_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ai_job_id: Uuid,
    scope: &CompiledSearchScope,
) -> Result<Vec<ContextSource>, JobExecutionError> {
    let rows = sqlx::query("SELECT source_kind,source_id,stable_id,authority,include_reason,snapshot_hash,snapshot_text,source_revision,permission_key,included,metadata_json FROM ai_context_sources WHERE job_id=$1 ORDER BY ordinal")
        .bind(ai_job_id).fetch_all(&mut **tx).await.map_err(transient_job)?;
    let mut sources = Vec::with_capacity(rows.len());
    for row in &rows {
        let metadata: Value = row.get("metadata_json");
        let kind: ContextSourceKind = parse_enum(row.get("source_kind"))?;
        let document_id = metadata
            .get("documentId")
            .cloned()
            .map(serde_json::from_value)
            .transpose()
            .map_err(|_| JobExecutionError::Permanent("AI_CONTEXT_INVALID"))?
            .flatten();
        let permission_key: Option<String> = row.get("permission_key");
        if row.get::<bool, _>("included")
            && let Some(document) = document_id
        {
            let expected = match kind {
                ContextSourceKind::Draft => draft_permission(scope, document),
                ContextSourceKind::PublishedRegion => published_permission(scope, document),
                ContextSourceKind::Discussion => draft_permission(scope, document),
                ContextSourceKind::Vocabulary | ContextSourceKind::UserInput => None,
            };
            if permission_key != expected {
                return Err(JobExecutionError::Permanent("AI_PERMISSION_CHANGED"));
            }
        }
        let value = ContextSource {
            source_id: row
                .get::<String, _>("source_id")
                .parse()
                .map_err(|_| JobExecutionError::Permanent("AI_CONTEXT_INVALID"))?,
            kind,
            stable_id: row.get("stable_id"),
            document_id,
            region_id: optional_uuid(&metadata, "regionId")?,
            version: optional_i64(&metadata, "version")?,
            draft_revision: optional_i64(&metadata, "draftRevision")?,
            authority: parse_enum(row.get("authority"))?,
            include_reason: parse_enum(row.get("include_reason"))?,
            snapshot_hash: row.get("snapshot_hash"),
            snapshot_text: row.get("snapshot_text"),
            permission_key,
            source_revision: row.get("source_revision"),
            retrieved_at: metadata
                .get("retrievedAt")
                .cloned()
                .map(serde_json::from_value)
                .transpose()
                .map_err(|_| JobExecutionError::Permanent("AI_CONTEXT_INVALID"))?
                .flatten(),
            included: row.get("included"),
        };
        if !value.is_valid() {
            return Err(JobExecutionError::Permanent("AI_CONTEXT_INVALID"));
        }
        validate_source_snapshot_tx(tx, scope.workspace_id, &value).await?;
        sources.push(value);
    }
    Ok(sources)
}

async fn validate_source_snapshot_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    workspace: Uuid,
    source: &ContextSource,
) -> Result<(), JobExecutionError> {
    let valid = match source.kind {
        ContextSourceKind::Draft => {
            let Some(document) = source.document_id else {
                return Err(JobExecutionError::Permanent("AI_CONTEXT_INVALID"));
            };
            let row = sqlx::query(
                "SELECT revision,content_json FROM drafts WHERE workspace_id=$1 AND document_id=$2",
            )
            .bind(workspace)
            .bind(document)
            .fetch_optional(&mut **tx)
            .await
            .map_err(transient_job)?;
            row.is_some_and(|row| {
                row.get::<i64, _>("revision") == source.source_revision
                    && snapshot_hash(&row.get::<Value, _>("content_json")) == source.snapshot_hash
            })
        }
        ContextSourceKind::PublishedRegion => {
            let (Some(document), Some(version)) = (source.document_id, source.version) else {
                return Err(JobExecutionError::Permanent("AI_CONTEXT_INVALID"));
            };
            let content: Option<Value> = sqlx::query_scalar("SELECT content_json FROM published_versions WHERE workspace_id=$1 AND document_id=$2 AND number=$3")
                .bind(workspace).bind(document).bind(version).fetch_optional(&mut **tx).await.map_err(transient_job)?;
            content.is_some_and(|content| snapshot_hash(&content) == source.snapshot_hash)
        }
        ContextSourceKind::Discussion => {
            let id = source
                .stable_id
                .strip_prefix("discussion:")
                .and_then(|value| Uuid::parse_str(value).ok())
                .ok_or(JobExecutionError::Permanent("AI_CONTEXT_INVALID"))?;
            let revision: Option<i64> = sqlx::query_scalar(
                "SELECT revision FROM discussions WHERE workspace_id=$1 AND id=$2",
            )
            .bind(workspace)
            .bind(id)
            .fetch_optional(&mut **tx)
            .await
            .map_err(transient_job)?;
            let text = discussion_text(tx, workspace, id)
                .await
                .map_err(|_| JobExecutionError::Transient("AI_STORAGE_UNAVAILABLE"))?;
            revision == Some(source.source_revision)
                && digest(text.as_bytes()) == source.snapshot_hash
        }
        ContextSourceKind::Vocabulary => {
            let revision: i64 = sqlx::query_scalar(
                "SELECT COALESCE(max(revision),0) FROM vocabulary_concepts WHERE workspace_id=$1",
            )
            .bind(workspace)
            .fetch_one(&mut **tx)
            .await
            .map_err(transient_job)?;
            revision == source.source_revision
        }
        ContextSourceKind::UserInput => true,
    };
    valid
        .then_some(())
        .ok_or(JobExecutionError::Permanent("AI_CONTEXT_STALE"))
}

async fn require_membership(
    pool: &PgPool,
    actor: Uuid,
    workspace: Uuid,
) -> Result<(), ContextError> {
    let allowed: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM memberships m JOIN workspaces w ON w.id=m.workspace_id WHERE m.workspace_id=$1 AND m.user_id=$2 AND m.status='ACTIVE' AND w.status IN ('ACTIVE','DELETION_SCHEDULED'))")
        .bind(workspace).bind(actor).fetch_one(pool).await.map_err(|_| ContextError::Storage)?;
    allowed.then_some(()).ok_or(ContextError::NotFound)
}

fn ai_job_view(row: &sqlx::postgres::PgRow) -> Result<AiJobView, ContextError> {
    let task: AiTask = serde_json::from_value(row.get("target_json"))
        .map_err(|_| ContextError::StorageAt("ai_job_target_decode"))?;
    Ok(AiJobView {
        id: row.get("id"),
        kind: parse_enum_context(row.get("kind"))?,
        target: task.target,
        status: parse_enum_context(row.get("status"))?,
        sequence: row.get("revision"),
        revision: row.get("revision"),
        result: row.try_get("result_json").ok(),
        proposal_id: row.try_get::<Option<Uuid>, _>("proposal_id").ok().flatten(),
        error_code: row.get("error_code"),
    })
}

fn priority_bucket(priority: i16) -> JobPriorityBucket {
    if priority >= 67 {
        JobPriorityBucket::Interactive
    } else if priority >= 34 {
        JobPriorityBucket::Normal
    } else {
        JobPriorityBucket::Background
    }
}

fn enum_text(value: &impl Serialize) -> Result<String, ContextError> {
    serde_json::to_value(value)
        .ok()
        .and_then(|value| value.as_str().map(ToOwned::to_owned))
        .ok_or(ContextError::Storage)
}

fn parse_enum<T: serde::de::DeserializeOwned>(value: String) -> Result<T, JobExecutionError> {
    serde_json::from_value(Value::String(value))
        .map_err(|_| JobExecutionError::Permanent("AI_CONTEXT_INVALID"))
}

fn parse_enum_context<T: serde::de::DeserializeOwned>(value: String) -> Result<T, ContextError> {
    serde_json::from_value(Value::String(value)).map_err(|_| ContextError::Storage)
}

fn optional_uuid(value: &Value, key: &str) -> Result<Option<Uuid>, JobExecutionError> {
    value
        .get(key)
        .cloned()
        .map(serde_json::from_value)
        .transpose()
        .map_err(|_| JobExecutionError::Permanent("AI_CONTEXT_INVALID"))
        .map(Option::flatten)
}

fn optional_i64(value: &Value, key: &str) -> Result<Option<i64>, JobExecutionError> {
    value
        .get(key)
        .cloned()
        .map(serde_json::from_value)
        .transpose()
        .map_err(|_| JobExecutionError::Permanent("AI_CONTEXT_INVALID"))
        .map(Option::flatten)
}

fn ai_job_id(payload: &Value) -> Result<Uuid, JobExecutionError> {
    payload
        .as_object()
        .filter(|value| value.len() == 1)
        .and_then(|value| value.get("aiJobId"))
        .and_then(Value::as_str)
        .and_then(|value| Uuid::parse_str(value).ok())
        .ok_or(JobExecutionError::Permanent("JOB_PAYLOAD_INVALID"))
}

async fn lock_generic_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    job: &Job,
    worker: &str,
) -> Result<String, JobExecutionError> {
    sqlx::query_scalar(
        "SELECT status::text FROM jobs WHERE id=$1 AND sequence=$2 AND lease_owner=$3 FOR UPDATE",
    )
    .bind(job.id)
    .bind(job.sequence)
    .bind(worker)
    .fetch_optional(&mut **tx)
    .await
    .map_err(transient_job)?
    .ok_or(JobExecutionError::Transient("JOB_LEASE_LOST"))
}

async fn finish_generic_success_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    id: Uuid,
    now: chrono::DateTime<chrono::Utc>,
) -> Result<(), JobExecutionError> {
    sqlx::query("UPDATE jobs SET status='SUCCEEDED',sequence=sequence+1,completed_at=$2,updated_at=$2,lease_owner=NULL,lease_until=NULL,last_error_code=NULL WHERE id=$1 AND status IN ('RUNNING','CANCEL_REQUESTED')")
        .bind(id).bind(now).execute(&mut **tx).await.map_err(transient_job)?;
    Ok(())
}

async fn append_ai_job_event_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    workspace_id: Uuid,
    user_id: Uuid,
    ai_job_id: Uuid,
    revision: i64,
    status: &str,
    now: chrono::DateTime<chrono::Utc>,
) -> Result<(), sqlx::Error> {
    let event_id = Uuid::now_v7();
    let correlation_id = ai_job_id.to_string();
    sqlx::query("INSERT INTO workspace_sequences(workspace_id) VALUES($1) ON CONFLICT(workspace_id) DO NOTHING")
        .bind(workspace_id).execute(&mut **tx).await?;
    let projection_sequence: i64 = sqlx::query_scalar("UPDATE workspace_sequences SET next_projection_sequence=next_projection_sequence+1 WHERE workspace_id=$1 RETURNING next_projection_sequence-1")
        .bind(workspace_id).fetch_one(&mut **tx).await?;
    sqlx::query("INSERT INTO outbox_events(id,workspace_id,aggregate_kind,aggregate_id,sequence,event_type,event_version,projection_sequence,payload_json,audience_kind,audience_id,correlation_id,occurred_at) VALUES($1,$2,'AIJob',$3,$4,'AIJobChanged.v1',1,$5,$6,'USER',$7,$8,$9)")
        .bind(event_id).bind(workspace_id).bind(ai_job_id).bind(revision + 1).bind(projection_sequence)
        .bind(serde_json::json!({"jobId":ai_job_id,"status":status,"jobSequence":revision + 1}))
        .bind(user_id).bind(&correlation_id).bind(now).execute(&mut **tx).await?;
    sqlx::query("INSERT INTO jobs(id,workspace_id,kind,payload_json,dedupe_key,status,priority,sequence,attempt,max_attempts,run_after,correlation_id,created_at,updated_at) VALUES($1,$2,'OUTBOX_TO_STREAM',$3,$4,'QUEUED',50,1,0,5,$5,$6,$5,$5) ON CONFLICT(kind,dedupe_key) DO NOTHING")
        .bind(Uuid::now_v7()).bind(workspace_id).bind(serde_json::json!({"outboxEventId":event_id}))
        .bind(format!("workspace-stream:{event_id}")).bind(now).bind(correlation_id)
        .execute(&mut **tx).await?;
    Ok(())
}

async fn finish_cancelled_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ai_job_id: Uuid,
    generic_job_id: Uuid,
    now: chrono::DateTime<chrono::Utc>,
) -> Result<(), JobExecutionError> {
    let row = sqlx::query("UPDATE ai_jobs SET status='CANCELLED',error_code='AI_CANCELLED',revision=revision+1,completed_at=$2 WHERE id=$1 AND status IN ('QUEUED','RUNNING','CANCEL_REQUESTED') RETURNING workspace_id,user_id,revision")
        .bind(ai_job_id).bind(now).fetch_optional(&mut **tx).await.map_err(transient_job)?;
    sqlx::query("UPDATE jobs SET status='CANCELLED',sequence=sequence+1,cancel_requested_at=COALESCE(cancel_requested_at,$2),completed_at=$2,updated_at=$2,lease_owner=NULL,lease_until=NULL WHERE id=$1 AND status IN ('RUNNING','CANCEL_REQUESTED')")
        .bind(generic_job_id).bind(now).execute(&mut **tx).await.map_err(transient_job)?;
    if let Some(row) = row {
        append_ai_job_event_tx(
            tx,
            row.get("workspace_id"),
            row.get("user_id"),
            ai_job_id,
            row.get("revision"),
            "CANCELLED",
            now,
        )
        .await
        .map_err(transient_job)?;
    }
    Ok(())
}

async fn finish_pre_runtime_failure_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ai_job_id: Uuid,
    generic_job_id: Uuid,
    code: &str,
    now: chrono::DateTime<chrono::Utc>,
) -> Result<(), JobExecutionError> {
    let row = sqlx::query("UPDATE ai_jobs SET status='FAILED',error_code=$2,revision=revision+1,completed_at=$3 WHERE id=$1 AND status IN ('QUEUED','RUNNING') RETURNING workspace_id,user_id,revision")
        .bind(ai_job_id).bind(code).bind(now).fetch_optional(&mut **tx).await.map_err(transient_job)?;
    if let Some(row) = row {
        append_ai_job_event_tx(
            tx,
            row.get("workspace_id"),
            row.get("user_id"),
            ai_job_id,
            row.get("revision"),
            "FAILED",
            now,
        )
        .await
        .map_err(transient_job)?;
    }
    finish_generic_success_tx(tx, generic_job_id, now).await
}

fn transient_job(_error: sqlx::Error) -> JobExecutionError {
    JobExecutionError::Transient("AI_STORAGE_UNAVAILABLE")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_text_ignores_non_content_metadata() {
        let value = serde_json::json!({"id":"secret","root":{"children":[{"type":"paragraph","children":[{"type":"text","text":"hello"}]}]}});
        assert_eq!(plain_text(&value), "hello");
    }
}
