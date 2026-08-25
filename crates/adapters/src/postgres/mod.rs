mod ai;
mod audit;
mod collaboration;
mod document;
mod error;
mod file;
mod governance;
mod idempotency;
mod identity;
mod jobs;
mod knowledge;
mod outbox;
mod permission;
mod proposal;
mod publishing;
mod retention;
mod retrieval;
mod search;
mod store;
mod stream;
mod transaction;

pub use ai::PostgresAiContextRepository;
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
pub use jobs::PostgresJobRepository;
pub use knowledge::PostgresKnowledgeRepository;
pub use outbox::{OutboxAppendError, OutboxEventInput, append_outbox_event};
pub use permission::PostgresPermissionRepository;
pub use proposal::PostgresWritingIntelligenceRepository;
pub use publishing::PostgresPublishingRepository;
pub use retention::PostgresRetentionRepository;
pub use retrieval::PostgresSearchRetrievalRepository;
pub use search::PostgresSearchProjectionRepository;
pub use store::{DatabaseSettings, PostgresPreflight, PostgresStore};
pub use stream::PostgresStreamRepository;
pub use transaction::PgUnitOfWork;
