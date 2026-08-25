# Workspace·Document Tree Journeys

- **문서 ID**: PLAN-36
- **상태**: 구현 기준

## 책임과 경계

SCR-01~04는 인증된 Session을 공통 전제로 사용한다. `/login`은 same-origin 상대 경로만
`returnTo`로 허용한다. `/invites/:token`은 인증 뒤 verified Google email과 token을 서버에서 함께
검증한 preview만 SSR loader에 넣는다. `/workspaces`는 목록과 생성 command를 소유한다.
Workspace shell은 permission-scoped Document Tree query와 mutation 진입을 소유한다.

server state는 API와 Query cache만 소유한다. form 입력·선택·dialog만 component state에 둔다.
mutation 성공 전 tree를 optimistic 변경하지 않고 response 반영 뒤 `document-tree`와 관련 Document
query를 invalidate한다.

## API client 계약

| Method | HTTP operation | 입력 | 결과 |
|---|---|---|---|
| `invitationPreview` | `getInvitationPreview` | token | `InvitationPreview` |
| `acceptInvitation` | `acceptInvitation` | token+command | `Membership` |
| `createWorkspace` | `createWorkspace` | name+command | `Workspace` |
| `documentTree` | `getDocumentTree` | workspaceId | `DocumentTree` |
| `createDocument` | `createDocument` | title+anchors+command | `Document` |
| `renameDocument` | `updateDocumentMetadata` | revision+title+command | `Document` |
| `previewDocumentMove` | `previewDocumentMove` | revision+anchors | `ImpactPreview` |
| `moveDocument` | `moveDocument` | revision+anchors+preview token+command | `Document` |
| `trashDocument` | `trashDocument` | revision+reason+command | `Document` |

모든 command는 browser cookie의 CSRF token과 mutation 시작 때 생성한 idempotency key를 끝까지
유지한다. move preview는 CSRF·revision을 요구하지만 idempotency key를 만들지 않는다. move commit은
preview의 anchor와 token을 그대로 사용한다.

## 화면 상태 전이

### Login·Invitation

anonymous invitation 진입은 `/login?returnTo=<encoded invitation path>`로 이동한다. login start도 같은
returnTo를 전달한다. 허용하지 않는 origin·scheme·backslash·double slash는 `/workspaces`로 정규화한다.
인증된 preview 성공은 Workspace 이름·role·expiry와 accept를 표시한다. invalid·expired·email mismatch는
같은 unavailable 화면이며 token을 표시하거나 log하지 않는다. accept 성공은 Workspace 목록을 새로
조회하고 preview의 Workspace home으로 이동한다.

### Workspace 목록·생성

목록 유무와 관계없이 `Create Workspace` form을 제공한다. trim 뒤 빈 이름과 200자를 넘는 이름은
client와 server가 같은 validation 실패로 막는다. SUBMITTING 동안 같은 command를 비활성화한다.
성공 response의 slug로 home에 이동한다. 실패 input은 보존한다.

### Tree

Tree는 API가 이미 permission filter한 node만 렌더링하며 client에서 제한 항목을 사후 제거하지 않는다.
각 node는 `effectiveAccess`를 함께 받아 Contributor에게 하위 생성, Editor에게 rename·move·trash action을
표시한다. root·선택 node 아래 생성, rename, move/reorder, trash를 제공한다. compact에서도 drag만
요구하지 않고 동일 Move dialog를 사용한다.

Move는 destination parent와 sibling anchor를 선택한 뒤 preview를 표시한다. permission/policy change 수와
expiry를 확인한 후에만 commit한다. preview 만료·revision conflict는 입력을 보존하고 tree를 재조회한다.
rename·trash stale도 같은 방식으로 current tree를 재조회한다. trash는 이유를 요구하고 성공 시 home으로
이동하며 subtree를 API 결과에서 제거한다.

## 권한·오류·복구

- loader는 Session → Membership → target permission 순서를 유지한다.
- `WORKSPACE_NOT_FOUND`, `DOCUMENT_NOT_FOUND`, `INVITATION_INVALID`는 제한 정보를 렌더링하지 않는다.
- `FORBIDDEN`은 shell 안 denied 상태를 사용하고 mutation form을 제거한다.
- `REVISION_CONFLICT`, `MOVE_PREVIEW_INVALID`, `DOCUMENT_TREE_CYCLE`은 입력을 보존하고 최신 tree를 재조회한다.
- network timeout은 idempotency key를 유지한 retry만 허용한다.
- tree query 실패는 editor·다른 screen cache를 폐기하지 않고 tree 영역만 unavailable로 만든다.

## 검증 계약

unit은 returnTo canonicalization, tree flatten/anchor selection과 command key 유지 조건을 검사한다.
component test는 empty/list/create, invitation invalid/success, nested tree와 mutation failure를 검사한다.
PostgreSQL integration은 token preview의 email·hash·expiry·single-use, permission prefilter, cycle·stale·
preview/commit transaction을 실행한다. TASK-044는 같은 흐름을 실제 browser에서 재실행한다.
