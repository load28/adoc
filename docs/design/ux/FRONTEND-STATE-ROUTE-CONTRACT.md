# Frontend State·Route Contract

- **문서 ID**: UX-15
- **상태**: 동결

## State owner

| State | 정본 | 보존 범위 | mutation 방식 |
|---|---|---|---|
| Session·Membership·Permission | server query | request/session | loader 재검증 |
| Workspace·Document·Draft·Review·Job | PostgreSQL API | query cache | command 후 key invalidation |
| 현재 route·mode·panel·filter | URL | history/share | typed search parameter |
| editor unsynced operations | browser memory+encrypted recovery store | tab/recovery TTL | revision ack 뒤 제거 |
| dialog·menu·selection | component state | current view | local reducer |
| theme·locale·panel width | user preference | session 간 | preference command+SSR cookie |
| SSE cursor | server event ID+session storage | reconnect | query reconciliation 후 advance |

server state를 전역 client store에 복제하지 않는다. TanStack Router loader와 Query cache key는
`[workspaceId, resourceKind, resourceId, viewParameters]`다. 다른 Workspace key와 data를 재사용하지
않고 Workspace 전환 시 in-memory restricted cache를 폐기한다.

## Typed route search

| Route | 허용 search parameter | default·invalid 처리 |
|---|---|---|
| Document | `mode=published|draft`, `panel`, `discussion`, `review`, `job`, `proposal`, `from`, `to`, `region` | unknown 제거, 권한 없는 panel 닫기 |
| Search | `q`, `kind[]`, `updatedAfter`, `cursor` | q trim, cursor는 history replace |
| Inbox | `status`, `kind[]`, `cursor`, `item` | unknown enum 제거 |
| Vocabulary | `q`, `status`, `concept` | detail ID invalid 시 목록 유지 |
| Trash | `q`, `parent`, `cursor` | inaccessible parent 제거 |
| Settings | section route+`subject`, `document`, `cursor` | Admin/Manage gate 뒤 parse |

URL에는 token, lease token, idempotency key, Draft body, AI prompt와 Message draft를 넣지 않는다.
Region deep link는 opaque region ID만 가지며 excerpt를 query에 넣지 않는다.

## SSR·CSR boundary

SSR loader는 session, Workspace shell, Published Document, public viewer와 initial collection page를
렌더링한다. Draft Editor, Tiptap instance, drag-and-drop, Diff interaction, Context Inspector와
SSE subscriber는 hydration 뒤 CSR island다. SSR과 client가 같은 query serializer·permission
result type을 사용하며 hydration 전에 mutation을 허용하지 않는다.

## Command state machine

`IDLE → VALIDATING → SUBMITTING → COMMITTED|FAILED|CONFLICT`를 공통 command state로 쓴다.
`SUBMITTING` 중 같은 action은 disabled다. network timeout은 idempotency key를 유지한다.
`COMMITTED`는 response resource를 cache에 먼저 반영한 뒤 관련 key를 invalidate한다.
`CONFLICT`는 local intent와 current server resource를 모두 보존한다.

## Draft operation buffer

editor transaction은 schema operation으로 변환해 순서를 부여한다. 250ms quiet 또는 20개
operation마다 batch를 보내되 block boundary command는 즉시 flush한다. 한 batch가 ack되기 전
다음 batch는 전송하지 않는다. ack revision이 expected+1이 아니면 buffer 전송을 멈추고
conflict recovery로 전환한다. Undo는 server-acked inverse Operation만 사용한다.

## SSE reconciliation

SSE는 cache를 직접 진실로 만들지 않고 invalidation signal로 사용한다. event sequence gap,
`STREAM_CURSOR_EXPIRED`, visibility 복귀와 reconnect 시 active route query를 재조회한다. local
Draft ack와 동일 operation ID event는 중복 toast 없이 revision만 확인한다.
