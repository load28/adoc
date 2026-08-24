# TASK-015: Permission·PublishPolicy 구현

- **상태**: 완료
- **유형**: 구현
- **구현 패키지**: IMP-08
- **시작일**: 2026-08-25
- **완료일**: 2026-08-25
- **커밋**: 이 태스크 완료 커밋

## 목적

Document tree의 User·Group Grant를 하나의 precedence 정책으로 해석하는 point/scope resolver와
상속 가능한 PublishPolicy를 구현한다. 이후 모든 Document·Search·AI use case가 사후 필터 없이
같은 authorization 결과를 소비할 수 있는 경계를 고정한다.

## 범위

- 포함: permission·policy domain 값과 merge, point resolver, scope compiler, explanation,
  PermissionGrant·PublishPolicy CRUD, PostgreSQL transaction·idempotency·outbox, Redis read-through
  cache와 revision fingerprint, Axum API, property·contract·통합 테스트
- 제외: Document 생성·이동 API(IMP-10), public link(IMP-11), outbox worker와 SSE(IMP-17),
  OpenSearch projection(IMP-18), TanStack 화면(IMP-22·26)

## 필수 설계 문서

- [x] `product/PRD.md`, `product/features/WORKSPACE-AND-GOVERNANCE.md`,
  `product/IMPLEMENTATION-SCOPE.md`
- [x] `domain/workspace-governance.md`, `domain/document-system.md`
- [x] UX: `design/ux/WORKSPACE-PERMISSION-FLOWS.md`, `design/ux/SCREEN-BEHAVIOR-SPECS.md`
- [x] 데이터: `design/data/schema.sql`, `design/data/DATA-DICTIONARY.md`,
  `design/data/DATABASE-INVARIANT-TRANSACTION-MATRIX.md`
- [x] API: `design/api/openapi.yaml`, `design/api/COMMAND-QUERY-CATALOG.md`,
  `design/api/ERROR-CATALOG.md`, `design/api/EVENT-CATALOG.md`
- [x] 보안: `design/security/AUTHORIZATION.md`, `design/security/THREAT-MODEL.md`
- [x] 테스트: `design/quality/TEST-STRATEGY.md`, `design/quality/FIXTURE-CATALOG.md`,
  `design/quality/CONTRACT-COVERAGE.md`
- [x] 구현 기준: `design/implementation/PERMISSION-PUBLISH-POLICY.md`,
  `design/implementation/MODULE-INTERFACE-CATALOG.md`

## 문서 준비 게이트

- [x] 구현 결과를 바꿀 미해결 질문이 없다.
- [x] precedence·상속·last manager·subject·tenant·revision race가 정의됐다.
- [x] point/scope 동등성, cache key·실패 정책과 invalidation 계약이 타입 수준으로 정의됐다.
- [x] IMP-08과 IMP-10·11·12·17·18·22·26의 책임 경계를 추적할 수 있다.
- [x] 구현 기준 문서를 근거로 코드를 작성할 수 있다.

## 사용자 결정

사용자는 전체 제품 구현, PostgreSQL·Redis와 권장안의 자율 적용을 확정했다.

## 의사결정

### 결정 1: point와 scope는 하나의 순수 compiler를 사용한다

- **상황**: 소비자별 SQL·merge 구현은 검색·AI에서 권한 결과가 달라지는 보안 결함을 만든다.
- **검토한 대안**: point별 recursive SQL / scope별 별도 ACL / 동일 snapshot compiler.
- **선택과 근거**: PostgreSQL이 snapshot을 공급하고 하나의 domain compiler가 두 결과를 계산한다.
  생성형 동등성 테스트로 모든 Document의 결과가 같음을 검증한다.

### 결정 2: cache 유효성은 PostgreSQL revision stamp가 소유한다

- **상황**: Redis delete message만 의존하면 invalidation 지연 중 이전 allow가 재사용될 수 있다.
- **검토한 대안**: TTL만 사용 / event delete / DB monotonic revision을 cache key에 포함.
- **선택과 근거**: Membership·Group·Grant·tree·policy trigger가 Workspace revision을 올린다. Redis
  장애·stale·decode 실패는 정본 PostgreSQL로 fallback하며 sensitive command는 cache를 쓰지 않는다.

### 결정 3: last manager는 영향 subtree 전체에서 보장한다

- **상황**: target Document만 검사하면 상속 grant 삭제로 descendant가 영구 관리 불가능해질 수 있다.
- **검토한 대안**: 현재 target만 확인 / Workspace Owner 우회 / 영향 subtree의 effective manager 확인.
- **선택과 근거**: 변경 snapshot으로 전체 subtree를 재평가하고 manager가 0인 Document가 하나라도
  있으면 transaction을 취소한다. Workspace role의 content 우회도 만들지 않는다.

## 구현 순서

1. IMP-08 상세 구현 계약과 영향받는 정본 domain·API·DDL을 고정한다.
2. permission·policy domain value, point/scope policy compiler와 property test를 구현한다.
3. PostgreSQL repository, transaction·idempotency·outbox와 Redis cache port를 구현한다.
4. Axum query·command route와 generated contract를 연결한다.
5. 실제 PostgreSQL·Redis integration, HTTP security와 전체 root gate를 실행한다.
6. 완료 기록 후 commit·push하고 다음 구현 패키지로 진행한다.

## 작업 내역

- 2026-08-25: IMP-08 태스크를 등록하고 제품·도메인·보안·API·DDL·테스트 정본을 확인했다.
- 2026-08-25: PLAN-14에 단일 precedence compiler, subtree last manager, PublishPolicy 후보,
  PostgreSQL revision stamp와 Redis fail-closed cache 계약을 고정했다.
- 2026-08-25: action capability, local revision, Workspace cache stamp, OpenAPI response와 stable error·event
  정본을 구현 가능한 타입 수준으로 보강했다.
- 2026-08-25: 단일 precedence compiler와 point·scope·explanation, PublishPolicy 상속·후보 검증을
  domain·application service로 구현했다.
- 2026-08-25: PostgreSQL command transaction·revision·last manager·idempotency·outbox와 Redis
  read-through cache·DB fallback을 구현하고 6개 Axum operation을 연결했다.
- 2026-08-25: generated Rust·TypeScript 계약, forward-only migration과 Docker upgrade 경로를 갱신했다.
- 2026-08-25: 전체 root gate와 PostgreSQL 16·Redis·backup·OpenSearch Compose 통합 검증을 통과했다.

## 이슈 및 해결

- 멱등 재시도가 최초 성공 뒤 증가한 local revision에 먼저 막혔다. command receipt를 actor의 sensitive
  권한 확인 직후 조회하고, 신규 command일 때만 revision을 검사하도록 transaction 순서를 수정했다.
- last manager와 reviewer 후보 검증이 active User마다 tree를 다시 조회했다. active Membership·Document
  tree·User/Group grant를 한 번에 읽는 Workspace snapshot으로 교체하고 같은 순수 compiler를 적용했다.
- Redis 연결 실패가 API 시작 자체를 중단해 cache가 가용성 조건이 됐다. 비가용 cache adapter가 모든
  read·write를 실패로 반환하고 PermissionService가 PostgreSQL 정본으로 fallback하도록 경계를 분리했다.
- Redis의 역직렬화 가능한 비정상 값이 effective 결과로 사용될 수 있었다. fingerprint 형식과 permission
  불변식·evidence 정렬을 검증하고 불일치 값은 miss로 처리하도록 수정했다.

## 검증

- [x] precedence matrix와 생성형 point/scope 동등성
- [x] PostgreSQL subject·tenant·revision·last manager·policy invariant
- [x] Redis cache hit·miss·stale fingerprint·dependency failure
- [x] HTTP CSRF·Origin·IDOR·problem response
- [x] OpenAPI generated Rust·TypeScript 계약
- [x] 실제 PostgreSQL 16·Redis Compose integration과 root gate

## 결과

Document tree의 User·Group grant를 하나의 순수 compiler로 해석하는 point·scope·explanation 경계를
구현했다. subtree last manager, PublishPolicy 상속·후보, local·Workspace revision, Redis 장애 fallback,
tenant 비노출과 command 원자성을 실제 PostgreSQL 16·Redis·Compose 검증으로 고정했다.
