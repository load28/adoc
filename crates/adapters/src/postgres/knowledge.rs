use adoc_application::{
    governance::{Command, GovernanceError},
    knowledge::{
        KnowledgeRepository, NormalizedVocabularyInput, Reference, ReferenceDisplaySnapshot,
        ReferencePage, VocabularyAction, VocabularyCommand, VocabularyConcept, VocabularyPage,
        VocabularyStatus, VocabularyTerm, VocabularyTermKind,
    },
    operations::{AuditAction, AuditEventInput, AuditTarget, AuditTargetKind},
    permission::{Access, compile_permission_scope},
};
use adoc_ports::BoxFuture;
use serde_json::{Value, json};
use sqlx::{PgPool, Postgres, Row, Transaction, postgres::PgRow};
use uuid::Uuid;

use super::{
    PostgresStore, append_audit_event,
    document::{require_access, require_effective_active},
    governance::{
        OutboxEvent, append_event, begin_workspace, check_revision, complete_workspace, map_store,
    },
    permission::scope_snapshot_tx,
};

#[derive(Clone)]
pub struct PostgresKnowledgeRepository {
    pool: PgPool,
}
impl PostgresKnowledgeRepository {
    pub fn new(store: &PostgresStore) -> Self {
        Self {
            pool: store.pool().clone(),
        }
    }
}

impl KnowledgeRepository for PostgresKnowledgeRepository {
    fn get_reference<'a>(
        &'a self,
        actor: Uuid,
        workspace: Uuid,
        document: Uuid,
        id: Uuid,
    ) -> BoxFuture<'a, Result<Reference, GovernanceError>> {
        Box::pin(async move {
            let mut tx = self.pool.begin().await.map_err(map_store)?;
            require_access(&mut tx, actor, workspace, document, Access::Viewer, false).await?;
            let row=sqlx::query("SELECT id,source_id,source_region_json,target_kind,target_id,target_region_json,snapshot_json,created_at FROM references_graph WHERE workspace_id=$1 AND source_kind='DOCUMENT' AND source_id=$2 AND id=$3")
                .bind(workspace).bind(document).bind(id).fetch_optional(&mut *tx).await.map_err(map_store)?.ok_or(GovernanceError::ReferenceNotFound)?;
            let result = reference(&row)?;
            tx.commit().await.map_err(map_store)?;
            Ok(result)
        })
    }
    fn list_backlinks<'a>(
        &'a self,
        actor: Uuid,
        workspace: Uuid,
        target: Uuid,
        cursor: Option<String>,
    ) -> BoxFuture<'a, Result<ReferencePage, GovernanceError>> {
        Box::pin(async move {
            let cursor = cursor
                .map(|value| Uuid::parse_str(&value).map_err(|_| GovernanceError::Validation))
                .transpose()?;
            let mut tx = self.pool.begin().await.map_err(map_store)?;
            require_access(&mut tx, actor, workspace, target, Access::Viewer, false).await?;
            require_effective_active(&mut tx, workspace, target).await?;
            let snapshot = scope_snapshot_tx(&mut tx, actor, workspace).await?;
            let allowed = compile_permission_scope(&snapshot.nodes)
                .map_err(|_| GovernanceError::Internal)?
                .into_iter()
                .filter_map(|(id, permission)| permission.access.can_view().then_some(id))
                .collect::<Vec<_>>();
            let target = target.to_string();
            let rows=sqlx::query("SELECT id,source_id,source_region_json,target_kind,target_id,target_region_json,snapshot_json,created_at FROM references_graph WHERE workspace_id=$1 AND source_id=ANY($2) AND deleted_at IS NULL AND target_kind IN ('DOCUMENT','REGION') AND target_id=$3 AND ($4::uuid IS NULL OR (created_at,id)<(SELECT created_at,id FROM references_graph WHERE workspace_id=$1 AND id=$4 AND deleted_at IS NULL AND target_kind IN ('DOCUMENT','REGION') AND target_id=$3)) ORDER BY created_at DESC,id DESC LIMIT 51")
                .bind(workspace).bind(&allowed).bind(&target).bind(cursor).fetch_all(&mut *tx).await.map_err(map_store)?;
            if cursor.is_some() && rows.is_empty() {
                let valid:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM references_graph WHERE workspace_id=$1 AND id=$2 AND deleted_at IS NULL AND target_kind IN ('DOCUMENT','REGION') AND target_id=$3)").bind(workspace).bind(cursor).bind(target).fetch_one(&mut *tx).await.map_err(map_store)?;
                if !valid {
                    return Err(GovernanceError::Validation);
                }
            }
            let items = rows
                .iter()
                .take(50)
                .map(reference)
                .collect::<Result<Vec<_>, _>>()?;
            let next_cursor =
                (rows.len() > 50).then(|| items.last().expect("nonempty").id.to_string());
            tx.commit().await.map_err(map_store)?;
            Ok(ReferencePage { items, next_cursor })
        })
    }
    fn list_vocabulary<'a>(
        &'a self,
        actor: Uuid,
        workspace: Uuid,
        cursor: Option<String>,
    ) -> BoxFuture<'a, Result<VocabularyPage, GovernanceError>> {
        Box::pin(async move {
            let cursor = cursor
                .map(|value| Uuid::parse_str(&value).map_err(|_| GovernanceError::Validation))
                .transpose()?;
            let mut tx = self.pool.begin().await.map_err(map_store)?;
            require_member(&mut tx, actor, workspace, false).await?;
            let rows=sqlx::query("SELECT id,canonical_term,definition,status::text,replacement_concept_id,revision,created_at FROM vocabulary_concepts WHERE workspace_id=$1 AND ($2::uuid IS NULL OR (canonical_term,id)>(SELECT canonical_term,id FROM vocabulary_concepts WHERE workspace_id=$1 AND id=$2)) ORDER BY canonical_term,id LIMIT 51")
                .bind(workspace).bind(cursor).fetch_all(&mut *tx).await.map_err(map_store)?;
            let mut items = Vec::new();
            for row in rows.iter().take(50) {
                items.push(load_concept(&mut tx, workspace, row).await?)
            }
            let next_cursor =
                (rows.len() > 50).then(|| items.last().expect("nonempty").id.to_string());
            tx.commit().await.map_err(map_store)?;
            Ok(VocabularyPage { items, next_cursor })
        })
    }
    fn get_vocabulary<'a>(
        &'a self,
        actor: Uuid,
        workspace: Uuid,
        id: Uuid,
    ) -> BoxFuture<'a, Result<VocabularyConcept, GovernanceError>> {
        Box::pin(async move {
            let mut tx = self.pool.begin().await.map_err(map_store)?;
            require_member(&mut tx, actor, workspace, false).await?;
            let result = get_concept(&mut tx, workspace, id, false).await?;
            tx.commit().await.map_err(map_store)?;
            Ok(result)
        })
    }
    fn mutate_vocabulary<'a>(
        &'a self,
        input: VocabularyCommand,
    ) -> BoxFuture<'a, Result<VocabularyConcept, GovernanceError>> {
        Box::pin(async move {
            let mut tx = self.pool.begin().await.map_err(map_store)?;
            require_member(&mut tx, input.command.actor_id, input.workspace_id, true).await?;
            if let Some(replay) =
                begin_workspace::<VocabularyConcept>(&mut tx, input.workspace_id, &input.command)
                    .await?
            {
                tx.commit().await.map_err(map_store)?;
                return Ok(replay);
            }
            match input.action {
                VocabularyAction::Create => insert_concept(&mut tx, &input).await?,
                VocabularyAction::Update => update_concept(&mut tx, &input).await?,
                VocabularyAction::Deprecate => deprecate_concept(&mut tx, &input).await?,
            }
            let result = get_concept(&mut tx, input.workspace_id, input.concept_id, false).await?;
            append_event(&mut tx,OutboxEvent{workspace_id:input.workspace_id,aggregate_kind:"VocabularyConcept",aggregate_id:input.concept_id,sequence:result.revision+1,event_type:"VocabularyChanged.v1",payload:json!({"conceptId":input.concept_id,"revision":result.revision,"action":action_text(input.action)}),occurred_at:input.command.now}).await?;
            audit_vocabulary(
                &mut tx,
                &input.command,
                input.workspace_id,
                match input.action {
                    VocabularyAction::Create => AuditAction::VocabularyCreated,
                    VocabularyAction::Update => AuditAction::VocabularyUpdated,
                    VocabularyAction::Deprecate => AuditAction::VocabularyDeprecated,
                },
                input.concept_id,
            )
            .await?;
            complete_workspace(
                &mut tx,
                input.workspace_id,
                &input.command,
                if matches!(input.action, VocabularyAction::Create) {
                    201
                } else {
                    200
                },
                &result,
            )
            .await?;
            tx.commit().await.map_err(map_store)?;
            Ok(result)
        })
    }
}

async fn audit_vocabulary(
    tx: &mut Transaction<'_, Postgres>,
    command: &Command,
    workspace: Uuid,
    action: AuditAction,
    id: Uuid,
) -> Result<(), GovernanceError> {
    append_audit_event(
        tx,
        AuditEventInput::user(
            workspace,
            command.actor_id,
            action,
            AuditTarget {
                kind: AuditTargetKind::Vocabulary,
                id,
            },
            command.now,
            &command.idempotency_key,
        ),
    )
    .await?;
    Ok(())
}

async fn require_member(
    tx: &mut Transaction<'_, Postgres>,
    actor: Uuid,
    workspace: Uuid,
    admin: bool,
) -> Result<(), GovernanceError> {
    let role:Option<String>=sqlx::query_scalar("SELECT m.role::text FROM memberships m JOIN workspaces w ON w.id=m.workspace_id WHERE m.workspace_id=$1 AND m.user_id=$2 AND m.status='ACTIVE' AND w.status='ACTIVE'").bind(workspace).bind(actor).fetch_optional(&mut **tx).await.map_err(map_store)?;
    if role
        .as_deref()
        .is_some_and(|value| !admin || matches!(value, "ADMIN" | "OWNER"))
    {
        Ok(())
    } else {
        Err(GovernanceError::WorkspaceNotFound)
    }
}
async fn insert_concept(
    tx: &mut Transaction<'_, Postgres>,
    command: &VocabularyCommand,
) -> Result<(), GovernanceError> {
    let input = command.input.as_ref().ok_or(GovernanceError::Validation)?;
    sqlx::query("INSERT INTO vocabulary_concepts(id,workspace_id,canonical_term,definition,created_by,created_at,updated_at) VALUES($1,$2,$3,$4,$5,$6,$6)")
        .bind(command.concept_id).bind(command.workspace_id).bind(&input.canonical_term).bind(&input.definition).bind(command.command.actor_id).bind(command.command.now).execute(&mut **tx).await.map_err(map_vocabulary_store)?;
    insert_terms(tx, command.workspace_id, command.concept_id, input).await
}
async fn update_concept(
    tx: &mut Transaction<'_, Postgres>,
    command: &VocabularyCommand,
) -> Result<(), GovernanceError> {
    let current = get_concept(tx, command.workspace_id, command.concept_id, true).await?;
    check_revision(
        current.revision,
        command
            .expected_revision
            .ok_or(GovernanceError::Validation)?,
    )?;
    if current.status != VocabularyStatus::Active {
        return Err(GovernanceError::VocabularyStateInvalid);
    }
    append_history(tx, command, &current).await?;
    let input = command.input.as_ref().ok_or(GovernanceError::Validation)?;
    sqlx::query("UPDATE vocabulary_concepts SET canonical_term=$3,definition=$4,revision=revision+1,updated_at=$5 WHERE workspace_id=$1 AND id=$2")
        .bind(command.workspace_id).bind(command.concept_id).bind(&input.canonical_term).bind(&input.definition).bind(command.command.now).execute(&mut **tx).await.map_err(map_vocabulary_store)?;
    sqlx::query("DELETE FROM vocabulary_terms WHERE workspace_id=$1 AND concept_id=$2")
        .bind(command.workspace_id)
        .bind(command.concept_id)
        .execute(&mut **tx)
        .await
        .map_err(map_store)?;
    insert_terms(tx, command.workspace_id, command.concept_id, input).await
}
async fn deprecate_concept(
    tx: &mut Transaction<'_, Postgres>,
    command: &VocabularyCommand,
) -> Result<(), GovernanceError> {
    if command.reason.as_deref().is_none_or(str::is_empty) {
        return Err(GovernanceError::Validation);
    }
    let mut ids = vec![command.concept_id];
    if let Some(replacement) = command.replacement_concept_id {
        ids.push(replacement);
    }
    ids.sort_unstable();
    ids.dedup();
    let locked=sqlx::query_scalar::<_,Uuid>("SELECT id FROM vocabulary_concepts WHERE workspace_id=$1 AND id=ANY($2) ORDER BY id FOR UPDATE").bind(command.workspace_id).bind(&ids).fetch_all(&mut **tx).await.map_err(map_store)?;
    if locked.len() != ids.len() {
        return Err(GovernanceError::VocabularyNotFound);
    }
    let current = get_concept(tx, command.workspace_id, command.concept_id, false).await?;
    check_revision(
        current.revision,
        command
            .expected_revision
            .ok_or(GovernanceError::Validation)?,
    )?;
    if current.status != VocabularyStatus::Active {
        return Err(GovernanceError::VocabularyStateInvalid);
    }
    if let Some(replacement) = command.replacement_concept_id {
        if replacement == command.concept_id {
            return Err(GovernanceError::VocabularyStateInvalid);
        }
        let replacement = get_concept(tx, command.workspace_id, replacement, false).await?;
        if replacement.status != VocabularyStatus::Active {
            return Err(GovernanceError::VocabularyStateInvalid);
        }
    }
    append_history(tx, command, &current).await?;
    sqlx::query("UPDATE vocabulary_concepts SET status='DEPRECATED',replacement_concept_id=$3,revision=revision+1,updated_at=$4 WHERE workspace_id=$1 AND id=$2")
        .bind(command.workspace_id).bind(command.concept_id).bind(command.replacement_concept_id).bind(command.command.now).execute(&mut **tx).await.map_err(map_store)?;
    Ok(())
}
async fn append_history(
    tx: &mut Transaction<'_, Postgres>,
    command: &VocabularyCommand,
    current: &VocabularyConcept,
) -> Result<(), GovernanceError> {
    sqlx::query("INSERT INTO vocabulary_concept_revisions(workspace_id,concept_id,revision,canonical_term,definition,status,replacement_concept_id,terms_json,change_reason,changed_by,changed_at) VALUES($1,$2,$3,$4,$5,$6::vocabulary_status,$7,$8,$9,$10,$11)")
        .bind(command.workspace_id).bind(command.concept_id).bind(current.revision+1).bind(&current.canonical_term).bind(&current.definition).bind(status_text(current.status)).bind(current.replacement_concept_id).bind(serde_json::to_value(&current.terms).map_err(|_|GovernanceError::Internal)?).bind(command.reason.as_deref()).bind(command.command.actor_id).bind(command.command.now).execute(&mut **tx).await.map_err(map_store)?;
    Ok(())
}
async fn insert_terms(
    tx: &mut Transaction<'_, Postgres>,
    workspace: Uuid,
    concept: Uuid,
    input: &NormalizedVocabularyInput,
) -> Result<(), GovernanceError> {
    for term in &input.terms {
        sqlx::query("INSERT INTO vocabulary_terms(workspace_id,concept_id,term,normalized_term,kind) VALUES($1,$2,$3,$4,$5)").bind(workspace).bind(concept).bind(&term.term).bind(&term.normalized_term).bind(term_kind_text(term.kind)).execute(&mut **tx).await.map_err(map_vocabulary_store)?;
    }
    Ok(())
}
async fn get_concept(
    tx: &mut Transaction<'_, Postgres>,
    workspace: Uuid,
    id: Uuid,
    lock: bool,
) -> Result<VocabularyConcept, GovernanceError> {
    let query = if lock {
        "SELECT id,canonical_term,definition,status::text,replacement_concept_id,revision FROM vocabulary_concepts WHERE workspace_id=$1 AND id=$2 FOR UPDATE"
    } else {
        "SELECT id,canonical_term,definition,status::text,replacement_concept_id,revision FROM vocabulary_concepts WHERE workspace_id=$1 AND id=$2"
    };
    let row = sqlx::query(query)
        .bind(workspace)
        .bind(id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(map_store)?
        .ok_or(GovernanceError::VocabularyNotFound)?;
    load_concept(tx, workspace, &row).await
}
async fn load_concept(
    tx: &mut Transaction<'_, Postgres>,
    workspace: Uuid,
    row: &PgRow,
) -> Result<VocabularyConcept, GovernanceError> {
    let id: Uuid = row.get("id");
    let terms=sqlx::query("SELECT term,kind FROM vocabulary_terms WHERE workspace_id=$1 AND concept_id=$2 ORDER BY kind,normalized_term").bind(workspace).bind(id).fetch_all(&mut **tx).await.map_err(map_store)?.iter().map(|row|Ok(VocabularyTerm{term:row.get("term"),kind:term_kind(row.get("kind"))?})).collect::<Result<Vec<_>,GovernanceError>>()?;
    Ok(VocabularyConcept {
        id,
        canonical_term: row.get("canonical_term"),
        definition: row.get("definition"),
        terms,
        status: status(row.get("status"))?,
        replacement_concept_id: row.get("replacement_concept_id"),
        revision: row.get("revision"),
    })
}
fn reference(row: &PgRow) -> Result<Reference, GovernanceError> {
    let kind: String = row.get("target_kind");
    let mut target = json!({"kind":kind,"id":row.get::<String,_>("target_id")});
    if let Some(region) = row.get::<Option<Value>, _>("target_region_json") {
        target["region"] = region
    }
    let snapshot: Value = row.get("snapshot_json");
    Ok(Reference {
        id: row.get("id"),
        source_document_id: row.get("source_id"),
        source_region: row.get("source_region_json"),
        target,
        snapshot: ReferenceDisplaySnapshot {
            title: snapshot
                .get("title")
                .and_then(Value::as_str)
                .ok_or(GovernanceError::Internal)?
                .to_owned(),
            snapshot_hash: snapshot
                .get("snapshotHash")
                .and_then(Value::as_str)
                .ok_or(GovernanceError::Internal)?
                .to_owned(),
        },
        created_at: row.get("created_at"),
    })
}
fn map_vocabulary_store(error: sqlx::Error) -> GovernanceError {
    if error
        .as_database_error()
        .is_some_and(|value| value.is_unique_violation())
    {
        GovernanceError::VocabularyTermConflict
    } else {
        map_store(error)
    }
}
fn status(value: String) -> Result<VocabularyStatus, GovernanceError> {
    match value.as_str() {
        "ACTIVE" => Ok(VocabularyStatus::Active),
        "DEPRECATED" => Ok(VocabularyStatus::Deprecated),
        _ => Err(GovernanceError::Internal),
    }
}
fn status_text(value: VocabularyStatus) -> &'static str {
    match value {
        VocabularyStatus::Active => "ACTIVE",
        VocabularyStatus::Deprecated => "DEPRECATED",
    }
}
fn term_kind(value: String) -> Result<VocabularyTermKind, GovernanceError> {
    match value.as_str() {
        "CANONICAL" => Ok(VocabularyTermKind::Canonical),
        "SYNONYM" => Ok(VocabularyTermKind::Synonym),
        "PROHIBITED" => Ok(VocabularyTermKind::Prohibited),
        _ => Err(GovernanceError::Internal),
    }
}
fn term_kind_text(value: VocabularyTermKind) -> &'static str {
    match value {
        VocabularyTermKind::Canonical => "CANONICAL",
        VocabularyTermKind::Synonym => "SYNONYM",
        VocabularyTermKind::Prohibited => "PROHIBITED",
    }
}
fn action_text(value: VocabularyAction) -> &'static str {
    match value {
        VocabularyAction::Create => "CREATED",
        VocabularyAction::Update => "UPDATED",
        VocabularyAction::Deprecate => "DEPRECATED",
    }
}
