# TASK-031: TanStack Shell·Atlaskit 구현

- **상태**: 완료
- **유형**: 구현
- **구현 패키지**: IMP-22
- **시작일**: 2026-08-25
- **완료일**: 2026-08-25
- **커밋**: —

## 목적

TanStack Start의 SSR·CSR 경계를 유지하면서 Atlaskit만으로 인증·Workspace·Document·설정 화면이
공유하는 접근 가능한 애플리케이션 셸, route, theme, locale 기반을 구현한다.

## 범위

- 포함: SSR root shell, typed route tree와 search parameter, session·Workspace loader 경계,
  Atlaskit AppProvider·token·reset, Light·Dark·System, 한국어·영어 i18n, 공통 loading·empty·error,
  responsive navigation, hydration·accessibility·license 검증
- 제외: Editor(IMP-23), Collaboration·Knowledge(IMP-24), AI(IMP-25), Settings·Audit·Public Viewer의
  실제 화면 기능(IMP-26)

## 필수 설계 문서

- `docs/product/PRD.md`
- `docs/design/ux/DESIGN-SYSTEM.md`
- `docs/design/ux/FRONTEND-STATE-ROUTE-CONTRACT.md`
- `docs/design/ux/SCREEN-INVENTORY.md`
- `docs/design/ux/COMMON-STATES.md`
- `docs/design/ux/ACCESSIBILITY.md`
- `docs/design/ux/RESPONSIVE-VISUAL-SPECS.md`
- `docs/design/api/openapi.yaml`
- `docs/design/security/AUTH-SESSION-CSRF.md`
- `docs/design/implementation/WORK-BREAKDOWN.md`
- `docs/design/implementation/TANSTACK-ATLASKIT-SHELL.md`

## 문서 준비 게이트

- [x] 제품 route와 SSR·CSR 소유권 확인
- [x] Atlaskit package·theme·token·license 경계 확정
- [x] Session·Workspace bootstrap과 cache 격리 계약 확정
- [x] typed search·공통 상태·반응형·접근성 계약 확정
- [x] hydration·route·a11y·license 테스트 전략 확정

## 사용자 결정

사용자는 React 기반 UI에 Jira와 같은 시각 언어를 적용하되 Jira 화면을 복제하거나 별도 디자인
시스템을 만들지 않고 공개 Atlaskit component를 직접 사용하도록 결정했다. 또한 중간 태스크 번호에
멈추지 않고 전체 구현 계획이 끝날 때까지 자율 진행하도록 결정했다.

## 의사결정

- 공개 Atlaskit package 중 IMP-22가 실제 사용하는 최소 집합만 정확한 버전으로 lock하고 후속
  domain 화면 dependency는 해당 태스크가 추가한다.
- AppProvider·CSS reset·tokens는 root에서 한 번만 적용하고 기능 module이 provider나 token 체계를
  만들 수 없게 한다.
- SSR과 CSR은 동일한 bootstrap serializer·typed route parser·API Problem codec을 사용한다.
- server state는 route/query cache만 소유하며 Workspace 전환 시 restricted cache를 폐기한다.
- 기능 leaf route는 stable topology를 예약하되 실제 screen과 loader는 IMP-23~26이 소유한다.
- license metadata가 없는 navigation-system은 exact version·공식 repository·공식 Apache-2.0 원문을
  검증하는 단일 exception gate로만 허용한다.

## 구현 순서

1. PLAN-28과 영향받는 정본 계약을 확정한다.
2. i18n·theme·session bootstrap primitive를 구현한다.
3. SSR shell과 typed route·responsive navigation을 구현한다.
4. 공통 상태와 접근성·hydration 검증을 구현한다.
5. root·Compose gate를 통과하고 완료 기록한다.

## 작업 내역

- 2026-08-25: IMP-22 구현 태스크를 등록하고 UX·route·session 정본 감사를 시작했다.
- 2026-08-25: PLAN-28에서 package·SSR bootstrap·route·theme·locale·API client·접근성·실패 복구와
  hydration 검증 계약을 확정하고 문서 준비 게이트를 통과했다.
- 2026-08-25: i18n·UI domain package와 session bootstrap을 구현하고 TanStack route tree, Atlaskit
  provider, 반응형 navigation, 공통 상태 화면을 연결했다.
- 2026-08-25: 고정 theme bootstrap, SSR locale, typed search, API Problem 정규화, 접근성 검증과
  Atlaskit 의존성 license gate를 구현했다.
- 2026-08-25: root 검증과 운영 Docker 이미지의 health·실제 `/login` SSR 응답을 검증했다.

## 이슈 및 해결

- Vite 8의 React plugin은 Babel plugin 주입 계약을 제공하지 않았다. token 변환 소유권을 Vite
  transform 계층으로 옮기고 `@rolldown/plugin-babel`에 Atlaskit token plugin을 등록했다.
- route 분할 뒤 Bun 단일 `outfile`은 동적 chunk를 표현할 수 없었다. server runtime을 `outdir`
  artifact로 만들고 Docker 진입점을 명시적인 runtime artifact로 변경했다.
- `@atlaskit/navigation-system` 배포 manifest에 license metadata가 없었다. 정확한 package version,
  공식 repository와 Apache-2.0 원문을 함께 검증하는 제한된 예외 gate로 해결했다.
- Atlaskit AppProvider가 초기화되지 않은 feature flag resolver를 호출했다. root provider에서 공개
  resolver를 한 번 초기화해 서버와 브라우저가 동일한 결정값을 사용하도록 했다.
- Docker SSR 검증이 browser에서만 계산되는 `data-color-mode`를 정적 HTML attribute로 기대했다.
  서버가 소유하는 theme preference와 hydration 전 bootstrap 실행 계약을 각각 검증하도록 분리했다.

## 검증

- [x] 문서 링크와 정본 경계 확인
- [x] hydration·typed route·theme·locale 검증
- [x] axe·keyboard·responsive contract 검증
- [x] dependency license·root·Compose gate

## 결과

SSR·CSR이 공유하는 TanStack route·bootstrap 계약과 Atlaskit application shell을 구현했다. 한국어·영어,
Light·Dark·System, 공통 상태, 반응형 navigation과 접근성·license·Docker 검증을 완료했다.
