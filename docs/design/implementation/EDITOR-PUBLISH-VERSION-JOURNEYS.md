# Editor·Publish·Version 사용자 여정 구현 계약

- **문서 ID**: PLAN-37
- **상태**: 구현 기준
- **적용 태스크**: TASK-040
- **상위 정본**: PROD-11·12·16, DOM-02·06, UX-05·06·16, SPEC-05~08·15

## 1. 완료 경계

이 계약은 SCR-05 Published, SCR-06 Draft Editor, SCR-09 Version History와 RQ-05~08의
브라우저 사용자 여정을 완성한다. SCR-15 Trash의 File 수명주기는 기존 PLAN-21·32를 재사용한다.
Discussion·Review composer의 세부 동작은 TASK-041이 소유하지만 Publish는 현재 PublishPolicy와 exact
Review 결과를 서버에서 검증한다.

완료는 placeholder가 아닌 실제 Content 렌더링, command 실행, Draft 저장, Publish, history·detail·diff,
restore와 conflict 복구가 같은 Document route에서 연결된 상태다. 브라우저 종류별 E2E와 시각 회귀는
TASK-044가 소유한다.

## 2. Route·query 정본

`/w/:slug/docs/:id`의 search parameter만 화면 상태를 소유한다.

| 상태 | URL | query |
|---|---|---|
| 발행 문서 | `?mode=published` | `getDocument` |
| 초안 편집 | `?mode=draft` | `getDocument`, `getDraft`, Lease |
| 발행 대화상자 | `?mode=draft&dialog=publish` | Draft revision과 Document revision |
| 버전 기록 | `?mode=published&panel=history` | `listVersions` |
| 버전 비교 | `?panel=history&from=:id&to=:id` | `compareVersions` |

Published 화면은 `DocumentDetail.publishedVersion.content`만 렌더링하며 Draft를 섞지 않는다. Draft 화면은
active Draft와 Lease 획득 뒤에만 mutation command를 활성화한다. query cache는 저장 ack의 revision을
반영하고 Publish·restore 성공 뒤 Document·Draft·Version query를 함께 무효화한다.
`publishedVersion`이 없는 Document는 유효한 미발행 상태다. Published 화면은 Document title, 아직 발행된
버전이 없다는 설명과 권한별 Draft 진입 action을 표시하며 Workspace 부재나 기능 준비 상태로 표현하지 않는다.

## 3. 하나의 command registry

`EditorCommand`는 `id`, label key, selection gate, 실행 함수와 shortcut을 갖는다. toolbar, slash palette,
keyboard와 block action menu는 이 registry의 같은 command ID를 실행한다. availability는 `EDITOR access ∩
active lease ∩ supported schema ∩ selection`으로 계산한다.

첫 구현 registry는 text bold·italic·underline·strike·code·link, paragraph·heading·quote·callout·divider·
toggle·bullet·ordered·task·code·table·image·file, undo·redo·save, block duplicate·delete·move up/down과
find·replace를 포함한다. Markdown input rule은 heading, quote, bullet, ordered, task, fenced code와 divider를
Tiptap extension에서 처리한다. 조합 입력 중에는 registry shortcut을 실행하지 않는다.

drag handle과 다중 block selection은 ProseMirror selection을 입력으로 registry의 `block.move`와 ordered
batch를 만든다. compact 화면은 동일 기능을 menu/dialog로 제공한다. drag 또는 pointer만 가능한 command는
허용하지 않는다.

## 4. Operation·undo·conflict

Tiptap transaction은 PLAN-29 adapter를 거쳐 CONTRACT-02 Operation으로만 저장한다. structural change는
explicit command boundary에서 즉시 flush하고 typing은 250ms quiet 또는 20개 operation에서 flush한다.
server ack의 inverse만 undo stack에 넣고 redo는 undo 직전의 정방향 batch를 새 revision에 재적용한다.

`DRAFT_REVISION_STALE`, `EDIT_LEASE_INVALID`, `EDIT_LEASE_HELD`, `PUBLISH_BASE_STALE`은 후속 전송을
중지한다. 브라우저는 local Content, server Draft와 base/current Published를 보존한다. 서로 겹치지 않는
stable block ID 변경만 자동 병합할 수 있고, 겹치는 block은 `local` 또는 `server`를 사용자가 선택한 뒤
새 Operation batch로 저장한다. last-write-wins와 Document 전체 덮어쓰기는 금지한다.

## 5. Import·export

Import는 `.md`, `.markdown`, `.txt`만 받는다. parser는 입력을 schema version 1의 stable-ID Content로
변환해 CONTRACT-01 validator를 통과시킨 뒤 현재 Draft와의 최소 Operation batch를 만든다. Markdown은
heading, paragraph, quote, bullet·ordered·task list, fenced code와 divider를 지원한다. 알 수 없는 HTML과
raw script는 실행하거나 rich content로 해석하지 않고 text로 보존한다. Import는 새 Draft revision을 만드는
명시적 command이며 현재 내용을 바꾼다는 확인을 요구한다.

Markdown·plain export는 현재 화면 snapshot을 순수 함수로 직렬화한다. Published mode는 immutable Version,
Draft mode는 editor의 현재 local snapshot을 사용한다. PDF export는 별도 서버 문서 복제 없이 semantic
Published renderer와 print stylesheet를 사용하고 `window.print()`를 연다. heading, link, alt, table semantic을
유지한다. File token·storage key·비공개 asset URL을 export payload에 넣지 않는다.

## 6. File 경계

drop·picker는 `CreateUpload → capability PUT → Complete`를 수행한다. READY asset만 image/file block으로
승격한다. upload 중 placeholder는 editor-local 상태이며 Content·recovery에 넣지 않는다. 실패하면 재시도와
제거를 제공하고 기존 Content를 바꾸지 않는다. upload 진행 중에는 Publish를 비활성화한다.

Published·Version File download는 Workspace, Document와 exact Version owner 권한을 서버가 먼저 검사한다.
브라우저는 storage key를 받지 않고 API URL만 사용한다. 과거 Version reference는 restore·Draft 제거와
무관하게 PLAN-21 GC reachability에 남는다.

## 7. Publish·history·restore UI

Publish dialog는 summary, current Draft revision, base/current Version, upload 상태와 정책 결과를 표시한다.
DIRECT는 즉시 command를 허용하고 REVIEW_REQUIRED는 exact approved Review가 없으면 Review panel 진입만
제공한다. Publish request는 Draft `If-Match`, 같은 idempotency key와 선택적인 current Lease pair를 보낸다.
성공하면 Published mode로 전환하며 반환된 immutable Version만 먼저 표시하고 queries를 재검증한다.

History는 version number·summary·publisher·time을 내림차순으로 표시한다. 두 Version 선택 시 server의
`DocumentOperation[]` diff와 양쪽 immutable snapshot을 함께 보여준다. Restore는 active Draft가 없을 때만
선택 Version base의 새 Draft를 만들고 Draft route로 이동한다. active Draft·stale Document revision은
현재 Draft를 보존한 채 명시적 conflict 상태를 표시한다.

## 8. 오류·복구·관측성

지원하지 않는 Content는 부분 렌더링하지 않고 raw JSON recovery download와 오류 code를 제공한다.
offline·timeout은 encrypted recovery와 같은 idempotency identity를 유지한다. Lease 상실은 editor를
read-only로 전환하고 unsynced recovery export를 제공한다. Import parsing, upload, Publish와 restore 오류는
각 command surface에 correlation ID와 재시도 가능 여부를 표시한다.

telemetry는 command ID, block type, operation count, revision, duration과 outcome code만 기록한다. Content,
selected text, filename, summary, File token과 operation payload는 기록하지 않는다.

## 9. 구현·검증 단위

1. `editor-schema`: Markdown/plain parser·serializer와 snapshot renderer input의 순수 함수
2. `ui-domain`: Version detail, Publish, restore client와 editor command·conflict state
3. `web/editor`: registry, palette, block·table actions, import/export, recovery와 Publish dialog
4. `web/document`: immutable Published renderer와 Version history·diff·restore
5. Rust·PostgreSQL: existing Publish·Version·File invariants의 missing query/negative integration 보강

unit gate는 import/export round trip, unsafe input, command availability, selection batch, 기본 ID factory와
conflict 선택을 검증한다. Web component gate는 Published·Draft·Publish·History 상태를 ko/en과 keyboard로
검증하고 미발행 Document가 Workspace 부재 문구를 포함하지 않음을 확인한다.
Compose gate는 save → publish → immutable read → diff → restore, stale base, active Draft, File READY·failure와
과거 Version download를 실제 PostgreSQL·ObjectStorage에서 검증한다.

## 10. 완료 조건

- SCR-05·06·09에 placeholder가 없고 정본 Content와 command가 동작한다.
- PROD-12의 content·command·import/export 항목에 UI 경로와 test가 있다.
- PublishedVersion은 읽기 전용이고 Publish·restore가 revision·policy·lease 계약을 우회하지 않는다.
- conflict와 upload failure에서 local 입력 또는 immutable snapshot을 잃지 않는다.
- root gate와 실제 의존 서비스 Compose integration이 통과한다.
