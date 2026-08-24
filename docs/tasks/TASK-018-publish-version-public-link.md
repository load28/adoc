# TASK-018: Publish·Version·Public Link 구현

- **상태**: 완료
- **유형**: 구현
- **구현 패키지**: IMP-11
- **시작일**: 2026-08-25
- **완료일**: 2026-08-25
- **커밋**: —

## 목적

active Draft 하나를 불변 Published Version으로 원자 발행하고 history·diff·과거 snapshot 복원을
제공한다. Manage 사용자가 만든 폐기 가능한 capability로 익명 사용자가 단일 Document의 최신
Published Version만 읽게 한다.

## 범위

- 포함: direct Publish, immutable Version·context, history/detail/diff, stale base conflict, 과거 Version
  기반 Draft restore, public link list/create/revoke, 익명 latest Published viewer, idempotency·outbox·race test
- 제외: Review 승인 판정(IMP-13), File byte delivery와 asset graph(IMP-15), Audit projection(IMP-16),
  Search projection·SSE(IMP-17·18), Public Viewer 화면(IMP-26)

## 필수 설계 문서

- [x] 제품·도메인: `product/features/DOCUMENT-LIFECYCLE.md`, `domain/document-system.md`
- [x] 상태·알고리즘: `design/specs/document/PUBLISH-VERSION.md`,
  `design/specs/document/REGION-OPERATION-DIFF.md`, `design/specs/STATE-TRANSITION-CATALOG.md`
- [x] 데이터: `design/data/schema.sql`, `design/data/DATABASE-INVARIANT-TRANSACTION-MATRIX.md`,
  `design/data/LIFECYCLE-RETENTION.md`
- [x] API·이벤트: `design/api/openapi.yaml`, `design/api/ERROR-CATALOG.md`,
  `design/contracts/event-payloads.schema.json`
- [x] 권한·보안: `design/security/AUTHORIZATION.md`, `design/security/THREAT-MODEL.md`,
  `design/implementation/PERMISSION-PUBLISH-POLICY.md`
- [x] 품질: `design/quality/TEST-STRATEGY.md`, `design/quality/SECURITY-TESTS.md`
- [x] 구현 기준: `design/implementation/PUBLISH-VERSION-PUBLIC-LINK.md`

## 문서 준비 게이트

- [x] 구현 결과를 바꿀 미해결 질문이 없다.
- [x] Publish·history·restore·public capability의 정상·실패·권한·동시성 흐름이 정의되어 있다.
- [x] API·DB·event·idempotency 계약과 lock 순서가 타입 수준으로 정의되어 있다.
- [x] IMP-13·15·16·17·18·26 책임 경계가 추적된다.
- [x] immutable·base conflict·publish/link race 완료 조건이 정의되어 있다.

## 사용자 결정

사용자는 전체 제품 구현과 권장안의 자율 적용을 확정했다.

## 의사결정

### 결정 1: Publish는 active Lease와 충돌하지 않는다

- active Lease가 없거나 만료·반환됐으면 EDITOR가 lease token 없이 발행할 수 있다.
- active Lease가 있으면 holder user·client instance·token이 모두 일치해야 한다.
- 이 규칙으로 저장 중인 다른 client의 Draft snapshot을 중간 상태로 발행하지 않는다.

### 결정 2: Review-required 정책은 IMP-13까지 fail closed다

- DIRECT 정책만 이 태스크에서 발행한다.
- REVIEW_REQUIRED는 승인 판정을 추정하지 않고 `REVIEW_REQUIRED`로 거부한다.

### 결정 3: Public link는 최신 Version을 동적으로 가리킨다

- token 원문은 최초 응답 한 번만 노출하고 SHA-256 hash만 저장한다.
- 조회마다 link·expiry·Document active·current Version을 한 query 경계에서 재검증한다.
- 일반 session API와 Workspace metadata는 capability로 접근할 수 없다.

### 결정 4: 과거 복원은 새 Draft만 만든다

- Version row와 context는 수정하지 않는다.
- active Draft가 있으면 복원을 거부하고, 없을 때 선택 Version을 base로 revision 0 Draft를 만든다.

## 구현 순서

1. Publish·Version·Public API와 DDL 계약 공백을 감사하고 PLAN-17을 고정한다.
2. domain model·application port·PostgreSQL transaction을 구현한다.
3. authenticated와 anonymous HTTP 경계를 분리해 연결한다.
4. immutable·base conflict·restore·token race 통합 테스트를 추가한다.
5. root·Compose gate 후 완료 기록, commit·push하고 IMP-12로 진행한다.

## 작업 내역

- 2026-08-25: IMP-11 태스크를 등록하고 PLAN-17로 Publish·Version·Public capability 경계를 고정했다.
- 2026-08-25: OpenAPI·event schema·canonical DDL과 migration 0007에 snapshot fingerprint,
  source revision, base Version, 공개 capability 제약을 반영하고 생성 계약을 갱신했다.
- 2026-08-25: Version domain model과 deterministic `DocumentOperation[]` diff, Application port,
  PostgreSQL transaction, 인증 API와 별도 익명 route를 구현했다.
- 2026-08-25: 발행이 Version·context 생성, current pointer 전환, Draft 종료, Lease 반환,
  outbox·idempotency를 하나의 transaction으로 보장하게 했다.
- 2026-08-25: immutable trigger, stale base metadata, one-time public token, revoke·expiry·trash·latest,
  권한 격리와 동시 Publish 단일 승자 barrier test를 추가했다.
- 2026-08-25: 전체 root gate와 격리 PostgreSQL·Redis·Docker Compose 통합 게이트를 통과했다.

## 이슈 및 해결

- 배포 전에는 PublishedVersion 쓰기 경로가 없으므로 migration 0007은 기존 Version row가 있으면
  임의 fingerprint를 만들지 않고 명시적으로 중단한다. 이 구조로 검증되지 않은 snapshot identity의
  조용한 이식을 막았다.

## 검증

- [x] immutable Version·단조 번호·base conflict
- [x] Publish transaction·Draft 종료·Lease race
- [x] history·diff·restore와 권한 negative corpus
- [x] public token create/revoke/expiry/latest/tenant scope race
- [x] generated contract와 전체 root·Compose gate

## 결과

Direct Publish, immutable history/detail/diff, 과거 snapshot 기반 Draft restore와 최신 Published를
동적으로 읽는 폐기 가능한 public capability를 구현했다. Version identity와 public secret 경계를
DDL·domain·application·HTTP에 고정했고 전체 root·Compose gate가 통과했다.
