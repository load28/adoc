# 화면 목록

- **문서 ID**: UX-02
- **상태**: 동결

| 화면 | 진입 조건 | 주요 상태 |
|---|---|---|
| Login·Callback | 미인증 | 시작, provider error, session established |
| Workspace 선택·생성 | 인증 | empty, creating, deletion pending |
| Invitation | token | valid, wrong account, expired, consumed |
| Document Published | VIEWER+ | loading, ready, no version, denied, trashed |
| Document Draft Editor | EDITOR + lease | acquiring, editing, read-only, recovery, conflict |
| Discussion Panel | CONTRIBUTOR+ | list, detail, closed, composing, failed |
| Review Panel | designated reviewer | requested, approved, changes, invalidated |
| History·Diff | VIEWER+ | versions, compare, restore confirmation |
| Search | Member | empty, querying, results, partial index outage |
| Inbox | Member | unread, actionable, resolved, target unavailable |
| Vocabulary | Viewer/Admin by action | browse, edit, conflict, deprecated |
| Trash | Manage | retention countdown, restore, impact, purge |
| Members·Groups | Admin | invite, pending, active, removed |
| Permission·Policy | Manage | effective, edit, impact preview, committed |
| Writing·AI Settings | Admin | rules, provider health, quota, usage |
| Audit | Admin | filter, event detail, export denied |
| Public Viewer | valid link | loading, ready, revoked/expired, no version |

Dialog는 destructive confirm, move impact, publish conflict, Proposal Diff, Context Inspector와
file validation을 포함한다. 모든 dialog는 URL-independent transient state지만 작업을
재개할 수 있는 server-side identity를 사용한다.
