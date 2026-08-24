# PostgreSQL Foundation Implementation Contract

- **문서 ID**: PLAN-10
- **상태**: 구현 기준
- **관련 package**: IMP-04
- **정본 DDL**: [schema.sql](../data/schema.sql)

## 책임과 경계

PostgreSQL 16은 domain state·Audit·Job·Outbox·멱등성의 정본이다. `adoc-adapters::postgres`가
SQLx pool, transaction handle, migration runner와 공통 persistence primitive를 소유한다.
domain·application crate는 SQLx type이나 SQLSTATE를 import하지 않는다. application은
`adoc-ports`의 opaque `Transaction`과 `UnitOfWork`만 사용하고 repository adapter만 실제
connection을 해석한다.

handler와 domain service가 SQL을 직접 실행하는 것을 금지한다. transaction 안에서는 PostgreSQL
query만 허용하며 Redis, OpenSearch, ObjectStorage와 provider 호출은 commit 이후 outbox·job으로
수행한다.

## Migration 계약

`docs/design/data/schema.sql`은 최신 fresh-install 구조를 소유한다. 최초 baseline은 다음 순서의 pure
transform으로 생성했으며, 한번 commit된 migration은 다시 생성하거나 수정하지 않는다.

1. UTF-8·LF와 마지막 newline을 확인한다.
2. 첫 실행문 `BEGIN;`과 마지막 실행문 `COMMIT;`을 제거한다.
3. 수동 편집 금지와 정본 경로를 나타내는 고정 header를 붙인다.
4. 그 밖의 byte는 순서와 공백을 포함해 보존한다.

baseline 이후 migration은 `NNNN_<slug>.sql` forward-only file로 작성하고 최신 canonical schema에
같은 결과를 반영한다. `migrations:seal`은 모든 migration의 filename·SHA-256을 정렬된 manifest에
기록한다. `migrations:check`는 연속 version, filename pattern, manifest 집합과 byte checksum을
검사하고 수정·삭제·끼워넣기를 거부한다. 새 migration은 마지막 version에만 추가할 수 있다.
SQLx migrator도 `_sqlx_migrations` checksum을 독립적으로 검사하며 불일치를 자동 보정하지 않는다.

초기 baseline은 clean PostgreSQL 16 database에 전체 적용한다. 이어 같은 migrator를 다시
실행하고 적용 version·checksum이 변하지 않는지 확인한다. 실제 후속 migration부터는 직전
version fixture를 먼저 만든 뒤 latest로 upgrade하고, current application read/write compatibility
query를 실행한다.

## Pool·Transaction 계약

pool input은 secret URL 원문과 `ADOC_DB_MAX_CONNECTIONS`의 검증된 값이다. URL은 log·Debug·error에
포함하지 않는다. pool은 acquire timeout 5초, idle timeout 10분, max lifetime 30분을 사용하고
연결마다 UTC timezone과 5초 `lock_timeout`, 30초 `statement_timeout`, application name을 설정한다.
startup preflight는 `SELECT 1`, server major version 16 이상, pending migration 0을 확인한다.

`UnitOfWork::execute`는 owned pool connection에서 `BEGIN`하고 opaque transaction을 operation에
전달한다. operation 성공은 `COMMIT`, 실패는 `ROLLBACK`한다. panic을 성공으로 바꾸거나 자동
fallback하지 않는다. commit 결과가 불명확하면 성공으로 추정하지 않고 caller가 같은 멱등성
key로 결과를 조회한다.

SQLSTATE는 원문 message 없이 안정 범주로 바꾼다.

| SQLSTATE·상황 | 범주 | 재시도 |
|---|---|---|
| `40001` | serialization | 같은 command identity로 최대 3회 |
| `40P01` | deadlock | 같은 command identity로 최대 3회 |
| `23...` | constraint | 금지 |
| acquire timeout·connection closed | unavailable | command 정책에 따름 |
| migration checksum·unknown | internal | 금지 |

공통 UoW는 retry loop를 숨기지 않는다. expected revision·권한·validation error를 식별해야 하는
application command runner가 transient SQLSTATE만 동일 command identity로 재실행한다.

## 멱등성·Outbox 계약

멱등성 identity는 `(workspace_id, actor_id, operation_id, key)`다. request body의 canonical bytes를
SHA-256 lowercase hex로 전달한다. reserve는 transaction 시작 직후 수행한다.

- 신규 key: lease·expiry와 함께 insert하고 `Acquired`를 반환한다.
- 같은 hash·완료 response: 저장된 status·JSON을 `Replay`한다.
- 같은 hash·유효 lease·미완료: `Busy`를 반환하며 mutation을 수행하지 않는다.
- 같은 hash·만료 lease: row lock 아래 lease를 인계하고 `Acquired`를 반환한다.
- 다른 hash·미만료 record: `KeyReused` conflict다.
- retention expiry가 지난 record: row lock 아래 새 hash·lease·expiry로 원자 교체할 수 있다.

completion은 같은 transaction에서 response status·JSON을 한 번만 기록한다. 완료되지 않은
멱등성 row와 domain mutation이 서로 다른 commit에 존재해서는 안 된다.

Outbox append input은 UUIDv7 event ID, Workspace, aggregate kind·ID, aggregate lock 아래 계산한 다음
sequence, versioned event type·payload, server UTC instant다. `(aggregate_kind, aggregate_id, sequence)`
충돌은 `EVENT_SEQUENCE_CONFLICT`의 persistence 범주로 변환한다. publisher claim·`published_at`과
consumer receipt는 IMP-17이 소유한다.

## 검증 계약

unit test는 migration transform·error mapping·입력 불변식을 검증한다. persistence integration은
mock DB가 아니라 PostgreSQL 16에서 다음 barrier를 검증한다.

1. clean apply와 second apply no-op
2. operation error 뒤 inserted row 0, success 뒤 commit row 1
3. 같은 멱등성 요청의 acquired→completed→replay
4. 다른 hash conflict와 유효 lease busy, 만료 lease takeover
5. outbox append payload 보존과 같은 aggregate sequence 충돌

통합 test는 격리된 database를 사용하고 종료 시 폐기한다. sleep으로 race를 통과시키지 않으며
후속 동시성 package는 barrier로 interleaving을 고정한다.
