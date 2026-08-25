# Collaboration·Knowledge UX 구현 설계

- **문서 ID**: PLAN-30
- **상태**: 확정
- **적용 패키지**: IMP-24

## 1. 책임과 route 경계

Document route의 `panel=discussion|review|history|references`와 `discussion`, `review`, `from`, `to`,
`region` search parameter가 공유 가능한 선택 상태를 소유한다. Inbox, Search, Vocabulary는 각각 기존
Workspace leaf route를 사용한다. panel 전환은 editor instance와 Draft buffer를 재생성하지 않는다.

UI query key는 `{workspaceId, resourceKind, resourceId, cursor, filter, permissionFingerprint}`의 canonical
tuple이다. Workspace 전환·permission fingerprint 변경 시 restricted cache를 폐기한다. 입력 중 composer,
focus, dialog는 URL이나 server cache에 저장하지 않는다.

## 2. 공통 query·command 상태

모든 query는 `loading → ready|empty|restricted|error`를 명시한다. cursor page는 응답 순서를 보존하고 ID로
중복 제거한다. 재요청 실패가 이미 허용된 데이터를 지우지는 않지만 권한 오류는 즉시 restricted로 바꾸고
본문 cache를 폐기한다.

command는 UI event마다 새 UUID idempotency key와 현재 entity revision을 고정한다. pending 동안 같은
control을 비활성화한다. 성공 응답을 받은 뒤에만 화면 상태를 확정하고 영향받는 query key를 무효화한다.
409·412는 최신 resource를 다시 읽고 사용자의 입력은 composer에 보존한다. offline에서는 새 command를
전송하지 않으며 성공처럼 표시하지 않는다.

## 3. Discussion·Message·Topic

목록은 OPEN 우선, server ordering을 유지한다. 새 Discussion은 title, 첫 RichMessage, 하나 이상의 Topic을
원자 command로 보낸다. Region에서 시작해도 Topic만 Region을 가리키며 Discussion identity는 Document에
남는다. detail은 Topic, Message page, 상태 revision을 함께 표시한다.

Message composer는 text, mention, internal Reference chip, READY Attachment만 전송한다. 삭제는 row 제거가
아니라 redacted state를 표시한다. close·reopen은 reason과 exact revision을 요구한다. 닫힌 Discussion은
읽기 가능하고 composer만 비활성화한다. AI action은 이 화면에서 Discussion을 자동 닫지 않는다.

## 4. Review·History·Reference

Review request는 현재 Draft revision, reviewer assignment와 policy snapshot을 확인하는 surface를 거친다.
decision 화면은 requested revision의 Diff와 현재 invalidation 상태를 함께 표시한다. approve와 changes
requested는 exact Review revision command이며 후자는 Discussion link를 명시한다. threshold 충족 여부를
client에서 재계산해 Publish를 허용하지 않는다.

History는 PublishedVersion cursor와 선택한 `from`·`to` Diff를 읽기 전용으로 표시한다. Backlink와
Reference target은 조회 시점 권한을 다시 확인한 API 결과만 렌더링한다. restricted endpoint는 제목,
snippet, 존재 개수 대신 `접근할 수 없는 참조` 한 상태만 제공한다. Region reanchor 실패는 다른 위치로
이동하지 않고 stale 상태와 원문 Document 진입점만 제공한다.

## 5. Inbox·Search·Vocabulary

Inbox는 unread/resolved를 독립 filter와 control로 제공한다. read는 확인 행위, resolve는 처리 완료다.
target deep link는 가장 구체적인 Message·Review·Proposal·Conflict route를 사용한다. bulk action도 각 item
revision을 포함한 명시적 command 집합이며 부분 실패를 개별 표시한다.

Search는 submit된 query만 URL과 API에 반영하고 입력 중 문자열은 local state다. Source는 kind, official
version/draft revision, region과 visibility evidence를 표시한다. OpenSearch unavailable은 오류로 표시하고
PostgreSQL 본문이나 AI 일반 지식으로 조용히 대체하지 않는다. 새 cursor에서 permission scope가 달라지면
첫 page부터 다시 조회한다.

Vocabulary는 canonical term, definition, aliases, status, revision을 표시한다. create·update·deprecate는
충돌 후보와 영향 Reference를 확인한 뒤 exact revision command를 보낸다. term conflict를 client 문자열
비교로 결정하지 않고 server normalization 결과를 표시한다.

## 6. Realtime·실패·보안

Workspace SSE는 session cursor로 재개한다. Collaboration·Knowledge event는 ID와 revision만 이용해 관련
query를 invalidate한다. event payload를 정본 row처럼 cache에 쓰지 않는다. cursor 만료는 전체 Workspace
bootstrap과 열린 resource를 다시 읽는다. reconnect는 bounded backoff와 visibility 복귀 재검증을 사용한다.

telemetry는 screen, transition, stable error code, latency와 result count만 기록한다. query, Message,
Document snippet, Vocabulary definition, mention, filename과 Reference title은 기록하지 않는다. HTML body는
제품 RichMessage renderer allowlist만 사용하며 임의 HTML 삽입을 금지한다.

## 7. UI·접근성·반응형

panel, tabs, buttons, forms, lozenge, modal, pagination과 message는 공개 Atlaskit component와 ADS token만
사용한다. wide panel은 canvas 오른쪽, compact panel은 focus-trapped full-screen dialog다. 닫으면 원래
trigger와 editor selection으로 focus를 복원한다.

tab은 arrow key, list는 semantic heading·list, unread count는 text alternative를 제공한다. command 결과는
`role=status`, conflict·restricted는 `role=alert`로 한 번 알린다. Diff는 색만으로 의미를 구분하지 않고
추가·삭제 label을 제공한다. infinite scroll만 제공하지 않고 cursor pagination control을 함께 둔다.

## 8. 검증 gate

- Discussion: create/detail/message/redaction/topic/close/reopen, revision conflict와 draft preservation
- Review: exact revision Diff, approve/changes requested, invalidation과 threshold 비추론
- Inbox: read·resolve 독립 상태, target deep link, cursor dedupe와 partial bulk failure
- Knowledge: permission-safe Source·Backlink, stale Region, search outage, Vocabulary normalization conflict
- Realtime: duplicate·gap·expired cursor, event-as-invalidation, visibility·offline recovery
- UI: typed search round trip, keyboard-only, compact·wide, axe, 한국어·영어와 Light·Dark
- repository: generated contract, root check와 실제 Docker SSR·API proxy gate
