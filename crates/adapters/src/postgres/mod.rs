mod document;
mod error;
mod governance;
mod idempotency;
mod identity;
mod outbox;
mod permission;
mod store;
mod transaction;

pub use document::PostgresDocumentRepository;
pub use governance::PostgresGovernanceRepository;
pub use idempotency::{
    IdempotencyDecision, IdempotencyError, IdempotencyIdentity, IdempotencyReservation,
    StoredResponse, complete_idempotency, reserve_idempotency,
};
pub use identity::PostgresIdentityRepository;
pub use outbox::{OutboxAppendError, OutboxEventInput, append_outbox_event};
pub use permission::PostgresPermissionRepository;
pub use store::{DatabaseSettings, PostgresPreflight, PostgresStore};
pub use transaction::PgUnitOfWork;
