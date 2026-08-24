# Container와 배포 구조

- **문서 ID**: ARCH-02
- **상태**: 동결

## 실행 unit

| Container | 책임 | 상태 |
|---|---|---|
| web | TanStack Start SSR·CSR, static asset, BFF-less browser client | stateless |
| api | Axum HTTP command/query, SSE, auth·authorization | stateless |
| worker | outbox, index, file, notification, AI orchestration | stateless |
| ai-runner | isolated CLI/API invocation adapter | ephemeral/worker |
| postgres | domain 정본·outbox·audit | persistent |
| redis | queue reservation·rate limit·ephemeral cursor | reconstructable |
| opensearch | permission-aware search projection | reconstructable |
| object-storage | local File bytes, later S3 adapter | persistent |

## Docker Compose

Compose는 local과 single-host self-hosted의 executable reference다. healthcheck, named volume,
secret file, migration one-shot job과 backup profile을 포함한다. web/API/worker image는 동일
commit SHA와 contract version label을 가진다.

## 수평 확장

API와 web은 shared session store 없이 signed opaque session ID로 PostgreSQL session을
조회하고 local sticky state를 두지 않는다. SSE reconnect는 cursor를 사용한다. Worker는
Redis queue claim과 PostgreSQL idempotency로 중복 실행을 허용하되 결과를 한 번만 commit한다.

## Failure isolation

OpenSearch·AI·preview가 unhealthy여도 Document query·save·publish가 배포 health에서 제외되지
않는다. dependency별 readiness를 기능 flag로 노출하고 core liveness와 분리한다.

## Provider 중립

특정 Cloud·Kubernetes manifest는 만들지 않는다. image, config, health, graceful shutdown과
volume contract는 container orchestrator가 교체돼도 유지한다.
