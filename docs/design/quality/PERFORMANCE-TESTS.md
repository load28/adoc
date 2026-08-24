# Performance Tests

- **문서 ID**: TEST-06
- **상태**: 동결

## Workload

Workspace 크기, tree depth, Document block 수, Permission Group 수, concurrent session, SSE,
Search corpus, File size와 AI queue depth를 [Capacity](../architecture/SCALABILITY-CAPACITY.md)의
기준으로 조합한다.

## Target

Document query p95 300ms, public viewer 500ms, command ack 500ms, Search 1.5s, AI first progress
2s를 정상 부하에서 검증한다. error rate와 saturation을 함께 기록한다.

## Test 종류

load 30분, stress until saturation, soak 8시간, spike 10배 5분과 dependency degradation을
실행한다. client think time과 realistic content size distribution을 사용한다.

## 핵심 검사

Permission resolver N+1, deep tree recursive query, large Diff, OpenSearch filter cardinality,
SSE slow consumer, Redis backlog, DB pool starvation과 local File streaming memory를 측정한다.

## Gate

평균값으로 p95·p99 실패를 숨기지 않는다. SLO target 초과는 capacity limit·architecture 변경
또는 사용자 승인된 목표 변경 없이는 통과시키지 않는다.
