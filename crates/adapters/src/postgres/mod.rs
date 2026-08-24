mod collaboration;
mod document;
mod error;
mod governance;
mod idempotency;
mod identity;
mod knowledge;
mod outbox;
mod permission;
mod publishing;
mod store;
mod transaction;

pub use collaboration::PostgresCollaborationRepository;
pub use document::PostgresDocumentRepository;
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
pub use store::{DatabaseSettings, PostgresPreflight, PostgresStore};
pub use transaction::PgUnitOfWork;
