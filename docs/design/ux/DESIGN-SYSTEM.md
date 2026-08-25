# UI Design System 계약

- **문서 ID**: UX-09
- **상태**: 구현 기준
- **기술 결정**: ADR-011
- **경험 원칙**: UX-18

## 1. 단일 기반과 source ownership

Tailwind CSS 4 CSS-first theme와 `apps/web/src/components/ui`의 shadcn/ui New York source를
단일 UI 기반으로 사용한다. component source는 package dependency의 black box가 아니라 저장소
코드이며 접근성·variant·token 변경을 application review 대상으로 삼는다.

- primitive: Button, Input, Textarea, Label, Checkbox, Badge, Card, Separator, Skeleton
- composite: Dialog, AlertDialog, DropdownMenu, Select, Tabs, Tooltip, Sheet, ScrollArea
- product composition: AppShell, PageHeader, StatusBanner, EmptyState, DataList, FilterBar,
  DetailSheet, ConfirmDialog, EditorToolbar
- domain component: DocumentTree, DocumentCanvas, DiffView, SourceItem, DiscussionThread

다른 UI library, raw Radix component의 화면 직접 사용, legacy Atlaskit import와 component 내부를
깨는 화면별 selector를 금지한다. product·domain component는 primitive를 조합하고 token을 새로
만들지 않는다.

## 2. Semantic color token

색은 Light·Dark 각각의 OKLCH CSS variable로 정의한다. 이름은 hue가 아니라 역할을 나타낸다.

| Token | 의미 | 허용 사용 |
|---|---|---|
| `background` / `foreground` | page canvas와 기본 text | body, article |
| `card` / `card-foreground` | 독립 surface | dialog, compact card |
| `popover` / `popover-foreground` | floating surface | menu, tooltip, select |
| `primary` / `primary-foreground` | 한 container의 primary action | primary button, selected control |
| `secondary` / `secondary-foreground` | 낮은 강조 action | secondary button, neutral badge |
| `muted` / `muted-foreground` | 보조 surface와 metadata | filter rail, helper text |
| `accent` / `accent-foreground` | hover·active navigation | nav item, command item |
| `destructive` | irreversible action·error | danger button, destructive status |
| `border` / `input` / `ring` | boundary·control·focus | border, field, focus outline |
| `success` / `warning` / `info` | committed·attention·informative | status icon+text+surface |

상태는 icon·label·description을 함께 사용한다. foreground text는 background와 WCAG 2.2 AA
대비를 만족한다. data visualization이 없는 화면에 category palette를 만들지 않는다.

## 3. Typography

system UI font stack을 기본으로 사용하고 한국어 glyph가 없는 전용 font를 사용하지 않는다.

| 역할 | 크기/행간 | 무게 | 사용 |
|---|---|---|---|
| Display | 30/36px | 650 | Login value, 빈 Workspace onboarding만 |
| Page title | 24/32px | 650 | H1 |
| Section title | 18/28px | 600 | H2, panel heading |
| Component title | 15/22px | 600 | card·row title |
| Body reading | 16/28px | 400 | Document article, long guidance |
| Body UI | 14/20px | 400 | form, table, navigation |
| Metadata | 13/18px | 400 | timestamp, helper, status detail |

11px 이하 text, all-caps 긴 label과 letter spacing으로 hierarchy를 만들지 않는다. 숫자 비교 column은
tabular number를 쓴다. 본문 폭은 72ch, 설명·form은 최대 640px다.

## 4. Spacing·size·shape

4px base scale을 사용한다. component 내부는 4·8·12·16px, component group은 16·24px, section은
32·48px를 사용한다. page gutter는 wide 32px, medium 24px, compact 20px다.

| 요소 | 계약 |
|---|---|
| Dense button/input | 높이 32px, target 최소 32×32px |
| Default button/input | 높이 36px, icon button 36×36px |
| Touch primary | compact 높이 44px |
| Navigation row | 40px, tree depth indent 16px |
| Data row | 최소 44px, multiline 56px 이상 |
| Radius | control 6px, card 8px, dialog 10px, pill은 status·chip만 |

page section마다 card를 쓰지 않는다. card는 독립 선택 단위, 경계가 필요한 compact 재배치와
auth surface에만 쓴다.

## 5. Elevation·border·motion

border와 spacing을 기본 계층 표현으로 사용한다. shadow는 dropdown·popover·dialog·floating
toolbar와 auth card에만 적용한다. nested card shadow를 금지한다.

- micro transition: 120ms, hover·pressed·color
- overlay enter/exit: 160~200ms, opacity + 4~8px translate
- layout 이동 animation: 원칙적으로 없음
- `prefers-reduced-motion`에서는 non-essential transition duration을 0으로 줄인다.
- loading spinner는 300ms 뒤 표시하며 skeleton shimmer animation은 reduced motion에서 멈춘다.

## 6. Component anatomy

### PageHeader

`Breadcrumb/Eyebrow → H1 + status → description/metadata → action cluster` 순이다. H1과 primary
action은 각각 하나다. compact에서는 action cluster를 H1 아래로 wrap하고 위험 action은 menu에 둔다.

### Section

`H2 → optional description → optional secondary action → content` 순이다. section 사이 32px,
heading과 content 사이 12~16px를 쓴다. 제목 없는 visual group은 fieldset·list 등 semantic이
명확할 때만 허용한다.

### Form

label → optional description → control → field error 순이다. 24px 간격의 field stack과 마지막
32px action row를 쓴다. placeholder는 label을 대체하지 않는다. destructive form은 일반 저장
form과 section을 분리한다.

### DataList·Table

비교 축이 3개 이상이고 row 간 비교가 중요하면 table을 쓴다. 그 외에는 semantic list를 쓴다.
toolbar는 search → filter → count → batch action 순이다. row primary target과 overflow action을
분리한다. compact에서 priority field를 list item에 남기고 나머지는 DetailSheet로 연다.

### StatusBanner

icon, 짧은 status title, 영향 설명, recovery action, optional correlation ID 순이다. `info`,
`success`, `warning`, `destructive`, `neutral` variant를 가진다. page-blocking과 inline variant를
구분한다.

### Dialog·Sheet

heading과 description은 필수다. form field, impact summary, error, footer action 순이다. primary
submit은 오른쪽, cancel은 왼쪽이며 destructive dialog는 destructive action을 마지막에 둔다.
compact에서는 복잡한 dialog를 full-height Sheet로 바꾼다.

## 7. Theme·locale

Light·Dark·System preference는 SSR HTML과 hydration 전에 적용해 flash를 막는다. token 값만
theme별로 달라지고 component structure와 emphasis hierarchy는 바뀌지 않는다. public viewer도
같은 theme contract를 사용한다.

한국어·영어에서 label이 30% 길어져도 control이 겹치지 않게 wrap한다. date·number는 사용자
locale·timezone으로 표시한다. icon만 방향성을 표현하지 않고 논리적 inline-start/end를 사용한다.

## 8. 금지 규칙

- raw hex·rgb·hsl·oklch를 component에 직접 작성
- arbitrary spacing·radius·shadow를 반복 사용
- 의미 없는 gradient, glass, neon, decorative chart와 oversized hero
- 12px body, 24px 미만 target, outline 제거, hover-only command
- button 안 link 또는 link 안 button
- toast만으로 command 결과·error·unsaved state 표현
- page별 자체 Button·Input·Dialog·Badge 구현

## 9. 품질 gate

1. source scan에서 Atlaskit import, raw color, inline style와 비표준 token이 0이다.
2. component variant snapshot이 Light·Dark에서 contrast와 focus를 통과한다.
3. Button·Dialog·Menu·Select·Tabs·Sheet가 keyboard·focus APG scenario를 통과한다.
4. 화면은 UX-19 Story의 page frame·state·responsive·accessibility 계약과 일치한다.
