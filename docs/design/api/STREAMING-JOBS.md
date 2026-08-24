# SSE와 Job Streaming

- **문서 ID**: API-05
- **상태**: 동결

## 연결

`GET /api/v1/stream?workspaceId=...&cursor=...`은 session·Membership 확인 뒤 `text/event-stream`
을 반환한다. 15초 heartbeat comment, 60초 idle proxy tolerance와 graceful shutdown event를
사용한다.

## Event

```text
id: opaque-cursor
event: AI_JOB_CHANGED
data: {eventId,workspaceId,sequence,type,version,occurredAt,payload}
```

payload는 UI invalidation과 progress에 필요한 최소 정보만 가진다. client는 event를 정본
state로 누적하지 않고 관련 query cache를 갱신한다.

## Resume

server는 bounded stream ledger에서 cursor 이후를 재생한다. cursor가 너무 오래됐거나
권한 revision이 바뀌면 `STREAM_RESET_REQUIRED` event를 보내고 client가 Workspace bootstrap,
Inbox와 active Job을 다시 query한다.

## AI progress

Job sequence는 monotonic이다. `QUEUED(position)`, `RUNNING(phase)`, `VALIDATING`, terminal 상태를
전달한다. generated content token은 기본 stream payload가 아니며 검증 완료 결과만 query한다.

## Backpressure

connection별 bounded buffer를 넘으면 connection을 종료하고 resume을 요구한다. 느린 client
때문에 producer나 domain transaction을 block하지 않는다.
