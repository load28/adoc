# TASK-011: PostgreSQL 영속성 기반 구현

- **상태**: 완료
- **유형**: 구현
- **구현 패키지**: IMP-04
- **시작일**: 2026-08-25
- **완료일**: 2026-08-25
- **커밋**: —

## 목적

PostgreSQL 16을 도메인 정본으로 사용하는 공통 영속성 기반을 만든다. 정본 DDL과 실행
마이그레이션의 불일치를 자동으로 차단하고, 이후 모든 command가 같은 transaction에서 상태·
outbox·멱등성 결과를 원자적으로 기록할 수 있는 SQLx 경계를 제공한다.

## 범위

- 포함: 정본 DDL 기반 migration, migration drift 검사, SQLx pool, transaction runner,
  idempotency reservation·replay·completion, aggregate outbox sequence·append, 실제 PostgreSQL 통합 테스트
- 제외: 개별 도메인 repository, command handler, outbox publisher·consumer, Redis wake-up, Docker Compose
  전체 harness, production migration orchestration과 backup

## 필수 설계 문서

- [x] `product/NON-FUNCTIONAL-REQUIREMENTS.md`
- [x] `design/adr/ADR-002-postgresql-sqlx.md`
- [x] `design/architecture/MODULE-ARCHITECTURE.md`
- [x] `design/architecture/TRANSACTION-EVENT-JOB.md`
- [x] `design/data/LOGICAL-SCHEMA.md`
- [x] `design/data/schema.sql`
- [x] `design/data/DATABASE-INVARIANT-TRANSACTION-MATRIX.md`
- [x] `design/data/MIGRATION-STRATEGY.md`
- [x] `design/implementation/MODULE-INTERFACE-CATALOG.md`
- [x] `design/implementation/POSTGRESQL-FOUNDATION.md`
- [x] `design/implementation/REPOSITORY-STRUCTURE.md`
- [x] `design/implementation/WORK-BREAKDOWN.md`
- [x] `design/quality/TEST-STRATEGY.md`
- [x] `design/quality/CONCURRENCY-RECOVERY-TESTS.md`
- [x] UX·HTTP API·사용자 권한: N/A — transport와 제품 권한 판정 전의 infrastructure 기반이다.

## 문서 준비 게이트

- [x] PostgreSQL 16과 SQLx, migration·SQL owner가 동결 설계에 정의됐다.
- [x] transaction의 lock·검증·state/audit/outbox 원자성 및 외부 I/O 금지가 정의됐다.
- [x] idempotency key의 request hash 재사용과 완료 response replay 계약이 정의됐다.
- [x] outbox aggregate sequence uniqueness와 consumer receipt 계약이 DDL에 정의됐다.
- [x] forward-only expand→migrate→contract와 실제 PostgreSQL 검증 조건이 정의됐다.
- [x] 구현 결과를 바꿀 미해결 질문이 없다.

## 사용자 결정

사용자는 PostgreSQL, Rust 단일 backend, 구조적 해결 원칙을 확정했다. 2026-08-25 이후에는
권장안을 별도 승인 없이 선택하라고 지시했다.

## 의사결정

### 결정 1: `schema.sql`에서 baseline migration을 결정적으로 생성한다

- **상황**: 정본 DDL과 실행 migration을 수동으로 이중 관리하면 구조가 필연적으로 어긋난다.
- **검토한 대안**: 수동 복사 / migration을 새 정본으로 변경 / 정본에서 결정적으로 생성하고 drift 검사.
- **선택과 근거**: `schema.sql`을 정본으로 유지하고 generator가 SQLx baseline을 만든다. CI는 재생성 diff가
  있으면 실패하므로 한 정책을 두 파일에서 수정할 수 없다.

### 결정 2: SQLx 구체 타입은 adapter 안에 가둔다

- **상황**: application과 domain이 `PgPool`·`Transaction`에 의존하면 계층 경계가 무너진다.
- **검토한 대안**: application에 SQLx 노출 / 범용 repository abstraction / adapter 소유 transaction runner.
- **선택과 근거**: connection pool·transaction·공통 persistence primitive를 `adoc-adapters`가 소유한다.
  이후 repository adapter도 같은 crate의 transaction context를 사용하고 domain은 SQLx를 알지 않는다.

### 결정 3: 초기 baseline의 upgrade 검증은 migration 재적용으로 고정한다

- **상황**: 아직 이전 출시 schema가 없어 인위적인 legacy DDL을 만들면 거짓 호환 계약이 된다.
- **검토한 대안**: 가짜 legacy migration / 의미 없는 두 번째 migration / baseline 적용 후 재실행 검증.
- **선택과 근거**: clean database 적용과 동일 database 재실행의 no-op을 검증한다. 실제 두 번째 schema 변경부터
  이전 version→latest upgrade fixture를 의무화한다.

### 결정 4: 멱등성 상태 전이는 transaction 안에서 행 잠금으로 직렬화한다

- **상황**: 같은 key 동시 요청과 commit 결과 불명확 상태에서 중복 mutation을 막아야 한다.
- **검토한 대안**: process mutex / Redis lock / PostgreSQL primary key·row lock.
- **선택과 근거**: 정본과 같은 PostgreSQL transaction에서 reserve하고 request hash를 비교한다. 완료된 결과는
  replay하고 진행 중 key는 명시적 busy 결과를 반환한다.

### 결정 5: SQLx 0.8.6을 Rust 1.90 호환선으로 고정한다

- **상황**: 구현 시점 최신 SQLx 0.9.0은 Rust 1.94 이상을 요구해 동결된 asdf Rust 1.90과
  호환되지 않았다.
- **검토한 대안**: Rust toolchain 선행 변경 / SQLx 0.9 강제 / Rust 1.90을 지원하는 최신 SQLx.
- **선택과 근거**: package 하나 때문에 toolchain 계약을 우회하지 않고 SQLx 0.8.6을 lockfile에
  고정했다. 전체 Clippy·test·build와 PostgreSQL 16 adapter contract로 호환성을 검증했다.

## 구현 순서

1. migration 생성·drift 검사를 상세 설계와 tool에 반영한다.
2. SQLx pool·transaction runner와 안정적인 persistence error를 구현한다.
3. idempotency와 outbox primitive를 같은 transaction context에 구현한다.
4. fresh apply·repeat apply·rollback·replay·hash conflict·sequence conflict를 실제 PostgreSQL에서 검증한다.
5. 전체 repository gate를 통과한 뒤 태스크를 완료하고 commit·push한다.

## 이슈 및 해결

### 이슈 1: SQLx 추가 후 dependency boundary 검사가 output buffer를 초과함

- **증상**: 전체 gate에서 `cargo metadata` 결과가 Node 기본 1 MiB buffer를 넘어 `ENOBUFS`로 종료됐다.
- **조사**: boundary 검사는 Workspace 간 edge만 필요하지만 third-party transitive package 전체를 읽고 있었다.
- **근본 원인**: 검사 입력 범위가 소유한 invariant보다 넓어 dependency 수에 따라 비결정적으로 실패했다.
- **구조적 해결**: boundary 검사는 `cargo metadata --no-deps`의 local path dependency만 사용한다. 전체
  dependency가 필요한 license 검사는 명시적인 16 MiB 상한을 둔다.

### 이슈 2: SPDX `AND` 표현식을 단일 license 문자열로 오판함

- **증상**: 허용된 Apache-2.0과 ISC를 함께 요구하는 `ring`을 license gate가 거부했다.
- **조사**: 기존 검사는 `OR`만 분리하고 `AND`·괄호 우선순위를 해석하지 않았다.
- **근본 원인**: SPDX expression을 문자열 목록으로 축약해 복합 조건의 의미를 잃었다.
- **구조적 해결**: `AND`·`OR`·`WITH`·괄호를 평가하는 parser와 precedence regression test를 추가했다.

### 이슈 3: 격리 환경에서 registry·Docker socket·localhost 연결이 차단됨

- **증상**: dependency download, Docker daemon 조회와 로컬 PostgreSQL 연결이 sandbox 권한으로 실패했다.
- **조사**: 같은 lockfile·container·test가 승인된 실행 경계에서는 통과함을 각각 확인했다.
- **근본 원인**: source 결함이 아니라 실행 환경의 network·Unix socket 정책이었다.
- **구조적 해결**: 허용된 경계에서 dependency를 고정 다운로드하고 명시 이름의 임시 PostgreSQL 16
  container로 통합 gate를 실행한 뒤 container를 제거했다. 이후 전체 gate는 offline으로 재실행했다.

## 검증

- [x] canonical schema와 generated migration byte-level drift 0
- [x] PostgreSQL 16 clean database에 migration 전체 적용
- [x] 적용된 database에 migration 재실행 시 no-op
- [x] transaction commit·rollback과 SQLSTATE 분류
- [x] idempotency reserve·same-request replay·different-hash conflict·busy 처리
- [x] outbox append와 aggregate sequence 충돌
- [x] root `bun run check`와 `git diff --check`

## 작업 내역

- 2026-08-25: IMP-04 태스크를 등록하고 정본 설계와 DDL의 migration·transaction·outbox·멱등성
  계약을 확인했다.
- 2026-08-25: PLAN-10에 migration 단일 정본, adapter/UoW 경계, SQLSTATE, pool, 멱등성·Outbox와
  실제 PostgreSQL 검증 계약을 코드보다 먼저 고정했다.
- 2026-08-25: canonical DDL generator와 drift check, embedded SQLx migration, PostgreSQL pool·
  preflight와 panic-safe opaque UnitOfWork를 구현했다.
- 2026-08-25: transaction-scoped 멱등성 reserve·completion·replay·lease takeover와 aggregate sequence
  Outbox append를 구현했다.
- 2026-08-25: PostgreSQL 16 clean schema 41개 table, migration 재실행 no-op, rollback, 멱등성 상태
  전이와 Outbox 충돌을 실제 container에서 검증하고 임시 container를 제거했다.
- 2026-08-25: dependency graph 증가로 드러난 boundary metadata 범위와 SPDX expression 검사를
  구조적으로 수정하고 전체 `bun run check`와 `git diff --check`를 통과했다.

## 결과

IMP-04를 완료했다. 정본 DDL과 실행 migration은 자동 drift gate로 연결됐고, 이후 repository가
opaque transaction 안에서 멱등성·domain write·Outbox를 원자적으로 조합할 수 있다. 다음 package는
이 기반을 실제 service topology와 healthcheck에 연결하는 IMP-05 Docker Compose harness다.
