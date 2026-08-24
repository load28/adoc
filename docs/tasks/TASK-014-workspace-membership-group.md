# TASK-014: Workspace·Membership·Group 구현

- **상태**: 완료
- **유형**: 구현
- **구현 패키지**: IMP-07
- **시작일**: 2026-08-25
- **완료일**: 2026-08-25
- **커밋**: 이 태스크 완료 커밋

## 목적

Workspace tenant 경계와 Owner·Admin·Member governance, 만료 가능한 invitation, active Member로만
구성되는 Group을 구현한다. 이후 Permission·Document·Job이 재사용할 Membership extractor와 revision
event를 고정한다.

## 범위

- 포함: Workspace 생성·조회·수정·삭제 예약·취소, Membership 조회·role 변경·제거, invitation
  생성·조회·폐기·수락, Group CRUD·member 변경, opaque invitation capability, session revoke,
  idempotency·outbox, PostgreSQL repository, Axum API와 contract·통합 테스트
- 제외: Document Permission·PublishPolicy(IMP-08), invitation mail worker(IMP-17), 실제 Workspace
  purge·Audit projection(IMP-16), TanStack 화면(IMP-22·26)

## 필수 설계 문서

- [x] `product/PRD.md`, `product/features/WORKSPACE-AND-GOVERNANCE.md`, `product/USER-JOURNEYS.md`
- [x] `domain/workspace-governance.md`
- [x] UX: `design/ux/WORKSPACE-PERMISSION-FLOWS.md`, `design/ux/SCREEN-BEHAVIOR-SPECS.md`
- [x] 데이터: `design/data/LOGICAL-SCHEMA.md`, `design/data/schema.sql`,
  `design/data/DATABASE-INVARIANT-TRANSACTION-MATRIX.md`, `design/data/LIFECYCLE-RETENTION.md`
- [x] API: `design/api/openapi.yaml`, `design/api/API-CONVENTIONS.md`,
  `design/api/COMMAND-QUERY-CATALOG.md`, `design/api/ERROR-CATALOG.md`
- [x] 보안: `design/security/AUTHENTICATION-SESSION.md`, `design/security/AUTHORIZATION.md`,
  `design/security/THREAT-MODEL.md`
- [x] 테스트: `design/quality/TEST-STRATEGY.md`, `design/quality/FIXTURE-CATALOG.md`,
  `design/quality/CONTRACT-COVERAGE.md`
- [x] 구현 기준: `design/implementation/WORKSPACE-MEMBERSHIP-GROUP.md`,
  `design/implementation/MODULE-INTERFACE-CATALOG.md`

## 문서 준비 게이트

- [x] 구현 결과를 바꿀 미해결 질문이 없다.
- [x] 정상·만료·폐기·email mismatch·last Owner·tenant·revision race가 정의됐다.
- [x] domain, repository transaction, token, outbox, HTTP 계약이 타입 수준으로 정의됐다.
- [x] IMP-07과 IMP-08·16·17·22·26의 책임 경계를 추적할 수 있다.
- [x] PLAN-13을 구현 기준으로 사용해 코드를 작성할 수 있다.

## 사용자 결정

사용자는 모든 Google domain 허용, PostgreSQL·Redis, 전체 제품 구현과 권장안의 자율 적용을
확정했다.

## 의사결정

### 결정 1: 마지막 Owner는 application transaction lock으로 보장한다

- **상황**: 단순 count 후 update는 동시 강등·제거에서 Owner 0명을 만들 수 있다.
- **검토한 대안**: UI 차단 / trigger / Workspace와 active Owner row를 잠그는 transaction.
- **선택과 근거**: Workspace와 active Owner 전체를 고정 순서로 잠근 뒤 결과 상태를 계산한다.

### 결정 2: invitation token은 서명 capability로 재생성한다

- **상황**: mail 재시도를 위해 token 원문을 DB·outbox에 저장하면 credential 유출 범위가 커진다.
- **검토한 대안**: 원문 저장 / commit 뒤 일회 전송 / invitation ID와 HMAC으로 결정적 재생성.
- **선택과 근거**: token hash와 key ID만 저장하고 outbox consumer가 같은 token을 재생성한다.

### 결정 3: Membership privilege 변경은 session revoke와 event를 원자 처리한다

- **상황**: cache invalidation이나 session 종료가 늦으면 제거된 사용자의 권한이 남는다.
- **검토한 대안**: TTL 대기 / Redis best effort / PostgreSQL session revoke+outbox 원자 commit.
- **선택과 근거**: 정본 DB에서 session을 즉시 revoke하고 revision event를 같은 transaction에 쓴다.

## 구현 순서

1. PLAN-13과 영향받는 정본 API·domain·DDL 계약을 고정한다.
2. governance domain value·state·capability와 application port·service를 구현한다.
3. PostgreSQL transaction, idempotency, session revoke와 outbox를 구현한다.
4. Axum Workspace extractor·CSRF command route와 web proxy를 연결한다.
5. unit·contract·PostgreSQL concurrency·HTTP security·Compose와 전체 gate를 실행한다.
6. 완료 기록 후 commit·push하고 IMP-08로 진행한다.

## 작업 내역

- 2026-08-25: IMP-07 태스크를 등록하고 제품·도메인·UX·API·DDL·보안 정본을 확인했다.
- 2026-08-25: PLAN-13에 tenant capability, last Owner lock, invitation capability, session·outbox
  원자성과 검증 gate를 고정했다.
- 2026-08-25: 기존 invitation을 안전하게 폐기하면서 capability key ID와 expiry 불변식을 추가하는
  forward-only migration을 작성했다.
- 2026-08-25: governance domain·application service·PostgreSQL repository와 20개 HTTP operation을
  구현하고 Rust·TypeScript 계약을 재생성했다.
- 2026-08-25: last Owner 동시성, tenant 비노출, invitation 재생·email 불일치 비소비, Group active
  member, idempotency와 outbox 원자성을 PostgreSQL 계약 테스트로 고정했다.
- 2026-08-25: 전체 root gate와 PostgreSQL 16·Redis·backup·OpenSearch Compose 통합 검증을 통과했다.

## 이슈 및 해결

- Compose 계약 테스트가 secret file 환경 변수를 읽지 않아 DB 검증을 건너뛰었다. 테스트 연결 설정이
  plain 값과 `_FILE` secret을 같은 우선순위 계약으로 읽도록 수정했다.
- test-runner 이미지가 이전 소스를 재사용했다. 통합 스크립트가 실행 전에 test-runner를 명시적으로
  build하도록 변경해 실행 소스와 검증 대상을 일치시켰다.
- Group 변경과 outbox append가 독립 매개변수를 과도하게 노출했다. `GroupMemberCommand`,
  `GroupMutation`, `OutboxEvent` 값 객체로 경계를 묶고 Clippy 무경고로 고정했다.
- idempotency 완료 receipt가 갱신되지 않아도 성공할 수 있었다. 갱신 행 수를 불변식으로 검사하고
  불일치 transaction을 원자적으로 취소하도록 수정했다.

## 검증

- [x] governance domain·application unit와 negative corpus
- [x] PostgreSQL last Owner·tenant·revision·idempotency concurrency
- [x] invitation one-shot·email mismatch non-consume·key rotation
- [x] Group active member·name collision·in-use invariant
- [x] HTTP CSRF·Origin·IDOR·problem response
- [x] 실제 PostgreSQL 16·Redis Compose integration과 root gate

## 결과

Workspace·Membership·Invitation·Group의 domain·application·PostgreSQL·HTTP 수직 경계를 구현했다.
last Owner, tenant 비노출, privilege 변경 session revoke, invitation capability, Group membership,
idempotency와 outbox 원자성을 실제 PostgreSQL·Compose 검증으로 고정했다.
