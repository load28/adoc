# Enterprise UI/UX 조사와 설계 근거

- **문서 ID**: UX-17
- **상태**: 구현 기준
- **조사일**: 2026-08-26
- **적용 범위**: 인증, Workspace, Document, 협업, 지식, 설정, 운영, 공개 Viewer 전체

## 1. 조사 질문

이 조사는 “엔터프라이즈처럼 보이는 화면”을 시각 취향으로 정의하지 않는다. 다음 질문에
공개된 1차 자료가 제시하는 검증 가능한 계약을 찾고, 제품 정본과 충돌하지 않는 원칙만
채택한다.

1. 정보량이 많은 작업 화면에서 사용자가 현재 위치와 다음 행동을 어떻게 빠르게 파악하는가.
2. desktop의 높은 정보 밀도와 compact 화면의 동등한 기능을 어떻게 함께 유지하는가.
3. 위험·대기·오류·권한·동시성 상태를 어떻게 오판 없이 전달하는가.
4. Figma 없이도 설계와 구현의 일치를 어떤 문서와 code token으로 검증하는가.

## 2. 기술 기반 조사

| 근거 | 확인한 사실 | 이 제품의 결정 |
|---|---|---|
| [React Versions](https://react.dev/versions) | 공식 최신 major는 React 19이며 19.2 문서와 patch release가 제공된다. | React 19.2 계열로 올리고 exact patch는 lockfile에 고정한다. |
| [Tailwind CSS v4.3](https://tailwindcss.com/blog/tailwindcss-v4-3) | Vite에서는 `tailwindcss`와 `@tailwindcss/vite` 최신 설치를 권장한다. | Tailwind 4.3 계열과 전용 Vite plugin을 사용한다. |
| [Tailwind v4 upgrade](https://tailwindcss.com/docs/upgrade-guide) | v4는 CSS-first 구성과 modern browser 기능을 사용한다. | JavaScript config를 만들지 않고 CSS theme를 정본으로 둔다. |
| [shadcn Tailwind v4](https://ui.shadcn.com/docs/tailwind-v4) | Tailwind 4·React 19를 지원하고 OKLCH·`data-slot`·New York 기본 방향을 제공한다. | New York source component와 OKLCH semantic token을 사용한다. |
| [shadcn Vite](https://ui.shadcn.com/docs/installation/vite) | 기존 Vite project에 Tailwind plugin, alias와 source component를 추가할 수 있다. | monorepo의 `apps/web` 안에 UI source를 소유한다. |
| [TanStack Start](https://tanstack.com/start/latest/docs/framework/react/overview) | typed route, SSR, streaming과 Vite 기반을 제공한다. | route·loader·SSR 기능 계약은 유지하고 표현 계층만 교체한다. |

## 3. 엔터프라이즈 작업 UI 조사

| 근거 | 확인한 원칙 | 채택 계약 |
|---|---|---|
| [Fluent 2 accessibility](https://fluent2.microsoft.design/accessibility) | 명확하고 예측 가능한 정보 구조, semantic code, focus order와 screen reader 명세를 설계 단계부터 기록한다. | 모든 화면 story에 landmark, heading, focus entry·return과 announcement를 기록한다. |
| [Fluent 2 Nav](https://fluent2.microsoft.design/components/web/react/core/nav/usage) | 짧고 scan 가능한 260px navigation을 쓰며 640px 이하에서는 overlay로 전환한다. hover action은 DOM과 대체 경로에 남긴다. | wide rail 264px, compact sheet, 항상 접근 가능한 overflow menu를 사용한다. |
| [GOV.UK layout](https://design-system.service.gov.uk/styles/layout/) | small screen first, 읽기 본문 약 75자 이하, content 목적에 맞는 제한 폭을 권장한다. | 문서 본문 72ch, form 640px, data workspace는 가용 폭을 사용한다. |
| [GOV.UK spacing](https://design-system.service.gov.uk/styles/spacing/) | 제한된 spacing scale과 responsive spacing으로 vertical rhythm을 만든다. | 4px base scale과 4·8·12·16·24·32·48px 의미 단계를 사용한다. |
| [GOV.UK focus](https://design-system.service.gov.uk/get-started/focus-states/) | focus는 배경과 무관하게 명확하고 일관되어야 한다. | 2px ring + 2px offset, 고대비 ring token을 모든 interactive primitive에 적용한다. |
| [WCAG 2.2 target size](https://www.w3.org/WAI/WCAG22/Understanding/target-size-minimum.html) | pointer target은 원칙적으로 24×24 CSS px 이상이어야 한다. | dense control도 최소 32px, icon target은 36px를 기본으로 한다. |
| [WCAG focus appearance](https://www.w3.org/WAI/WCAG22/Understanding/focus-appearance.html) | 2 CSS px perimeter 수준과 3:1 변화 대비가 강한 focus의 단순 기준이다. | 그림자만으로 focus를 표현하지 않고 outline 면적과 대비를 검증한다. |
| [ARIA APG patterns](https://www.w3.org/WAI/ARIA/apg/patterns/) | dialog, menu, tree, combobox와 grid는 서로 다른 keyboard·focus 계약을 가진다. | shadcn/Radix primitive를 쓰고 composite마다 APG keyboard 시나리오를 검증한다. |

## 4. 조사에서 도출한 엔터프라이즈 기준

### 4.1 명료한 운영 문맥

화면은 `어디에 있는가 → 어떤 상태인가 → 무엇을 해야 하는가 → 결과가 무엇인가`를 첫
viewport에서 설명해야 한다. Workspace, Document, mode와 주요 상태를 header에 고정하고
primary action은 한 영역에 하나만 둔다.

### 4.2 점진적 복잡성

자주 쓰는 읽기·편집·검색은 즉시 보인다. 권한 근거, impact, provider detail과 raw audit
metadata는 drawer·detail panel로 연다. 숨긴 정보가 작업 판단에 필요하면 trigger 옆에
summary를 남긴다.

### 4.3 상태의 공간적 안정성

loading, empty, error와 ready가 같은 page frame을 공유한다. action 위치와 heading 높이를
유지하며 결과를 toast에만 맡기지 않는다. long-running state는 해당 resource header에 남는다.

### 4.4 안전한 높은 밀도

Enterprise density는 작은 글자와 좁은 target이 아니다. 14px body, 32~36px control, 44px row,
명확한 group border와 충분한 section gap을 사용한다. 표는 비교가 필요한 필드에만 쓰고
compact에서는 label-value detail로 재배치한다.

### 4.5 기능 동등 반응형

compact 화면에서 정보를 삭제하지 않는다. rail은 Sheet, context panel은 full-height Sheet,
table row는 detail card, side-by-side diff는 unified diff로 표현만 바꾼다. route와 command ID는
동일하게 유지한다.

### 4.6 접근성은 명세의 한 축

색·pixel과 같은 수준으로 semantic, accessible name, focus order, keyboard command, live region,
reduced motion과 200% reflow를 화면 story에 기록한다. 자동 검사는 semantic 오류를 찾고,
keyboard·screen reader 수동 증거는 별도로 남긴다.

## 5. 채택하지 않은 패턴

- dashboard용 장식 chart, KPI와 gradient를 근거 없이 추가하지 않는다.
- 모든 내용을 card 안에 중첩하지 않는다. page section은 whitespace와 heading으로 먼저 나눈다.
- icon-only global navigation, hover-only action과 tooltip-only 설명을 사용하지 않는다.
- 위험 action을 primary action 옆에 같은 강조도로 두지 않는다.
- status를 색 하나, spinner 하나 또는 toast 하나로만 표현하지 않는다.
- desktop table을 수평 축소해 compact에 그대로 노출하지 않는다.

## 6. 조사 적용 검증

모든 화면은 UX-18 원칙 ID와 UX-19 story ID를 참조한다. 구현 review는 화면별로 `page frame`,
`primary task`, `state family`, `responsive transformation`, `accessibility contract` 다섯 항목을
대조한다. 외부 제품의 화면 모양은 복제하지 않고 위 계약의 충족만 검증한다.
