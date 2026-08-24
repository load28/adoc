# Draft와 Edit Lease

- **문서 ID**: SPEC-07
- **상태**: 동결

## Draft state

Document당 active Draft 0..1. `CreateDraft(baseVersionId?)`, `SaveOperations`, `ReplaceDraftFrom
Version`, `DiscardDraft` command를 제공한다. 모든 content mutation은 expected revision과
lease token을 요구한다.

## Lease

lease TTL은 90초, holder heartbeat는 30초다. PostgreSQL server time만 사용한다. acquire는 expired
row를 lock·교체하고 token hash를 저장한다. holder는 userId와 clientInstanceId의 쌍이며 browser 여러
tab은 서로 다른 client instance다. 원 token은 acquire·force acquire 때 한 번만 반환한다.

## Takeover

holder가 release하면 API상 absent가 되지만 persisted lease revision tombstone은 event 순서를 위해 유지한다.
release 또는 expiry 후 새 client가 acquire한다. Manage 강제 takeover는 경고,
reason과 Audit 대상 outbox를 요구하고 기존 holder SSE에 즉시 알린다. lease 상실 뒤 save는
`LEASE_LOST`로 거부한다.

## Autosave

client는 typing transaction을 1초 idle 또는 5초 max interval로 Operation batch한다. 동일
idempotency key 재전송은 같은 revision result를 받는다.

## Local recovery

encrypted-at-rest browser storage에 documentId, baseRevision, Operation batch와 timestamp를
둔다. 성공 save 뒤 삭제한다. login user·Workspace가 다르면 내용을 표시·전송하지 않는다.
