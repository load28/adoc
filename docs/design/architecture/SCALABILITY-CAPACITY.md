# Scalability와 Capacity

- **문서 ID**: ARCH-08
- **상태**: 동결

## 기준 부하 model

설계 검증 기준은 Workspace 10,000개, active Member 100,000명, 저장 Document 10,000,000개,
동시 session 10,000개, 초당 query 1,000, command 200, SSE connection 10,000이다. 이는 판매
한도가 아니라 partition·index·test 기준이다.

## 확장 축

- web/API: stateless horizontal scaling
- PostgreSQL: connection pool, read replica는 immutable/read query만, tenant hot-spot 관찰
- Worker: queue kind별 독립 replica와 concurrency semaphore
- OpenSearch: workspace routing key, alias 기반 reindex
- File: streaming I/O, content-length 제한, local volume에서 S3 adapter로 전환

## AI quota

interactive가 background보다 우선한다. system global, Workspace, User 순으로 concurrency를
예약한다. Admin은 monthly provider usage budget과 per-user cap을 설정한다. admission control은
Job 생성 전에 수행하고 사용량·reset 시각을 표시한다.

## Backpressure

DB pool, queue depth, SSE buffer와 provider rate limit은 bounded다. limit 도달 시 memory queue를
늘리지 않고 `429/503 + retryAfter` 또는 queued position을 반환한다.

## Large document

content size, block count, table dimension, upload size와 export runtime에 configurable hard
limit을 둔다. limit은 UI·API·import·AI Operation validation에서 같은 config를 사용한다.
