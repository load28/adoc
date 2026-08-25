# TASK-044: Browser E2E·접근성·시각·호환성 검증

- **상태**: 완료
- **유형**: 품질
- **시작일**: 2026-08-25
- **완료일**: 2026-08-25
- **커밋**: —

## 목적

TEST-09 전체 사용자 여정을 실제 Chromium·Firefox·WebKit에서 실행하고 WCAG·responsive·visual
계약을 검증해 jsdom 수준의 간접 증거를 대체한다.

## 범위

- 포함: Playwright 기반 SSR/CSR E2E, multi-session, failure manifestation, axe, keyboard-only, focus,
  compact/wide visual snapshot, ko/en, Chrome/Edge/Firefox/Safari 대응 browser engine matrix
- 제외: 실제 사람 screen-reader 세션과 production credential은 environment evidence로 분리

## 필수 설계 문서

- PROD-04~06·09, UX-02·03·10·12·13·15·16, SEC-02~04
- TEST-01~04·06~09, OPS-02, PLAN-03·34·35 및 이 태스크에서 작성할 browser harness 계약

## 문서 준비 게이트

- [x] scenario fixture·session·browser·viewport matrix 정의
- [x] deterministic visual threshold와 font/time/animation 고정 정의
- [x] keyboard·focus·axe 실패 조건과 수동 evidence 경계 정의
- [x] CI project matrix·artifact·failure reproduction 정의

## 사용자 결정

없음.

## 의사결정

- API에 test-only login endpoint를 추가하는 방식, browser request interception, 별도 fixture executable을
  검토했다. production runtime 분기를 만들지 않고 실제 session HMAC·PostgreSQL 계약을 재사용하는 별도
  fixture executable을 선택했다.
- vendor browser channel을 직접 설치하는 방식과 Playwright 고정 engine revision을 검토했다. repository
  gate의 재현성을 위해 Chromium·Firefox·WebKit revision을 고정하고 실제 Chrome·Edge·Safari stable은
  release environment evidence로 분리했다.
- 반응형 CSS를 수치 검사만 하는 방식과 exact visual baseline을 검토했다. 접근성 tree·reflow assertion과
  고정 Linux 환경의 0 pixel visual baseline을 함께 사용한다.

## 작업 내역

- 2026-08-25: TASK-043 완료 뒤 후속 DAG의 여섯 번째 품질 태스크로 시작했다.
- 2026-08-25: PLAN-41에 실제 dependency fixture, 3개 engine, wide·compact, ko·en,
  axe·keyboard·focus·visual, artifact·CI project 계약을 고정했다.
- 2026-08-25: TEST-09의 15개 ID와 제목을 exact browser manifest로 고정하고 누락·추가·제목 변경을
  negative self-test가 거부하게 했다.
- 2026-08-25: production image, PostgreSQL, Redis, local ObjectStorage와 실제 session HMAC을 쓰는 전용
  fixture executable을 구성했다. API에는 test-only endpoint나 runtime 분기를 추가하지 않았다.
- 2026-08-25: Chromium·Firefox·WebKit의 wide acceptance 45건과 compact 접근성·반응형·시각 품질 9건을
  고정 Playwright Linux image에서 실행했다. 12개 exact visual baseline을 저장했다.
- 2026-08-25: CI에 Compose browser gate와 실패 screenshot·trace·seed artifact 보존을 연결했다.
- 2026-08-25: 기준선 갱신과 독립 재실행에서 각각 browser test 54/54가 통과했다. 전체 root gate와
  PostgreSQL·Redis·OpenSearch·ObjectStorage·backup·restore Compose integration도 통과했다.

## 이슈 및 해결

- API Document detail이 OpenAPI의 `publishedVersion`을 반환하지 않았다. application DTO와 같은 transaction의
  adapter 조회를 연결해 계약 자체를 충족했다.
- Vite SSR artifact를 Bun이 다시 bundle하면서 Atlaskit과 app에 서로 다른 React instance가 생겼다. Vite가
  dependency까지 self-contained SSR bundle을 만들고 Bun은 runtime entry만 compile하도록 경계를 분리했다.
- Ajv runtime code generation이 production CSP에 차단됐다. 계약 생성 단계에서 standalone validator를
  미리 만들고 runtime은 생성된 함수만 호출하게 했다.
- Web runtime이 `/api/v1`만 proxy해 anonymous `/public/v1` 요청을 app 404로 처리했다. request routing을
  독립 계약으로 추출하고 authenticated·public API namespace를 같은 upstream 경계로 연결했다.
- UI mutation의 `If-Match`가 OpenAPI와 달리 unquoted revision이었다. API client 한 곳에서 quoted entity tag를
  생성하게 해 모든 mutation에 같은 optimistic concurrency 계약을 적용했다.
- `application/problem+json`을 JSON으로 읽지 않아 lease 충돌을 dependency failure로 오인했다. JSON 계열
  media type 판정을 `application/json`과 `+json`에 공통 적용했다.
- Atlaskit router adapter가 query 포함 href 전체를 TanStack path로 전달해 토론 ID를 오염시켰다. app link는
  path·search·hash로 구조화하고 API·public·external link는 native anchor가 소유하게 했다.
- browser manifest checker가 구현하지 않기로 확정한 offline marker를 요구했다. multi-context와 public
  boundary처럼 browser suite가 실제 소유하는 증거 marker로 검사 계약을 정정했다.
- 실제 session·Document content 계약을 준비하는 fixture executable을 일반 tool 계층으로 분류해 application
  의존 경계를 위반했다. production 계약을 재사용하는 실행형 test support로 분류해 허용된 검증 계층을
  명시했다.
- browser test의 cookie 존재 검증 뒤 non-null assertion과 switch scope가 lint 계약을 위반했다. 검증 뒤
  명시적 실패 분기를 두고 scenario 지역 변수를 block scope로 제한했다.
- Compose OpenSearch healthcheck가 cluster status를 보지 않고 HTTP 200만 확인했다. single-node에서 유효한
  yellow 이상과 `timed_out=false`를 readiness 조건으로 강화했으나 같은 실패가 반복되어 단독 원인에서는
  제외했다.
- 동일 image의 mapping·Nori·vector·alias·update와 targeted Search Projection은 격리 환경에서 통과했다.
  Compose build 중 Docker data filesystem 사용률이 89%에서 OpenSearch 90% disk watermark를 넘는 것이
  원인이었다. 재생성 가능한 build cache만 정리해 23GiB를 확보하고 image와 제품 데이터를 보존했다.
- OpenSearch 4xx가 모두 `SEARCH_REQUEST_REJECTED`로 합쳐져 실패 경계를 숨겼다. alias·update·bulk·count·
  document·index create별 안정된 내부 오류 코드로 분류해 이후 진단이 입력이나 응답 본문에 의존하지 않게
  했다.

## 검증

- [x] TEST-09 scenario 1:1 browser execution — 15개 exact scenario × 3 engine
- [x] Chromium·Firefox·WebKit wide/compact gate — 54/54 통과
- [x] automated a11y·keyboard·visual gate — axe 0 violation, keyboard·reflow·12 snapshot 통과
- [x] failure screenshot·trace·seed artifact — CI retention과 local cleanup 확인
- [x] root·Compose regression gate — `bun run check`, `bun run compose:integration` 통과

## 결과

고정 Linux browser image에서 기준선을 갱신한 뒤 새 Compose project로 기준선 수정 없이 재실행했다.
Chromium·Firefox·WebKit의 54개 browser test가 모두 통과했다. TEST-09의 저장·장애 주입 증거는 exact
Compose suite가, 사용자가 관찰하는 route·권한·협업·복구 결과는 같은 ID의 browser suite가 소유한다.
