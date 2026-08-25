# Transaction, Event와 Job

- **문서 ID**: ARCH-06
- **상태**: 동결

## Command transaction

authorization snapshot 확인 → aggregate rows lock 또는 expected revision update → invariant
검증 → state·Audit·outbox를 한 PostgreSQL transaction에 기록 → commit 뒤 response.

외부 system 호출을 transaction 안에서 수행하지 않는다.

## Outbox

`outbox_event(id, workspace_id, aggregate, sequence, type, payload, audience, occurred_at, published_at)`를
사용한다. Browser 대상 audience는 producer transaction이 구조화하고 consumer가 payload key로 추론하지
않는다. Worker는 delivery Job claim 후 consumer별 idempotency ledger에 기록한다. event ordering은 같은
aggregate sequence에서만 보장한다.

## Queue

Redis는 interactive·normal·background priority queue를 제공한다. PostgreSQL `job` row가
상태 정본이고 Redis item은 Job ID wake-up signal이다. Redis 유실·duplicate signal은 허용하며 periodic
reconcile이 due queued row를 다시 signal한다.

## Job state

`QUEUED → RUNNING → {SUCCEEDED|FAILED|CANCELLED|TIMED_OUT}`. retry는 attempt를 추가하고 같은
Job identity를 유지한다. cancellation request와 process termination을 분리한다.

Outbox→Browser delivery도 첫 closed Job kind로 실행한다. domain transaction은 Outbox와 delivery Job을 함께
commit하고 handler는 Stream ledger·consumer receipt·Job success를 한 transaction에 기록한다.

## Retry

transient dependency 오류만 exponential backoff+jitter로 재시도한다. validation,
authorization, stale revision과 quota는 재시도하지 않는다. max attempt 뒤 dead-letter 상태와
operator action을 남긴다.

## Projection consistency

client는 write response의 revision과 projection watermark를 받을 수 있다. read-your-write가
필요한 Document query는 PostgreSQL을 사용하고 Search는 eventual 상태를 명시한다.
