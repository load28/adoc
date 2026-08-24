# Observability와 SLO

- **문서 ID**: OPS-03
- **상태**: 동결

## SLI·SLO

핵심 Document read/write good event 비율을 월 99.9%로 한다. unauthorized response, corrupt
Version과 acknowledged-but-lost write는 latency와 무관하게 bad event다. Search·AI는 별도
availability SLI로 core error budget을 공유하지 않는다.

## Metric

request count·latency·error code, DB pool, transaction retry, queue depth·age, outbox lag, index
watermark, SSE connection·reset, lease conflict, AI usage·quota, File validation·GC와 backup age.

## Trace

requestId→correlationId→outbox event→Job attempt→provider request ID를 연결한다. content,
prompt, query, title와 token은 span attribute에 넣지 않는다.

## Log

structured JSON에 service, version, workspace opaque bucket, actor kind, code와 duration을 둔다.
PII redaction test를 CI에서 수행하고 security audit와 application debug retention을 분리한다.

## Alert

multi-window burn-rate, oldest queue age, backup/RPO, purge stuck, permission invariant, outbox lag와
provider credential failure를 alert한다. 단일 5xx보다 사용자 영향과 error budget으로 page한다.
