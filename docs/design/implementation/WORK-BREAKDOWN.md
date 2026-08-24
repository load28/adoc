# Implementation Work Breakdown

- **문서 ID**: PLAN-08
- **상태**: 동결
- **출시 단위**: 전체 제품 한 번

아래 package는 구현 dependency DAG다. 각 package는 별도 태스크 문서를 만들고 필수 설계 문서를
읽은 뒤 시작한다. 부분 package 완료를 MVP나 제품 출시로 선언하지 않는다.

## Foundation DAG

| ID | Package | 선행 | 산출물 | 완료 gate |
|---|---|---|---|---|
| IMP-01 | Monorepo·toolchain | 설계 freeze | asdf-pinned Cargo/Bun workspace, dependency rules | clean bootstrap·forbidden edge test |
| IMP-02 | Contract generation | IMP-01 | JSON Schema/OpenAPI/AsyncAPI Rust·TS type | corpus 양언어 동일 판정 |
| IMP-03 | Configuration·telemetry | IMP-01 | typed config, tracing, metrics, redaction | config negative corpus |
| IMP-04 | PostgreSQL foundation | IMP-01~03 | migration, SQLx pool, UoW, outbox/idempotency | schema.sql fresh+upgrade apply |
| IMP-05 | Docker Compose harness | IMP-01~04 | all service, health, secret, volume | clean up/down, backup profile |

## Domain DAG

| ID | Package | 선행 | 산출물 | 완료 gate |
|---|---|---|---|---|
| IMP-06 | Identity·session | IMP-02~05 | Google OIDC, session, preferences | auth threat/contract tests |
| IMP-07 | Workspace·membership·group | IMP-06 | governance aggregates/API | last Owner·tenant tests |
| IMP-08 | Permission·policy | IMP-07 | point/scope resolver, cache invalidation | property equivalence·matrix |
| IMP-09 | Content·Operation reducer | IMP-02 | Rust/TS schema, reducer, inverse | shared fixture/property tests |
| IMP-10 | Document tree·Draft·Lease | IMP-04,08,09 | tree, autosave command, lease | barrier race tests |
| IMP-11 | Publish·Version·Public link | IMP-10 | review-independent publish core, history | immutable/base conflict tests |
| IMP-12 | Discussion·Message·Inbox | IMP-08,10 | collaboration API/projection | history·dedupe tests |
| IMP-13 | Review | IMP-11,12 | revision-bound approval gate | threshold/edit race tests |
| IMP-14 | Reference·Vocabulary | IMP-09,12 | knowledge graph/rules | target permission·term unique |
| IMP-15 | File·ObjectStorage | IMP-05,09,10 | local/S3 port, reference, range | adapter suite·GC race |
| IMP-16 | Audit·Retention | IMP-04,07,11,15 | audit sequence, trash/workspace purge | append-only·ledger·restore |

## Search·AI DAG

| ID | Package | 선행 | 산출물 | 완료 gate |
|---|---|---|---|---|
| IMP-17 | Job runtime·SSE | IMP-04,05,07 | PostgreSQL job, Redis wake, resumable SSE | loss/replay/cancel tests |
| IMP-18 | Search projection | IMP-08,11,14,17 | OpenSearch mapping, indexer, rebuild | prefilter·ordering canary |
| IMP-19 | Hybrid retrieval·Source | IMP-18 | RRF, Source provenance | relevance+non-leak suite |
| IMP-20 | AI Context·runtime adapters | IMP-14,17,19 | Context Builder, CLI/API adapter | same port suite, source coverage |
| IMP-21 | AI Result·Proposal·rules | IMP-09,13,20 | validation, Diff, apply/Undo | stale/dependency/hard rule tests |

## Web·release DAG

| ID | Package | 선행 | 산출물 | 완료 gate |
|---|---|---|---|---|
| IMP-22 | TanStack shell·Atlaskit | IMP-02,06~08 | SSR shell, theme, routes, i18n | hydration·license·a11y |
| IMP-23 | Document Editor UX | IMP-09~11,15,22 | Tiptap, operation buffer, recovery | IME·keymap·multi-session |
| IMP-24 | Collaboration·Knowledge UX | IMP-12~14,18,22 | panels, Inbox, Search, Vocabulary | screen behavior contract |
| IMP-25 | AI UX | IMP-17,20,21,22 | Inspector, progress, Proposal Diff | no-direct-apply·Source views |
| IMP-26 | Settings·Audit·Public UX | IMP-07,08,16,22 | settings, trash, viewer | public scope·responsive matrix |
| IMP-27 | System hardening | IMP-01~26 | security, performance, DR, observability | NFR SLO/RPO/RTO evidence |
| IMP-28 | Full acceptance release | IMP-27 | one versioned artifact | TEST-09 전체+release runbook |

## 태스크 생성 규칙

구현 시작 시 IMP ID 하나를 하나의 task 문서로 등록한다. package가 너무 크면 내부 checklist를
나누되 설계·코드 책임을 여러 독립 태스크로 쪼개 계약 소유권을 흐리지 않는다. 각 태스크는
관련 PRD requirement, domain invariant, API operation, DDL table, screen, test ID를 명시한다.
새 제품 결정이 필요하면 구현을 중지하고 설계 변경 task를 먼저 완료한다.
