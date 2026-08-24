# Container Runtime Implementation Contract

- **문서 ID**: PLAN-11
- **상태**: 구현 기준
- **관련 package**: IMP-05

## Artifact와 공급망

Rust builder는 asdf와 같은 Rust 1.90으로 `adoc-api`·`adoc-worker`를 `--locked --release` build한다.
Web builder는 Bun 1.3.13과 frozen lockfile로 TanStack Start artifact를 만든다. Rust runtime stage는
compiler·package manager·source를 포함하지 않고 UID/GID `10001`로 실행한다. Web은 같은 Bun 실행
파일이 runtime이자 package command이므로 source·dependency tree를 제외한 공식 Bun runtime image의
`bun` 사용자로 실행한다. image에는 release SHA·contract version OCI label을 기록한다.

PostgreSQL은 `16.15-bookworm`, Redis는 `8.2.9-bookworm`, OpenSearch는 검증된 3.x patch를 exact
tag로 고정한다. floating `latest`·major-only tag를 금지한다. Redis는 별도 network service이며
application binary에 결합하지 않는다. image 갱신은 security·license·backup/restore 검증을 같은
태스크에서 수행한다.

## Compose service와 profile

기본 `core` 실행은 `postgres → migrate → api·worker`, `redis`, `web`이다. local object storage는
별도 HTTP service가 아니라 API·worker에 동일 경로로 mount되는 `object_data` named volume이다.
service dependency는 container 시작이 아니라 health 또는 successful completion 조건을 사용한다.
`volume-init`은 root 소유로 생성되는 `object_data`를 UID/GID `10001`에 넘기는 단일 목적의 one-shot이다.
애플리케이션에 root 권한을 주지 않으며 API·worker는 초기화 성공 뒤에만 시작한다.

| Profile | Service | 정본 여부 | 실패 영향 |
|---|---|---|---|
| default/core | postgres, migrate, redis, api, worker, web | PostgreSQL·object_data만 정본 | core readiness 실패 |
| search | opensearch | 재구축 가능 | Search·AI retrieval만 degraded |
| ai-local | ai-runner | ephemeral | AI 기능만 degraded |
| observability | otel-collector | telemetry sink | 제품 core는 유지 |
| backup | backup | backup_data staging | one-shot 실패·alert |
| test | test-runner | ephemeral build/test | 제품 실행에 포함하지 않음 |

`migrate`는 API image의 전용 `migrate` subcommand를 사용한다. 이 command는 database secret과 pool
설정만 읽고 migration·PostgreSQL version·pending 0을 검사한 뒤 종료한다. OIDC·AI·ObjectStorage
credential을 받지 않는다.

## Secret·network·volume

Compose에는 secret 원문 environment를 넣지 않는다. `infra/docker/bootstrap-local.sh`가 gitignore된
`infra/docker/.local/secrets`에 mode `0600` file을 원자 생성한다. application은 `_FILE`, PostgreSQL은
`POSTGRES_PASSWORD_FILE`, Redis는 file을 읽는 entrypoint를 사용한다. example에는 값이 아니라
형식만 둔다.

host 공개 port는 web·api만 loopback에 bind한다. PostgreSQL·Redis·OpenSearch·worker·collector는
internal network에만 둔다. `postgres_data`, `object_data`, `backup_data`는 persistent named volume이고
Redis·OpenSearch data는 삭제 후 재구축 가능한 별도 volume이다. 일반 `down`은 volume을 지우지
않고 test gate만 project 이름을 확인한 뒤 `down --volumes`로 격리 volume을 제거한다.

## Health와 종료

API는 `/health/live`에서 process event loop만, `/health/ready`에서 typed config·migration current·
PostgreSQL을 검사한다. Redis·OpenSearch·AI 상태는 ready JSON의 dependency map에 표시하되 core
HTTP status를 실패시키지 않는다. Web health는 SSR process와 API route reachability를 분리한다.
Worker는 process liveness와 PostgreSQL·migration·Redis readiness를 구분한 health command를 제공한다.

모든 application health output은 secret path·URL·provider message를 포함하지 않는다. container는
`SIGTERM` 뒤 `ADOC_SHUTDOWN_GRACE` 안에 listener를 닫고 새 작업을 받지 않으며 transaction·lease를
정리한다. grace 초과는 non-zero 종료다.

## Backup profile

backup one-shot은 공유 `backup_data`의 원자 directory lock으로 같은 destination의 중복 실행을 막고
PostgreSQL custom-format dump와 object_data tar를 임시 directory에 만든다. lock과 partial directory는
process 종료 trap에서 정리한다. manifest에는 UTC creation time, release SHA, migration version,
각 artifact byte size·SHA-256을 기록한다. 모두 성공한 뒤 timestamp directory를 backup_data에
rename하고 `latest` pointer를 교체한다. partial directory는 성공으로 간주하지 않는다.

local profile은 암호화·외부 destination을 제공하지 않으므로 RPO/RTO 충족을 표시하지 않는다.
production은 encrypted external destination과 restore drill evidence가 없으면 backup readiness를
degraded로 노출한다.

## 실행 Gate

`scripts/check-compose.mjs`는 exact tag, profile, secret-file-only, no privileged/root, health dependency,
volume classification과 host port를 정적으로 검사한다. Docker integration은 고유 project name에서
build → secret bootstrap → core up → migration exit 0 → health → search degraded isolation → backup
artifact checksum → down → test volume cleanup 순으로 실행한다. 정적 검사를 mock Compose 실행으로
대체하지 않는다.
