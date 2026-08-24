# TASK-023: Audit·Retention 구현

- **상태**: 완료
- **유형**: 구현
- **구현 패키지**: IMP-16
- **시작일**: 2026-08-25
- **완료일**: 2026-08-25
- **커밋**: —

## 목적

중요한 도메인 변경을 같은 PostgreSQL transaction의 구조화된 Audit Event로 남기고, Document·Workspace
삭제를 단계별 ledger로 안전하게 재개한다. immutable history와 ObjectStorage byte가 부분 실패나 worker
재시작 때문에 유실되거나 되살아나지 않도록 retention 경계를 완성한다.

## 범위

- 포함: Workspace 단조 Audit sequence, append-only event, 권한 기반 cursor query, 구현된 중요 command의
  원자 Audit 연결, Document·Workspace purge claim/step ledger, object deletion row, Audit tombstone, worker,
  API·DDL·migration·통합 테스트
- 제외: OpenSearch 실제 delete consumer(IMP-18), 일반 Job·SSE runtime(IMP-17), backup 저장소의 실제 key
  expiry(IMP-27), Audit·Trash UI(IMP-26), 아직 구현되지 않은 AI command의 Audit(IMP-21)

## 필수 설계 문서

- [x] 제품·도메인: `product/features/FILES-AND-AUDIT.md`, `domain/operations.md`
- [x] 상태·수명주기: `design/specs/operations/AUDIT.md`, `design/data/LIFECYCLE-RETENTION.md`
- [x] 데이터·API: `design/data/schema.sql`, `design/api/openapi.yaml`, `design/api/EVENT-CATALOG.md`
- [x] 권한·보안: `design/security/AUTHORIZATION.md`, `design/security/PRIVACY-RETENTION.md`
- [x] 실패·복구·동시성: `design/data/DATABASE-INVARIANT-TRANSACTION-MATRIX.md`,
  `design/quality/CONCURRENCY-RECOVERY-TESTS.md`
- [x] 테스트 전략: `design/quality/TEST-STRATEGY.md`, `design/quality/SECURITY-TESTS.md`
- [x] 구현 기준: `design/implementation/AUDIT-RETENTION.md`

## 문서 준비 게이트

- [x] 구현 결과를 바꿀 미해결 질문이 없다.
- [x] 정상·예외·권한·동시성 흐름이 정의되어 있다.
- [x] Audit·ledger·object deletion 경계가 타입·row 수준으로 정의되어 있다.
- [x] 구현 단위와 테스트 완료 조건을 PLAN-22에서 추적할 수 있다.
- [x] 정본 검토와 PLAN-22 확정 뒤 코드 작성이 가능하다.

## 사용자 결정

사용자는 기존 전체 제품 설계와 구조적 권장안을 별도 승인 없이 적용하고, AGENTS.md 원칙을 지켜 다음
태스크를 이전과 같은 방식으로 진행하도록 확정했다.

## 의사결정

### 결정 1: Audit append를 transaction primitive로 둔다

- **상황**: 각 repository가 sequence·JSON·redaction을 따로 구현하면 원자성과 action vocabulary가 갈라진다.
- **검토한 대안**: command 이후 비동기 consumer / DB trigger 자동 생성 / 명시적 transaction primitive.
- **선택과 근거**: command transaction이 actor·action·target을 아는 명시적 append primitive를 호출한다.
  sequence row lock과 insert가 실패하면 domain mutation도 rollback한다.

### 결정 2: Purge는 durable step machine이다

- **상황**: DB row와 ObjectStorage byte를 하나의 transaction으로 삭제할 수 없다.
- **검토한 대안**: 단일 긴 transaction / 매번 전체 상태 추론 / 단계별 durable ledger.
- **선택과 근거**: target당 하나의 ledger가 claim·step·retry를 소유한다. 각 step은 멱등이며 worker 재시작
  후 마지막 완료 경계부터 재개한다.

### 결정 3: Object deletion 목록은 ledger 하위 row다

- **상황**: domain cascade 뒤 storage key를 다시 계산할 수 없고, 큰 목록을 JSON 한 row에 넣으면 lock과
  update가 커진다.
- **검토한 대안**: ledger JSON array / 삭제 직전 memory 보관 / key당 deletion row.
- **선택과 근거**: `purge_object_deletions`가 storage key별 상태·attempt·error를 보존한다. byte delete는
  멱등이며 전체 row 완료 뒤 ledger를 완료한다.

### 결정 4: Local Compose의 admin credential은 retention 경계 테스트에만 허용한다

- **상황**: 현재 Local Compose는 일반·retention URL이 같은 PostgreSQL bootstrap credential이다.
- **검토한 대안**: local role provisioning 확대 / purge 미검증 / adapter에서 credential capability 검증.
- **선택과 근거**: retention repository는 전용 생성자와 URL에서만 만들어지고 production은 별도
  `adoc_retention` login을 요구한다. Local·test superuser는 명시적 transaction marker가 있는 purge step에만
  immutable mutation을 허용하며, API repository에는 purge primitive를 노출하지 않는다.

## 구현 순서

1. PLAN-22와 canonical DDL·OpenAPI·action vocabulary를 확정한다.
2. Audit domain·append/query adapter와 API를 구현한다.
3. 이미 구현된 중요 command transaction에 Audit append를 연결한다.
4. Document·Workspace purge ledger와 ObjectStorage deletion worker를 구현한다.
5. append-only·sequence·restore/claim race·retry·tombstone 통합 테스트와 전체 gate를 수행한다.

## 작업 내역

- 2026-08-25: TASK-023을 등록하고 Audit·Retention 정본과 기존 DDL·command transaction을 감사했다.
- 2026-08-25: PLAN-22에 Audit append primitive와 durable purge step machine을 확정했다.
- 2026-08-25: Audit 타입·append/query adapter·관리자 API를 구현하고 구현된 중요 command transaction에
  구조화된 Audit append를 연결했다.
- 2026-08-25: Document·Workspace purge claim, 단계형 ledger, storage key별 삭제 상태, Audit tombstone과
  worker 실행 경계를 구현했다.
- 2026-08-25: migration 0016과 generated contract를 봉인하고 PostgreSQL·Redis·ObjectStorage Docker
  통합 테스트 및 저장소 전체 gate를 통과했다.

## 이슈 및 해결

- 첫 ObjectStorage 실패 뒤 retry claim이 저장된 `DOMAIN_PURGED` 단계를 `ACCESS_REVOKED`로 되돌려 작업을
  취소했다. 최초 `PENDING` claim에서만 접근을 회수하고 retry는 저장된 단계에서 재개하도록 고쳤다.
- 상태 전이 SQL이 존재하지 않는 enum type cast를 사용했다. CHECK 제약이 있는 실제 text 저장 계약에 맞춰
  cast를 제거했다.
- Local superuser 전체에 immutable history 삭제를 허용해 기존 Vocabulary 이력 테스트가 회귀했다.
  `adoc_retention` 역할 또는 marker가 있는 retention transaction만 허용하는 공통 DB predicate로 좁혔다.

## 검증

- [x] Audit sequence 단조성·동시 append·append-only trigger
- [x] 중요 command와 Audit의 commit/rollback 원자성
- [x] Audit query permission·cursor·민감 metadata 제한
- [x] Document restore 대 purge claim race·단계 재개
- [x] Workspace access revoke·domain purge·object deletion·tombstone
- [x] generated contract·migration·root·Compose gate

## 결과

IMP-16을 완료했다. 중요 변경은 domain mutation과 같은 transaction의 append-only Audit Event로 남고,
Document·Workspace 영구 삭제는 durable step ledger에서 접근 회수, domain purge, byte 삭제, Audit redaction을
재시작 가능하게 수행한다. `bun run check`와 `bun run compose:integration`을 통과했다.
