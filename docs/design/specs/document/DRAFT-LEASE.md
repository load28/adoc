# Draft와 Edit Lease

- **문서 ID**: SPEC-07
- **상태**: 동결

## Draft state

Document당 active Draft 0..1. `CreateDraft(baseVersionId?)`, `SaveOperations`, `ReplaceDraftFrom
Version`, `DiscardDraft` command를 제공한다. 모든 content mutation은 expected revision과
lease token을 요구한다.

## Lease

lease TTL은 90초, holder heartbeat는 30초다. server time만 사용한다. acquire는 expired row를
compare-and-swap하고 token hash를 저장한다. browser 여러 tab도 서로 다른 client instance로
취급한다.

## Takeover

holder가 release하거나 expiry 후 새 사용자가 acquire한다. Manage 강제 takeover는 경고,
reason과 Audit를 요구하고 기존 holder SSE에 즉시 알린다. lease 상실 뒤 save는
`LEASE_LOST`로 거부한다.

## Autosave

client는 typing transaction을 1초 idle 또는 5초 max interval로 Operation batch한다. 동일
idempotency key 재전송은 같은 revision result를 받는다.

## Local recovery

encrypted-at-rest browser storage에 documentId, baseRevision, Operation batch와 timestamp를
둔다. 성공 save 뒤 삭제한다. login user·Workspace가 다르면 내용을 표시·전송하지 않는다.
