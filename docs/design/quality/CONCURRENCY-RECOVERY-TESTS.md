# Concurrency와 Recovery Tests

- **문서 ID**: TEST-03
- **상태**: 동결

## Race matrix

- lease acquire vs acquire, heartbeat vs takeover, expired save
- autosave duplicate vs reorder, save vs Review decision
- Draft edit vs approval, approval vs Publish
- two Publish commands, Publish vs trash, move vs Permission change
- Proposal apply vs manual edit, cancellation vs late AI result
- File attach vs GC, Workspace restore vs purge lease

각 test는 barrier로 정확한 interleaving을 만들고 final DB invariant·outbox count·client error를
검증한다. sleep 기반 timing test를 금지한다.

## Fault injection

commit 전후 connection loss, Redis flush, OpenSearch timeout, ObjectStorage partial write, worker
crash, SSE disconnect와 AI child process hang을 주입한다.

## Expected result

command는 한 번 commit되거나 명확히 미commit 상태여야 한다. unknown outcome은 같은
idempotency key query로 결정한다. projection·notification은 중복될 수 있지만 consumer
receipt로 최종 한 번의 의미만 반영한다.

## Backup restore

point-in-time restore 뒤 outbox replay, index rebuild, File checksum, current Version pointer,
Draft revision과 deletion ledger를 검증한다. RPO 15분·RTO 4시간 측정 evidence를 남긴다.
