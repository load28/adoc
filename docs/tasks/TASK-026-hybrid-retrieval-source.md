# TASK-026: Hybrid Retrieval·Source 구현

- **상태**: 완료
- **유형**: 구현
- **구현 패키지**: IMP-19
- **시작일**: 2026-08-25
- **완료일**: 2026-08-25
- **커밋**: —

## 목적

권한이 허용된 현재 검색 projection만 대상으로 lexical·vector 후보를 결합하고, 결과가
원문 근거와 안정적으로 연결되는 Hybrid Retrieval·Source 계약을 구현한다. 검색 점수나
후처리 단계가 권한 경계를 우회하지 않으며 중복 Region과 반복 요청이 결정적인 결과를
만들도록 한다.

## 범위

- 포함: query normalization, permission scope compiler, lexical·vector retrieval, RRF·dedupe,
  Source provenance, cursor·limit, 검색 API application service, OpenSearch adapter, 실제 통합 테스트
- 제외: embedding provider와 AI Context(IMP-20), 검색 화면(IMP-24), 운영자 검색 관리 UI(IMP-27)

## 필수 설계 문서

- [x] 제품·도메인: `product/features/KNOWLEDGE.md`, `domain/knowledge.md`
- [x] 검색 projection·retrieval·Source 정본: `design/adr/ADR-003-search-projection.md`,
  `design/data/OPENSEARCH-PROJECTION-SCHEMA.md`,
  `design/specs/knowledge/INDEX-RETRIEVAL-SOURCE.md`,
  `design/implementation/SEARCH-PROJECTION.md`
- [x] API·오류·권한 계약: `design/api/openapi.yaml`,
  `design/api/COMMAND-QUERY-CATALOG.md`, `design/api/ERROR-CATALOG.md`,
  `design/api/EVENT-CATALOG.md`, `design/specs/governance/PERMISSION-RESOLVER.md`,
  `design/security/AUTHORIZATION.md`
- [x] 알고리즘·실패·동시성 계약: `design/specs/ALGORITHM-CATALOG.md`,
  `design/architecture/SCALABILITY-CAPACITY.md`, `design/operations/OBSERVABILITY-SLO.md`
- [x] 테스트·성능·보안 기준: `design/quality/TEST-STRATEGY.md`,
  `design/quality/PERFORMANCE-TESTS.md`, `design/quality/SECURITY-TESTS.md`
- [x] TASK-026 구현 기준 문서: `design/implementation/HYBRID-RETRIEVAL-SOURCE.md`

## 문서 준비 게이트

- [x] 구현 결과를 바꿀 미해결 질문이 없다.
- [x] 정상·예외·권한·동시성 흐름이 정의되어 있다.
- [x] PostgreSQL permission resolver·OpenSearch·HTTP 경계가 타입 수준으로 정의되어 있다.
- [x] 구현 단위와 relevance·non-leak 완료 조건을 문서에서 추적할 수 있다.
- [x] PLAN-25와 정본 감사를 근거로 코드 작성 가능을 확인했다.

## 사용자 결정

사용자는 기존 설계와 구조적 권장안을 별도 승인 없이 적용하고, AGENTS.md 원칙에 따라
이전과 같은 방식으로 구현을 계속하도록 확정했다.

## 의사결정

### 결정 1: composite permission key를 bounded terms batch로 조회한다

- **상황**: 문서별 scope·fingerprint OR clause는 Workspace 크기에 따라 OpenSearch boolean
  clause 한도를 넘지만 사용자는 Document 수 제한을 두지 않기로 했다.
- **검토한 대안**: clause 한도 상향 / 검색 가능한 Document 제한 / user·group을 index에 복제 /
  composite key의 bounded batch.
- **선택과 근거**: scope와 fingerprint의 hash key를 4,096개 단위로 query하고 modality별
  global top 100을 다시 선정한다. 권한 의미를 바꾸지 않고 규모 제한도 만들지 않는다.

### 결정 2: retrieval port는 optional query vector를 받는다

- **상황**: IMP-19가 hybrid 알고리즘을 소유하지만 embedding provider는 선행 관계상 IMP-20이
  소유한다.
- **검토한 대안**: provider를 앞당겨 결합 / lexical만 구현 / provider-neutral vector input.
- **선택과 근거**: 같은 retrieval port가 vector 유무를 모두 처리한다. HTTP Search는 lexical
  경계를 먼저 사용하고 IMP-20이 동일 port에 vector를 공급해 권한·RRF·Source를 재사용한다.

### 결정 3: cursor를 query·scope·index·ranking version에 결합한다

- **상황**: 결과를 offset만으로 넘기면 권한 변경·alias cutover·projection 변경 뒤 다음 page가
  다른 권한 snapshot을 섞을 수 있다.
- **검토한 대안**: 무상태 offset / 서버측 result cache / binding cursor와 재계산.
- **선택과 근거**: bounded top 30을 결정적으로 재계산하고 모든 의미 fingerprint가 같을 때만
  offset을 허용한다. 변경되면 409로 새 검색을 요구한다.

### 결정 4: stale permission projection은 별도 내부 복구 event로 수선한다

- **상황**: exact key filter는 stale hit를 안전하게 제외하지만 projection drift를 자동으로
  고치지는 않는다.
- **검토한 대안**: 사용자 응답 후 필터 / 기존 DocumentChanged 위장 / 별도 bounded drift probe와
  repair event.
- **선택과 근거**: ranking과 분리된 probe가 mismatch Document를 찾고
  `SearchProjectionRepairScheduled.v1`을 멱등 생성한다. 안전한 결과와 자동 복구를 분리한다.

## 구현 순서

1. 제품·Knowledge·권한·검색·API·품질 정본을 감사한다.
2. TASK-026에 필요한 상세 구현 문서를 코드보다 먼저 작성한다.
3. query·permission·retrieval·RRF·Source 경계를 구현한다.
4. relevance·non-leak·ordering·failure 통합 테스트를 구현한다.
5. 전체 gate를 통과하고 태스크를 완료한다.

## 작업 내역

- 2026-08-25: TASK-026을 등록하고 IMP-19 정본 감사를 시작했다.
- 2026-08-25: 제품·도메인·권한·projection·API·알고리즘·품질 정본을 교차 감사했다.
- 2026-08-25: PLAN-25에 scope compiler, bounded candidate query, vector 경계, RRF weight,
  cursor·Source·drift repair·실패·테스트 계약을 확정했다.
- 2026-08-25: DATA-09·SPEC-12·ALG-007·API 계약을 보강하고 문서 준비 게이트를 통과했다.
- 2026-08-25: Knowledge 계층에 query·vector 검증, permission key, RRF·dedupe·Source
  snapshot을 구현했다.
- 2026-08-25: PostgreSQL scope compiler·drift repair와 OpenSearch bounded `_msearch`
  adapter를 구현하고 검색 API에 연결했다.
- 2026-08-25: generated contract를 갱신하고 lexical·vector 결합, cursor, 권한 변경
  non-leak, 멱등 repair와 재투영을 실제 PostgreSQL·OpenSearch 통합 테스트로 검증했다.

## 이슈 및 해결

- **증상**: 첫 Compose 통합 검증에서 변경 전후 region 기대 건수가 실제 투영 건수와
  일치하지 않았다.
- **조사**: projection mutation과 seed content를 대조해 변경 전에는 paragraph 한 건,
  변경 후에는 pagination 검증용 paragraph 두 건이 생성됨을 확인했다.
- **근본 원인**: 테스트 데이터를 두 region으로 확장하면서 변경 전후 기대값을 함께
  갱신하지 않았다.
- **구조적 해결**: region 단위 투영 계약에 맞춰 변경 전 1건·변경 후 2건을 각각 검증하고,
  전체 Compose 계약을 재실행했다.

## 검증

- [x] 문서 링크와 정본 경계 확인
- [x] PRD·도메인·상세 설계 간 모순 확인
- [x] relevance·permission non-leak·Source 통합 검증
- [x] generated contract·root·Compose gate

## 결과

권한 resolver가 만든 exact scope snapshot 안에서만 BM25·kNN 후보를 조회하고 고정 RRF로
결합하는 Hybrid Retrieval을 구현했다. 결과는 version 또는 draft revision에 결합된 Source를
반환하며 cursor가 query·권한·index generation·ranking version의 혼합을 차단한다. stale
permission projection은 응답에 노출하지 않고 멱등 repair event와 기존 projection job으로
자동 복구한다. `bun run check`와 `bun run compose:integration`을 모두 통과했다.
