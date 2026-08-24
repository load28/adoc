# Test Strategy

- **문서 ID**: TEST-01
- **상태**: 동결

## 계층

| 계층 | 검증 |
|---|---|
| Domain unit | invariant, state transition, resolver·merge property |
| Persistence integration | PostgreSQL constraint, transaction, SQLx query |
| Adapter contract | ObjectStorage, AI Runtime, OpenSearch, Redis 동일 port suite |
| API contract | OpenAPI request·problem·idempotency·revision |
| Browser component | Tiptap schema, Atlaskit interaction, a11y |
| End-to-end | 전체 사용자 여정, multi-session, dependency failure |

## Test data

builder는 Workspace마다 isolated ID namespace를 만든다. golden content는 schema version을
명시하고 production 원문을 복사하지 않는다. clock, ID, queue와 provider는 controllable port다.

## 필수 property test

Permission point/scope 동등성, tree acyclic, Operation apply/inverse, Diff stability, retry
idempotency, event sequence monotonic과 retention reference safety를 생성형 입력으로 검증한다.

## 환경

unit 외 integration은 실제 PostgreSQL·Redis·OpenSearch·local ObjectStorage container를 사용한다.
mock으로 SQL·search permission 의미를 대체하지 않는다.

## Gate

format·lint → unit → schema contract → integration → browser/a11y → E2E → security·performance
순이다. flaky retry로 통과시키지 않고 quarantine은 owner·기한·issue를 요구한다.
