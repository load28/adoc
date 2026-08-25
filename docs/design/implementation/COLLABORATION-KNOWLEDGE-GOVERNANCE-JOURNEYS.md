# Collaboration·Knowledge·Governance 사용자 여정 구현 계약

- **문서 ID**: PLAN-38
- **상태**: 구현 기준
- **태스크**: TASK-041

## 1. 목적과 경계

이 문서는 SCR-07·08·10~14·16~21의 primary action을 이미 구현된 Rust aggregate와 1:1로 연결한다.
Web은 상태를 추론하거나 직접 합성하지 않는다. 모든 mutation은 OpenAPI command의 exact resource
revision, idempotency key, CSRF와 필요한 경우 Draft lease를 전송하고 성공 응답 뒤 query를 다시 읽는다.

화면은 다음 네 경계를 가진다.

| 경계 | 소유 상태 | mutation 정본 |
|---|---|---|
| Document collaboration | Discussion·Topic·Message·Review·Reference | Document·Discussion·Review·Draft aggregate |
| Workspace knowledge | Inbox·Search·Vocabulary | Inbox projection·Retrieval·Vocabulary aggregate |
| AI Inspector | Context·Job·Result·Proposal | AI Job·Proposal와 Draft lease |
| Settings | Membership·Group·Permission·Policy·Writing·Audit | Governance·Permission·Audit aggregate |

## 2. Route와 선택 상태

Document route는 `panel`, `discussion`, `review`, `job`, `proposal`을 canonical search state로 소유한다.
Inbox item은 target kind에 따라 가장 구체적인 Document panel 또는 AI proposal URL로 이동한다. Inbox,
Search, Vocabulary와 Settings는 Workspace leaf route를 유지한다. Settings의 `document`, `subjectKind`,
`subjectId`, Audit의 `action`, `actor`, `targetKind`, `from`, `to`, `cursor`만 공유 가능한 query state다.

composer text, mention, attachment, topic draft, deprecate reason, confirmation과 선택 중인 proposal operation은
브라우저 local state다. URL 갱신은 이 입력을 초기화하지 않는다. 선택 ID는 `encodeURIComponent`를 거친다.

## 3. Collaboration command 계약

Discussion 생성은 title, 첫 RichMessage, 하나 이상의 Topic을 하나의 command로 보낸다. Topic kind는 닫힌
`TEXT|DOCUMENT|REGION|EXTERNAL` 집합이고 kind별 필수 값이 없는 command는 UI에서 막는다. 열린 Discussion은
title 수정, Topic 추가·삭제, Message 생성·수정·redact, reason을 포함한 close를 제공한다. 닫힌 Discussion은
reason을 포함한 reopen만 제공한다. Message attachment는 READY FileAsset 업로드가 완료된 ID만 포함한다.

Review 요청은 현재 Draft revision과 effective PublishPolicy를 함께 보여 준 뒤 exact Draft revision으로
생성한다. `REQUESTED` Review는 본인 assignment의 approve 또는 Discussion ID가 필수인 request changes와
requester의 reason 필수 cancel만 제공한다. status·assignment·threshold는 응답을 표시할 뿐 client에서
재계산하지 않는다. stale Review는 최신 detail을 다시 읽고 입력한 reason과 Discussion 선택을 보존한다.

## 4. Inbox와 Realtime 계약

Inbox의 read와 resolve는 독립 command다. `read-all`은 클릭 시각 `before`를 고정해 그 시각 이전 항목만
처리하고 새 알림을 삼키지 않는다. item primary action은 target deep link 이동이며 권한 상실로 redacted된
target은 이동을 제공하지 않고 resolve만 허용한다. cursor page는 server 순서를 보존하고 ID로 중복 제거한다.

SSE event는 resource ID·revision으로 query invalidation만 수행한다. event payload로 row를 덮어쓰지 않는다.
중복 event는 무시하고 gap·cursor expiry는 열린 detail과 목록을 HTTP로 다시 읽는다.

## 5. Knowledge와 AI 계약

Reference 생성·삭제는 source Document의 현재 Draft revision, client instance와 active lease를 요구한다.
source Region과 target은 schema의 discriminated union으로 구성하며 target 조회 권한을 API가 먼저 확인한다.
Backlink는 server가 반환한 snapshot만 표시하고 제한된 source의 title·count를 추측하지 않는다.

Vocabulary는 create뿐 아니라 canonical term·definition·alias 전체 편집과 `ACTIVE → DEPRECATED`를 제공한다.
deprecate는 reason과 optional replacement를 exact concept revision으로 보낸다. 충돌 시 server normalization
오류를 표시하고 client 문자열 비교로 중복을 결정하지 않는다. Search는 URL query, cursor와 permission-safe
Source snapshot을 사용하며 결과는 target kind에 맞는 viewer link를 제공한다.

AI task registry는 `COMPOSE|REWRITE|REVIEW|DISCUSSION_APPLY|CONFLICT_MERGE|KNOWLEDGE_QUERY` 여섯 종류로 닫는다.
kind별 target과 instruction을 같은 Context preview input으로 만든다. preview fingerprint가 없거나 만료·입력
변경·revision 변경이면 실행을 막는다. Job cancel과 Proposal reject/apply는 exact revision을 사용하고 apply는
사용자가 선택한 dependency-closed operation과 active Draft lease가 있어야 한다. AI는 직접 Publish하지 않는다.

## 6. Governance 계약

Group은 create·rename·delete 외에 active Membership을 member로 추가·제거한다. 동일 Group의 모든 변경은
현재 Group revision을 직렬화하고 응답 뒤 groups query를 재조회한다. Owner·last owner·권한 충돌은 server
problem을 그대로 표시한다.

Permission은 USER와 GROUP grant, `NO_ACCESS|VIEWER|CONTRIBUTOR|EDITOR`, manage flag를 편집하고 explicit
grant를 exact permission collection revision으로 삭제한다. explanation은 subject kind·ID에 대한 effective
result, inheritance step과 fingerprint를 표시한다. PublishPolicy는 `DIRECT` 또는 `REVIEW_REQUIRED`와 approval
수, `ANY_EDITOR|USERS|GROUPS` reviewer rule을 하나의 exact revision command로 저장한다.

Writing configuration은 닫힌 `writing-rules-v1` registry와 server가 허용한 override만 표시·저장한다. 현재
OpenAPI가 override를 허용하지 않으므로 임의 rule editor를 만들지 않고 immutable baseline 확인 상태를 명확히
표시한다. AI provider credential은 화면에 입력·표시하지 않는다.

## 7. Audit query와 보안 순서

Audit filter는 `action`, `actorUserId`, `targetKind`, `from`, `to`, `cursor`의 bounded server query다. 저장소는
먼저 Workspace Admin/Owner를 확인한 후 parameterized predicate를 적용하고 `(sequence,id)` 역순 cursor를
유지한다. UI는 event detail에서 actor, target, before, after, metadata, correlation과 redaction 상태를
구조화해 표시하며 임의 문장이나 redacted 값을 복원하지 않는다.

모든 Search·AI Context·Backlink·File·Audit query는 permission scope를 만든 뒤 조회한다. 결과를 만든 다음
client 또는 handler에서 사후 필터링하지 않는다. 403/404 이후 restricted query cache를 제거한다. telemetry는
resource ID, 검색어, message, filename, definition, prompt, source title을 기록하지 않는다.

## 8. 실패와 복구

409·412는 최신 aggregate를 재조회하고 local composer 값을 보존한다. permission denial은 숨겨진 resource의
존재를 드러내지 않는다. upload failure는 Message 전송을 막고 재업로드 또는 attachment 제거를 제공한다.
Review invalidation, Proposal stale, Context expiry와 Job cancellation은 서로 다른 terminal state로 표시한다.
offline command를 성공처럼 표시하거나 자동 queue하지 않는다. 재시도는 새로운 idempotency key를 사용한다.

## 9. 구현과 검증 단위

1. UI-domain: 누락된 Topic·Message·Review·Inbox·Reference·Vocabulary·Group·Permission·Policy·Audit typed method와
   exact header/query test를 구현한다.
2. Collaboration·Knowledge: aggregate state별 control, target deep link, lifecycle form과 component test를
   구현한다.
3. AI: 6종 task registry, preview invalidation, Job·Proposal terminal action test를 구현한다.
4. Settings·Audit: Group member, grant delete/explain, Policy form, Audit filter/detail과 server integration test를
   구현한다.
5. root gate와 실제 PostgreSQL·Redis·OpenSearch·ObjectStorage Compose integration을 통과한다.

완료 판정은 각 primary action이 typed API method에 연결되고 success·stale·denied가 직접 assertion되며,
Audit·Inbox·Outbox projection이 실제 PostgreSQL 통합 테스트에서 일치할 때만 가능하다.
