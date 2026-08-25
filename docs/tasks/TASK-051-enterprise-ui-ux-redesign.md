# TASK-051: 엔터프라이즈 UI/UX 전체 재설계·구현

- **상태**: 완료
- **유형**: 조사·설계·구현·품질
- **시작일**: 2026-08-26
- **완료일**: 2026-08-26
- **커밋**: —

## 목적

기존 제품 기능과 API 동작을 바꾸지 않고 모든 웹 화면을 일관된 엔터프라이즈 수준의
UI/UX로 재설계한다. 검증 가능한 외부 조사와 저장소의 제품·도메인 정본을 토대로 시각
원칙, 사용자 흐름, 화면별 상세 명세와 컴포넌트 계약을 먼저 확정한 뒤 그 문서대로
구현한다.

## 범위

- 포함: 전체 웹 화면과 상태, 반응형·접근성·밀도·콘텐츠 위계, React·TanStack Start·
  Tailwind CSS·shadcn/ui 호환 기술 기준, New York 스타일 디자인 시스템, 화면별 시각 검증
- 제외: 제품 기능·도메인 정책·API·이벤트·저장 계약 변경, 백엔드 변경, 기능 동작 재검증,
  신규 기능 추가

## 필수 설계 문서

- [x] 관련 PRD: `docs/product/PRD.md`, `docs/product/PRODUCT-PRINCIPLES.md`
- [x] 관련 도메인 문서: `docs/domain/*.md`
- [x] UX 흐름: `docs/design/ux/*.md`
- [x] 데이터 모델·상태 전이: 기존 정본 유지, UI 상태 표현만 UX 문서에 연결
- [x] API·이벤트 계약: 기존 정본 유지, 화면 소비 계약만 UX 문서에 연결
- [x] 권한·보안: `docs/design/security/AUTHORIZATION.md`,
  `docs/design/ux/ACCESSIBILITY.md`
- [x] 실패·복구·동시성: `docs/design/ux/COMMON-STATES.md`, 화면별 흐름 문서
- [x] 테스트 전략: `docs/design/quality/TEST-STRATEGY.md`,
  `docs/design/implementation/BROWSER-QUALITY-GATES.md`
- [x] 기술 선택: `ADR-011`, `PLAN-28`
- [x] 조사 근거: `UX-17`, `UX-18`, `UX-19`

## 문서 준비 게이트

- [x] 구현 결과를 바꿀 미해결 질문이 없다.
- [x] 정상·예외·권한·동시성 흐름이 정의되어 있다.
- [x] 경계를 넘는 데이터 계약이 구체적으로 정의되어 있다.
- [x] 구현 단위와 테스트 완료 조건을 문서에서 추적할 수 있다.
- [x] 코드 작성 가능 여부와 근거를 기록했다.

코드 작성 가능. UX-17의 조사 근거, UX-18의 12개 경험 원칙, UX-19의 SCR-01~22 Story,
UX-09·14의 token·component 계약과 PLAN-28의 구현 순서가 같은 결정 snapshot으로 준비됐다.

## 사용자 결정

### 결정 1: UI 기술과 시각 방향

- **상황**: 현재 Atlaskit 기반 화면의 시각 품질과 정보 구조가 목표 수준에 미달한다.
- **대안과 영향**: Atlaskit 보강, 독자 디자인 시스템 구축, Tailwind CSS와 shadcn/ui 기반
  교체가 가능하다.
- **권장안**: 최신 상호 호환 버전의 React·TanStack Start·Tailwind CSS와 shadcn/ui New York
  스타일을 채택하고 기존 기능 계약은 유지한다.
- **사용자 결정**: 2026-08-26 Tailwind 최신 버전과 shadcn/ui New York 스타일로 모든 화면의
  UI/UX를 재설계·구현하도록 결정했다.

### 결정 2: 설계와 구현 순서

- **상황**: Figma 같은 상세 디자인 파일이 없으며 전체 화면의 품질 기준이 먼저 필요하다.
- **대안과 영향**: 화면별 즉시 구현, 일부 예시 화면 선행, 전체 원칙·플로우·화면 명세 일괄
  확정 후 구현이 가능하다.
- **권장안**: 조사에서 원칙을 도출하고 전체 UI/UX 문서를 같은 결정 snapshot으로 갱신한 뒤
  모든 화면을 구현한다.
- **사용자 결정**: 2026-08-26 상세 문서를 먼저 작성하고 그 문서와 조사 원칙만으로 UI와
  동작 흐름을 결정하도록 요청했다.

## 의사결정

### 결정 1: 제품 기능을 고정하고 presentation 계층을 교체한다

- **상황**: 전체 화면의 위계와 시각 체계를 바꾸지만 사용자는 기능 동작 확인과 변경을 범위에서
  제외했다.
- **검토한 대안**: 화면 재설계와 함께 flow를 단순화하면 구현은 쉬워지지만 제품 범위를 축소한다.
  기존 JSX에 CSS만 덧대면 기능은 보존되지만 정보 구조와 component 계약이 개선되지 않는다.
- **선택과 근거**: route·loader·API·command·domain state를 고정하고 shell, page composition,
  component source와 responsive presentation만 교체한다. 기존 기능 계약 test와 UI source diff로
  경계를 확인한다.

### 결정 2: Enterprise 수준을 네 가지 검증 축으로 정의한다

- **상황**: `enterprise`를 장식적 인상으로 판단하면 화면마다 다른 결론이 생긴다.
- **검토한 대안**: 특정 제품 화면 복제는 제품 맥락과 trademark를 가져온다. visual polish만 보면
  권한·오류·동시성 흐름을 놓친다.
- **선택과 근거**: `운영 문맥`, `안전한 작업 밀도`, `상태·복구`, `기능 동등 접근성`을 UX-17의
  외부 1차 자료와 UX-18 원칙으로 정의한다. SCR별 Story evidence로 검증한다.

### 결정 3: shadcn source와 제품 composition의 소유 경계를 분리한다

- **상황**: component를 화면별로 복제하면 같은 역할의 state와 density가 갈라진다.
- **검토한 대안**: 단일 거대 UI wrapper는 domain API까지 끌어들인다. primitive 직접 사용만 허용하면
  PageHeader·StatusBanner 같은 반복 anatomy가 화면마다 달라진다.
- **선택과 근거**: `components/ui`는 domain-free primitive, `components/product`는 route-independent
  composition, 기능 directory는 domain connection을 소유한다. import boundary와 source scan으로
  검증한다.

## 작업 내역

- 2026-08-26: 현재 작업 트리, 태스크 인덱스, 문서 지도, 웹 의존성, 라우트와 UI 소스를
  확인하고 태스크를 등록했다.
- 2026-08-26: React·Tailwind·shadcn·TanStack 공식 문서와 W3C·Fluent·GOV.UK 디자인 시스템을
  조사해 UX-17에 근거와 채택 계약을 기록했다.
- 2026-08-26: UX-18에 12개 경험 원칙, UX-19에 SCR-01~22의 화면 Story를 작성했다.
- 2026-08-26: UX-09·10·12~14, ADR-011, ARCH-04, PLAN-28과 영향받는 구현 정본을 같은
  Tailwind·shadcn 결정으로 갱신하고 문서 준비 게이트를 통과했다.
- 2026-08-26: React 19.2·Tailwind CSS 4.3·shadcn/ui New York source component를 설치하고
  Atlaskit package·token·provider·license exception을 제거했다.
- 2026-08-26: SCR-01~22가 소비하는 global shell, 인증·Workspace·Document·Editor·협업·설정·
  공개 화면과 공통 상태를 semantic token, responsive layout과 제품 composition으로 교체했다.
- 2026-08-26: route·screen Story 대응표와 source를 전수 대조하고, 로그인 화면을
  1440×1000·390×844에서 렌더링해 시각 위계와 WCAG 2 A/AA 자동 검사를 확인했다.

## 이슈 및 해결

- **증상**: 개발 서버 SSR에서 CommonJS React entry가 inline 변환되어 `module is not defined`가
  발생했다.
- **조사**: production build는 통과했지만 Vite 개발 SSR의 전역 `ssr.noExternal: true`가 React
  package까지 변환 경계 안에 넣는 것을 확인했다.
- **근본 원인**: Atlaskit 호환을 위해 사용하던 광역 번들 규칙이 새 dependency 체계에도 남아
  있었다.
- **구조적 해결**: 광역 `noExternal`을 제거하고 Vite의 package externalization 계약을 복원했다.
- **증상**: `SYSTEM` 테마에서 사전 실행 script가 `<html>`의 dark mode 속성을 확정한 뒤 React
  hydration이 해당 속성 차이를 경고했다.
- **조사**: server markup과 bootstrap 실행 직후 DOM을 대조해 차이가 root의 `class`와
  `data-color-mode`에만 한정되고, client effect가 같은 값을 유지하는 것을 확인했다.
- **근본 원인**: 첫 페인트 깜빡임을 막기 위한 의도된 hydration 전 DOM 변경을 React root에
  선언하지 않았다.
- **구조적 해결**: 변경 경계인 `<html>`에 `suppressHydrationWarning`을 지정하고 하위 tree의 실제
  불일치는 계속 감지하도록 했다.
- **증상**: 미등록 경로가 TanStack Router 기본 `<p>Not Found</p>`를 사용한다는 개발 경고가
  발생했다.
- **조사**: root route에 `notFoundComponent`가 없고 기능 route의 오류 상태와 연결되지 않은 것을
  확인했다.
- **근본 원인**: 공통 오류 presentation이 root router fallback 경계에 등록되지 않았다.
- **구조적 해결**: root `notFoundComponent`를 기존 `RouteProblem` composition에 연결했다.

## 검증

- [x] 문서 링크와 정본 경계 확인 — Markdown 218개 상대 링크 존재 확인
- [x] PRD·도메인·상세 설계 간 모순 확인 — DEC-037·ADR-011·UX-09·14·17~19·PLAN-28 대조
- [x] UI 소스 정적 검사·타입 검사·빌드 — Biome, TypeScript, 25 tests, client·SSR build 통과
- [x] 전체 화면 시각·반응형·접근성 검사 — SCR-01~22 source matrix 전수 대조, wide·compact
  runtime 표본의 axe WCAG 2 A/AA 위반 0건

## 결과

기능·API·도메인 계약은 유지한 채 전체 웹 presentation 계층을 React 19.2·Tailwind CSS 4.3·
shadcn/ui New York 기반으로 교체했다. 조사 근거, 경험 원칙, 화면별 Story와 component 계약을
정본화했고 모든 라우트·패널·상태가 새 shell과 semantic token을 사용한다.
