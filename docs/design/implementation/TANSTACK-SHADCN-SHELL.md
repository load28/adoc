# TanStack·Tailwind·shadcn Shell 구현 계약

- **문서 ID**: PLAN-28
- **상태**: 구현 기준
- **구현 패키지**: IMP-22
- **정본**: UX-01~03·09~19, ADR-001·011, SEC-02

## 1. 책임과 변경 경계

IMP-22는 SSR document, provider, route frame, navigation, locale·theme, API query boundary와 공통
UI primitive를 소유한다. TASK-051은 presentation과 UI dependency만 교체하며 route, loader, API,
command, domain state와 기능 결과를 변경하지 않는다.

- `components/ui`: shadcn/ui primitive source, domain import 금지
- `components/product`: AppShell, PageHeader, StatusBanner, EmptyState, layout composition
- 기능 directory: query·command·permission을 presentation에 연결하는 domain component
- `packages/ui-domain`: API client와 browser application state, React UI source 금지

## 2. 정확한 dependency 계약

implementation 시점의 상호 호환 최신 stable을 exact version으로 lock한다.

- React·React DOM 19.2.x와 일치하는 `@types/react`, `@types/react-dom`
- 현재 repository의 TanStack Start·Router·Query 최신 compatible release
- Vite 8.x, `@vitejs/plugin-react`와 `@tailwindcss/vite`
- Tailwind CSS 4.3.x, `tw-animate-css`
- shadcn/ui latest CLI의 New York style source, Radix primitive, Lucide icons
- `class-variance-authority`, `clsx`, `tailwind-merge`

`components.json`은 `style: new-york`, `rsc: false`, TypeScript, CSS variable, `@/*` alias를 명시한다.
shadcn CLI는 초기 source 취득에만 사용하며 이후 overwrite upgrade는 별도 task와 visual gate를
요구한다.

## 3. Build·CSS entry

Vite plugin 순서는 TanStack Start → Tailwind CSS → React다. root CSS는 한 번 import하고 다음
순서로 구성한다.

SSR dependency bundling은 Vite command별로 분리한다. 개발 server는 CommonJS package를 Node·Bun
해석 경계에 externalize한다. production build는 `ssr.noExternal: true`로 dependency를 모두 bundle해
`node_modules`가 없는 Web runtime image에서도 실행 가능한 self-contained server artifact를 만든다.
두 mode의 설정을 하나의 고정값으로 합치지 않는다.

```text
tailwindcss import
→ tw-animate-css
→ dark custom variant
→ semantic Light·Dark variables
→ @theme inline mapping
→ base semantic reset
→ editor·document domain layer
```

JavaScript Tailwind config, CSS module, Sass와 runtime CSS-in-JS를 추가하지 않는다. 기존 feature
CSS는 token을 소비하는 domain geometry만 남기고 control·color·typography는 Tailwind component로
옮긴다.

## 4. Route topology·SSR

기존 UX-01 route tree를 그대로 유지한다. SSR loader는 session → Workspace membership → target
permission 순서이며 permission 확인 전 protected title을 HTML에 넣지 않는다. public route는 별도
minimal layout과 asset graph를 유지한다.

SSR과 client는 같은 class와 token을 출력한다. viewport 의존 DOM 분기는 hydration 전에 만들지 않고
CSS responsive presentation 또는 hydration 뒤 Sheet state로 처리한다. shadcn primitive가 사용하는 ID는
SSR에서 안정적이어야 하며 hydration warning은 gate failure다.

## 5. Theme·locale provider

legacy AppProvider를 제거한다. root `<html>`의 `data-theme-preference`와 `class=dark`를 hydration 전
bootstrap이 설정한다. 허용값은 `LIGHT|DARK|SYSTEM`뿐이며 System은 `prefers-color-scheme`을 구독한다.

`ProductAppProvider`는 translation context와 theme preference synchronization만 소유한다. locale은
`<html lang>`과 catalog가 일치한다. theme·locale action은 account menu 안에서 label과 현재 값을
동시에 표시한다.

## 6. AppShell anatomy

```text
body[min-h-svh]
├─ SkipLink
├─ GlobalHeader[56px]
│  ├─ nav trigger + product wordmark
│  ├─ workspace-scoped search
│  └─ workspace switcher + account menu
└─ ShellBody
   ├─ WorkspaceRail[264px, Wide]
   │  ├─ primary nav
   │  └─ DocumentTree[scroll]
   ├─ Main[min-width:0]
   │  └─ route page frame
   └─ ContextPanel[360px, route 선택]
```

header는 sticky, rail은 viewport 안에서 독립 scroll, main만 route page를 소유한다. nested `main`을
금지한다. Compact에서는 rail과 panel을 동시에 열 수 없는 Sheet로 바꾸며 trigger가 focus를 돌려받는다.
현재 route link는 `aria-current=page`, document tree selection은 별도 state다.

## 7. 공통 page composition

- `PageFrame`: 일반 max 1440px, document mode는 full width, responsive gutter
- `PageHeader`: breadcrumb/eyebrow, H1+Badge, description/metadata, action cluster
- `SectionHeader`: H2, 설명, secondary action
- `StatusBanner`: status·impact·recovery·correlation
- `RoutePending`: page anatomy와 같은 skeleton
- `RouteEmpty`: 이유·guidance·권한 있는 primary action 하나
- `RouteProblem`: stable code, 영향, retry-safe action과 correlation ID
- `DetailSheet`: compact row·panel detail, URL state 연결 가능

기능 화면은 raw page padding과 H1 layout을 직접 만들지 않고 이 composition을 사용한다.

## 8. Navigation·account interaction

global header의 Workspace switcher는 current Workspace 이름과 chevron을 가진 button이다. account menu는
user summary, locale, theme, Workspace 목록 link와 logout 순서다. logout은 다른 preference action과
같은 visual group에 inline button으로 놓지 않는다.

primary nav는 Home, Search, Inbox, Vocabulary다. 관리 nav는 Separator 아래 Trash, Settings다.
각 항목은 icon+label, active surface와 optional count를 가진다. document tree는 `문서` section heading,
create action, hierarchical row와 overflow menu로 구성한다.

## 9. 보안·접근성

기존 same-origin API, CSRF, canonical route와 restricted cache 계약은 유지한다. dynamic URL은 기존
typed helper를 사용하고 사용자 입력을 HTML로 삽입하지 않는다.

skip link, landmark, H1, 2px focus ring, 32px control target, dialog focus와 200% reflow를 공통 shell에서
보장한다. icon-only button은 Tooltip 이전에 accessible name을 가진다. route pending 뒤 main heading으로
focus가 이동하며 background query refresh는 focus를 이동시키지 않는다.

## 10. 구현 순서

1. dependency·Vite·alias·CSS theme와 components.json
2. UI primitive·variant·test
3. root provider·theme·global CSS
4. AppShell·navigation·document tree
5. auth·Workspace·common states
6. Document·Editor·context panel
7. Search·Inbox·Vocabulary·Settings·Trash·Public Viewer
8. legacy Atlaskit·CSS·license exception 제거

## 11. 검증 계약

1. `rg @atlaskit apps/web packages` 결과 0, package manifest·lockfile에서도 0
2. typecheck·unit·SSR build와 hydration warning 0, production SSR의 bare package import 0
3. component keyboard·focus·axe test
4. SCR-01~22 ready와 가능한 공통 state의 Story 대조
5. 1440×1000, 1024×768, 390×844, 200% zoom layout
6. ko·en, Light·Dark·System, reduced motion
7. root license·security·browser quality gate
