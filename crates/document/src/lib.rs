#![forbid(unsafe_code)]

//! Document bounded context.

mod content;
mod model;
mod reducer;

pub use content::{ValidatedContent, canonical_hash, normalize_content};
pub use model::{
    Affinity, AttrPatch, DocumentOperation, OperationError, OperationErrorCode,
    OperationPrecondition, OperationScope, ReducerInput, ReducerResult, ReferenceEffect,
    ReferenceSnapshot, ReferenceTarget, RegionResolution, RegionResolutionStatus, SetMarksMode,
    TextAnchor,
};
pub use reducer::{apply_operations, reanchor_region};
