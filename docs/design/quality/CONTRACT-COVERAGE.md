# Contract Coverage

- **문서 ID**: TEST-08
- **상태**: 동결

## Artifact gate

| Contract | Positive corpus | Negative corpus | 실행 gate |
|---|---|---|---|
| Content Schema | C-EMPTY, C-FULL, C-KO | depth, duplicate ID, unknown attr, unsafe URL | Rust·TypeScript validator 동일 판정 |
| Operation Schema | 9 kind×각 Region | missing precondition, dependency cycle, stale hash | reducer property+API 422/409 |
| AI Contracts | 6 Task kind, 4 Result status | ungrounded claim, invalid Source, unsafe Operation | adapter contract |
| Event Schema | 23 type별 payload | type/payload mismatch, old/out-of-order sequence | producer·consumer contract |
| OpenAPI | 정본의 모든 `operationId` | bad auth, field, revision, idempotency | request/response snapshot |
| PostgreSQL DDL | 모든 table·constraint·trigger | tenant mismatch, immutable update, duplicate key | PostgreSQL 16 integration |
| OpenSearch mapping | Published·Draft projection | dynamic field, wrong dimension, stale fingerprint | real OpenSearch contract |

## Operation test template

Catalog의 모든 Query는 `success`, `AUTH_REQUIRED`, cross-tenant not-found, insufficient permission과
pagination/filter 경계를 실행한다. 모든 Command는 여기에 valid commit, field validation,
duplicate idempotency same body, reused key different body, stale revision과 transaction rollback을
추가한다. Audit·Outbox는 Event Catalog와 Audit action registry가 선언한 exact count를 검증하며
부작용이 없는 Command는 0개임을 검증한다. Lease command는 expired·other holder, asynchronous
command는 cancel·timeout·redelivery도 추가한다.

## Requirement coverage

| Requirement group | Unit/property | Integration/contract | E2E scenario |
|---|---|---|---|
| Workspace·Governance | permission resolver, last Owner | composite FK, grant trigger, 22 governance API | `workspace governance and precedence` |
| Document·Editor | tree acyclic, reducer/inverse | Draft/Lease/Version transaction | `edit review publish immutable version` |
| Collaboration | Review threshold, mention target | Message history, Inbox dedupe | `discussion review and inbox` |
| Knowledge | RRF/dedupe, reference validity | pre-filtered OpenSearch projection | `search never leaks denied source` |
| AI | context budget, dependency closure | provider/result/schema/cancel | `proposal requires human approval` |
| File·Public | reference set/GC | ObjectStorage range/checksum | `public viewer exact published scope` |
| Retention·Operations | purge state property | ledger/outbox/rebuild/restore | `trash restore and permanent purge` |

## Traceability assertion

CI는 다음 집합 차이를 0으로 만든다: PRD requirement IDs−traceability rows, Catalog operation IDs−
OpenAPI IDs, OpenAPI IDs−API contract test IDs, Event enum−producer tests−consumer tests, state
transition IDs−domain tests, Screen action IDs−Operation/local action IDs. wildcard test ID로 여러
operation을 덮었다고 간주하지 않고 parameterized case가 각 exact ID를 report해야 한다.

## Failure evidence

gate output은 seed, fixture ID, operation ID, contract version, correlation ID와 dependency version을
남긴다. Restricted content, prompt, token과 file bytes는 남기지 않는다. flaky rerun으로 성공을
대체하지 않으며 같은 seed 재현이 불가능한 실패는 test infrastructure 결함으로 처리한다.
