# Publish와 Version

- **문서 ID**: SPEC-08
- **상태**: 동결

## Preconditions

EDITOR, active Draft, expected revision, valid lease 또는 explicit publish-without-edit lease,
READY File, valid Reference, policy gate와 no unresolved base conflict를 요구한다.

## Transaction

Document row lock → current Version 확인 → policy·Review 재검증 → next number 계산 → immutable
Version·context insert → current pointer update → Draft close → Audit·outbox insert → commit.

## Conflict

Draft baseVersionId와 document.currentVersionId가 다르면 `PUBLISH_BASE_STALE`과 세 Version ID를
반환한다. merge result는 Draft Operation으로 적용되어 revision을 올리고 approval을
무효화한다.

## Restore

과거 Version snapshot을 current schema로 읽고 active Draft가 없을 때 새 Draft를 생성한다.
active Draft가 있으면 사용자가 discard 또는 먼저 Publish할 때까지 거부한다.

## Public latest

Public link는 versionId를 고정하지 않고 document current pointer를 transactionally 읽는다.
새 Publish 직후 최신 Version을 제공하며 Version이 없거나 trash이면 generic unavailable이다.
