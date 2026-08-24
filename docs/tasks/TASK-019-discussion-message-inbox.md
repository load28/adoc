# TASK-019: Discussion·Message·Inbox 구현

- **상태**: 완료
- **유형**: 구현
- **구현 패키지**: IMP-12
- **시작일**: 2026-08-25
- **완료일**: 2026-08-25
- **커밋**: 본 태스크 완료 커밋에 포함

## 목적

Document 권한 경계 안에서 복수 Topic Discussion과 append-history Message를 제공한다. Mention을
중복 없는 Inbox item으로 투영하고 read와 resolve 상태를 독립적으로 관리한다.

## 범위

- 포함: Discussion list/detail/create/title/close/reopen, Topic add/remove, Message create/edit/redact와
  revision history, Mention 검증·Inbox dedupe, Inbox list/read/read-all/resolve, idempotency·outbox
- 제외: Review(IMP-13), Reference graph(IMP-14), Attachment/File reference(IMP-15), SSE consumer(IMP-17), UI(IMP-24)

## 필수 설계 문서

- [x] 제품·도메인: `product/features/COLLABORATION.md`, `domain/collaboration.md`
- [x] 상태·UX: `design/specs/collaboration/DISCUSSION.md`,
  `design/specs/collaboration/REVIEW-INBOX.md`, `design/specs/STATE-TRANSITION-CATALOG.md`,
  `design/ux/COLLABORATION-FLOWS.md`
- [x] 데이터·API: `design/data/schema.sql`, `design/api/openapi.yaml`,
  `design/api/ERROR-CATALOG.md`, `design/contracts/event-payloads.schema.json`
- [x] 보안·품질: `design/security/AUTHORIZATION.md`, `design/specs/AUTHORIZATION-MATRIX.md`,
  `design/quality/TEST-STRATEGY.md`
- [x] 구현 기준: `design/implementation/DISCUSSION-MESSAGE-INBOX.md`

## 문서 준비 게이트

- [x] aggregate·상태·권한·edit window·redaction 의미가 정의되어 있다.
- [x] Topic target와 Mention의 tenant·permission 검증이 정의되어 있다.
- [x] Message history와 Inbox dedupe·read·resolve 저장 계약이 정의되어 있다.
- [x] API·transaction lock·idempotency·outbox와 후속 패키지 경계가 정의되어 있다.
- [x] 구현 결과를 바꿀 미해결 질문이 없다.

## 사용자 결정

사용자는 전체 제품 구현에서 구조적 권장안을 별도 승인 없이 적용하도록 확정했다.

## 의사결정

### 결정 1: Message 삭제는 물리 삭제가 아니라 redaction이다

- `deleted_at`과 tombstone body를 저장하고 row·revision history는 retention까지 보존한다.
- 작성자는 생성 후 15분 안에 수정·redact할 수 있고 Editor는 redaction만 시간 제한 없이 수행한다.

### 결정 2: Mention Inbox는 source key로 transaction 내 upsert한다

- `mention:{messageId}:{userId}`가 dedupe key다.
- active Membership과 target Document CONTRIBUTOR를 command 전에 검증하고 실패 시 전체 command를 거부한다.
- Message edit는 새 recipient를 upsert하고 제거된 recipient item을 resolve하되 read 상태를 바꾸지 않는다.

### 결정 3: 미구현 dependency는 fail closed다

- Attachment ID가 있으면 IMP-15 전까지 `DEPENDENCY_UNAVAILABLE`이다.
- Topic은 TEXT·DOCUMENT·REGION·EXTERNAL을 지원하지만 Reference graph materialization은 IMP-14가 소유한다.

## 구현 순서

1. PLAN-18과 canonical DDL·OpenAPI 공백을 고정한다.
2. Collaboration domain·Application port·PostgreSQL transaction을 구현한다.
3. HTTP route와 generated contract를 연결한다.
4. history·dedupe·permission·revision barrier 통합 테스트를 추가한다.
5. 전체 gate 후 완료 기록·commit·push하고 IMP-13으로 진행한다.

## 작업 내역

- 2026-08-25: TASK-019를 등록하고 PLAN-18로 aggregate·history·Inbox 경계를 고정했다.
- 2026-08-25: 정본 DDL·OpenAPI와 생성 계약에 Discussion·Topic·Message·Inbox 계약을 반영했다.
- 2026-08-25: Collaboration domain·application service·PostgreSQL adapter·HTTP route를 구현했다.
- 2026-08-25: Message revision 불변 이력, Mention dedupe, Inbox 단조 상태와 transactional outbox를 구현했다.
- 2026-08-25: 접근 불가 Discussion 존재 비공개와 비활성 Membership의 Inbox 차단을 통합 테스트로 고정했다.

## 이슈 및 해결

- 초기 구현에서 Inbox 변경 outbox가 빠진 것을 확인했다. Inbox 상태 변경과 같은 transaction에서
  aggregate sequence를 증가시키고 `InboxChanged.v1`을 기록하도록 구조적으로 해결했다.
- 테스트 fixture가 존재하지 않는 Membership 상태를 사용했다. 정본 DDL의 `SUSPENDED` 상태로 수정했다.

## 검증

- [x] Discussion state·Topic 최소 1개·target permission
- [x] Message edit window·append history·redaction
- [x] Mention membership·permission·Inbox dedupe
- [x] Inbox cursor·filter·read·resolve·tenant isolation
- [x] generated contract·root·Compose gate

## 결과

IMP-12를 완료했다. `bun run check`와 `bun run compose:integration`이 모두 통과했다.
