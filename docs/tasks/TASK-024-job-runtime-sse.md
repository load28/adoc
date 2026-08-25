# TASK-024: Job Runtime·SSE 구현

- **상태**: 완료
- **유형**: 구현
- **구현 패키지**: IMP-17
- **시작일**: 2026-08-25
- **완료일**: 2026-08-25
- **커밋**: —

## 목적

PostgreSQL을 정본으로 하는 범용 Job 실행기와 Outbox consumer를 만들고, Redis를 유실 가능한 wake-up
신호로만 사용한다. 브라우저에는 권한이 적용된 Workspace 변경을 bounded·resumable SSE로 전달해 queue
유실, worker 재시작, 중복 event, 연결 단절과 느린 consumer가 domain transaction을 손상시키지 않게 한다.

## 범위

- 포함: Job claim·lease·retry·cancel·dead-letter, Redis wake·reconcile, Outbox claim·consumer receipt,
  Workspace stream ledger·cursor·reset·heartbeat·backpressure, worker·API wiring, DDL·migration·contract·통합 테스트
- 제외: OpenSearch projection consumer(IMP-18), AI provider 실행과 AI Job API(IMP-20~21), Web SSE cache
  reconciliation UI(IMP-22·25), invitation mail provider, 운영자 replay UI(IMP-27)

## 필수 설계 문서

- [x] 제품·도메인: `product/IMPLEMENTATION-SCOPE.md`, `domain/operations.md`
- [x] 시스템 경계: `design/architecture/TRANSACTION-EVENT-JOB.md`, `design/adr/ADR-004-http-sse.md`
- [x] 상태·데이터: `design/specs/STATE-TRANSITION-CATALOG.md`, `design/data/schema.sql`,
  `design/data/DATABASE-INVARIANT-TRANSACTION-MATRIX.md`
- [x] API·이벤트: `design/api/STREAMING-JOBS.md`, `design/api/EVENT-CATALOG.md`,
  `design/api/asyncapi.yaml`, `design/contracts/event-payloads.schema.json`
- [x] 권한·보안: `design/security/AUTHORIZATION.md`, `design/specs/AUTHORIZATION-MATRIX.md`
- [x] 실패·복구·운영: `design/quality/CONCURRENCY-RECOVERY-TESTS.md`,
  `design/operations/OBSERVABILITY-SLO.md`, `design/architecture/SCALABILITY-CAPACITY.md`
- [x] 테스트 전략: `design/quality/TEST-STRATEGY.md`, `design/quality/SECURITY-TESTS.md`
- [x] 구현 기준: `design/implementation/JOB-RUNTIME-SSE.md`

## 문서 준비 게이트

- [x] 구현 결과를 바꿀 미해결 질문이 없다.
- [x] 정상·예외·권한·동시성 흐름이 정의되어 있다.
- [x] Job·Outbox·Redis·SSE 경계가 타입·row·cursor 수준으로 정의되어 있다.
- [x] 구현 단위와 테스트 완료 조건을 PLAN-23에서 추적할 수 있다.
- [x] 정본 검토와 PLAN-23 확정 뒤 코드 작성이 가능하다.

## 사용자 결정

사용자는 기존 설계와 구조적 권장안을 별도 승인 없이 적용하고, AGENTS.md 원칙에 따라 다음 태스크를
이전과 같은 방식으로 계속 진행하도록 확정했다.

## 의사결정

### 결정 1: Outbox delivery를 첫 범용 Job handler로 둔다

- **상황**: 실제 handler가 없는 Job 추상화와 별도 Outbox loop는 runtime 계약을 검증하지 못하거나 중복한다.
- **검토한 대안**: 사용처 없는 runner / Outbox 전용 loop / delivery Job.
- **선택과 근거**: Outbox와 `OUTBOX_TO_STREAM` Job을 같은 transaction에 만들고 consumer receipt까지 generic
  runner로 실행한다.

### 결정 2: Redis를 유실 가능한 wake-up으로 제한한다

- **상황**: Redis payload가 정본이면 flush와 failover 때 Job을 잃는다.
- **검토한 대안**: Redis-only queue / PostgreSQL polling-only / PostgreSQL 정본+Redis wake.
- **선택과 근거**: Redis에는 Job ID만 중복 허용 signal하고 PostgreSQL reconcile이 누락을 복구한다.

### 결정 3: producer가 구조화된 Event audience를 기록한다

- **상황**: consumer가 payload key나 삭제된 row로 권한을 추론하면 tenant leak과 불안정한 예외 분기가 생긴다.
- **검토한 대안**: 전송 시 payload heuristic / 모든 Member에게 전송 / producer audience descriptor.
- **선택과 근거**: `INTERNAL|WORKSPACE|ADMIN|USER|DOCUMENT` audience를 domain transaction에서 확정하고
  SSE가 현재 권한을 다시 검사한다.

### 결정 4: PostgreSQL replay ledger와 Redis Pub/Sub wake를 분리한다

- **상황**: 10,000 SSE connection에 polling만 쓰면 DB load가 크고 Redis Pub/Sub만 쓰면 reconnect replay가 없다.
- **검토한 대안**: connection polling / Pub/Sub-only / durable ledger+shared wake hub.
- **선택과 근거**: 24시간 Stream ledger가 cursor 정본이고 API instance당 한 Pub/Sub subscriber가 bounded
  process-local hub를 깨운다. wake 유실은 heartbeat DB 조회가 복구한다.

## 구현 순서

1. 관련 정본의 Job·Outbox·SSE 계약을 감사하고 PLAN-23을 확정한다.
2. PostgreSQL Job·Outbox claim과 consumer receipt를 구현한다.
3. Redis wake-up과 PostgreSQL reconcile을 worker에 연결한다.
4. 권한 필터형 stream ledger와 resumable SSE API를 구현한다.
5. loss·replay·cancel·cursor reset·backpressure 통합 테스트와 전체 gate를 수행한다.

## 작업 내역

- 2026-08-25: TASK-024를 등록하고 IMP-17 정본 감사를 시작했다.
- 2026-08-25: PLAN-23에서 Job 상태 머신, Redis reconcile, Outbox audience, Stream ledger·cursor·권한·
  backpressure 계약을 확정했다.
- 2026-08-25: DATA-04·07·08, ARCH-06, API-04·05, AsyncAPI·Event Schema 정본을 PLAN-23과 일치시켰다.
- 2026-08-25: 모든 Outbox producer에 구조화된 audience와 폐쇄형 payload를 적용하고 같은 transaction에서
  `OUTBOX_TO_STREAM` Job을 생성하도록 연결했다.
- 2026-08-25: PostgreSQL claim·lease·retry·cancel·dead-letter·consumer receipt와 Redis wake·reconcile을
  범용 Job runtime 및 Worker loop에 연결했다.
- 2026-08-25: 24시간 Workspace stream ledger, 현재 권한 필터, 불투명 cursor, reset, heartbeat,
  bounded backpressure를 `/api/v1/stream` SSE에 연결했다.
- 2026-08-25: migration 0017과 generated contract를 봉인하고 전체 저장소 및 Docker Compose 통합 gate를
  통과했다.

## 이슈 및 해결

- Rust 문자열을 PostgreSQL enum column에 직접 bind해 Audit·Retention 최종 Outbox append가 실패했다.
  SQL 경계에서 `event_audience_kind`와 `document_access`로 명시적으로 cast해 모든 producer와 consumer가
  같은 저장 타입 계약을 사용하도록 고쳤다.
- 백그라운드 Worker와 contract test가 같은 purge lease를 claim할 수 있었다. 통합 테스트 구간에서는
  서비스 기동 검증을 끝낸 Worker를 중지하고 각 contract test가 자체 clock·worker를 소유하도록 격리했다.
- 반복 Docker 검증이 `adoc-task017`의 미사용 이미지로 디스크를 소진했다. Compose project label로 범위를
  확인한 뒤 해당 프로젝트의 dangling 이미지만 제거하고 검증을 재개했다.

## 검증

- [x] Job lease·retry·cancel·dead-letter·crash recovery
- [x] Redis loss 뒤 PostgreSQL reconcile·중복 wake 멱등성
- [x] Outbox consumer receipt·aggregate ordering·중복 delivery
- [x] SSE permission filter·resume·expired cursor reset·heartbeat·backpressure
- [x] generated contract·migration·root·Compose gate

## 결과

IMP-17을 완료했다. Domain transaction은 구조화된 audience를 가진 Outbox와 delivery Job을 원자적으로
기록한다. Worker는 PostgreSQL을 정본으로 lease·retry·cancel·dead-letter를 수행하고 Redis 신호 유실을
reconcile한다. API는 24시간 ledger에서 현재 권한을 다시 확인해 resumable SSE를 제공한다.
`bun run check`와 `bun run compose:integration`을 통과했다.
