# TASK-025: Search Projection 구현

- **상태**: 완료
- **유형**: 구현
- **구현 패키지**: IMP-18
- **시작일**: 2026-08-25
- **완료일**: 2026-08-25
- **커밋**: —

## 목적

PostgreSQL의 공식 문서·지식·권한 상태를 OpenSearch의 버전 관리된 검색
projection으로 멱등적으로 반영한다. 이벤트 중복·역순·유실, OpenSearch 일시 장애,
권한 변경과 전체 재구축 중에도 낡은 내용이나 권한을 노출하지 않는 projection 계약을
구현한다.

## 범위

- 포함: OpenSearch index template·mapping·alias, projection model, Job consumer, 멱등 upsert·delete,
  권한 사전 필터 필드, generation rebuild·atomic cutover, drift canary, DDL·구성·통합 테스트
- 제외: Hybrid query·RRF·Source 응답(IMP-19), 검색 UI(IMP-24), embedding provider·AI Context
  (IMP-20), 운영자 재구축 UI(IMP-27)

## 필수 설계 문서

- [x] 제품·도메인: `product/features/KNOWLEDGE.md`, `domain/knowledge.md`
- [x] 검색 아키텍처: `design/adr/ADR-003-search-projection.md`,
  `design/specs/knowledge/INDEX-RETRIEVAL-SOURCE.md`
- [x] 데이터·알고리즘: `design/data/OPENSEARCH-PROJECTION-SCHEMA.md`,
  `design/specs/ALGORITHM-CATALOG.md`, `design/data/LIFECYCLE-RETENTION.md`
- [x] 권한·보안: `design/specs/governance/PERMISSION-RESOLVER.md`,
  `design/security/AUTHORIZATION.md`
- [x] 실행·복구: `design/architecture/TRANSACTION-EVENT-JOB.md`,
  `design/architecture/SCALABILITY-CAPACITY.md`, `design/operations/OBSERVABILITY-SLO.md`
- [x] 테스트: `design/quality/TEST-STRATEGY.md`,
  `design/quality/CONCURRENCY-RECOVERY-TESTS.md`, `design/quality/PERFORMANCE-TESTS.md`
- [x] 구현 기준: `design/implementation/SEARCH-PROJECTION.md`

## 문서 준비 게이트

- [x] 구현 결과를 바꿀 미해결 질문이 없다.
- [x] 정상·예외·권한·동시성 흐름이 정의되어 있다.
- [x] PostgreSQL·Job·OpenSearch·alias 경계가 타입·버전 수준으로 정의되어 있다.
- [x] 구현 단위와 prefilter·ordering·rebuild 완료 조건을 문서에서 추적할 수 있다.
- [x] PLAN-24와 정본 감사를 근거로 코드 작성 가능을 확인했다.

## 사용자 결정

사용자는 기존 설계와 구조적 권장안을 별도 승인 없이 적용하고, AGENTS.md 원칙에
따라 다음 태스크를 이전과 같은 방식으로 계속 진행하도록 확정했다.

## 의사결정

### 결정 1: 문서 기준 scope token을 사용한다

- **상황**: user·group ID를 index에 펼쳐 저장하면 Membership 변경이 전체 문서 fan-out을
  만들고 권한 정보를 과도하게 복제한다.
- **검토한 대안**: user·group token 복제 / 사용자별 index / 문서 scope token·ancestry fingerprint.
- **선택과 근거**: 문서 token과 조상 permission revision fingerprint를 쓴다. query scope
  compiler가 현재 resolver 결과를 `{scope,fingerprint}`로 compile하여 prefilter 동등성을
  golden test로 검증할 수 있다.

### 결정 2: Workspace 단위 projection sequence를 도입한다

- **상황**: 하나의 index Region은 Document·Draft·Version·Permission event의 영향을 받으므로
  aggregate sequence를 서로 비교할 수 없다.
- **검토한 대안**: 시간 / aggregate별 watermark / Workspace 단조 sequence.
- **선택과 근거**: producer transaction이 Workspace별 projection sequence를 할당한다. 동일
  Region의 역순·중복 쓰기를 OpenSearch external version으로 결정적으로 차단한다.

### 결정 3: event snapshot 대신 current-state materialization을 사용한다

- **상황**: 지연된 upsert event가 삭제·권한 변경 뒤 도착할 수 있다.
- **검토한 대안**: event에 content snapshot 복제 / 변경 diff 재생 / PostgreSQL 현재 상태
  materialization.
- **선택과 근거**: event는 target과 ordering만 제공하고 consumer가 현재 상태를 읽는다.
  늦은 event도 권한이 회수된 이전 내용을 재생하지 못한다.

### 결정 4: Job dispatcher를 provider-중립 경계로 분리한다

- **상황**: JobRuntime이 OpenSearch 세부 구현을 알면 후속 AI·mail handler마다 runtime
  분기가 늘어난다.
- **검토한 대안**: repository에 handler 추가 / runtime의 kind match 확장 / `JobExecutor`
  dispatcher port.
- **선택과 근거**: runtime은 claim·transition만 소유하고 dispatcher가 kind를 폐쇄형 handler에
  라우팅한다. 후속 provider가 runtime 상태 머신을 변경하지 않고 추가될 수 있다.

### 결정 5: 재구축 세대에 실시간 변경을 동시 투영한다

- **상황**: snapshot watermark 이후 발생한 변경이 alias cutover 전에 새 세대에 반영되지
  않으면 전환 직후 검색 결과가 일시적으로 과거 상태가 된다.
- **검토한 대안**: cutover 전 outbox 재생 / 재구축 중 쓰기 중단 / rebuild alias dual-write.
- **선택과 근거**: 새 세대를 준비할 때 종류별 rebuild alias를 연결하고 일반 Job이 active와
  rebuild 세대에 같은 sequence mutation을 적용한다. CAS ordering이 snapshot과 실시간
  변경의 순서를 흡수하고, 검증 후 read·write alias를 원자 전환한다.

## 구현 순서

1. 검색 projection·권한·Job·rebuild 정본을 감사한다.
2. PLAN-24에 mapping, event sequencing, failure recovery, rebuild·cutover 계약을 확정한다.
3. OpenSearch adapter와 projection Job consumer를 구현한다.
4. generation rebuild와 drift canary를 구현한다.
5. 실제 PostgreSQL·Redis·OpenSearch 통합 테스트와 전체 gate를 수행한다.

## 작업 내역

- 2026-08-25: TASK-025를 등록하고 IMP-18 정본 감사를 시작했다.
- 2026-08-25: DATA-09·SPEC-12를 감사하고 PLAN-24에 projection unit, permission
  prefilter, Workspace ordering, Job dispatch, rebuild·failure·test 계약을 확정했다.
- 2026-08-25: 문서 준비 게이트를 통과하고 코드 작성 가능을 확인했다.
- 2026-08-25: 역순 Region replacement에서 삭제된 ID를 낮은 sequence가 재생성하는
  ABA race를 발견했다. Region을 즉시 삭제하지 않고 sequence를 보존한 tombstone과
  scripted compare-and-set을 쓰도록 PLAN-24를 코드 전에 보강했다.
- 2026-08-25: Workspace projection sequence와 search Job producer, current-state
  materializer, strict OpenSearch mapping·alias·tombstone adapter를 구현했다.
- 2026-08-25: generation ledger, rebuild dual-write, 검증·원자 cutover·abort cleanup과
  실제 PostgreSQL·Redis·OpenSearch 통합 계약을 구현했다.
- 2026-08-25: `bun run check`와 `bun run compose:integration`을 통과했다.

## 이슈 및 해결

- 최신 replacement가 제거한 Region ID는 OpenSearch external version 정보도 함께 사라져,
  느린 과거 Job이 같은 ID를 재생성할 수 있었다. 삭제 대신 sequence tombstone을 보존하고
  scripted upsert가 row의 logical sequence를 비교하도록 구조적으로 해결했다.
- OpenSearch와 Rust 통합 test binary를 함께 병렬 link하며 Compose 메모리 한도를
  초과했다. test-runner의 Cargo build concurrency를 1로 고정해 환경 용량에 무관하게
  계약 test binary를 순차 link하도록 했다.
- OpenSearch 3.3.2 공식 image에 `analysis-nori` plugin이 없어 Korean analyzer mapping이
  거부됐다. standard analyzer로 폴백하지 않고, 버전 고정 파생 image에 공식
  `analysis-nori` plugin을 설치해 정본의 검색 품질 계약을 유지했다.
- rebuild alias 존재 확인 직후 cutover가 일어나면 이어지는 mirror 요청이 404를 받을 수
  있었다. alias 404를 영구 요청 오류가 아닌 재시도 가능한 세대 전환으로 분류해 Job이
  현재 write alias에 current state를 다시 적용하도록 했다.

## 검증

- [x] 문서 링크와 정본 경계 확인
- [x] PRD·도메인·상세 설계 간 모순 확인
- [x] OpenSearch mapping·prefilter·ordering·rebuild 통합 검증
- [x] generated contract·migration·root·Compose gate

## 결과

PostgreSQL 정본의 문서·초안·권한·지식 변경을 Workspace 단조 sequence로 OpenSearch에
투영한다. scripted CAS tombstone이 중복·역순·ABA를 차단하고, versioned alias와 rebuild
dual-write가 무중단 재구축을 보장한다. 실제 Compose 환경에서 권한 prefilter, ordering,
tombstone과 저장·복구 계약을 검증했다.
