# Job Runtime·Outbox·SSE 구현 설계

- **문서 ID**: PLAN-23
- **상태**: 확정
- **구현 패키지**: IMP-17
- **관련 태스크**: TASK-024

## 1. 목적과 경계

이 설계는 PostgreSQL Job row와 Outbox Event를 정본으로 유지하면서 Redis를 유실 가능한 wake-up
가속기로만 사용하는 범용 비동기 실행 경계를 정의한다. 첫 실제 handler는 Outbox Event를 권한 descriptor가
포함된 bounded Workspace Stream ledger로 투영한다. API는 이 ledger를 현재 권한으로 다시 검사한 뒤 SSE로
전달한다.

Job runner는 AI provider, OpenSearch와 mail provider의 업무 로직을 소유하지 않는다. 이후 package는 closed
`JobKind` handler를 추가해 같은 claim·lease·retry·cancel 계약을 사용한다. SSE는 변경 알림이며 query state를
대체하지 않는다.

## 2. 확정 결정

### 2.1 Outbox delivery 자체를 첫 범용 Job handler로 사용한다

검토한 대안은 사용처 없는 추상 Job runner, Outbox 전용 polling loop, Outbox consumer delivery를 Job으로
표현하는 방식이다. 사용처 없는 runner는 실제 장애 계약을 검증하지 못하고 전용 loop는 retry·lease를 중복
구현한다. 따라서 domain transaction이 Outbox Event와 `OUTBOX_TO_STREAM` Job을 함께 생성한다. Job payload는
`outboxEventId`만 포함하고 `kind + dedupeKey` unique로 한 delivery만 의미 있게 실행한다.

### 2.2 Redis는 wake-up ID만 보관하고 reconcile이 복구한다

Redis list에는 Job payload가 아니라 Job ID만 넣는다. 세 priority bucket은 `interactive`, `normal`,
`background`이고 PostgreSQL priority를 bucket으로 정규화한다. worker reconciler는 due `QUEUED` row를 bounded
batch로 찾아 Redis에 signal한다. duplicate signal은 허용하며 claim conditional update가 한 worker만 RUNNING으로
만든다. Redis flush·재시작·signal 누락은 다음 reconcile에서 복구한다.

### 2.3 Outbox producer가 audience를 transaction에서 확정한다

consumer가 payload key나 삭제된 aggregate를 추측해 권한을 정하지 않는다. Outbox Event는 다음 audience 중
하나를 필수로 가진다.

| kind | 식별자·조건 | 전달 대상 |
|---|---|---|
| `INTERNAL` | 없음 | browser stream 미투영 |
| `WORKSPACE` | 없음 | 현재 active Member |
| `ADMIN` | 없음 | 현재 Admin·Owner |
| `USER` | user ID | 동일 active user |
| `DOCUMENT` | document ID + minimum access | 현재 Effective Permission 충족 user |

문서가 이미 purge되어 current permission을 계산할 수 없으면 event를 전달하지 않고 client query reconciliation이
삭제 상태를 확정한다. audience는 browser payload에 포함하지 않는다. 기존 migration row는 안전한 기본값
`INTERNAL`로 backfill해 과거 event가 넓게 노출되지 않게 한다.

### 2.4 SSE replay는 PostgreSQL Stream ledger, 저지연 wake는 process-local hub다

Redis Pub/Sub 한 연결을 API instance마다 유지하고 process-local bounded broadcast hub로 fan-out한다. worker는
Stream row commit 뒤 `{workspaceId,sequence}`만 publish한다. Pub/Sub 유실은 정합성에 영향을 주지 않으며 SSE
connection은 heartbeat마다 PostgreSQL을 다시 조회한다. connection별 broadcast lag는 연결을 종료해 client가
cursor로 재접속하게 한다.

## 3. 저장 계약

### 3.1 Job

`jobs`는 `id`, optional `workspace_id`, closed `kind`, bounded `payload_json`, `dedupe_key`, `status`,
`priority`, `sequence`, `attempt/max_attempts`, `run_after`, lease, cancellation, error, correlation과 terminal time을
가진다. `kind+dedupe_key`는 unique다. payload에는 credential·본문·prompt를 넣지 않는다.

```text
QUEUED
  ├─ claim → RUNNING
  └─ cancel request → CANCEL_REQUESTED → CANCELLED
RUNNING
  ├─ success → SUCCEEDED
  ├─ transient failure → QUEUED(run_after, sequence+1)
  ├─ permanent failure → FAILED
  ├─ attempts exhausted → DEAD_LETTER
  ├─ deadline → TIMED_OUT
  └─ cancel request → CANCEL_REQUESTED → CANCELLED
```

claim은 `FOR UPDATE SKIP LOCKED`와 conditional update를 사용한다. attempt는 claim 때 증가한다. expired RUNNING
lease는 transient recovery로 QUEUED에 되돌리고 sequence를 증가시킨다. terminal row는 다시 active가 되지
않는다. operator replay는 새 Job identity와 `replay_of_job_id`를 만든다.

### 3.2 Outbox와 receipt

Outbox Event는 기존 aggregate ordering key에 audience descriptor와 correlation ID를 추가한다. producer는 domain
state·Audit·Outbox·stream delivery Job을 같은 transaction에 commit한다. `consumer_receipts(consumer,event_id)`가
handler side effect의 한 번 의미를 보장한다. handler는 receipt가 있으면 성공 replay로 종료한다.

### 3.3 Workspace Stream ledger

`workspace_stream_events`는 Workspace gapless sequence, 원 Outbox ID unique, normalized event type/version,
payload, audience snapshot, occurred/created/expires time을 가진 append-only row다. sequence는
`workspace_sequences.next_stream_sequence`를 row lock으로 할당한다. active replay window는 24시간이며 정리
worker가 만료 row를 삭제한다. 삭제는 stream retention credential이 아니라 일반 worker가 정책에 따라 수행할 수
있고, domain history나 Audit 삭제와 연결하지 않는다.

## 4. Job 실행 protocol

1. reconciler가 expired lease를 회수하고 due Job ID를 priority Redis list에 signal한다.
2. executor가 priority 순서로 bounded ID batch를 가져온다.
3. repository가 exact ID·due status를 conditional claim하고 lease owner/time과 attempt를 기록한다.
4. registry가 closed `JobKind` handler를 선택한다. 알 수 없는 kind는 retry하지 않는 FAILED다.
5. handler는 cancellation을 시작 전과 외부 side effect 경계마다 확인한다.
6. success·retry·failure transition은 current lease owner와 sequence를 조건으로 commit한다.
7. transition 뒤 남은 active Job이면 새 wake signal을 보낼 수 있으나 실패해도 DB row는 유지한다.

backoff는 `min(5s * 2^(attempt-1), 5m)`에 Job ID 기반 deterministic jitter를 더한다. validation, unknown kind,
authorization과 malformed payload는 permanent다. dependency unavailable과 transaction serialization은
transient다.

## 5. Outbox→Stream handler

handler transaction은 Outbox row를 lock하고 다음 순서를 수행한다.

1. `workspace-stream` receipt 존재 시 성공 반환
2. `INTERNAL` audience면 receipt 기록 후 종료
3. event type·version과 payload를 generated Event contract로 검증
4. Workspace sequence 할당과 Stream row insert
5. receipt 기록과 Outbox `published_at` 갱신
6. Job success transition

Stream insert와 receipt는 같은 transaction이므로 crash 뒤 duplicate row가 생기지 않는다. contract가 잘못된
producer event는 `EVENT_CONTRACT_INVALID` permanent failure로 보내고 원 payload를 log하지 않는다.

## 6. Cursor·SSE protocol

cursor는 versioned opaque base64url `{version,workspaceId,sequence,eventId}`다. 다른 Workspace cursor, malformed
cursor와 미래 sequence는 request error다. cursor sequence보다 ledger의 minimum sequence가 크면 server는
`STREAM_RESET_REQUIRED` 한 건을 보내고 종료한다. cursor가 없으면 연결 시점의 current high watermark를
시작점으로 삼으며, client는 먼저 Workspace bootstrap query를 수행한다.

SSE frame은 `id`, normalized `event`, generated Event Envelope `data`를 가진다. 15초마다 heartbeat comment를
보낸다. 한 query page는 100건, connection output buffer는 256건이다. DB page 내 event는 sequence 순서로
평가한다. 전달하지 않는 event도 connection cursor를 전진시켜 권한 없는 row 때문에 재조회 loop가 생기지 않게
한다.

## 7. 권한 재검사

connection 시작 시 session-selected Workspace와 active Membership을 확인한다. 각 page는 현재 Membership
revision을 읽고 바뀌면 `STREAM_RESET_REQUIRED` 후 종료한다. `ADMIN`, `USER`, `DOCUMENT` audience는 event
시점 snapshot이 아니라 현재 상태로 검사한다. DOCUMENT는 동일 Permission Resolver의 point 계산을 사용하며
`VIEWER < CONTRIBUTOR < EDITOR` minimum을 적용한다. 권한 확인 실패는 event를 숨기고 존재를 드러내지 않는다.

Workspace가 deletion scheduled면 active Member stream은 유지하지만 PURGING·DELETED로 전이되면 연결을
종료한다. Public Viewer와 anonymous request에는 SSE를 제공하지 않는다.

## 8. Port·module 배치

- `operations`: Job·Outbox delivery·Stream domain type와 transition 결과
- `application::jobs`: runner, handler registry, backoff, cancellation orchestration
- `application::stream`: cursor·page·audience query service
- `adapters::postgres`: JobRepository, OutboxStreamHandler, StreamRepository
- `adapters::redis`: JobSignalQueue와 StreamWakePublisher/Subscriber
- `worker`: reconcile·execute·stream cleanup loop
- `api`: single Redis subscriber hub와 `/api/v1/stream` Axum SSE route

Redis와 PostgreSQL adapter error 원문은 browser·log에 노출하지 않는다. shutdown은 새 claim을 멈추고 현재
handler에 bounded grace를 준 뒤 lease expiry 복구에 맡긴다.

## 9. 관측성과 제한

metric은 queue depth/oldest age, claim/retry/dead-letter, expired lease recovery, outbox lag, receipt duplicate,
stream ledger lag, active SSE, reset, filtered event와 broadcast lag를 bounded label로 기록한다. trace는
correlation ID→outbox ID→Job ID→attempt→Stream sequence를 연결한다. payload·document ID·user ID는 metric
label이나 log field로 쓰지 않는다.

Job payload는 64KiB, event payload는 64KiB를 넘지 않는다. reconcile batch는 설정된
`outbox_batch_size`, executor concurrency는 DB pool보다 작게 제한한다. Redis 장애 중에도 worker가 일정 주기로
PostgreSQL에서 직접 due Job을 claim해 무한 정지를 막는다.

## 10. 검증 gate

- 동일 aggregate event ordering과 consumer receipt duplicate 억제
- concurrent worker에서 exact-once claim, lease expiry recovery와 stale owner completion 거부
- Redis flush·duplicate signal·disconnect 뒤 PostgreSQL reconcile
- queued/running cancel, transient retry backoff, attempts exhausted dead-letter
- audience별 current permission filter와 cross-Workspace cursor 거부
- disconnect resume, expired cursor reset, heartbeat와 slow consumer termination
- malformed event permanent failure 시 payload 비노출
- `bun run contracts:check`, migration seal/check, `bun run check`, `bun run compose:integration`
