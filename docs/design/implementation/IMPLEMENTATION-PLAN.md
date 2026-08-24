# 전체 구현 계획

- **문서 ID**: PLAN-02
- **상태**: 동결
- **출시 원칙**: 아래 순서는 dependency DAG이며 MVP·부분 출시가 아니다.

## W-01 Foundation

monorepo, typed config, Docker Compose, PostgreSQL migration, telemetry, OpenAPI·AsyncAPI generation,
CI gate와 test container harness를 구축한다.

## W-02 Identity·Governance

Google OIDC·session → Workspace·Invitation·Group → Permission Resolver point/scope equivalence →
PublishPolicy → public link capability를 구현한다.

## W-03 Document core

Tree → Content schema·migration → Region·Operation·Diff → Draft·Lease·autosave recovery → immutable
Publish·History·restore·conflict를 구현한다.

## W-04 Collaboration

Discussion·Topic·Message·Reference → Mention·Inbox projection → Review revision·policy gate → SSE
cursor delivery를 구현한다.

## W-05 Knowledge

Vocabulary·Reference → outbox OpenSearch projection → Permission-safe hybrid Search → Source
provenance와 rebuild tooling을 구현한다.

## W-06 Writing Intelligence

Task registry·Context Builder → Job admission·Redis → Codex CLI/OpenAI adapters → structured Result·
Proposal·Diff·Undo → Writing Rules·evaluation을 구현한다.

## W-07 File·Audit·Retention

upload·validation·preview·authorized delivery → reference lifecycle·GC → Audit → trash·Workspace
purge와 backup deletion ledger를 구현한다.

## W-08 Web UX

TanStack Start shell, ko/en, Atlaskit theme → responsive tree/editor/panels → all common/error states →
public viewer → accessibility와 visual regression을 각 domain vertical과 병행 통합한다.

## W-09 Operations·hardening

SLO dashboard, backup·restore, migration·release/incident/support runbook, security·load·DR test를
완료한다.

## 통합 gate

각 W는 관련 spec의 unit·integration을 통과해야 다음 consumer를 unblock한다. 전체 완료는
W-01~09와 TEST-02 A-01~08이 한 artifact에서 모두 통과할 때만 선언한다.
