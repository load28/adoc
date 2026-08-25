# Browser Quality Gates

- **문서 ID**: PLAN-41
- **상태**: 구현 기준
- **구현 태스크**: TASK-044

## 1. 검증 경계

브라우저 게이트는 production Web image와 API image를 실제 PostgreSQL·Redis·local ObjectStorage에
연결해 실행한다. HTTP를 page route interception으로 대체하지 않는다. TEST-09의 도메인 불변식은
기존 exact Compose suite가 소유하고, 각 scenario의 사용자가 관찰할 수 있는 결과를 실제 브라우저
journey가 같은 ID로 추가 증명한다. 두 evidence 중 하나만 존재하면 인수 시나리오는 완료가 아니다.

Google production credential은 사용하지 않는다. test profile의 별도 fixture executable이 production과
같은 session HMAC 계약으로 session row를 만들고 raw token은 프로세스 stdout으로 한 번만 전달한다.
API에 test-only 인증 endpoint나 runtime 분기를 추가하지 않는다. fixture는 isolated Compose project에서만
실행되고 project volume 제거와 함께 폐기된다.

## 2. 실행 행렬

| 축 | 값 | 필수 범위 |
|---|---|---|
| engine | Chromium·Firefox·WebKit | TEST-09 15개 journey 전부 |
| viewport | wide 1440×1000·compact 390×844 | 핵심 route·reflow·visual snapshot |
| locale | ko·en | fixture data·login·Workspace shell·문서·공개 Viewer |
| session | owner·member·anonymous | 권한 격리·multi-session·public boundary |

Chromium은 Chrome·Edge, Firefox는 Firefox, WebKit은 Safari의 지원 엔진 계약이다. 실제 vendor stable
채널의 차이는 production release 전 environment evidence로 남기며 repository gate는 고정 Playwright
browser revision을 사용한다.

## 3. deterministic fixture와 시나리오

`adoc-browser-fixtures`는 고정 UUID namespace와 semantic label을 사용한다. fixture output은 schema
version, session token, user·Workspace·Document·public token ID만 포함한다. DB URL, HMAC key와 CSRF
token은 artifact에 기록하지 않는다. 인증 만료는 실제 실행 시각을 기준으로 계산하고 화면에 보이는
발행 시각은 고정한다. browser process는 timezone `Asia/Seoul`, locale, reduced-motion, color scheme과
pixel ratio를 고정한다.

TEST-09 제목과 browser manifest는 exact 집합이어야 한다. 각 journey는 다음 공통 assertion을 가진다.

- navigation과 주요 action이 실제 API response를 거쳐 완료된다.
- page error·console error·unhandled rejection·실패한 same-origin request가 없다.
- 기대 landmark·heading·accessible name·focus 이동을 확인한다.
- scenario가 요구하는 denied·stale·recovery 결과는 사용자에게 구분 가능한 상태로 보인다.

multi-session은 서로 다른 BrowserContext를 동시에 열어 lease·permission 결과를 검증한다. browser
journey는 토론 close/reopen 뒤 Message 보존, lease 경쟁, 비공개 검색 격리, 공개 Viewer와 복구된 Inbox를
실제 UI에서 검증한다. stale proposal, File 보존, purge와 Redis fault injection은 TEST-09의 같은 ID를 가진
exact Compose suite가 상태·저장 경계에서 소유한다. browser가 구현하지 않은 fault injection을 간접 UI
assertion으로 대체하지 않는다.

## 4. 접근성·시각 안정성

axe-core의 WCAG 2.2 A·AA 자동 규칙에서 violation이 하나라도 있으면 실패한다. keyboard-only 검증은
skip link, logical tab order, dialog initial focus·trap·trigger 복귀, editor command 대체 조작과 visible
focus를 확인한다. 자동화로 증명할 수 없는 VoiceOver·NVDA 읽기 품질은 release environment evidence로
분리하고 자동 통과로 표시하지 않는다.

visual snapshot은 font loading 완료, CSS animation·transition 제거, caret 숨김, reduced motion, locale,
timezone과 viewport를 고정한 뒤 찍는다. pixel ratio는 1이고 snapshot 허용 차이는 0 pixel이다. OS font
rasterization 차이를 제거하기 위해 snapshot은 고정 Linux browser container/CI image에서만 승인한다.
wide 문서와 compact login·Workspace·공개 Viewer baseline을 같은 task에서 명시적으로 갱신한다. 기준선
검증 명령에서는 누락 baseline을 생성하지 않는다.

## 5. 실행·artifact·재현

`bun run browser:check`는 별도 Compose project를 bootstrap·build·wait하고 fixture를 seed한 뒤 Playwright를
실행한다. browser runner는 Web service의 network namespace를 공유하고 `http://localhost:8080`으로 접근해
production과 같은 Secure cookie·same-origin 계약을 지킨다. Web proxy의 public API origin도 같은 origin으로
고정한다. 성공·실패와 관계없이 container와 volume을 정리하며 기존 `adoc` project는 건드리지 않는다.

CI는 고정 `mcr.microsoft.com/playwright:v1.62.1-noble` image에서 3개 wide project와 3개 compact project를
worker 하나로 실행한다. 45개 acceptance 실행과 9개 compact 품질 실행을 retry 없이 검증한다. 실패 시
`artifacts/browser/`에 screenshot, trace와 seed manifest를 남긴다. browser binary가 고정되지 않은 host
실행은 gate로 인정하지 않는다.

`artifacts/`는 실행 도구가 원본 형식으로 생성하는 Git 비추적 evidence 경계다. schema·scenario·release
evidence는 각각의 전용 validator가 검사하며 source formatter·linter의 입력에서는 제외한다. 따라서 이전
브라우저 실행 산출물이 남아 있어도 repository source gate의 결과는 달라지지 않는다.

SSR bundle은 Vite가 dependency까지 하나의 server artifact로 만들고 Bun은 그 artifact만 runtime executable로
만든다. 이 경계는 SSR과 client에 서로 다른 React instance가 포함되는 것을 방지한다. CSP를 위해 JSON Schema
validator도 build 시 standalone 함수로 생성하며 browser runtime에서 동적 코드를 만들지 않는다.

## 6. Gate

다음 조건을 모두 만족해야 한다.

1. TEST-09와 browser manifest의 15개 scenario ID·제목 집합 차이가 0이다.
2. Chromium·Firefox·WebKit에서 15개 journey가 모두 실제 Compose endpoint를 사용해 통과한다.
3. 12개 wide·compact baseline, axe·keyboard·focus와 visual exact snapshot이 통과한다.
4. multi-session·권한 거부·공개 경계·복구 결과가 실행되고 manifest negative self-test가 통과한다.
5. browser binary나 production credential이 없어 실행하지 못한 검증을 통과로 기록하지 않는다.
