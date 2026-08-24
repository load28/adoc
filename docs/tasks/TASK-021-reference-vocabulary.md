# TASK-021: Reference·Vocabulary 구현

- **상태**: 완료
- **유형**: 구현
- **구현 패키지**: IMP-14
- **시작일**: 2026-08-25
- **완료일**: 2026-08-25
- **커밋**: 본 태스크 완료 커밋에 포함

## 목적

Document·Region·Discussion·Vocabulary·External target을 안정적으로 연결하고 Workspace 공통 용어를
충돌 없이 관리한다. 권한 없는 source가 Backlink 수와 cursor에도 노출되지 않도록 query 경계를 고정한다.

## 범위

- 포함: Reference create/delete/backlink, target snapshot, Draft operation 연결, Vocabulary CRUD·deprecate,
  term normalization·unique, concept history, idempotency·outbox
- 제외: Search projection(IMP-18), AI Context(IMP-20), hard delete·Audit(IMP-16), UI(IMP-24)

## 필수 설계 문서

- [x] 제품·도메인: `product/features/KNOWLEDGE.md`, `domain/knowledge.md`
- [x] 기능·상태: `design/specs/knowledge/REFERENCE-VOCABULARY.md`,
  `design/specs/STATE-TRANSITION-CATALOG.md`, `design/specs/ALGORITHM-CATALOG.md`
- [x] 데이터·API: `design/data/schema.sql`, `design/api/openapi.yaml`,
  `design/api/ERROR-CATALOG.md`, `design/contracts/document-operation.schema.json`
- [x] 보안·품질: `design/security/AUTHORIZATION.md`, `design/specs/AUTHORIZATION-MATRIX.md`,
  `design/quality/TEST-STRATEGY.md`, `design/quality/CONCURRENCY-RECOVERY-TESTS.md`
- [x] 구현 기준: `design/implementation/REFERENCE-VOCABULARY.md`

## 문서 준비 게이트

- [x] source·target·snapshot·Backlink permission 불변식이 정의되어 있다.
- [x] Vocabulary normalization·unique·state·history 계약이 정의되어 있다.
- [x] Draft revision·lease·idempotency·outbox transaction 경계가 정의되어 있다.
- [x] 내부 target 종류와 External URL 검증이 구체화되어 있다.
- [x] 구현 결과를 바꿀 미해결 질문이 없다.

## 사용자 결정

사용자는 구조적으로 권장하는 방향을 별도 승인 없이 적용하도록 확정했다.

## 의사결정

### 결정 1: Reference와 Draft operation은 같은 command다

- 별도 Reference row와 editor content가 어긋나는 이중 쓰기를 금지한다.
- API 입력은 application에서 ADD/REMOVE_REFERENCE operation으로 변환해 기존 reducer와 transaction을 재사용한다.

### 결정 2: 용어 충돌은 DB unique key와 domain normalization을 함께 사용한다

- domain은 사용자에게 안정적인 validation을 제공한다.
- PostgreSQL unique index는 동시 command race의 최종 barrier다.

### 결정 3: Backlink는 permission prefilter다

- source 권한이 없는 row는 가져온 뒤 숨기지 않고 SQL permission scope 안에서만 후보가 된다.

## 구현 순서

1. PLAN-20과 canonical DDL·OpenAPI를 고정한다.
2. Knowledge domain·Application port·PostgreSQL transaction을 구현한다.
3. Draft operation·lease·permission-safe Backlink를 연결한다.
4. HTTP route·generated contract·통합 테스트를 연결한다.
5. 전체 gate 후 완료 기록·commit·push하고 IMP-15로 진행한다.

## 작업 내역

- 2026-08-25: TASK-021을 등록하고 PLAN-20으로 Reference·Vocabulary 구현 경계를 고정했다.
- 2026-08-25: Reference target validation·snapshot·Draft operation 원자 transaction을 구현했다.
- 2026-08-25: permission-prefilter Backlink와 idempotent soft delete를 연결했다.
- 2026-08-25: Vocabulary Unicode normalization·Workspace unique·immutable revision을 구현했다.
- 2026-08-25: OpenAPI·generated contract·forward migration·HTTP route를 연결했다.
- 2026-08-25: External URL canonicalization·폐기 사유 이력·replacement ordered lock을 보강했다.
- 2026-08-25: root gate와 Docker PostgreSQL·Redis 통합 gate를 통과했다.

## 이슈 및 해결

- Reference 삭제 후 동일 command를 재시도하면 row가 없어 idempotency 확인 전에 실패했다. 삭제 row를
  tombstone으로 보존하고 현재 projection에서 제외해 replay 입력과 물리 삭제 책임을 분리했다.
- Reference operation UUID를 매 호출마다 생성해 동일 idempotency key의 request hash가 변했다. actor·
  Workspace·Document·key에서 결정적 UUID를 파생해 command identity를 안정화했다.
- 0011 migration 봉인 후 soft-delete 수명주기 보강이 필요했다. 봉인 파일은 유지하고 0012 forward
  migration으로 추가해 append-only 계약을 지켰다.
- 0012 봉인 후 폐기 사유의 영구 보존 누락을 확인했다. 0013 forward migration으로 immutable history
  계약을 보강했다.

## 검증

- [x] Reference source·target permission·snapshot
- [x] Draft lease·revision·operation atomicity
- [x] Backlink permission prefilter·cursor·tenant isolation
- [x] Vocabulary normalization·unique·state·history
- [x] generated contract·root·Compose gate

## 결과

IMP-14를 완료했다. `bun run check`와 `bun run compose:integration`이 모두 통과했다.
