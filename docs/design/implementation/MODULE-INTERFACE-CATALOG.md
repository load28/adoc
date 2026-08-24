# Module Interface Catalog

- **문서 ID**: PLAN-06
- **상태**: 동결

## 공통 application contract

```rust
struct CommandContext { actor: Actor, workspace_id: WorkspaceId,
  idempotency: IdempotencyKey, correlation_id: CorrelationId, now: Instant }
struct QueryContext { actor: Actor, workspace_id: WorkspaceId, correlation_id: CorrelationId }
struct Expected<T> { value: T, revision: Revision }
enum AppError { Validation, Unauthorized, Forbidden, NotFound, Conflict,
  RateLimited, DependencyUnavailable, Internal }
```

Command handler는 `Result<Committed<T>, AppError>`, query handler는 `Result<T, AppError>`를
반환한다. domain crate는 async runtime·DB·HTTP type을 받지 않는다. serialized type은
`packages/contracts`와 JSON Schema에서 생성하고 domain value object로 변환한다.

## Context interface

| Owner crate | 제공 application service | 필요한 port | 발생 event |
|---|---|---|---|
| identity | Google callback, session rotate/revoke, preference | `OidcProvider`, `SessionRepository`, `Clock` | SessionChanged |
| governance | Workspace, Invitation, Member, Group, Permission, Policy | `GovernanceRepository`, `PermissionProjection`, `Mailer` | Membership/Group/Permission/PolicyChanged |
| document | Tree, Draft, Lease, Operation, Publish, PublicLink | `DocumentRepository`, `ContentValidator`, `FileReferencePort` | Document/Draft/Lease/Version/PublicLinkChanged |
| collaboration | Discussion, Message, Review, Inbox | `CollaborationRepository`, `InboxProjector` | Discussion/Review/InboxChanged |
| knowledge | Reference, Vocabulary, Search, Source | `KnowledgeRepository`, `SearchIndex`, `EmbeddingProvider` | Reference/VocabularyChanged |
| writing_intelligence | Task, Context, Job, Result, Proposal | `AIJobRepository`, `AIRuntime`, `ContextSourcePort`, `UsageMeter` | AIJobChanged, ProposalApplied |
| operations | File, Audit, retention, job/outbox | `ObjectStorage`, `AuditRepository`, `JobRepository`, `PurgePort` | File/PurgeChanged |

## 핵심 port signature

```rust
trait UnitOfWork {
  async fn execute<T>(&self, f: impl FnOnce(&mut Transaction) -> AppFuture<T>)
    -> Result<T, AppError>;
}
trait PermissionResolver {
  async fn point(&self, tx: &mut Transaction, actor: UserId, document: DocumentId)
    -> Result<EffectivePermission, AppError>;
  async fn scope(&self, actor: UserId, workspace: WorkspaceId)
    -> Result<PermissionScope, AppError>;
}
trait DocumentRepository {
  async fn lock_document_draft_lease(&self, tx: &mut Transaction, id: DocumentId)
    -> Result<DocumentEditState, AppError>;
  async fn commit_operations(&self, tx: &mut Transaction, state: Expected<Draft>,
    operations: NonEmpty<DocumentOperation>) -> Result<Draft, AppError>;
  async fn append_version(&self, tx: &mut Transaction, input: PublishSnapshot)
    -> Result<PublishedVersion, AppError>;
}
trait ObjectStorage {
  async fn begin(&self, key: StorageKey, limits: UploadLimits) -> Result<UploadHandle, StorageError>;
  async fn stat(&self, key: &StorageKey) -> Result<ObjectMetadata, StorageError>;
  async fn read(&self, key: &StorageKey, range: Option<ByteRange>) -> Result<ByteStream, StorageError>;
  async fn delete(&self, key: &StorageKey) -> Result<DeleteOutcome, StorageError>;
}
trait AIRuntime {
  async fn run(&self, request: ValidatedAIRequest, cancel: CancellationToken)
    -> Result<AIProviderResponse, AIRuntimeError>;
}
trait SearchIndex {
  async fn upsert_if_newer(&self, projection: SearchProjection) -> Result<(), SearchError>;
  async fn tombstone_if_newer(&self, tombstone: SearchTombstone) -> Result<(), SearchError>;
  async fn search(&self, query: ScopedSearchQuery) -> Result<SearchPage, SearchError>;
}
```

`Transaction`은 application 내부 opaque type이다. context repository끼리 서로의 concrete SQL
type을 import하지 않는다. owner service가 같은 transaction에 필요한 port method만 조합한다.

## Transport·frontend interface

Axum route는 OpenAPI operationId와 같은 handler name을 사용하고 request를 generated type으로
deserialize한다. extractor 순서는 `Session → CSRF(command) → Workspace → Idempotency → Handler`다.
SSE payload는 Event Schema를 그대로 사용하되 server가 actor permission으로 event type과 target을
filter한다.

TanStack route loader/action은 generated client만 호출한다. feature module public surface는
`route`, `queryKeys`, `screens`, `domainComponents`다. feature 간 store import를 금지하고 navigation
또는 typed resource ID로 연결한다. editor만 `packages/editor-schema`의 reducer·codec을 공유하며
server validator corpus와 동일 fixture를 실행한다.

## Adapter conformance

LocalObjectStorage와 S3ObjectStorage, CodexCliRuntime과 OpenAIResponsesRuntime은 동일 port contract
suite를 통과한다. fake adapter는 unit test에서만 쓰고 integration gate는 실제 PostgreSQL,
Redis, OpenSearch와 local filesystem adapter를 사용한다. adapter error는 stable domain-independent
category로 변환하고 provider 원문을 Browser나 log에 노출하지 않는다.
