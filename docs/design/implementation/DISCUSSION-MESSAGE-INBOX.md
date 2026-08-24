# Discussion·Message·Inbox 구현 계약

- **문서 ID**: PLAN-18
- **상태**: 구현 기준
- **구현 패키지**: IMP-12

## 책임과 aggregate

Discussion aggregate는 Document ID, title, OPEN/CLOSED, ordered Topic과 revision을 소유한다. Message는
Discussion에 속하지만 자체 revision·history lock을 가진다. InboxItem은 `(workspace,user,sourceKey)`로
deduplicate되는 사용자 projection이며 source aggregate와 별도 read·resolve 상태를 가진다.

## Discussion·Topic

create는 CONTRIBUTOR, effective active Document, trim title 1~500, Topic 1개 이상과 첫 Message를 요구한다.
Discussion과 Topic·Message·Mention Inbox·outbox를 한 transaction으로 만든다. Topic rank는 32자리
고정 폭 lexical sequence다. TEXT는 non-empty text, DOCUMENT는 same Workspace Document VIEWER,
REGION은 same Workspace Document VIEWER와 canonical Region, EXTERNAL은 HTTPS URL만 허용한다.

title 변경은 creator 또는 current Document EDITOR다. close·reopen과 Topic 추가는 CONTRIBUTOR,
Topic 제거는 creator 또는 EDITOR이며 마지막 Topic을 제거할 수 없다. 모든 mutation은 Discussion lock,
expected revision, workspace idempotency를 사용한다. CLOSED Discussion은 reopen 외 content mutation을 거부한다.

## Message·history

Message body는 IMP-09 `ValidatedContent`, mention은 unique active Member, attachment는 IMP-15 전까지 빈 배열만
허용한다. create는 OPEN Discussion CONTRIBUTOR다. update는 author만 생성 server time부터 15분 안에
가능하다. mutation 전에 현재 body와 mention ID를 `message_revisions(revision=N+1)`에 append한 뒤 current
Message revision을 N+1로 바꾼다.

delete operation은 물리 DELETE가 아니라 canonical empty body와 `deleted_at`을 기록하는 REDACTED 전이다.
author는 같은 15분 window, Document EDITOR는 window 없이 가능하다. 이미 redacted Message의 edit·delete는
`MESSAGE_STATE_INVALID`다. history API는 이 패키지의 외부 계약에 노출하지 않지만 DB append-only trigger와
integration test로 보존하고 향후 Audit·AI Context가 소비한다.

## Mention·Inbox

recipient는 actor 자신을 포함할 수 있다. 각 mention은 recipient가 active Member이며 source Document
CONTRIBUTOR인지 permission point resolver로 prefilter한다. 검증 실패는 recipient 존재를 구분하지 않는
`DISCUSSION_TARGET_INVALID`다. 성공 시 `mention:{messageId}:{userId}` source key로 MENTIONED InboxItem을
upsert한다. replay는 item을 늘리지 않는다. edit에서 빠진 recipient의 unresolved item은 system resolve하고
새 recipient는 생성한다. readAt은 절대 대신 쓰지 않는다.

Inbox query는 actor 자신의 row만 `(created_at DESC,id DESC)` cursor 50으로 읽는다. UNREAD는 read_at null,
ACTIONABLE은 resolved_at null, RESOLVED는 resolved_at non-null이다. read·resolve는 monotonic timestamp
`COALESCE` update이며 동일 command replay와 이미 반영된 상태 모두 같은 표현을 반환한다. read-all은
`created_at <= before`만 갱신해 command 시작 뒤 도착한 item을 읽음 처리하지 않는다.

## Transaction·event

lock 순서는 workspace idempotency → Discussion → Message → Inbox recipient UUID 순이다. permission은 lock
전에 계산하고 transaction 안에서 effective active와 Membership을 다시 확인한다. `DiscussionChanged.v1`,
`MessageChanged.v1`, `InboxChanged.v1` outbox에는 ID·revision·action만 담고 body·title·mention을 넣지 않는다.

## 검증 계약

- create all-or-nothing, 마지막 Topic 제거와 CLOSED mutation 거부
- stale Discussion·Message revision single winner
- prior Message body·mention snapshot append-only와 redaction 보존
- invalid/foreign/insufficient mention은 전체 command rollback
- idempotency replay와 outbox replay에서 Inbox source key 단일 row
- 다른 user·Workspace Inbox와 inaccessible Discussion 존재 비노출
