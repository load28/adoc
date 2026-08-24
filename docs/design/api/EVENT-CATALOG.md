# Event Catalog

- **문서 ID**: API-04
- **상태**: 동결

| Event | Producer | Consumer | 핵심 payload |
|---|---|---|---|
| MembershipChanged | Governance | permission cache, Inbox, lease cleanup | userId, before, after |
| GroupChanged | Governance | permission cache·projection | groupId, member delta |
| PermissionChanged | Governance | Search permission projection, SSE | documentId, affected root, revision |
| PublishPolicyChanged | Governance | Review policy cache, SSE | documentId, revision, effective policy |
| DocumentMoved | Document | Search, Reference display, SSE | documentId, before/after parent |
| DraftChanged | Document | Review invalidator, SSE | documentId, revision, operation IDs |
| LeaseChanged | Document | SSE | documentId, holder, expiry |
| VersionPublished | Document | Search, File ref, Inbox, public cache | versionId, number, source revision |
| DiscussionChanged | Collaboration | Inbox, AI context invalidator | discussionId, revision, action |
| ReviewChanged | Collaboration | Inbox, publish gate cache | reviewId, draftRevision, status |
| VocabularyChanged | Knowledge | index, AI rule cache | conceptId, revision |
| AIJobChanged | Writing Intelligence | SSE, usage metrics | jobId, sequence, status |
| ProposalApplied | Writing Intelligence | Audit, Discussion UI | proposalId, documentId, revision |
| FileChanged | Operations | preview, UI | assetId, status |
| PublicLinkChanged | Document | public cache | linkId, documentId, status |

## 규칙

Event는 이미 발생한 사실을 과거형 의미로 표현한다. consumer가 producer table을 직접
수정하지 않는다. payload에 본문·prompt·file bytes를 넣지 않고 필요한 stable ID·revision만
전달한다. schema version은 envelope와 payload에 명시한다.
