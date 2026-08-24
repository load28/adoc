# Module Architecture

- **문서 ID**: ARCH-03
- **상태**: 동결

## Monorepo 경계

```text
apps/web
apps/api
apps/worker
crates/{kernel,identity,governance,document,collaboration,knowledge,ai,operations}
crates/{application,ports,adapters,telemetry,test-support}
packages/{contracts,editor-schema,ui-domain,i18n}
infra/{docker,migrations,opensearch}
```

`packages/ui-domain`은 design system이 아니라 Atlaskit을 조합한 domain UI다.

## 의존 방향

Domain → kernel만 의존한다. Application → domain·ports. Adapter → application·ports와 외부
SDK. Transport → application. Domain은 Axum, SQLx, Redis, OpenSearch, Tiptap과 Provider SDK를
모른다.

## Bounded context

- Identity: User, OIDC Identity, Session
- Governance: Workspace, Membership, Group, Permission, PublishPolicy
- Document: Tree, Draft, Content, Lease, Version, Public Link
- Collaboration: Discussion, Review, Inbox
- Knowledge: Reference, Vocabulary, Retrieval Source
- Writing Intelligence: AITask, Context, Result, Proposal
- Operations: FileAsset, Audit, retention

Context 간 직접 table write를 금지한다. 같은 transaction이 필요한 invariant는 owner context의
application service가 port를 호출하고 outbox로 후속 projection을 전달한다.

## 공통 kernel

WorkspaceId, typed ID, Revision, Version, UTC Instant, Actor, IdempotencyKey, DomainError와
PermissionScope만 공유한다. 편의를 위한 generic repository나 base entity는 만들지 않는다.
