#![forbid(unsafe_code)]

//! Document bounded context.

mod content;
mod model;
mod reducer;
mod tree;
mod version;

pub use content::{ValidatedContent, canonical_hash, normalize_content};
pub use model::{
    Affinity, AttrPatch, DocumentOperation, OperationError, OperationErrorCode,
    OperationPrecondition, OperationScope, ReducerInput, ReducerResult, ReferenceEffect,
    ReferenceSnapshot, ReferenceTarget, RegionResolution, RegionResolutionStatus, SetMarksMode,
    TextAnchor,
};
pub use reducer::{apply_operations, reanchor_region};
pub use tree::{
    Document, DocumentStatus, DocumentTitle, Draft, EditLease, LeaseDecision, TreeRank,
    TreeValidationError, validate_lease_acquire, validate_lease_holder,
};
pub use version::{DiffError, DocumentDiff, PublishedVersion, VersionPage, structural_diff};
