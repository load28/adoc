use std::collections::BTreeSet;

use adoc_application::{
    ai::{
        ProposalApply, ProposalReject, ProposalStatus, ProposalView, WritingConfigurationUpdate,
        WritingConfigurationView, WritingIntelligenceRepository, validate_dependency_selection,
    },
    document::{DocumentOperation, MutationResult},
    governance::GovernanceError,
    operations::{
        AuditAction, AuditEventInput, AuditTarget, AuditTargetKind, EventAudience, StreamAccess,
    },
};
use adoc_ports::BoxFuture;
use serde_json::{Value, json};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use super::{
    PostgresStore, append_audit_event,
    document::{DraftApplyTx, apply_draft_operations_tx, require_access},
    governance::{OutboxEvent, append_event, begin_workspace, complete_workspace, map_store},
};

#[derive(Clone)]
pub struct PostgresWritingIntelligenceRepository {
    pool: PgPool,
}

impl PostgresWritingIntelligenceRepository {
    #[must_use]
    pub fn new(store: &PostgresStore) -> Self {
        Self {
            pool: store.pool().clone(),
        }
    }
}

impl WritingIntelligenceRepository for PostgresWritingIntelligenceRepository {
    fn get_proposal<'a>(
        &'a self,
        actor: Uuid,
        workspace: Uuid,
        proposal: Uuid,
    ) -> BoxFuture<'a, Result<ProposalView, GovernanceError>> {
        Box::pin(async move {
            let mut tx = self.pool.begin().await.map_err(map_store)?;
            let row = sqlx::query("SELECT id,job_id,document_id,base_revision,operations_json,status::text,revision,applied_revision,applied_operation_ids,created_at,resolved_at FROM proposals WHERE workspace_id=$1 AND id=$2")
                .bind(workspace).bind(proposal).fetch_optional(&mut *tx).await.map_err(map_store)?.ok_or(GovernanceError::ProposalNotFound)?;
            require_access(
                &mut tx,
                actor,
                workspace,
                row.get("document_id"),
                adoc_application::permission::Access::Contributor,
                false,
            )
            .await
            .map_err(|_| GovernanceError::ProposalNotFound)?;
            let view = proposal_view(&row)?;
            tx.commit().await.map_err(map_store)?;
            Ok(view)
        })
    }

    fn apply_proposal<'a>(
        &'a self,
        input: ProposalApply,
    ) -> BoxFuture<'a, Result<MutationResult, GovernanceError>> {
        Box::pin(async move {
            let mut tx = self.pool.begin().await.map_err(map_store)?;
            if let Some(replay) =
                begin_workspace::<MutationResult>(&mut tx, input.workspace_id, &input.command)
                    .await?
            {
                tx.commit().await.map_err(map_store)?;
                return Ok(replay);
            }
            let row = sqlx::query("SELECT id,job_id,document_id,base_revision,operations_json,status::text,revision,applied_revision,applied_operation_ids,created_at,resolved_at FROM proposals WHERE workspace_id=$1 AND id=$2 FOR UPDATE")
                .bind(input.workspace_id).bind(input.proposal_id).fetch_optional(&mut *tx).await.map_err(map_store)?.ok_or(GovernanceError::ProposalNotFound)?;
            let status = row.get::<String, _>("status");
            if status == "STALE" {
                return Err(GovernanceError::ProposalStale);
            }
            if status != "OPEN" {
                return Err(GovernanceError::ProposalStateInvalid);
            }
            let document_id: Uuid = row.get("document_id");
            let base_revision: i64 = row.get("base_revision");
            if input.expected_draft_revision != base_revision {
                return Err(GovernanceError::ProposalStale);
            }
            let operations: Vec<DocumentOperation> =
                serde_json::from_value(row.get("operations_json"))
                    .map_err(|_| GovernanceError::Internal)?;
            let selected: BTreeSet<Uuid> = input
                .operation_ids
                .clone()
                .unwrap_or_else(|| {
                    operations
                        .iter()
                        .map(|operation| operation.base().op_id)
                        .collect()
                })
                .into_iter()
                .collect();
            if input
                .operation_ids
                .as_ref()
                .is_some_and(|ids| ids.len() != selected.len())
            {
                return Err(GovernanceError::ProposalDependencyInvalid);
            }
            validate_dependency_selection(&operations, Some(&selected))
                .map_err(|_| GovernanceError::ProposalDependencyInvalid)?;
            let chosen: Vec<_> = operations
                .into_iter()
                .filter(|operation| selected.contains(&operation.base().op_id))
                .collect();
            let result = match apply_draft_operations_tx(
                &mut tx,
                DraftApplyTx {
                    actor_id: input.actor_id,
                    workspace_id: input.workspace_id,
                    document_id,
                    client_instance_id: input.client_instance_id,
                    expected_draft_revision: input.expected_draft_revision,
                    token_hash: &input.token_hash,
                    operations: chosen,
                    now: input.command.now,
                },
            )
            .await
            {
                Err(
                    GovernanceError::RevisionConflict { .. }
                    | GovernanceError::OperationPreconditionFailed,
                ) => return Err(GovernanceError::ProposalStale),
                other => other?,
            };
            let proposal_revision: i64 = sqlx::query_scalar("UPDATE proposals SET status='APPLIED',revision=revision+1,applied_revision=$3,applied_operation_ids=$4,resolved_at=$5 WHERE workspace_id=$1 AND id=$2 AND status='OPEN' RETURNING revision")
                .bind(input.workspace_id).bind(input.proposal_id).bind(result.revision).bind(&result.applied_operation_ids).bind(input.command.now).fetch_one(&mut *tx).await.map_err(map_store)?;
            append_event(&mut tx, OutboxEvent {
                workspace_id: input.workspace_id,
                aggregate_kind: "Proposal",
                aggregate_id: input.proposal_id,
                sequence: proposal_revision + 1,
                event_type: "ProposalApplied.v1",
                payload: json!({"proposalId":input.proposal_id,"documentId":document_id,"appliedOperationIds":result.applied_operation_ids,"resultRevision":result.revision}),
                audience: EventAudience::document(document_id, StreamAccess::Contributor),
                occurred_at: input.command.now,
            }).await?;
            append_audit_event(
                &mut tx,
                AuditEventInput::user(
                    input.workspace_id,
                    input.actor_id,
                    AuditAction::AiProposalApplied,
                    AuditTarget {
                        kind: AuditTargetKind::AiProposal,
                        id: input.proposal_id,
                    },
                    input.command.now,
                    &input.command.idempotency_key,
                ),
            )
            .await?;
            complete_workspace(&mut tx, input.workspace_id, &input.command, 200, &result).await?;
            tx.commit().await.map_err(map_store)?;
            Ok(result)
        })
    }

    fn reject_proposal<'a>(
        &'a self,
        input: ProposalReject,
    ) -> BoxFuture<'a, Result<ProposalView, GovernanceError>> {
        Box::pin(async move {
            let mut tx = self.pool.begin().await.map_err(map_store)?;
            if let Some(replay) =
                begin_workspace::<ProposalView>(&mut tx, input.workspace_id, &input.command).await?
            {
                tx.commit().await.map_err(map_store)?;
                return Ok(replay);
            }
            let row = sqlx::query("SELECT id,job_id,document_id,base_revision,operations_json,status::text,revision,applied_revision,applied_operation_ids,created_at,resolved_at FROM proposals WHERE workspace_id=$1 AND id=$2 FOR UPDATE")
                .bind(input.workspace_id).bind(input.proposal_id).fetch_optional(&mut *tx).await.map_err(map_store)?.ok_or(GovernanceError::ProposalNotFound)?;
            let document_id: Uuid = row.get("document_id");
            require_access(
                &mut tx,
                input.actor_id,
                input.workspace_id,
                document_id,
                adoc_application::permission::Access::Contributor,
                false,
            )
            .await
            .map_err(|_| GovernanceError::ProposalNotFound)?;
            if row.get::<String, _>("status") != "OPEN" {
                return Err(GovernanceError::ProposalStateInvalid);
            }
            if row.get::<i64, _>("revision") != input.expected_proposal_revision {
                return Err(GovernanceError::RevisionConflict {
                    current_revision: row.get("revision"),
                });
            }
            let updated = sqlx::query("UPDATE proposals SET status='REJECTED',revision=revision+1,resolved_at=$3,validation_json=validation_json || jsonb_build_object('rejectionReasonRecorded',true) WHERE workspace_id=$1 AND id=$2 RETURNING id,job_id,document_id,base_revision,operations_json,status::text,revision,applied_revision,applied_operation_ids,created_at,resolved_at")
                .bind(input.workspace_id).bind(input.proposal_id).bind(input.command.now).fetch_one(&mut *tx).await.map_err(map_store)?;
            let result = proposal_view(&updated)?;
            complete_workspace(&mut tx, input.workspace_id, &input.command, 200, &result).await?;
            tx.commit().await.map_err(map_store)?;
            Ok(result)
        })
    }

    fn get_writing_configuration<'a>(
        &'a self,
        actor: Uuid,
        workspace: Uuid,
    ) -> BoxFuture<'a, Result<WritingConfigurationView, GovernanceError>> {
        Box::pin(async move {
            require_member(&self.pool, actor, workspace, false).await?;
            let row = sqlx::query("SELECT baseline_version,overrides_json,revision FROM writing_configurations WHERE workspace_id=$1").bind(workspace).fetch_optional(&self.pool).await.map_err(map_store)?;
            row.map_or_else(
                || Ok(default_writing_configuration()),
                |row| writing_configuration(&row),
            )
        })
    }

    fn update_writing_configuration<'a>(
        &'a self,
        input: WritingConfigurationUpdate,
    ) -> BoxFuture<'a, Result<WritingConfigurationView, GovernanceError>> {
        Box::pin(async move {
            let mut tx = self.pool.begin().await.map_err(map_store)?;
            require_member_tx(&mut tx, input.actor_id, input.workspace_id, true).await?;
            if let Some(replay) = begin_workspace::<WritingConfigurationView>(
                &mut tx,
                input.workspace_id,
                &input.command,
            )
            .await?
            {
                tx.commit().await.map_err(map_store)?;
                return Ok(replay);
            }
            if input.input.baseline_version != adoc_application::ai::WRITING_RULE_BASELINE_VERSION
                || !input.input.overrides.is_empty()
            {
                return Err(GovernanceError::WritingConfigurationInvalid);
            }
            let current = sqlx::query(
                "SELECT revision FROM writing_configurations WHERE workspace_id=$1 FOR UPDATE",
            )
            .bind(input.workspace_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(map_store)?;
            let current_revision = current.as_ref().map_or(0, |row| row.get("revision"));
            if current_revision != input.expected_revision {
                return Err(GovernanceError::RevisionConflict { current_revision });
            }
            let revision = current_revision + 1;
            let row = sqlx::query("INSERT INTO writing_configurations(workspace_id,baseline_version,overrides_json,revision,updated_by,updated_at) VALUES($1,$2,'[]'::jsonb,$3,$4,$5) ON CONFLICT(workspace_id) DO UPDATE SET baseline_version=EXCLUDED.baseline_version,overrides_json=EXCLUDED.overrides_json,revision=EXCLUDED.revision,updated_by=EXCLUDED.updated_by,updated_at=EXCLUDED.updated_at RETURNING baseline_version,overrides_json,revision")
                .bind(input.workspace_id).bind(&input.input.baseline_version).bind(revision).bind(input.actor_id).bind(input.command.now).fetch_one(&mut *tx).await.map_err(map_store)?;
            let result = writing_configuration(&row)?;
            append_audit_event(
                &mut tx,
                AuditEventInput::user(
                    input.workspace_id,
                    input.actor_id,
                    AuditAction::WritingConfigurationChanged,
                    AuditTarget {
                        kind: AuditTargetKind::Workspace,
                        id: input.workspace_id,
                    },
                    input.command.now,
                    &input.command.idempotency_key,
                ),
            )
            .await?;
            complete_workspace(&mut tx, input.workspace_id, &input.command, 200, &result).await?;
            tx.commit().await.map_err(map_store)?;
            Ok(result)
        })
    }
}

fn proposal_view(row: &sqlx::postgres::PgRow) -> Result<ProposalView, GovernanceError> {
    Ok(ProposalView {
        proposal_id: row.get("id"),
        job_id: row.get("job_id"),
        document_id: row.get("document_id"),
        base_revision: row.get("base_revision"),
        operations: serde_json::from_value(row.get("operations_json"))
            .map_err(|_| GovernanceError::Internal)?,
        status: match row.get::<String, _>("status").as_str() {
            "OPEN" => ProposalStatus::Open,
            "APPLIED" => ProposalStatus::Applied,
            "REJECTED" => ProposalStatus::Rejected,
            "STALE" => ProposalStatus::Stale,
            "CANCELLED" => ProposalStatus::Cancelled,
            _ => return Err(GovernanceError::Internal),
        },
        revision: row.get("revision"),
        applied_revision: row.get("applied_revision"),
        applied_operation_ids: row.get("applied_operation_ids"),
        created_at: row.get("created_at"),
        resolved_at: row.get("resolved_at"),
    })
}

fn default_writing_configuration() -> WritingConfigurationView {
    WritingConfigurationView {
        baseline_version: adoc_application::ai::WRITING_RULE_BASELINE_VERSION.to_owned(),
        overrides: Vec::new(),
        revision: 0,
    }
}
fn writing_configuration(
    row: &sqlx::postgres::PgRow,
) -> Result<WritingConfigurationView, GovernanceError> {
    Ok(WritingConfigurationView {
        baseline_version: row.get("baseline_version"),
        overrides: serde_json::from_value(row.get::<Value, _>("overrides_json"))
            .map_err(|_| GovernanceError::Internal)?,
        revision: row.get("revision"),
    })
}

async fn require_member(
    pool: &PgPool,
    actor: Uuid,
    workspace: Uuid,
    admin: bool,
) -> Result<(), GovernanceError> {
    let role: Option<String> = sqlx::query_scalar("SELECT role::text FROM memberships WHERE workspace_id=$1 AND user_id=$2 AND status='ACTIVE'").bind(workspace).bind(actor).fetch_optional(pool).await.map_err(map_store)?;
    if role
        .as_deref()
        .is_some_and(|role| !admin || matches!(role, "ADMIN" | "OWNER"))
    {
        Ok(())
    } else {
        Err(GovernanceError::WorkspaceNotFound)
    }
}
async fn require_member_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    actor: Uuid,
    workspace: Uuid,
    admin: bool,
) -> Result<(), GovernanceError> {
    let role: Option<String> = sqlx::query_scalar("SELECT role::text FROM memberships WHERE workspace_id=$1 AND user_id=$2 AND status='ACTIVE'").bind(workspace).bind(actor).fetch_optional(&mut **tx).await.map_err(map_store)?;
    if role
        .as_deref()
        .is_some_and(|role| !admin || matches!(role, "ADMIN" | "OWNER"))
    {
        Ok(())
    } else {
        Err(GovernanceError::WorkspaceNotFound)
    }
}
