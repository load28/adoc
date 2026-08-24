# Collaboration 요구사항

- **문서 ID**: PROD-13
- **상태**: 동결

## Discussion

Discussion은 하나의 Document에 속하고 복수 Topic을 가진다. Topic은 text, Document,
Region 또는 External Resource다. Message는 mention, Reference와 Attachment를 포함한다.
사람만 Close·Reopen할 수 있고 Topic 변경이나 Publish가 과거 Message를 지우지 않는다.

## AI 토론 반영

AI는 합의, 미합의, 아이디어와 정보 부족을 Source Message별로 구분한다. 다중 Region 또는
문서 전체 반영은 Proposal·Diff·승인을 요구한다. AI가 Discussion을 닫지 않는다.

## Review

Review는 Draft revision과 PublishPolicy snapshot에 고정된다. Reviewer는 approve 또는
changes requested를 선택하며 수정 요청은 Discussion으로 연결한다. revision 변경 시 과거
결정은 이력으로 남고 active approval에서는 제외된다.

## Inbox

Mention, Review Requested, Changes Requested, Proposal Ready와 Publish Conflict를 정확한
target과 연결한다. `readAt`과 `resolvedAt`을 분리하고 동일 원인 event는 중복 item을 만들지
않는다.

## 알림

SSE는 상태 변화를 빠르게 전달하지만 정본이 아니다. reconnect 시 cursor 이후 event를
재전달하고 gap이면 Inbox query로 재동기화한다.
