use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Affinity {
    Before,
    After,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TextAnchor {
    pub offset: usize,
    pub affinity: Affinity,
    pub context_hash: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(
    tag = "kind",
    rename_all = "SCREAMING_SNAKE_CASE",
    rename_all_fields = "camelCase"
)]
pub enum OperationScope {
    Document,
    Block {
        block_id: Uuid,
    },
    BlockRange {
        start_block_id: Uuid,
        end_block_id: Uuid,
    },
    Section {
        heading_id: Uuid,
    },
    TextRange {
        block_id: Uuid,
        from: TextAnchor,
        to: TextAnchor,
        quote_hash: String,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OperationPrecondition {
    pub draft_revision: i64,
    #[serde(default)]
    pub target_hash: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(
    tag = "action",
    rename_all = "SCREAMING_SNAKE_CASE",
    rename_all_fields = "camelCase"
)]
pub enum AttrPatch {
    Set { value: Value },
    Remove,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SetMarksMode {
    Add,
    Remove,
    Replace,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReferenceTarget {
    pub kind: String,
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub region: Option<OperationScope>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OperationBase {
    pub op_id: Uuid,
    pub scope: OperationScope,
    pub precondition: OperationPrecondition,
    #[serde(default)]
    pub depends_on: Vec<Uuid>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(
    tag = "kind",
    rename_all = "SCREAMING_SNAKE_CASE",
    rename_all_fields = "camelCase"
)]
pub enum DocumentOperation {
    InsertBlock {
        #[serde(flatten)]
        base: OperationBase,
        parent_id: Option<Uuid>,
        index: usize,
        block: Value,
    },
    DeleteBlock {
        #[serde(flatten)]
        base: OperationBase,
        block_id: Uuid,
    },
    MoveBlock {
        #[serde(flatten)]
        base: OperationBase,
        block_id: Uuid,
        new_parent_id: Option<Uuid>,
        new_index: usize,
    },
    ReplaceText {
        #[serde(flatten)]
        base: OperationBase,
        range: OperationScope,
        content: Vec<Value>,
    },
    SetBlockAttrs {
        #[serde(flatten)]
        base: OperationBase,
        block_id: Uuid,
        attrs: std::collections::BTreeMap<String, AttrPatch>,
    },
    SetMarks {
        #[serde(flatten)]
        base: OperationBase,
        range: OperationScope,
        mode: SetMarksMode,
        marks: Vec<Value>,
    },
    ReplaceRegion {
        #[serde(flatten)]
        base: OperationBase,
        region: OperationScope,
        blocks: Vec<Value>,
    },
    AddReference {
        #[serde(flatten)]
        base: OperationBase,
        reference_id: Uuid,
        source_region: OperationScope,
        target: ReferenceTarget,
    },
    RemoveReference {
        #[serde(flatten)]
        base: OperationBase,
        reference_id: Uuid,
        source_region: OperationScope,
        target: ReferenceTarget,
    },
}

impl DocumentOperation {
    pub fn base(&self) -> &OperationBase {
        match self {
            Self::InsertBlock { base, .. }
            | Self::DeleteBlock { base, .. }
            | Self::MoveBlock { base, .. }
            | Self::ReplaceText { base, .. }
            | Self::SetBlockAttrs { base, .. }
            | Self::SetMarks { base, .. }
            | Self::ReplaceRegion { base, .. }
            | Self::AddReference { base, .. }
            | Self::RemoveReference { base, .. } => base,
        }
    }

    pub fn base_mut(&mut self) -> &mut OperationBase {
        match self {
            Self::InsertBlock { base, .. }
            | Self::DeleteBlock { base, .. }
            | Self::MoveBlock { base, .. }
            | Self::ReplaceText { base, .. }
            | Self::SetBlockAttrs { base, .. }
            | Self::SetMarks { base, .. }
            | Self::ReplaceRegion { base, .. }
            | Self::AddReference { base, .. }
            | Self::RemoveReference { base, .. } => base,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReferenceSnapshot {
    pub reference_id: Uuid,
    pub source_region: OperationScope,
    pub target: ReferenceTarget,
}

#[derive(Clone, Debug)]
pub struct ReducerInput {
    pub content: Value,
    pub base_revision: i64,
    pub operations: Vec<DocumentOperation>,
    pub references: Vec<ReferenceSnapshot>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(
    tag = "kind",
    rename_all = "SCREAMING_SNAKE_CASE",
    rename_all_fields = "camelCase"
)]
pub enum ReferenceEffect {
    Add { reference: ReferenceSnapshot },
    Remove { reference: ReferenceSnapshot },
}

#[derive(Clone, Debug)]
pub struct ReducerResult {
    pub content: Value,
    pub content_fingerprint: String,
    pub applied_operation_ids: Vec<Uuid>,
    pub inverse_operations: Vec<DocumentOperation>,
    pub reference_effects: Vec<ReferenceEffect>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RegionResolutionStatus {
    Resolved,
    Moved,
    Ambiguous,
    Orphaned,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegionResolution {
    pub status: RegionResolutionStatus,
    pub region: Option<OperationScope>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OperationErrorCode {
    SchemaInvalid,
    ContentInvalid,
    BatchInvalid,
    DependencyInvalid,
    RegionNotFound,
    RegionAmbiguous,
    PreconditionFailed,
    TargetConflict,
    NoEffect,
    LimitExceeded,
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
#[error("operation failed: {code:?}")]
pub struct OperationError {
    pub code: OperationErrorCode,
    pub operation_id: Option<Uuid>,
}

impl OperationError {
    pub(crate) fn new(code: OperationErrorCode, operation_id: Option<Uuid>) -> Self {
        Self { code, operation_id }
    }
}
