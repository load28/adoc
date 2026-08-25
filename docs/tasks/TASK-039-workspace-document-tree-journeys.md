# TASK-039: Workspace·Document Tree 사용자 여정 완성

- **상태**: 완료
- **유형**: 구현
- **시작일**: 2026-08-25
- **완료일**: 2026-08-25
- **커밋**: —

## 목적

로그인 뒤 Workspace 생성·초대 수락·전환과 Document Tree 생성·탐색·변경이 Web에서 끝까지
동작하도록 SCR-01~04와 RQ-01·02·04를 완성한다.

## 범위

- 포함: invitation preview/accept, Workspace create/switch/home, permission-scoped tree, Document
  create/rename/move/sort/trash 진입, loading·empty·denied·conflict·responsive 상태
- 제외: 본문 편집·Publish와 Settings의 상세 Governance mutation

## 필수 설계 문서

- PROD-05·10·11, DOM-01·02, UX-01~04·10·12~15, SPEC-01·02·04·18·19
- DATA-07·08, API-01·02·06~08, SEC-02·03, TEST-01·03·04·07·08
- PLAN-35 및 이 태스크에서 작성할 구현 계약

## 문서 준비 게이트

- [x] route loader·action·cache·permission·revision 계약을 상세 설계에 고정
- [x] invitation·tree의 정상·empty·denied·stale·복구 흐름 정의
- [x] 필요한 API client method와 exact test ID 정의
- [x] 구현 가능 여부와 문서 근거 기록

## 사용자 결정

없음. 기존 동결 설계와 Atlaskit 공개 component 정책을 따른다.

## 의사결정

- 초대 토큰을 브라우저 전역 상태에 보관하는 안과 인증 뒤 같은 capability URL로 복귀하는 안을
  검토했다. 후자를 선택해 token 노출 범위와 별도 저장 상태를 줄였다.
- Tree를 낙관적으로 변경하는 안과 서버 응답 뒤 canonical query를 무효화하는 안을 검토했다.
  revision·권한 판정의 단일 진실 소스를 유지하기 위해 후자를 선택했다.
- 이동 목적지를 모두 노출하고 서버 오류로 거부하는 안과 유효한 부모 권한을 가진 목적지만 노출하는
  안을 검토했다. 권한 계약을 UI 행동 가능성에도 동일하게 반영하도록 후자를 선택했다.

## 작업 내역

- 2026-08-25: TASK-038 후속 DAG의 첫 구현 태스크로 시작했다.
- 2026-08-25: `PLAN-36`에 route·API client·tree mutation·권한·오류·복구·test 계약을 고정했다.
- 2026-08-25: UX의 invitation preview 누락을 API-02·06·08과 SPEC-01에 먼저 보강했다.
- 2026-08-25: invitation preview의 token·email·expiry 검증과 Tree `effectiveAccess`를 Rust service·
  PostgreSQL adapter·Axum·OpenAPI에 구현했다.
- 2026-08-25: Workspace 생성, 초대 수락, permission-scoped Tree의 생성·rename·move preview/commit·
  trash Web 흐름과 ko/en 상태를 Atlaskit으로 구현했다.
- 2026-08-25: 고정 operation 개수로 새 계약을 거부하던 검사를 OpenAPI operation ID 집합 기반으로
  교체했다.
- 2026-08-25: 이동 목적지의 부모 접근 수준을 검사해 서버에서 거부될 수밖에 없는 순서 변경 제어를
  숨겼다.

## 이슈 및 해결

- `/login`에 query가 없을 때 search validator가 기본 `returnTo`를 URL에 강제해 SSR smoke가 307을
  반환했다. 명시된 query만 정규화하고 component에서 안전한 기본값을 적용해 canonical URL을 보존했다.

## 검증

- [x] SCR-01~04 component·route integration test
- [x] cross-tenant·permission·stale revision negative test
- [x] ko/en·compact·keyboard·recovery 상태
- [x] root gate와 Compose integration

- `bun run check`: 계약 109개, format, lint, typecheck, unit, build, 보안·라이선스 게이트 통과
- `bun run compose:integration`: PostgreSQL·Redis·OpenSearch 통합, backup restore, SSR smoke 통과
- 성능 smoke p95: API ready 3.852ms, Web live 0.673ms, SSR login 5.7ms

## 결과

Workspace 생성·전환·초대 수락과 권한 범위 Document Tree의 생성·이름 변경·이동·휴지통 흐름을
Web부터 Rust·PostgreSQL까지 연결했다. 모든 변경은 서버 revision과 권한 판정 뒤 canonical Tree를
다시 조회한다.
