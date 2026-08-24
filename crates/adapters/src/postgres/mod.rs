mod audit;
mod collaboration;
mod document;
mod error;
mod file;
mod governance;
mod idempotency;
mod identity;
mod knowledge;
mod outbox;
mod permission;
mod publishing;
mod retention;
mod store;
mod transaction;

pub use audit::{PostgresAuditRepository, append_audit_event};
pub use collaboration::PostgresCollaborationRepository;
pub use document::PostgresDocumentRepository;
pub use file::PostgresFileRepository;
pub use governance::PostgresGovernanceRepository;
pub use idempotency::{
    IdempotencyDecision, IdempotencyError, IdempotencyIdentity, IdempotencyReservation,
    StoredResponse, complete_idempotency, reserve_idempotency,
};
pub use identity::PostgresIdentityRepository;
pub use knowledge::PostgresKnowledgeRepository;
pub use outbox::{OutboxAppendError, OutboxEventInput, append_outbox_event};
pub use permission::PostgresPermissionRepository;
pub use publishing::PostgresPublishingRepository;
pub use retention::PostgresRetentionRepository;
pub use store::{DatabaseSettings, PostgresPreflight, PostgresStore};
pub use transaction::PgUnitOfWork;
