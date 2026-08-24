# Acceptance Scenarios

- **문서 ID**: TEST-02
- **상태**: 동결

## A-01 Workspace·권한

Google login, Workspace 생성, 초대, Group 구성과 nested Document Grant를 수행한다. 개인 grant,
Group deny, 최고 Group access와 Manage 조건이 정해진 precedence대로 UI·API·Search에서 같다.

## A-02 작성·발행

전체 editor block을 작성·import하고 autosave한다. Review Required subtree에서 approval을 받고
Publish한다. Published Version은 이후 Draft 변경에도 동일하게 렌더링된다.

## A-03 충돌·복구

두 session의 lease, offline buffer, stale save와 concurrent Publish를 재현한다. 입력은
사라지지 않고 stale 상태가 3-way Diff로 이동하며 사람 승인 없이 merge되지 않는다.

## A-04 협업

복수 Topic Discussion, mention, Attachment, close/reopen, Review changes request와 Inbox read·
resolve를 수행한다. Publish 뒤에도 관련 맥락과 link가 유지된다.

## A-05 Knowledge·AI

Vocabulary·Reference를 만들고 hybrid Search와 grounded query를 실행한다. 권한 밖 문서는
후보·Source·count에 나타나지 않는다. 외부 web은 opt-in일 때만 Source로 표시된다.

## A-06 AI 적용

scoped Rewrite는 Undo되고, Discussion Apply는 Proposal Diff 승인 후 적용된다. stale Proposal,
invalid Operation과 forced Writing Rule 위반은 Draft를 바꾸지 않는다.

## A-07 File·공개 link

upload·scan·Version reference·GC를 검증한다. Public link는 최신 단일 Published 문서와 embedded
File만 보며 tree·History·Discussion·Search·AI endpoint는 접근하지 못한다.

## A-08 삭제·복구

trash 복구, 30일 purge, Audit redaction, Search·File cleanup과 backup restore 후 deletion ledger
재적용을 검증한다.
