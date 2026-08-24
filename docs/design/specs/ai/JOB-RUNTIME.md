# AI Job과 Runtime

- **문서 ID**: SPEC-14
- **상태**: 동결

## Admission

AITask permission, expected revision, provider health, global·Workspace·User concurrency와 budget을
검사한 뒤 PostgreSQL Job을 생성하고 Redis priority queue에 signal한다.

## Execution

Worker claim → Context snapshot materialize → isolated Runtime execute → progress event → output
limit → structured validation → result commit. process crash는 lease expiry 뒤 retry한다.

## Priority

interactive rewrite/query > user-requested review/compose > indexing-linked background evaluation.
weighted fair queue로 큰 Workspace가 전체 slot을 점유하지 못하게 한다.

## Cancellation·timeout

cancel command가 `CANCEL_REQUESTED`를 기록하고 Runtime cancellation token/process signal을
보낸다. timeout은 adapter별 hard deadline이다. late result는 terminal Job에 commit하지 않는다.

## Usage

provider request ID, model, input/output unit, latency와 cost estimate를 저장한다. prompt·content는
usage log에 저장하지 않는다. budget reservation은 시작 전에, actual reconciliation은 완료 후
수행한다.

## Streaming

SSE event는 job sequence, phase, progress와 terminal summary만 전달한다. unvalidated generated
Operation을 실행 가능한 payload로 stream하지 않는다.
