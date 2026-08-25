# SSE와 Job Streaming

- **문서 ID**: API-05
- **상태**: 동결

## 연결

`GET /api/v1/stream?workspaceId=...&cursor=...`은 session·Membership 확인 뒤 `text/event-stream`
을 반환한다. query `cursor`와 `Last-Event-ID`를 동시에 보내면 값이 같아야 한다. 15초 heartbeat
comment, 60초 idle proxy tolerance와 graceful shutdown event를 사용한다.

## Event

```text
id: opaque-cursor
event: AI_JOB_CHANGED
data: {eventId,workspaceId,sequence,type,version,occurredAt,payload}
```

payload는 UI invalidation과 progress에 필요한 최소 정보만 가진다. client는 event를 정본
state로 누적하지 않고 관련 query cache를 갱신한다.

## Resume

server는 24시간 bounded PostgreSQL stream ledger에서 cursor 이후를 재생한다. cursor가 없으면 연결
시점 high watermark 이후부터 전달한다. cursor가 너무 오래됐거나 권한 revision이 바뀌면
`STREAM_RESET_REQUIRED` event를 보내고 연결을 종료한다. client는 Workspace bootstrap, Inbox와 active
Job을 다시 query한 뒤 새 cursor 없이 연결한다.

## AI progress

Job sequence는 monotonic이다. `QUEUED(position)`, `RUNNING(phase)`, `VALIDATING`, terminal 상태를
전달한다. generated content token은 기본 stream payload가 아니며 검증 완료 결과만 query한다.

## Backpressure

connection별 bounded buffer를 넘으면 connection을 종료하고 resume을 요구한다. 느린 client
때문에 producer나 domain transaction을 block하지 않는다.

Outbox producer가 event audience를 구조화해 기록하고 server는 전송 시 현재 Membership·Role·Document
Permission을 재검사한다. 필터된 event도 connection cursor는 전진한다. Redis Pub/Sub은 새 ledger row wake-up만
담으며 유실 시 heartbeat DB 조회가 복구한다.
