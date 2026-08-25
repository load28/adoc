# Screen Behavior Specs

- **문서 ID**: UX-13
- **상태**: 동결

## 공통 screen contract

route loader는 session → Workspace Membership → target permission 순서로 검사한다. loader가
성공하기 전 Workspace title, Document title과 preload data를 HTML에 넣지 않는다. command는
server commit 전 성공 UI를 표시하지 않으며 오류는 [Error Catalog](../api/ERROR-CATALOG.md)의
code로 state를 선택한다.

모든 화면은 [Experience Principles](EXPERIENCE-PRINCIPLES.md)와
[화면 Story](SCREEN-STORIES.md)의 같은 SCR ID를 따른다. 이 문서는 loader·command·완료 결과를,
UX-19는 첫 viewport·정보 위계·반응형·접근성 표현을 소유한다.

공통 화면 anatomy는 `AppShell → PageHeader → optional StatusBanner → primary task → secondary
section → contextual surface`다. auth·public route는 AppShell 대신 전용 layout을 사용한다.

| Screen ID | Route | Loader query | Primary action | 완료 뒤 |
|---|---|---|---|---|
| SCR-01 | `/login` | `getSession` | `beginGoogleLogin` | 검증된 returnTo 또는 Workspace 선택 |
| SCR-02 | `/invites/:token` | token preview | `acceptInvitation` | 초대 Workspace home |
| SCR-03 | `/workspaces` | `listWorkspaces` | `createWorkspace` | 새 Workspace home |
| SCR-04 | `/w/:slug/home` | `getWorkspace`, `getDocumentTree` | `createDocument` | 생성 Document published route |
| SCR-05 | `/w/:slug/docs/:id?mode=published` | `getDocument` | Edit·Publish 진입 | 같은 route의 draft 또는 publish dialog |
| SCR-06 | `/w/:slug/docs/:id?mode=draft` | `getDocument`, `getDraft` | `applyDraftOperations` | server revision 갱신 |
| SCR-07 | `...?panel=discussion[&discussion=id]` | `listDiscussions`, `getDiscussion` | `createMessage` | detail 끝에 committed Message |
| SCR-08 | `...?panel=review[&review=id]` | `getReview` | `submitReviewDecision` | 결정 결과와 Inbox resolve |
| SCR-09 | `...?panel=history[&from=&to=]` | `listVersions`, `compareVersions` | restore Draft | draft route |
| SCR-10 | `...?panel=references` | `listBacklinks` | Source 열기 | 권한 있는 target route |
| SCR-11 | `...?panel=ai[&job=&proposal=]` | `getAIJob`, `getProposal` | `applyProposal` | draft revision 갱신 |
| SCR-12 | `/w/:slug/search?q=` | `searchKnowledge` | result 열기 | Document+Region route |
| SCR-13 | `/w/:slug/inbox` | `listInbox` | `resolveInboxItem` | 다음 unresolved item |
| SCR-14 | `/w/:slug/vocabulary` | `listVocabulary` | create/update concept | detail drawer 갱신 |
| SCR-15 | `/w/:slug/trash` | `listTrashedDocuments` | restore/purge Document | 목록 재조회 |
| SCR-16 | `/w/:slug/settings/members` | `listMembers`, `listInvitations` | invite/update/remove | table revision 갱신 |
| SCR-17 | `/w/:slug/settings/groups` | `listGroups` | group/member mutation | group revision 갱신 |
| SCR-18 | `/w/:slug/settings/permissions` | permission query | grant/policy mutation | impact 결과·tree 재조회 |
| SCR-19 | `/w/:slug/settings/writing` | `getWritingConfiguration` | `updateWritingConfiguration` | 새 version 표시 |
| SCR-20 | `/w/:slug/settings/ai` | `getAIConfiguration`, `getAIUsage`, `getAIProviderHealth` | `updateAIConfiguration` | health·limit 재조회 |
| SCR-21 | `/w/:slug/settings/audit` | `listAuditEvents` | filter | sequence cursor 갱신 |
| SCR-22 | `/p/:token` | `getPublicDocument` | embedded link·asset 열기 | viewer 내부 또는 asset response |

## Document screen

Wide layout은 Tree, DocumentCanvas, Context Panel landmark를 가진다. Published mode는 최신
Published Version만 기본 표시하고 active Draft 존재 badge를 별도로 보여준다. Draft mode는
lease 상태가 `HELD_BY_SELF`일 때만 mutation command를 활성화한다. lease를 잃으면 buffered
operation을 더 보내지 않고 Recovery drawer에 보존한다.

Document 변경 action의 순서는 validation → impact preview → confirmation → command → query
invalidation이다. move, permission, publish, trash와 purge는 optimistic update를 금지한다.
revision conflict는 기존 입력을 폐기하지 않고 server state와 local intent의 Diff를 연다.

## Panel과 dialog

panel identity는 URL에 남기고 composer text, selection, open menu는 client state로 둔다.
Discussion·Review·AI identity는 reload로 복구된다. destructive dialog는 대상 이름, 영향,
복구 가능 시점과 command failure state를 자체적으로 가진다. dialog가 닫혀도 실행 중 command를
취소하지 않으며 완료 결과를 Flag와 관련 screen query에 반영한다.

## Empty·denied·degraded

empty는 데이터를 만들 권한이 있을 때만 primary action을 보인다. known Workspace 내부에서
Document 권한을 잃으면 shell은 유지하되 본문을 제거한다. Search·AI 장애는 해당 screen만
unavailable로 만들며 Document read/edit는 유지한다. Public Viewer는 invalid, revoked, expired,
unpublished를 모두 동일한 not-found view로 렌더링한다.

## Compact behavior

Compact에서는 Tree와 Context Panel을 full-screen overlay route state로 표현하며 동시에 열지
않는다. editor toolbar는 bottom action에서 열리고 software keyboard 위 safe area를 사용한다.
모든 desktop action은 compact에서도 제공하되 drag는 Move dialog, side-by-side Diff는 unified
Diff로 바꾼다.
