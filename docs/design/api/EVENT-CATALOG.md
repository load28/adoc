# Event Catalog

- **문서 ID**: API-04
- **상태**: 동결

| Event | Producer | Consumer | 핵심 payload |
|---|---|---|---|
| WorkspaceChanged | Governance | SSE | workspaceId, revision, action |
| MembershipChanged | Governance | permission cache, Inbox, lease cleanup | userId, before, after |
| InvitationChanged | Governance | settings SSE | invitationId, revision, action |
| GroupChanged | Governance | permission cache·projection | groupId, member delta |
| PermissionChanged | Governance | Search permission projection, SSE | documentId, affected root, revision |
| PublishPolicyChanged | Governance | Review policy cache, SSE | documentId, revision, effective policy |
| DocumentChanged | Document | Search, Reference display, SSE | documentId, action, revision, tree revision |
| DocumentMoved | Document | Search, Reference display, SSE | documentId, before/after parent, tree revision |
| DraftChanged | Document | Review invalidator, SSE | documentId, revision, operation IDs |
| LeaseChanged | Document | SSE | documentId, holder, expiry |
| VersionPublished | Document | Search, File ref, Inbox, public cache | versionId, number, source revision |
| DiscussionChanged | Collaboration | Inbox, AI context invalidator | discussionId, revision, action |
| MessageChanged | Collaboration | Discussion SSE, AI context invalidator | messageId, revision, action |
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

Browser Stream으로 투영하는 Event는 producer가 `INTERNAL|WORKSPACE|ADMIN|USER|DOCUMENT` audience를
같은 transaction에 기록한다. SSE payload의 일반 변경 알림은 `entityId`, `revision`, `action`으로
정규화하고 상세 상태는 해당 query로 다시 읽는다. `InvitationDeliveryRequested` 같은 내부 명령 Event는
`INTERNAL`이며 Browser contract로 검증하거나 전송하지 않는다.
