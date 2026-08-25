# shadcn/ui Component Matrix

- **문서 ID**: UX-14
- **상태**: 구현 기준
- **정본 원칙**: [UI Design System](DESIGN-SYSTEM.md)

## Shell·navigation

| UI 역할 | shadcn/ui·primitive | 제품 composition | 계약 |
|---|---|---|---|
| Global header | Button, DropdownMenu, Avatar, Separator | `GlobalHeader` | 56px, product·search·account |
| Workspace rail | ScrollArea, Tooltip, Collapsible | `WorkspaceRail` | 264px, route link semantic |
| Compact navigation | Sheet, ScrollArea | `WorkspaceNavSheet` | 640px 이하 overlay, focus return |
| Document tree | Collapsible, DropdownMenu, Button | `DocumentTree` | APG tree keyboard, menu move 대체 |
| Context panel | Tabs, Sheet, ScrollArea | `ContextPanel` | wide 360px, compact full height |
| Page heading | Badge, Button, Breadcrumb | `PageHeader` | H1·status·primary action 각 하나 |

## Input·feedback

| UI 역할 | shadcn/ui·primitive | 적용 위치 | 계약 |
|---|---|---|---|
| Action | Button | 모든 command | variant·pending·disabled·accessible name |
| Text input | Input, Textarea, Label | create·settings·composer | description·field error 연결 |
| Enum | Select, RadioGroup | role·permission·filter | unknown 상태와 keyboard |
| Boolean | Checkbox, Switch | source·rule·capability | label 전체 target |
| Search/select | Command, Popover | member·document·concept picker | APG combobox |
| Menu | DropdownMenu | row·document·block action | hover 외 동일 경로 |
| Dialog | Dialog, AlertDialog | form·confirmation·impact | focus trap·initial·return |
| Status | Badge, Alert | review·lease·job·error | text+icon, color 독립 |
| Notice | Alert, Toast | committed·degraded·error | surface 정본, toast 보조 |
| Loading | Skeleton, Spinner | route·row·command | 300ms 지연, layout 보존 |
| Help | Tooltip, HoverCard | shortcut·용어 | 판단 필수 정보는 inline |

## Data·content

| UI 역할 | shadcn/ui·primitive | 제품 composition | 경계 |
|---|---|---|---|
| Table | Table, Checkbox, DropdownMenu | `DataTable` | 비교축이 있을 때만, compact detail |
| List | Item, Separator, Badge | `ResourceList` | row primary link와 menu 분리 |
| Filter | Input, Select, Popover, Badge | `FilterBar` | active filter가 항상 보임 |
| Tabs | Tabs | settings·context panel | route/search가 state 정본 |
| Pagination | Button | `CursorPagination` | opaque cursor, 결과 위치 유지 |
| Empty | Card, Button | `EmptyState` | 이유와 권한 있는 action 하나 |
| Code | ScrollArea | `CodeBlock` | restricted payload masking |
| Source | Badge, HoverCard, Sheet | `SourceItem` | status·snapshot·permission 유지 |
| Diff | Tabs, ScrollArea, Checkbox | `DiffView` | operation dependency와 mode 변환 |

## Screen mapping

| Story 묶음 | 핵심 composition | 화면 |
|---|---|---|
| 인증·시작 | `AuthLayout`, `InvitationCard`, `WorkspaceList` | SCR-01~03 |
| 문서 작업 | `WorkspaceShell`, `DocumentHeader`, `EditorToolbar`, `ContextPanel` | SCR-04~11 |
| 지식·개인 작업 | `SearchResults`, `InboxList`, `ConceptDetail` | SCR-12~14 |
| 거버넌스·운영 | `SettingsLayout`, `DataTable`, `ImpactDialog`, `AuditDetail` | SCR-15~21 |
| 공개 읽기 | `PublicDocumentLayout`, `DocumentContent` | SCR-22 |

## Source 규칙

CLI가 생성한 primitive는 `components/ui`, 제품 composition은 `components/product`, domain component는
기존 기능 directory에 둔다. `components/ui`는 API call, route, 권한과 domain type을 import하지
않는다. product composition은 route-independent presentation만 소유한다. domain component만 query,
command gate와 domain state를 연결한다.
