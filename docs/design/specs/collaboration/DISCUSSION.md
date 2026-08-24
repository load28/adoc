# Discussion

- **문서 ID**: SPEC-09
- **상태**: 동결

## State

`OPEN ↔ CLOSED`. Delete state는 없고 retention purge만 제거한다. Message edit는 원 revision을
append-only history로 남긴다.

## Commands

CreateDiscussion, AddTopic, RemoveTopic, AddMessage, EditMessage, CloseDiscussion,
ReopenDiscussion. expected Discussion 또는 Message revision과 idempotency key를 요구한다.

## Topic

TEXT는 non-empty text, DOCUMENT·REGION은 same Workspace target, EXTERNAL은 normalized URL과
display metadata를 가진다. target 권한 상실은 Reference를 삭제하지 않고 redacted render한다.

## Mention

active Member만 mention할 수 있고 source_key로 Inbox를 deduplicate한다. Mention recipient가
Document CONTRIBUTOR 미만이면 존재를 노출하지 않고 validation error로 처리한다.

## Attachment

READY FileAsset과 Discussion FileReference를 atomically 생성한다. Message 삭제 기능이 없으므로
reference도 retention까지 유지한다.
