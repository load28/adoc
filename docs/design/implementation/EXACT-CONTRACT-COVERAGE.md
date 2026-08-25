# Exact Contract Coverage

- **문서 ID**: PLAN-40
- **상태**: 구현 기준
- **관련 태스크**: TASK-043

## 목적

OpenAPI operation, Event schema와 상태 전이 표의 모든 항목에 고유한 test evidence를 연결한다.
도메인 단위 통합 테스트가 존재한다는 이유만으로 내부 operation 누락을 숨기지 않고 source 집합과
evidence 집합의 차이를 CI에서 0으로 만든다.

## Operation case profile

각 operation은 `query`, `command`, `lease-command`, `async-command`, `navigation`, `stream`, `binary`,
`callback` 중 정확히 하나의 profile을 가진다.

- query: success, auth, tenant, permission, validation, pagination 또는 `not-applicable`
- command: query 공통 case와 commit, idempotency replay·conflict, stale, rollback, Audit, Outbox
- lease-command: command case와 expired lease, other holder, token·client mismatch
- async-command: command case와 cancel, deadline, redelivery, terminal-state immutability
- navigation·callback·stream·binary: 해당 protocol 성공, auth/capability, malformed input, disclosure 경계

`not-applicable`은 빈칸이 아니다. operation별 reason code를 manifest에 명시한다. Audit·Outbox가 없는
read operation이나 상태를 만들지 않는 preview처럼 설계상 부작용이 없어야 하는 항목만 허용한다.

## Evidence 경계

각 operation 행은 정확한 test file과 test ID를 가진다. test ID의 `*`, prefix, 범위 표현을 금지한다.
한 parameterized test가 여러 operation을 실행할 수 있지만 실행 결과는 각 operation ID를 별도 case로
report한다. command evidence는 commit state와 같은 transaction의 Audit·Outbox count를 관찰하고 실패
case는 세 count가 모두 변하지 않았음을 확인한다.

PostgreSQL은 test별 UUID namespace와 transaction cleanup을 사용한다. Redis key는 Workspace UUID
namespace를 사용하며 OpenSearch index는 run UUID suffix를 사용한다. Compose test runner만 실제
dependency evidence를 만들 수 있고 로컬 unit skip은 완료 증거로 계산하지 않는다.

## Event coverage

`event-payloads.schema.json`의 닫힌 type enum이 Event ID 정본이다. 모든 ID는 다음 세 증거를 가진다.

1. producer: domain commit과 같은 transaction에서 canonical outbox row를 생성
2. consumer: payload validation, audience authorization과 idempotent delivery
3. negative: unknown type, payload mismatch, sequence gap 또는 denied audience 거부

내부 전용 Event도 producer·consumer evidence가 필요하지만 Browser stream evidence는 요구하지 않는다.
Event Catalog는 schema enum과 동일한 23개 행을 유지한다.

## State transition coverage

SPEC-17 표의 각 행은 `aggregate|from|trigger|to` 네 값을 NFC 정규화해 exact ID를 만든다. `/`, `·`,
범위 표현을 전개한 각각의 상태도 evidence manifest에서 독립 ID를 가진다. positive evidence는 guard를
만족한 전이를, negative evidence는 guard 실패와 stale revision의 side-effect zero를 검증한다.

## 검사와 완료 조건

검사기는 OpenAPI 109개, Event schema 23개, SPEC-17의 모든 전이 ID와 manifest를 대조한다. profile별
필수 case, test file·test ID, duplicate·wildcard·orphan을 검증하고 negative self-test로 각 실패를
재현한다. root gate와 실제 Compose integration이 모두 통과해야 완료한다.
