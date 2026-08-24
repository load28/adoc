# Conceptual Data Model

- **문서 ID**: DATA-01
- **상태**: 동결

## Aggregate 관계

```text
Workspace
├─ Membership ─ User
├─ Group ─ GroupMember
├─ Document ─ TreePosition
│  ├─ PermissionGrant / PublishPolicy
│  ├─ Draft ─ EditLease
│  ├─ PublishedVersion[]
│  ├─ Discussion[] ─ Topic[] ─ Message[]
│  ├─ Review[]
│  └─ PublicViewerLink[]
├─ VocabularyConcept[]
├─ Reference[]
├─ FileAsset[] ─ FileReference[]
├─ AIJob[] ─ AIResult / Proposal
├─ InboxItem[]
└─ AuditEvent[]
```

## Ownership

Aggregate owner만 자신의 invariant row를 변경한다. Reference와 FileReference는 서로 다른
owner를 연결하지만 target을 cascade delete하지 않는다. deletion planner가 impact를 계산한
뒤 각 owner command를 순서화한다.

## Identity와 snapshot

현재 상태 연결은 typed ID를 사용한다. 과거 의미 재현이 필요한 Version, Review, Source와
Audit은 표시 snapshot을 별도 보존한다. snapshot은 현재 entity를 갱신하지 않는다.

## Mutable와 immutable

Workspace 설정, Membership, Group, Document metadata, Draft, Discussion과 Job은 revision으로
변경한다. PublishedVersion, Message body revision, Review decision, Source snapshot와
AuditEvent는 append-only다.

## Projection

Inbox, Backlink query, Search index, preview와 analytics는 source aggregate에서 재생성 가능한
projection이다. 사용자 처리 상태처럼 재생성 불가능한 Inbox read·resolved는 PostgreSQL에
정본으로 둔다.
