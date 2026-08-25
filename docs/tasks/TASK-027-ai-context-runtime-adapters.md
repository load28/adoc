# TASK-027: AI Context·Runtime Adapter 구현

- **상태**: 완료
- **유형**: 구현
- **구현 패키지**: IMP-20
- **시작일**: 2026-08-25
- **완료일**: 2026-08-25
- **커밋**: —

## 목적

사용자 권한과 명시적 작업 범위 안에서만 근거가 추적되는 AI Context를 구성하고, 같은
application port를 통해 로컬 CLI와 OpenAI API runtime을 교체할 수 있게 구현한다. provider
차이가 Task 정책·Source·보안·취소·사용량 계약을 바꾸지 않도록 한다.

## 범위

- 포함: Task registry, Context Builder, embedding 경계, permission-safe retrieval 연결,
  provider-neutral runtime port, 로컬 CLI·OpenAI API adapter, 취소·timeout·usage·redaction,
  실제 adapter 계약 테스트
- 제외: AI Result·Proposal 적용과 Writing Rule validation(IMP-21), AI 화면(IMP-25), 외부 Web
  retrieval provider

## 필수 설계 문서

- [x] 제품·도메인: `product/features/WRITING-INTELLIGENCE.md`,
  `domain/writing-intelligence.md`, `domain/knowledge.md`
- [x] AI Task·Context·Result·계약: `design/specs/ai/TASK-CONTEXT-RESULT.md`,
  `design/specs/ai/JOB-RUNTIME.md`, `design/contracts/ai-contracts.schema.json`,
  `design/architecture/INTEGRATION-ARCHITECTURE.md`
- [x] UX·API·Job·검색 경계: `design/ux/KNOWLEDGE-AI-FLOWS.md`,
  `design/api/openapi.yaml`, `design/api/COMMAND-QUERY-CATALOG.md`,
  `design/implementation/JOB-RUNTIME-SSE.md`,
  `design/implementation/HYBRID-RETRIEVAL-SOURCE.md`
- [x] 데이터 계약: `design/data/LOGICAL-SCHEMA.md`, `design/data/schema.sql`,
  `design/data/DATABASE-INVARIANT-TRANSACTION-MATRIX.md`
- [x] 권한·보안·실패·운영: `design/security/AUTHORIZATION.md`,
  `design/security/THREAT-MODEL.md`, `design/security/AI-AND-FILE-SECURITY.md`,
  `design/architecture/SCALABILITY-CAPACITY.md`,
  `design/operations/OBSERVABILITY-SLO.md`
- [x] 테스트 기준: `design/quality/TEST-STRATEGY.md`,
  `design/quality/SECURITY-TESTS.md`, `design/quality/PERFORMANCE-TESTS.md`,
  `design/quality/CONCURRENCY-RECOVERY-TESTS.md`,
  `design/quality/AI-WRITING-EVALUATION.md`
- [x] TASK-027 구현 기준 문서: `design/implementation/AI-CONTEXT-RUNTIME-ADAPTERS.md`

## 문서 준비 게이트

- [x] 구현 결과를 바꿀 미해결 질문이 없다.
- [x] 정상·예외·권한·동시성 흐름이 정의되어 있다.
- [x] Context item·runtime request/response·usage 계약이 타입 수준으로 정의되어 있다.
- [x] 구현 단위와 source coverage·same-port 완료 조건을 문서에서 추적할 수 있다.
- [x] PLAN-26과 정본 감사를 근거로 코드 작성 가능을 확인했다.

## 사용자 결정

사용자는 기존 설계와 구조적 권장안을 별도 승인 없이 적용하고, AGENTS.md 원칙에 따라
이전과 같은 방식으로 구현을 계속하도록 확정했다.

## 의사결정

### 결정 1: Context preview를 무상태 fingerprint로 admission과 결합한다

- **상황**: 사용자가 Job 실행 전에 Source를 확인·제외해야 하지만 preview와 실행 사이 state
  변경도 차단해야 한다.
- **검토한 대안**: Job 생성 후 표시 / 서버측 preview session / current Context 무상태 재구성.
- **선택과 근거**: 같은 Builder의 canonical fingerprint를 preview와 create에서 비교한다. 별도
  임시 정본 없이 권한·revision·Source 변경을 모두 stale로 검출한다.

### 결정 2: Context snapshot을 AI 전용 row에 저장하고 generic Job에는 ID만 둔다

- **상황**: Worker가 사용자가 확인한 본문을 실행해야 하지만 범용 Job payload는 본문·prompt를
  금지한다.
- **검토한 대안**: 실행 시 최신 본문 / generic payload / AI Context 전용 snapshot.
- **선택과 근거**: bounded snapshot column과 permission evidence를 사용한다. retry 재현성과
  payload 비노출을 동시에 보장한다.

### 결정 3: DB snapshot 사이에서 외부 retrieval을 실행하고 revision을 재검사한다

- **상황**: 일관된 Context가 필요하지만 embedding·OpenSearch를 DB transaction 안에서 호출할
  수 없다.
- **검토한 대안**: 긴 transaction / 사후 검증 없음 / A·외부·B staged builder.
- **선택과 근거**: 두 repeatable-read snapshot의 stamp를 비교하고 한 번만 bounded retry한다.
  transaction pool 점유 없이 혼합 snapshot을 차단한다.

### 결정 4: 두 runtime은 tool 없는 strict structured-output port를 공유한다

- **상황**: Codex CLI와 OpenAI Responses의 wire 형식은 다르지만 제품 결과 의미는 같아야 한다.
- **검토한 대안**: provider별 application service / free-form JSON parse / 동일 runtime port.
- **선택과 근거**: Task별 JSON Schema, cancellation, usage와 stable failure를 port에 고정한다.
  OpenAI는 store·tools를 끄고 CLI는 빈 read-only job root에서 실행한다.

### 결정 5: Codex CLI 입력은 stdin의 bounded canonical JSON으로 전달한다

- **상황**: 빈 작업 루트에서 입력 파일 경로만 전달하면 모델이 파일 읽기 tool을 호출해야 하며,
  tool 없는 단발 실행 계약과 충돌한다.
- **검토한 대안**: 입력 파일 읽기 허용 / application 경로 전달 / canonical JSON stdin 전달.
- **선택과 근거**: schema만 read-only 파일로 두고 입력은 shell을 거치지 않는 stdin으로 전달한다.
  저장소 접근을 열지 않으면서 provider가 동일 artifact를 직접 받는다.

## 구현 순서

1. 제품·도메인·AI·검색·권한·보안·운영 정본을 감사한다.
2. TASK-027에 필요한 상세 구현 문서를 코드보다 먼저 작성한다.
3. Task registry·Context Builder·embedding·runtime port와 adapter를 구현한다.
4. same-port·source coverage·non-leak·취소·timeout 계약을 검증한다.
5. 전체 gate를 통과하고 태스크를 완료한다.

## 작업 내역

- 2026-08-25: TASK-027을 등록하고 IMP-20 정본 감사를 시작했다.
- 2026-08-25: 제품·도메인·AI·검색·권한·보안·운영·품질 정본을 교차 감사했다.
- 2026-08-25: 공식 OpenAI Responses·Embeddings 계약과 설치된 Codex CLI exec surface를
  확인했다.
- 2026-08-25: PLAN-26에 preview, staged Context Builder, snapshot 저장, runtime port,
  cancellation·usage·실패·테스트 계약을 확정했다.
- 2026-08-25: API·AI·Integration·Data 정본과 migration·generated contract를 보강하고 문서
  준비 게이트를 통과했다.
- 2026-08-25: Writing Intelligence 계층에 closed Task registry, Context Source identity·limit,
  canonical artifact fingerprint와 runtime request/result 타입을 구현했다.
- 2026-08-25: provider-neutral runtime·embedding port와 Codex CLI·OpenAI Responses·Embeddings
  adapter를 구현하고 동일 output 의미, refusal·tool·output limit 경계를 검증했다.
- 2026-08-25: PostgreSQL staged Context materializer와 permission evidence 재검사, Context snapshot,
  AI·generic Job 원자 admission 및 사용자 대상 `AIJobChanged` Outbox를 구현했다.
- 2026-08-25: preview·create/list/get/cancel API와 Redis signal 복구, Worker runtime 조립,
  cancel·timeout·terminal late-result guard를 연결했다.
- 2026-08-25: 취소 Idempotency-Key receipt, local CLI process-group TERM→KILL, 실제 PostgreSQL
  Source coverage·non-leak·Outbox 통합 테스트를 보강했다.
- 2026-08-25: `bun run check` 전체 gate와 TASK-027 Compose PostgreSQL 계약을 통과했다.

## 이슈 및 해결

- sealed migration `0019`를 후속 admission 계약에 맞춰 수정하려는 시도를 migration check가
  차단했다. 기존 파일을 원복하고 추가 계약을 `0020` append-only migration으로 분리했다.
- 실제 PostgreSQL fixture가 document rank 길이, PublishedVersion 필수 snapshot metadata,
  Reference Region nullability·target-kind 제약을 차례로 위반했다. 제품 경로에 예외를 넣지 않고
  fixture를 정본 DDL과 동일한 유효 데이터로 교정했다.
- 문서 Source의 permission key가 없을 때 검증을 건너뛸 수 있는 조건을 발견했다. 모든 문서
  Source가 현재 scope의 exact permission key와 일치해야 한다는 단일 불변식으로 강화했다.
- 월간 사용량 집계의 PostgreSQL `sum(bigint)` 결과가 `numeric`인데 Rust 경계가 `i64`로
  가정해 admission이 실패했다. SQL 계약에서 bounded 합계를 `bigint`로 명시해 타입 경계를
  고정했다.
- 전체 Compose 검증은 TASK-027 AI 계약과 선행 PostgreSQL 계약을 통과한 뒤 기존
  `search_projection`의 `SEARCH_REQUEST_REJECTED`에서 중단됐다. TASK-027 범위 밖의 독립된
  검색 검증 결함으로 분리해 후속 태스크에서 다룬다.

## 검증

- [x] 문서 링크와 정본 경계 확인
- [x] PRD·도메인·상세 설계 간 모순 확인
- [x] same-port·source coverage·permission non-leak 검증
- [x] generated contract·root·TASK-027 Compose 계약 gate

## 결과

권한 증거와 revision을 재검사하는 staged Context Builder, AI 전용 snapshot·admission,
provider-neutral runtime port와 Codex CLI·OpenAI adapter를 구현했다. preview부터 Worker 실행,
취소, timeout, usage, 사용자 Outbox까지 같은 도메인 계약으로 연결했고 실제 PostgreSQL에서
Source coverage와 비노출 불변식을 검증했다.
