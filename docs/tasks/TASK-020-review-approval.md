# TASK-020: Review·Approval 구현

- **상태**: 완료
- **유형**: 구현
- **구현 패키지**: IMP-13
- **시작일**: 2026-08-25
- **완료일**: 2026-08-25
- **커밋**: 본 태스크 완료 커밋에 포함

## 목적

정확한 Draft revision과 PublishPolicy snapshot에 묶인 다중 reviewer 승인 흐름을 구현한다. Draft 변경,
정책 변화와 권한 상실이 과거 승인을 우회해 Publish에 사용되지 않도록 하나의 transaction 계약으로
연결한다.

## 범위

- 포함: Review request/get/decision/cancel, assignment와 immutable decision history, threshold reducer,
  Inbox projection, Draft invalidation, REVIEW_REQUIRED Publish gate, idempotency·outbox
- 제외: Reference·Vocabulary(IMP-14), File readiness(IMP-15), Audit projection(IMP-16), SSE consumer(IMP-17), UI(IMP-24)

## 필수 설계 문서

- [x] 제품·도메인: `product/features/COLLABORATION.md`, `domain/collaboration.md`,
  `domain/workspace-governance.md`
- [x] 상태·UX: `design/specs/collaboration/REVIEW-INBOX.md`,
  `design/specs/STATE-TRANSITION-CATALOG.md`, `design/specs/ALGORITHM-CATALOG.md`,
  `design/ux/DRAFT-PUBLISH-FLOWS.md`, `design/ux/COLLABORATION-FLOWS.md`
- [x] 데이터·API: `design/data/schema.sql`, `design/api/openapi.yaml`,
  `design/api/ERROR-CATALOG.md`, `design/contracts/event-payloads.schema.json`
- [x] 보안·품질: `design/security/AUTHORIZATION.md`, `design/specs/AUTHORIZATION-MATRIX.md`,
  `design/quality/TEST-STRATEGY.md`, `design/quality/CONCURRENCY-RECOVERY-TESTS.md`
- [x] 구현 기준: `design/implementation/REVIEW-APPROVAL.md`

## 문서 준비 게이트

- [x] snapshot·assignment·상태·threshold 불변식이 정의되어 있다.
- [x] request·decision·cancel·Draft invalidation·Publish 연결이 정의되어 있다.
- [x] 권한 상실·정책 변화·동시 결정·stale revision 실패 계약이 정의되어 있다.
- [x] Inbox·outbox·idempotency·decision history 저장 경계가 정의되어 있다.
- [x] 구현 결과를 바꿀 미해결 질문이 없다.

## 사용자 결정

사용자는 구조적으로 권장하는 방향을 별도 승인 없이 적용하도록 확정했다.

## 의사결정

### 결정 1: 요청자는 reviewer snapshot에서 제외한다

- 형식적인 자기 승인으로 Review 의미가 사라지는 것을 막는다.
- 요청 시 남은 eligible reviewer가 threshold보다 적으면 원자적으로 거부한다.

### 결정 2: 정책 변화는 snapshot을 덮어쓰지 않는다

- 기존 Review는 `policyOutdated`로 표시하고 Publish에 재사용하지 않는다.
- 새 요청 시 outdated active Review를 INVALIDATED로 종료하고 새 snapshot을 만든다.

### 결정 3: current assignment와 immutable decision history를 함께 둔다

- threshold 계산은 잠근 current projection으로 수행한다.
- 감사 가능한 결정 변경 이력은 append-only table에 보존한다.

## 구현 순서

1. PLAN-19와 canonical DDL·OpenAPI를 고정한다.
2. Review domain·Application port·PostgreSQL transaction을 구현한다.
3. Draft invalidation과 Publish review gate를 연결한다.
4. HTTP route·generated contract·통합 테스트를 연결한다.
5. 전체 gate 후 완료 기록·commit·push하고 IMP-14로 진행한다.

## 작업 내역

- 2026-08-25: TASK-020을 등록하고 PLAN-19로 Review snapshot·threshold·Publish 경계를 고정했다.
- 2026-08-25: Review domain reducer·application service·PostgreSQL transaction·HTTP route를 구현했다.
- 2026-08-25: assignment current projection과 append-only decision revision, Inbox·outbox를 연결했다.
- 2026-08-25: Draft mutation invalidation과 REVIEW_REQUIRED Publish snapshot gate를 연결했다.
- 2026-08-25: canonical OpenAPI·DDL·generated contract와 Docker 통합 테스트를 갱신했다.

## 이슈 및 해결

- Draft의 `ON DELETE CASCADE`가 Publish 시 Review와 immutable decision history 삭제를 시도해
  transaction이 실패했다. Review의 `draft_id`를 immutable snapshot identity로 분리하고 active unique
  index를 REQUESTED 상태에 한정하는 forward-only 0010 migration으로 해결했다.
- 0009 migration 봉인 뒤 수명주기 수정 필요성을 확인했다. 봉인된 파일은 유지하고 교정을 0010으로
  추가해 migration append-only 계약을 지켰다.

## 검증

- [x] Review snapshot·self-review exclusion·threshold
- [x] decision history·stale revision·serialized decision
- [x] Draft mutation invalidation·policy outdated·permission loss
- [x] Inbox dedupe·resolve·tenant isolation
- [x] REVIEW_REQUIRED Publish·version snapshot
- [x] generated contract·root·Compose gate

## 결과

IMP-13을 완료했다. `bun run check`와 `bun run compose:integration`이 모두 통과했다.
