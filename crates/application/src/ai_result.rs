use std::collections::BTreeSet;

use adoc_writing_intelligence::{AiTarget, AiTask, AiTaskKind};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use unicode_normalization::UnicodeNormalization;
use uuid::Uuid;

use crate::document::{DocumentOperation, OperationScope};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AiResultStatus {
    Ready,
    InsufficientContext,
    ConflictingContext,
    NoChange,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FindingSeverity {
    Blocking,
    Warning,
    Advisory,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AiFinding {
    pub finding_id: Uuid,
    pub rule_id: String,
    pub severity: FindingSeverity,
    pub region: OperationScope,
    pub reason: String,
    pub suggestion: Option<String>,
    pub source_ids: Vec<Uuid>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ClaimCertainty {
    Supported,
    Conflicting,
    Insufficient,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AiClaim {
    pub text: String,
    pub source_ids: Vec<Uuid>,
    pub certainty: ClaimCertainty,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AiConflict {
    pub description: String,
    pub source_ids: Vec<Uuid>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AiResult {
    pub schema_version: u8,
    pub task_kind: AiTaskKind,
    pub status: AiResultStatus,
    pub operations: Vec<DocumentOperation>,
    pub findings: Vec<AiFinding>,
    pub claims: Vec<AiClaim>,
    pub uncertainties: Vec<String>,
    pub conflicts: Vec<AiConflict>,
    pub used_source_ids: Vec<Uuid>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResultApplication {
    None,
    BoundedRewrite,
    Proposal,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AiResultValidationError {
    Schema,
    TaskKind,
    Status,
    SourceMembership,
    Scope,
    Revision,
    Dependency,
    RuleBlocked,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResultValidationSummary {
    pub validator_version: &'static str,
    pub writing_rule_version: String,
    pub vocabulary_revision: i64,
    pub status: &'static str,
    pub application: &'static str,
}

pub fn validate_result(
    value: Value,
    task: &AiTask,
    included: &BTreeSet<Uuid>,
) -> Result<(AiResult, ResultApplication), AiResultValidationError> {
    let result: AiResult =
        serde_json::from_value(value).map_err(|_| AiResultValidationError::Schema)?;
    if result.schema_version != 1 {
        return Err(AiResultValidationError::Schema);
    }
    if result.task_kind != task.kind {
        return Err(AiResultValidationError::TaskKind);
    }
    if result.operations.len() > 500
        || result.findings.len() > 500
        || result.claims.len() > 500
        || result.uncertainties.len() > 500
        || result.conflicts.len() > 500
        || result.used_source_ids.len() > 200
    {
        return Err(AiResultValidationError::Schema);
    }
    let status_valid = match result.status {
        AiResultStatus::Ready => true,
        _ => result.operations.is_empty(),
    };
    if !status_valid
        || (matches!(task.kind, AiTaskKind::Review | AiTaskKind::KnowledgeQuery)
            && !result.operations.is_empty())
        || (result.claims.iter().any(|claim| {
            claim.source_ids.is_empty() && claim.certainty == ClaimCertainty::Supported
        }) && result.status != AiResultStatus::InsufficientContext)
    {
        return Err(AiResultValidationError::Status);
    }
    let mut sources = BTreeSet::new();
    sources.extend(result.used_source_ids.iter().copied());
    result
        .findings
        .iter()
        .for_each(|value| sources.extend(value.source_ids.iter().copied()));
    result
        .claims
        .iter()
        .for_each(|value| sources.extend(value.source_ids.iter().copied()));
    for conflict in &result.conflicts {
        if conflict.source_ids.len() < 2 {
            return Err(AiResultValidationError::Schema);
        }
        sources.extend(conflict.source_ids.iter().copied());
    }
    if !sources.is_subset(included) {
        return Err(AiResultValidationError::SourceMembership);
    }
    for operation in &result.operations {
        if operation.base().precondition.draft_revision != task.expected_revision {
            return Err(AiResultValidationError::Revision);
        }
        if !within_target(operation, &task.target) {
            return Err(AiResultValidationError::Scope);
        }
    }
    validate_dependency_selection(&result.operations, None)?;
    let application = application_policy(task, &result);
    Ok((result, application))
}

pub fn validate_dependency_selection(
    operations: &[DocumentOperation],
    selected: Option<&BTreeSet<Uuid>>,
) -> Result<(), AiResultValidationError> {
    let all: BTreeSet<_> = operations.iter().map(|value| value.base().op_id).collect();
    if all.len() != operations.len() {
        return Err(AiResultValidationError::Dependency);
    }
    let selected = selected.unwrap_or(&all);
    if selected.is_empty() || !selected.is_subset(&all) {
        return Err(AiResultValidationError::Dependency);
    }
    if operations.iter().any(|value| {
        selected.contains(&value.base().op_id)
            && value
                .base()
                .depends_on
                .iter()
                .any(|dependency| !selected.contains(dependency))
    }) {
        return Err(AiResultValidationError::Dependency);
    }
    Ok(())
}

pub fn prohibited_term_in_content<'a>(content: &Value, terms: &'a [String]) -> Option<&'a str> {
    let normalized_terms: Vec<_> = terms
        .iter()
        .map(|term| term.nfc().collect::<String>().to_lowercase())
        .collect();
    let mut texts = Vec::new();
    collect_text(content, &mut texts);
    texts.into_iter().find_map(|text| {
        let normalized = text.nfc().collect::<String>().to_lowercase();
        normalized_terms
            .iter()
            .position(|term| !term.is_empty() && normalized.contains(term))
            .map(|index| terms[index].as_str())
    })
}

fn collect_text<'a>(value: &'a Value, output: &mut Vec<&'a str>) {
    match value {
        Value::Object(object) => {
            if object.get("type").and_then(Value::as_str) == Some("text")
                && let Some(text) = object.get("text").and_then(Value::as_str)
            {
                output.push(text);
            }
            object
                .values()
                .for_each(|child| collect_text(child, output));
        }
        Value::Array(values) => values.iter().for_each(|child| collect_text(child, output)),
        _ => {}
    }
}
fn application_policy(task: &AiTask, result: &AiResult) -> ResultApplication {
    if result.operations.is_empty()
        || matches!(task.kind, AiTaskKind::Review | AiTaskKind::KnowledgeQuery)
    {
        return ResultApplication::None;
    }
    if task.kind == AiTaskKind::Rewrite
        && matches!(task.target, AiTarget::Region { .. })
        && result.operations.iter().all(|value| {
            matches!(
                value,
                DocumentOperation::ReplaceText { .. } | DocumentOperation::SetMarks { .. }
            )
        })
    {
        ResultApplication::BoundedRewrite
    } else {
        ResultApplication::Proposal
    }
}
fn within_target(operation: &DocumentOperation, target: &AiTarget) -> bool {
    match target {
        AiTarget::Document { .. } | AiTarget::Discussion { .. } => true,
        AiTarget::WorkspaceQuery { .. } => false,
        AiTarget::Region { region, .. } => serde_json::from_value::<OperationScope>(region.clone())
            .ok()
            .is_some_and(|scope| operation.base().scope == scope),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::{OperationBase, OperationPrecondition};
    fn operation(id: Uuid, depends_on: Vec<Uuid>) -> DocumentOperation {
        DocumentOperation::InsertBlock {
            base: OperationBase {
                op_id: id,
                scope: OperationScope::Document,
                precondition: OperationPrecondition {
                    draft_revision: 0,
                    target_hash: None,
                },
                depends_on,
            },
            parent_id: None,
            index: 0,
            block: serde_json::json!({"id":Uuid::from_u128(99),"type":"paragraph","children":[]}),
        }
    }
    #[test]
    fn dependency_selection_is_closed() {
        let first = Uuid::from_u128(1);
        let second = Uuid::from_u128(2);
        let operations = vec![operation(first, vec![]), operation(second, vec![first])];
        assert_eq!(
            validate_dependency_selection(&operations, Some(&BTreeSet::from([second]))),
            Err(AiResultValidationError::Dependency)
        );
        assert!(
            validate_dependency_selection(&operations, Some(&BTreeSet::from([first, second])))
                .is_ok()
        );
    }
    #[test]
    fn prohibited_rule_scans_only_text_nodes() {
        let terms = vec!["금지 용어".to_owned()];
        assert_eq!(
            prohibited_term_in_content(
                &serde_json::json!({"type":"text","text":"금지 용어"}),
                &terms
            ),
            Some("금지 용어")
        );
        assert_eq!(
            prohibited_term_in_content(&serde_json::json!({"label":"금지 용어"}), &terms),
            None
        );
    }
}
