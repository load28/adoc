#![forbid(unsafe_code)]

//! Writing intelligence bounded context.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use uuid::Uuid;

pub const TASK_DEFINITION_VERSION: &str = "ai-task-registry-v1";
pub const MAX_CONTEXT_SOURCES: usize = 200;
pub const MAX_SOURCE_BYTES: usize = 65_536;
pub const MAX_CONTEXT_BYTES: usize = 4 * 1024 * 1024;
pub const MAX_OUTPUT_BYTES: usize = 2 * 1024 * 1024;
pub const WRITING_RULE_BASELINE_VERSION: &str = "writing-rules-v1";
pub const RESULT_VALIDATOR_VERSION: &str = "ai-result-validator-v1";

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AiTaskKind {
    Compose,
    Rewrite,
    Review,
    DiscussionApply,
    ConflictMerge,
    KnowledgeQuery,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE", deny_unknown_fields)]
pub enum AiTarget {
    Document { document_id: Uuid },
    Region { document_id: Uuid, region: Value },
    Discussion { discussion_id: Uuid },
    WorkspaceQuery { question: String },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SemanticRetrieval {
    Optional,
    Required,
    Disabled,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TaskDefinition {
    pub kind: AiTaskKind,
    pub semantic_retrieval: SemanticRetrieval,
    pub timeout_class: &'static str,
    pub application_policy: &'static str,
    pub evaluation_set_version: &'static str,
}

impl TaskDefinition {
    pub fn accepts(self, target: &AiTarget) -> bool {
        matches!(
            (self.kind, target),
            (
                AiTaskKind::Compose
                    | AiTaskKind::Rewrite
                    | AiTaskKind::Review
                    | AiTaskKind::ConflictMerge,
                AiTarget::Document { .. } | AiTarget::Region { .. }
            ) | (AiTaskKind::DiscussionApply, AiTarget::Discussion { .. })
                | (AiTaskKind::KnowledgeQuery, AiTarget::WorkspaceQuery { .. })
        )
    }
}

pub fn task_definition(kind: AiTaskKind) -> TaskDefinition {
    let (semantic_retrieval, timeout_class, application_policy) = match kind {
        AiTaskKind::Rewrite => (
            SemanticRetrieval::Optional,
            "INTERACTIVE",
            "BOUNDED_REWRITE",
        ),
        AiTaskKind::KnowledgeQuery => (SemanticRetrieval::Optional, "INTERACTIVE", "ANSWER_ONLY"),
        AiTaskKind::Review => (SemanticRetrieval::Optional, "STANDARD", "FINDINGS_ONLY"),
        AiTaskKind::Compose => (SemanticRetrieval::Optional, "STANDARD", "PROPOSAL"),
        AiTaskKind::DiscussionApply | AiTaskKind::ConflictMerge => {
            (SemanticRetrieval::Optional, "STANDARD", "PROPOSAL")
        }
    };
    TaskDefinition {
        kind,
        semantic_retrieval,
        timeout_class,
        application_policy,
        evaluation_set_version: "ai-eval-v1",
    }
}

#[must_use]
pub fn runtime_output_schema(kind: AiTaskKind) -> Value {
    let kind = serde_json::to_value(kind).unwrap_or(Value::Null);
    let (operation, definitions) = embedded_operation_schema();
    let mut schema = serde_json::json!({
        "type": "object",
        "additionalProperties": false,
        "required": ["schemaVersion", "taskKind", "status", "operations", "findings", "claims", "uncertainties", "conflicts", "usedSourceIds"],
        "properties": {
            "schemaVersion": {"const": 1},
            "taskKind": {"const": kind},
            "status": {"type":"string","enum":["READY","INSUFFICIENT_CONTEXT","CONFLICTING_CONTEXT","NO_CHANGE"]},
            "operations": {"type": "array", "maxItems": 500, "items": operation},
            "findings": {"type": "array", "maxItems": 500, "items": {"type":"object","additionalProperties":false,"required":["findingId","ruleId","severity","region","reason","suggestion","sourceIds"],"properties":{"findingId":{"type":"string","format":"uuid"},"ruleId":{"type":"string"},"severity":{"type":"string","enum":["BLOCKING","WARNING","ADVISORY"]},"region":{"$ref":"#/$defs/region"},"reason":{"type":"string"},"suggestion":{"type":["string","null"]},"sourceIds":{"type":"array","items":{"type":"string","format":"uuid"}}}}},
            "claims": {"type": "array", "maxItems": 500, "items": {"type":"object","additionalProperties":false,"required":["text","sourceIds","certainty"],"properties":{"text":{"type":"string"},"sourceIds":{"type":"array","items":{"type":"string","format":"uuid"}},"certainty":{"type":"string","enum":["SUPPORTED","CONFLICTING","INSUFFICIENT"]}}}},
            "uncertainties": {"type":"array","maxItems":500,"items":{"type":"string","maxLength":5000}},
            "conflicts": {"type":"array","maxItems":500,"items":{"type":"object","additionalProperties":false,"required":["description","sourceIds"],"properties":{"description":{"type":"string"},"sourceIds":{"type":"array","minItems":2,"items":{"type":"string","format":"uuid"}}}}},
            "usedSourceIds": {"type":"array","maxItems":200,"uniqueItems":true,"items":{"type":"string","format":"uuid"}}
        },
        "$defs": definitions
    });
    canonicalize_schema(&mut schema);
    schema
}

fn embedded_operation_schema() -> (Value, serde_json::Map<String, Value>) {
    let mut operation: Value = serde_json::from_str(include_str!(
        "../../../docs/design/contracts/document-operation.schema.json"
    ))
    .expect("canonical operation schema must be valid JSON");
    let mut content: Value = serde_json::from_str(include_str!(
        "../../../docs/design/contracts/document-content.schema.json"
    ))
    .expect("canonical content schema must be valid JSON");
    rewrite_refs(&mut operation, false);
    rewrite_refs(&mut content, true);
    let operation_root = serde_json::json!({"oneOf": operation.get("oneOf").cloned().unwrap_or(Value::Array(Vec::new()))});
    let mut definitions = operation
        .get_mut("$defs")
        .and_then(Value::as_object_mut)
        .map(std::mem::take)
        .unwrap_or_default();
    if let Some(content_definitions) = content.get_mut("$defs").and_then(Value::as_object_mut) {
        for (name, value) in std::mem::take(content_definitions) {
            definitions.insert(format!("content_{name}"), value);
        }
    }
    (operation_root, definitions)
}

fn rewrite_refs(value: &mut Value, content_schema: bool) {
    match value {
        Value::Object(object) => {
            if let Some(reference) = object
                .get_mut("$ref")
                .and_then(|value| value.as_str())
                .map(str::to_owned)
            {
                let rewritten = if let Some(name) =
                    reference.strip_prefix("document-content.schema.json#/$defs/")
                {
                    format!("#/$defs/content_{name}")
                } else if content_schema {
                    reference
                        .strip_prefix("#/$defs/")
                        .map_or(reference.clone(), |name| format!("#/$defs/content_{name}"))
                } else {
                    reference
                };
                object.insert("$ref".to_owned(), Value::String(rewritten));
            }
            object
                .values_mut()
                .for_each(|child| rewrite_refs(child, content_schema));
        }
        Value::Array(values) => values
            .iter_mut()
            .for_each(|child| rewrite_refs(child, content_schema)),
        _ => {}
    }
}

fn canonicalize_schema(value: &mut Value) {
    if let Value::Object(object) = value {
        object.remove("$schema");
        object.remove("$id");
        object.remove("title");
        object.values_mut().for_each(canonicalize_schema);
    } else if let Value::Array(values) = value {
        values.iter_mut().for_each(canonicalize_schema);
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AiTask {
    pub kind: AiTaskKind,
    pub workspace_id: Uuid,
    pub actor_id: Uuid,
    pub target: AiTarget,
    pub expected_revision: i64,
    pub external_web_enabled: bool,
    pub instruction: Option<String>,
}

impl AiTask {
    pub fn is_valid(&self) -> bool {
        self.expected_revision >= 0
            && !self.external_web_enabled
            && self
                .instruction
                .as_ref()
                .is_none_or(|value| value.chars().count() <= 10_000)
            && match &self.target {
                AiTarget::WorkspaceQuery { question } => {
                    !question.trim().is_empty() && question.chars().count() <= 10_000
                }
                _ => true,
            }
            && task_definition(self.kind).accepts(&self.target)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ContextSourceKind {
    Draft,
    PublishedRegion,
    Discussion,
    Vocabulary,
    UserInput,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SourceAuthority {
    UserExplicit,
    Official,
    Vocabulary,
    DiscussionConfirmed,
    RelatedInternal,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum IncludeReason {
    CurrentTarget,
    ExplicitReference,
    DiscussionContext,
    VocabularyPolicy,
    RetrievedRelated,
    UserProvided,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContextSource {
    pub source_id: Uuid,
    pub kind: ContextSourceKind,
    pub stable_id: String,
    pub document_id: Option<Uuid>,
    pub region_id: Option<Uuid>,
    pub version: Option<i64>,
    pub draft_revision: Option<i64>,
    pub authority: SourceAuthority,
    pub include_reason: IncludeReason,
    pub snapshot_hash: String,
    pub snapshot_text: String,
    pub permission_key: Option<String>,
    pub source_revision: i64,
    pub retrieved_at: Option<DateTime<Utc>>,
    pub included: bool,
}

impl ContextSource {
    pub fn assign_id(&mut self) {
        self.source_id = deterministic_source_id(self.kind, &self.stable_id, &self.snapshot_hash);
    }

    pub fn is_valid(&self) -> bool {
        self.source_revision >= 0
            && self.snapshot_text.len() <= MAX_SOURCE_BYTES
            && is_sha256(&self.snapshot_hash)
            && self
                .permission_key
                .as_ref()
                .is_none_or(|key| is_sha256(key))
            && self.source_id
                == deterministic_source_id(self.kind, &self.stable_id, &self.snapshot_hash)
            && (self.version.is_some() ^ self.draft_revision.is_some()
                || matches!(
                    self.kind,
                    ContextSourceKind::Discussion
                        | ContextSourceKind::Vocabulary
                        | ContextSourceKind::UserInput
                ))
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContextArtifact {
    pub schema_version: u8,
    pub task: AiTask,
    pub task_definition_version: String,
    pub sources: Vec<ContextSource>,
    pub writing_rule_version: String,
    pub vocabulary_revision: i64,
    pub permission_scope_fingerprint: String,
    pub estimated_input_units: u64,
}

impl ContextArtifact {
    pub fn normalize_and_fingerprint(&mut self, maximum_units: u64) -> Option<String> {
        self.sources.sort_by(|left, right| {
            left.authority
                .cmp(&right.authority)
                .then_with(|| left.include_reason.cmp(&right.include_reason))
                .then_with(|| left.stable_id.cmp(&right.stable_id))
                .then_with(|| left.source_id.cmp(&right.source_id))
        });
        let serialized = serde_json::to_vec(self).ok()?;
        self.estimated_input_units = serialized.len() as u64;
        let serialized = serde_json::to_vec(self).ok()?;
        (self.schema_version == 1
            && self.task.is_valid()
            && self.task_definition_version == TASK_DEFINITION_VERSION
            && self.vocabulary_revision >= 0
            && is_sha256(&self.permission_scope_fingerprint)
            && self.sources.len() <= MAX_CONTEXT_SOURCES
            && self.sources.iter().all(ContextSource::is_valid)
            && serialized.len() <= MAX_CONTEXT_BYTES
            && self.estimated_input_units <= maximum_units)
            .then(|| hex::encode(Sha256::digest(&serialized)))
    }
}

pub fn deterministic_source_id(
    kind: ContextSourceKind,
    stable_id: &str,
    snapshot_hash: &str,
) -> Uuid {
    let digest = Sha256::digest(format!("ai-source:v1:{kind:?}:{stable_id}:{snapshot_hash}"));
    let mut bytes = [0_u8; 16];
    bytes.copy_from_slice(&digest[..16]);
    bytes[6] = (bytes[6] & 0x0f) | 0x50;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    Uuid::from_bytes(bytes)
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RuntimePhase {
    Started,
    Generating,
    Finalizing,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RuntimeEvent {
    pub phase: RuntimePhase,
    pub provider_sequence: u64,
    pub progress: Option<u8>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RuntimeRequest {
    pub job_id: Uuid,
    pub task_kind: AiTaskKind,
    pub model: String,
    pub policy_artifact: Value,
    pub context_artifact: Value,
    pub output_schema: Value,
    pub timeout_millis: u64,
    pub max_output_bytes: usize,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RuntimeUsage {
    pub input_units: u64,
    pub output_units: u64,
    pub estimated_microunits: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RuntimeResult {
    pub provider_request_id: Option<String>,
    pub model: String,
    pub output_json: Value,
    pub usage: RuntimeUsage,
    pub latency_millis: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_rejects_wrong_target_and_external_web() {
        let task = AiTask {
            kind: AiTaskKind::KnowledgeQuery,
            workspace_id: Uuid::from_u128(1),
            actor_id: Uuid::from_u128(2),
            target: AiTarget::Document {
                document_id: Uuid::from_u128(3),
            },
            expected_revision: 0,
            external_web_enabled: false,
            instruction: None,
        };
        assert!(!task.is_valid());
    }

    #[test]
    fn source_id_and_artifact_fingerprint_are_deterministic() {
        let hash = "a".repeat(64);
        let id = deterministic_source_id(ContextSourceKind::Draft, "draft:1", &hash);
        assert_eq!(
            id,
            deterministic_source_id(ContextSourceKind::Draft, "draft:1", &hash)
        );
        assert_ne!(
            id,
            deterministic_source_id(ContextSourceKind::Draft, "draft:2", &hash)
        );
    }
}
