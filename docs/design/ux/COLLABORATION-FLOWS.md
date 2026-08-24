# Collaboration 흐름

- **문서 ID**: UX-07
- **상태**: 동결

## Discussion 생성

문서 panel에서 제목·첫 Message 입력 → text Topic 기본 생성 → 필요 시 Document·Region·외부
Topic 추가 → mention과 Attachment 검증 → commit. Region에서 시작해도 Discussion은 Region
하위 객체가 아니다.

## Message와 Reference

composer는 내부 link paste를 Reference chip으로 변환한다. 권한을 잃은 target은 제목 대신
`접근할 수 없는 참조`로 표시한다. 전송 실패 초안은 local에 보존하되 server에 전송된 것처럼
표시하지 않는다.

## Close·Reopen

Contributor가 close reason을 입력하고 닫는다. AI는 action을 제공받지 않는다. Reopen은
기존 Message를 유지하고 새 event를 추가한다.

## Review·Inbox

Review 요청은 reviewer Inbox item과 SSE event를 만든다. reviewer는 exact revision Diff를
보고 approve 또는 changes requested를 제출한다. Inbox item은 읽음과 처리 완료를 별도
control로 제공한다.

## AI Apply

Discussion 선택 → 포함 Topic·Message·Reference 확인 → context conflict 표시 → job 실행 →
Proposal Diff에서 Source별 근거 확인 → 전체 또는 Operation별 승인 → expected revision
검증 후 적용. 적용 뒤에도 Discussion은 자동 close하지 않는다.
