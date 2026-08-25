# Document Editor UX 구현 계약

- **문서 ID**: PLAN-29
- **상태**: 구현 기준
- **적용 패키지**: IMP-23

## 1. 책임 경계

Tiptap·ProseMirror는 브라우저 입력, selection과 접근 가능한 editing surface만 소유한다. 저장 정본은
CONTRACT-01 Content와 CONTRACT-02 Operation이다. `editor-schema`가 Tiptap JSON과 제품 Content 사이의
strict adapter, transaction-to-operation 변환과 client-side dry-run을 소유한다. Web screen은 Draft·Lease
API orchestration만 소유하고 reducer 의미를 복제하지 않는다.

IMP-23은 Discussion·Review·AI·Publish 정책을 결정하지 않는다. 해당 action의 진입점과 panel route만
유지하며 실제 기능은 IMP-24·25가 소유한다. Tiptap extension은 제품 schema node를 조용히 drop하거나
unknown node를 paragraph로 바꾸지 않는다.

## 2. Editor lifecycle

Document route의 Published shell은 SSR 가능하지만 `mode=draft` island는 hydration 뒤 시작한다. 순서는
Document와 Draft query → `clientInstanceId` 생성·tab session 유지 → Lease acquire → adapter validation →
Tiptap mount다. Draft가 없으면 create-or-get command 뒤 acquire한다. 권한·lease 실패는 같은 Content의
read-only surface로 전환한다.

상태는 `BOOTSTRAPPING → ACQUIRING → EDITING|READ_ONLY|UNSUPPORTED → RECOVERING|CONFLICT`다. Lease는
90초 TTL, 30초 renew이며 visibility 복귀 때 즉시 renew한다. pagehide에서는 keepalive release를 시도하되
release 성공을 가정하지 않는다. 다른 tab은 다른 client instance이므로 server 단일 lease가 직렬화한다.

## 3. Tiptap schema와 adapter

StarterKit의 history는 끄고 제품 node·mark registry를 schema version 1에 고정한다. paragraph, heading,
blockquote, list, task list, code block, table, image, file, callout, toggle와 divider는 stable `blockId` attr를
갖는다. text mark는 bold·italic·underline·strike·code·link와 제품 token 기반 highlight·color를 지원한다.

Import는 먼저 Content validator를 통과한 뒤 registry가 전체 tree를 Tiptap JSON으로 변환한다. export는
모든 block ID와 semantic attr를 보존하고 normalize된 Content를 반환한다. 등록되지 않은 type·attr·mark는
`UNSUPPORTED_CONTENT`와 원본 recovery export를 제공하며 부분 mount하지 않는다.

각 committed ProseMirror transaction은 before/after 제품 Content를 비교한다. stable ID가 보존된 block의
text·attr·mark는 좁은 Operation으로, 삽입·삭제·이동은 해당 block Operation으로 만든다. 하나의 structural
command가 복합 변경이면 dependency DAG를 만든다. adapter가 동일 의미를 증명할 수 없는 transaction은
정확한 최소 BLOCK·BLOCK_RANGE `REPLACE_REGION`으로 표현한다. DOCUMENT 전체 교체 fallback은 금지한다.

## 4. Composition·command·undo

`compositionstart`부터 `compositionend`까지 transaction을 UI에 반영하되 Operation 변환과 flush timer를
중지한다. 종료 transaction에서 한국어·emoji UTF-16 boundary를 계산해 한 번의 Operation group을 만든다.
shortcut은 `event.isComposing` 또는 keyCode 229일 때 실행하지 않는다.

command registry의 availability는 schema capability, selection, permission, lease와 lifecycle의 교집합이다.
toolbar·menu·keymap은 같은 command ID를 호출한다. browser/OS shortcut은 가로채지 않는다. Undo·Redo는
ProseMirror history가 아니라 ack 응답의 inverse batch stack만 사용하며 500ms typing window와 explicit
command boundary를 한 group으로 고정한다.

## 5. Operation buffer와 server ack

buffer item은 `{groupId, sequence, baseRevision, operations, createdAt}`이다. 250ms quiet, 20 operations 또는
structural boundary에 flush한다. in-flight batch는 항상 하나다. request는 동일 Idempotency-Key,
If-Match revision, Lease token과 client instance를 재시도 동안 유지한다.
same-origin SPA는 `Path=/`의 non-HttpOnly `adoc_csrf` cookie를 읽어 `X-CSRF-Token`에 그대로 넣는다.
session cookie는 계속 HttpOnly이며 CSRF token은 URL·storage·telemetry에 저장하지 않는다.

ack revision은 expected+1이고 applied operation ID가 요청 집합과 일치해야 한다. 성공하면 Draft revision,
fingerprint와 inverse stack을 갱신하고 해당 recovery record를 삭제한다. timeout은 같은 request identity로
재조회·재시도한다. revision·lease·precondition conflict는 후속 전송을 정지하고 server Draft를 다시 읽어
local operations와 함께 conflict 화면에 보존한다. 임의 rebase나 last-write-wins는 금지한다.

## 6. Recovery와 offline

각 tab은 AES-GCM 256-bit key를 생성하고 tab 범위 session storage에 raw key와 recovery session ID를
base64url·UUID로 저장한다. IndexedDB record는 Workspace·Document·Draft·recovery session·group ID,
base revision, operation sequence,
IV, ciphertext, schema version과 expiry만 가진다. ciphertext는 operation batch와 작성 시각을 포함한다.
AAD는 Workspace·Document·Draft·recovery session·group·schema version canonical tuple이다.

record는 operation enqueue 전에 기록하고 ack 뒤 삭제한다. TTL은 7일이다. key 부재·인증 실패·schema
불일치는 평문이나 부분 복구로 폴백하지 않고 encrypted artifact 삭제·export 선택을 제공한다. offline에서는
typing과 암호화 기록은 계속하되 send를 중지한다. online·visibility 복귀 때 Lease와 current Draft를 먼저
재검증한 뒤 exact revision일 때만 재개한다. unsynced record가 있으면 beforeunload 경고를 등록한다.

## 7. Paste·drop·File

paste parser 우선순위는 plain text override → allowlisted HTML → internal Document URL → File이다. DOM parser는
script·style·event attribute, unknown node와 unsafe URL을 제거한 뒤 adapter validator를 통과시킨다. 검증할
수 없는 rich fragment는 plain text로 사용자가 명시적으로 선택해야 하며 silent conversion하지 않는다.

File drop은 local upload placeholder를 만들고 SHA-256·size·MIME로 CreateUpload → capability PUT → Complete를
순서대로 실행한다. READY 응답만 stable asset ID의 image/file block Operation으로 바꾼다. 실패·취소
placeholder는 Content에 저장하지 않는다. UPLOADING·VALIDATING placeholder가 하나라도 있으면 Review·Publish
action을 차단한다. upload token은 memory에만 두고 URL·recovery·log에 넣지 않는다.

## 8. UI·접근성·반응형

모든 chrome은 공개 Atlaskit Button, Icon, Menu, Modal, Form, Spinner, InlineMessage와 ADS token으로 구성한다.
편집 canvas만 ProseMirror semantic DOM을 사용하되 색·간격·focus는 ADS token이다. 저장·lease·upload 상태는
`role=status` live region, conflict는 `role=alert`로 묶어 변화 단위로 알린다.

wide는 tree·canvas·panel, medium은 drawer, compact는 단일 canvas와 full-screen command surface다. drag의
동일 command를 move menu가 제공한다. toolbar는 roving focus, 현재 mark의 `aria-pressed`, shortcut hint를
제공하고 dialog 종료 시 selection과 editor focus를 복원한다.

## 9. 실패·관측성·보안

Editor telemetry는 state transition, batch operation count·latency, conflict code, recovery result와 upload
phase만 기록한다. Content, selection text, filename, token, operation payload는 기록하지 않는다. API Problem은
stable code와 correlation ID만 상태 machine에 입력한다. Content와 recovery byte는 외부 origin으로 보내지 않는다.

프로덕션 Bun runtime은 `/assets/` 아래 Vite hash asset만 `dist/client`에서 제공한다. percent decoding 뒤
정규화 결과가 원래 경로와 다르거나 상위 경로를 포함하면 거부한다. 허용된 asset은 명시적 MIME,
`nosniff`, immutable 1년 cache를 적용하고 GET·HEAD 외 method는 거부한다. 존재하지 않는 파일은 SSR로
fallback하지 않고 404로 닫는다. 초기 theme bootstrap도 같은 asset graph와 경계를 사용한다.

## 10. 검증 gate

- adapter: 모든 schema node·mark round trip, unknown 거부, stable ID와 UTF-16 한글·emoji fixture
- input: composition 중 command·flush 0회, 종료 뒤 단일 group, toolbar/keymap 동일 command
- buffer: 250ms·20개·boundary, single in-flight, timeout identity, ack mismatch·conflict 정지
- recovery: enqueue-before-send, AES-GCM AAD, ack delete, wrong key·TTL·offline·visibility 복귀
- lease: 두 tab acquire 단일 승자, renew/release, expiry 중 unsynced preservation
- file: upload phase, token non-persistence, READY 승격, failed placeholder와 publish block
- UI: axe, keyboard-only, compact·wide, 한국어·영어, Light·Dark와 reduced motion
- repository: generated contract, dependency license, root check와 Compose SSR gate
- runtime asset: traversal·잘못된 encoding·method·404 거부와 실제 hash asset GET
