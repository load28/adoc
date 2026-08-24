# State Transition Catalog

- **문서 ID**: SPEC-17
- **상태**: 동결

상태는 명시된 command만 바꾼다. expected revision 불일치는 어떤 side effect도 만들지 않는다.
아래 `같은 transaction`에는 Audit와 Outbox가 포함된다.

## Governance·Document

| Aggregate | From | Command | To | Guard | 같은 transaction |
|---|---|---|---|---|---|
| Workspace | — | create | ACTIVE | slug unique | Owner Membership·configuration 기본값 |
| Workspace | ACTIVE | schedule deletion | DELETION_SCHEDULED | Owner, 30일 | delete_after, session notification |
| Workspace | DELETION_SCHEDULED | cancel deletion | ACTIVE | purge 미시작 | delete_after 제거 |
| Workspace | DELETION_SCHEDULED | retention start | PURGING | delete_after 도래 | purge ledger |
| Workspace | PURGING | retention complete | DELETED | 모든 단계 성공 | minimal tombstone |
| Membership | — | invitation accept | ACTIVE | token·email·expiry | invitation ACCEPTED |
| Membership | ACTIVE | suspend | SUSPENDED | last Owner 아님 | lease 취소·permission invalidation |
| Membership | ACTIVE/SUSPENDED | remove | REMOVED | last Owner 아님 | group 제거·lease 취소 |
| Document | — | create | ACTIVE | parent Contributor | empty identity |
| Document | ACTIVE | trash | TRASHED | Editor | root timestamp·descendant lease 종료·review invalidation |
| Document | TRASHED | restore | ACTIVE | Editor, valid parent | root status·projection upsert |
| Document | TRASHED | purge start | PURGING | retention/Admin Manage | purge ledger·tombstone event |
| Document | PURGING | purge complete | row removed | refs·versions·files 처리 | minimal Audit tombstone |

Document move는 status transition이 아니라 ACTIVE→ACTIVE revision transition이다. cycle, new parent
permission, preview token fingerprint와 sibling rank를 commit 직전에 다시 검증한다.

## Draft·Lease·Publish

| Aggregate | From | Command/event | To | Guard·result |
|---|---|---|---|---|
| Draft | absent | create/get | revision 0 | current Version snapshot 또는 empty content |
| Draft | revision N | apply Operations | revision N+1 | lease, permission, schema, all preconditions atomic |
| Draft | revision N | Proposal apply | revision N+1 | dependency-closed operations, base N |
| Draft | present | publish | absent | Version·Context 생성 뒤 Draft 제거 |
| Lease | absent/expired | acquire | HELD | Contributor; force는 Manage+reason |
| Lease | HELD_BY_SELF | renew | HELD_BY_SELF | token·lease revision 일치 |
| Lease | HELD_BY_SELF | release | absent | token 일치 |
| Lease | HELD_BY_OTHER | force acquire | HELD_BY_SELF | Manage, impact confirm, reason |
| Version | — | publish | immutable row | policy, review, base/current, files, writing hard rules |

Draft mutation은 같은 Document의 REQUESTED·APPROVED Review를 INVALIDATED로 바꾸고 결정 효력을
제거한다.
Version number는 Document row lock 아래 current max+1이다. Version과 VersionContext는 일반 role로
UPDATE·DELETE할 수 없다.

## Collaboration·Knowledge

| Aggregate | From | Command/event | To | Guard |
|---|---|---|---|---|
| Discussion | — | create | OPEN | Topic 1개 이상 |
| Discussion | OPEN | close | CLOSED | Contributor, reason |
| Discussion | CLOSED | reopen | OPEN | Contributor, reason |
| Message | — | create | ACTIVE rev 0 | Discussion OPEN |
| Message | ACTIVE rev N | update | ACTIVE rev N+1 | author, edit window, prior Revision append |
| Message | ACTIVE | delete | REDACTED | author window 또는 Editor |
| Review | — | request | REQUESTED | current Draft revision, no active Review |
| Review | REQUESTED | approve | REQUESTED/APPROVED | assigned reviewer; threshold 계산 |
| Review | REQUESTED | changes request | CHANGES_REQUESTED | assigned reviewer, Discussion 선택 가능 |
| Review | REQUESTED | cancel | CANCELLED | requester 또는 Editor |
| Review | REQUESTED | Draft changed | INVALIDATED | revision 불일치 |
| Review | APPROVED | Draft changed | INVALIDATED | revision 불일치 |
| Review | APPROVED | publish | APPROVED 유지 | exact Draft revision을 Version Context에 snapshot |
| Vocabulary | — | create | ACTIVE | term Workspace unique |
| Vocabulary | ACTIVE | update | ACTIVE rev+1 | term conflict 없음 |
| Vocabulary | ACTIVE | deprecate | DEPRECATED | replacement optional |

APPROVED는 required approval 수를 충족한 시점의 Review aggregate 상태다. 한 reviewer가 결정을
바꾸면 threshold를 다시 계산한다. CHANGES_REQUESTED 후 재검토는 기존 Review를 되살리지 않고
새 Draft revision에 새 Review를 만든다.

## AI·File·Job

| Aggregate | From | Trigger | To | Guard·retry |
|---|---|---|---|---|
| AI Job | — | create | QUEUED | permission·quota·concurrency 예약 |
| AI Job | QUEUED | worker claim | RUNNING | lease+attempt increment |
| AI Job | QUEUED/RUNNING | cancel request | CANCEL_REQUESTED | owner |
| AI Job | RUNNING | validated result | SUCCEEDED | result schema·groundedness 검증 |
| AI Job | RUNNING | permanent failure | FAILED | non-retryable 또는 attempts 소진 |
| AI Job | RUNNING | deadline | TIMED_OUT | provider cancellation 요청 |
| AI Job | CANCEL_REQUESTED | worker ack | CANCELLED | provider/process 종료 확인 |
| Proposal | — | large AI result | OPEN | Operation schema·dependency graph valid |
| Proposal | OPEN | apply | APPLIED | base revision current |
| Proposal | OPEN | reject/cancel | REJECTED/CANCELLED | target permission |
| Proposal | OPEN | Draft changes | STALE | base revision 불일치 |
| File | — | create upload | UPLOADING | size·mime·quota |
| File | UPLOADING | checksum complete | READY | byte count·SHA-256 일치 |
| File | UPLOADING | failure/expiry | FAILED | object cleanup 예약 |
| File | READY | delete request | DELETED | reference 0, 30일 purge_after |
| Job | QUEUED | claim | RUNNING | `SKIP LOCKED`, lease |
| Job | RUNNING | transient failure | QUEUED | attempt < max, backoff |
| Job | RUNNING | attempts exhausted | DEAD_LETTER | operator alert |

terminal state는 다시 active state로 돌아가지 않는다. 재실행은 새 identity가 아니라 명시된
retry transition만 사용하고, DEAD_LETTER operator replay는 새 Job과 원 Job link를 만든다.
