# Atlaskit Component Matrix

- **문서 ID**: UX-14
- **상태**: 동결
- **정본 원칙**: [UI Design System](DESIGN-SYSTEM.md)

## Shell·layout

| UI 역할 | 공개 package·primitive | 제품 adapter | 금지 |
|---|---|---|---|
| Theme·reset | `@atlaskit/app-provider`, `@atlaskit/css-reset`, `@atlaskit/tokens` | `ProductAppProvider` | raw theme variable |
| Layout | `@atlaskit/primitives`, `@atlaskit/page-layout` | `WorkspaceShell` | 자체 grid token |
| Global·side navigation | `@atlaskit/navigation-system` | `WorkspaceNavigation`, `DocumentTree` | Jira DOM 복제 |
| Responsive overlay | `@atlaskit/drawer`, `@atlaskit/modal-dialog` | `CompactTree`, `ContextPanel` | 별도 drawer 구현 |
| Drag·reorder | `@atlaskit/pragmatic-drag-and-drop` | `TreeReorder`, `BlockReorder` | HTML DnD 직접 구현 |

## Input·feedback

| UI 역할 | 공개 package | 적용 위치 | domain 규칙 |
|---|---|---|---|
| Button·icon | `@atlaskit/button`, `@atlaskit/icon` | 모든 action | icon-only accessible label 필수 |
| Form | `@atlaskit/form`, `@atlaskit/textfield`, `@atlaskit/textarea` | 생성·설정·message | server fieldErrors 연결 |
| Select | `@atlaskit/select`, `@atlaskit/checkbox` | role·permission·filter | enum unknown state 처리 |
| User selection | `@atlaskit/user-picker`, `@atlaskit/avatar` | invite·review·group | Workspace Member만 후보 |
| Menu | `@atlaskit/dropdown-menu`, `@atlaskit/popup` | block·document action | 같은 command ID 호출 |
| Dialog | `@atlaskit/modal-dialog` | confirmation·impact·context | focus 복귀·pending 유지 |
| Status | `@atlaskit/lozenge`, `@atlaskit/badge` | review·lease·job | 색 외 text label 필수 |
| Feedback | `@atlaskit/flag`, `@atlaskit/inline-message` | commit·degraded·error | correlation ID 제공 |
| Loading | `@atlaskit/skeleton`, `@atlaskit/spinner`, `@atlaskit/progress-indicator` | route·command·job | 300ms 미만 spinner 억제 |
| Help | `@atlaskit/tooltip` | shortcut·icon action | 필수 정보를 tooltip에만 두지 않음 |

## Data·content

| UI 역할 | 공개 package·primitive | 제품 adapter | 경계 |
|---|---|---|---|
| Table | `@atlaskit/dynamic-table` | members, audit, permission | compact detail drawer 제공 |
| Tabs | `@atlaskit/tabs` | settings, history | route가 state 정본 |
| Pagination | `@atlaskit/pagination` | versions, audit | server opaque cursor adapter |
| Date | `@atlaskit/datetime-picker` | expiry filter·input | user timezone 변환 |
| Empty state | `@atlaskit/empty-state` | collection empty | 허용 action 하나만 표시 |
| Code | `@atlaskit/code` | code block·audit JSON | restricted payload 마스킹 |
| Link | `@atlaskit/link` | internal·external link | external rel·warning 정책 |
| Tag | `@atlaskit/tag` | filters·Vocabulary term | term identity와 분리 |

## Domain component

| Component | ADS 조합 | 소유하는 domain behavior | 소유하지 않는 것 |
|---|---|---|---|
| `DocumentCanvas` | primitives, button, popup | mode·lease·selection 연결 | content schema validation |
| `BlockRenderer` | primitives, code, link | schema node별 rendering | 저장·permission |
| `RegionHighlight` | tokenized primitive | Region anchor·stale 표시 | operation 적용 |
| `DiffView` | primitives, tabs, lozenge | Operation dependency·approval 선택 | merge 결정 |
| `SourceChip` | tag, popup, lozenge | source status·snapshot 열기 | permission 우회 cache |
| `DocumentTree` | navigation, drag-and-drop | rank·move preview 연결 | cycle 검증 |
| `ContextInspector` | modal, checkbox, tag | source include/exclude intent | retrieval authorization |

제품 adapter는 `apps/web/src/features/*`에 둔다. ADS wrapper를 별도 공통 UI library로 만들지
않고 domain component가 공개 package를 직접 조합한다. import는 package public entry만 허용하며
license allowlist, token lint, SSR render와 visual regression을 CI에서 검사한다.
