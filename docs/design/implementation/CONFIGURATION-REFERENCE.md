# Configuration Reference

- **문서 ID**: PLAN-07
- **상태**: 동결

모든 설정은 시작 시 typed parser로 검증한다. unknown key, 잘못된 enum·duration·URL과 secret의
plain environment 직접 값은 시작 실패다. `_FILE`은 Docker secret file 경로를 뜻한다.

## Core·HTTP

| Key | 대상 | 기본값·제약 | reload |
|---|---|---|---|
| `ADOC_ENV` | all | `development|test|production`, 필수 | 재시작 |
| `ADOC_RELEASE_SHA` | all | immutable image label, 필수 | 재시작 |
| `ADOC_HTTP_BIND` | web/api | `0.0.0.0:8080/8081` | 재시작 |
| `ADOC_PUBLIC_ORIGIN` | web/api | HTTPS absolute, production 필수 | 재시작 |
| `ADOC_API_UPSTREAM` | web | internal API origin, Browser에 노출 금지 | 재시작 |
| `ADOC_SHUTDOWN_GRACE` | all | `30s`, 5~120s | 재시작 |
| `ADOC_LOG_LEVEL` | all | `info`, content logging 금지 | signal reload |
| `ADOC_OTEL_ENDPOINT` | all | optional HTTPS/gRPC | 재시작 |

## Database·queue·search

| Key | 기본값·제약 | secret |
|---|---|---|
| `ADOC_DATABASE_URL_FILE` | PostgreSQL TLS URL, 필수 | ✓ |
| `ADOC_RETENTION_DATABASE_URL_FILE` | `adoc_retention` 전용, worker만 | ✓ |
| `ADOC_DB_MAX_CONNECTIONS` | api 30, worker 20, 1~200 | — |
| `ADOC_REDIS_URL_FILE` | Redis TLS 또는 Compose URL | ✓ |
| `ADOC_QUEUE_NAMESPACE` | release-independent stable prefix | — |
| `ADOC_OPENSEARCH_URL` | HTTP(S) endpoint | — |
| `ADOC_OPENSEARCH_CREDENTIAL_FILE` | JSON user/password, production 필수 | ✓ |
| `ADOC_SEARCH_INDEX_PREFIX` | `adoc`, pattern `[a-z0-9-]+` | — |
| `ADOC_EMBEDDING_DIMENSION` | `1536`, mapping과 exact match | — |

## Auth·security

| Key | 기본값·제약 | secret |
|---|---|---|
| `ADOC_GOOGLE_CLIENT_ID_FILE` | Google OIDC client ID | ✓ |
| `ADOC_GOOGLE_CLIENT_SECRET_FILE` | Google OIDC secret | ✓ |
| `ADOC_SESSION_HMAC_KEY_FILE` | 32 byte 이상, key ID 포함 | ✓ |
| `ADOC_CSRF_HMAC_KEY_FILE` | session key와 별도 | ✓ |
| `ADOC_TOKEN_HASH_PEPPER_FILE` | invitation/public link token pepper | ✓ |
| `ADOC_TRUSTED_PROXY_CIDRS` | 명시 목록, 기본 empty | — |
| `ADOC_SESSION_TTL` | `12h`, 최대 7d | — |
| `ADOC_PUBLIC_LINK_MAX_TTL` | `365d`; no-expiry도 정책상 가능 | — |

key rotation은 `current`와 `previous` key ID 두 개를 읽고 새 token은 current로만 발급한다.
secret file 권한이 group/world-readable이면 production 시작을 거부한다.

회전 가능한 HMAC·pepper secret file은 UTF-8 JSON object다. `current`는 필수이고 `previous`는
선택이며 각 entry는 `id`와 `value`를 가진다. key ID는 서로 달라야 하고 빈 값은 거부한다.

```json
{"current":{"id":"key-2026-08","value":"..."},"previous":{"id":"key-2026-07","value":"..."}}
```

파일은 원자적으로 교체한다. 새 서명은 `current`만 사용하고 검증은 두 key를 모두 허용한다.
preflight와 log에는 path·value를 출력하지 않고 source kind와 key ID만 출력한다.

## File·AI

| Key | 기본값·제약 | secret |
|---|---|---|
| `ADOC_OBJECT_STORAGE_DRIVER` | `local|s3`, 최초 `local` | — |
| `ADOC_LOCAL_OBJECT_ROOT` | container volume 절대 경로 | — |
| `ADOC_S3_BUCKET`, `ADOC_S3_REGION`, `ADOC_S3_ENDPOINT` | driver=s3일 때 | — |
| `ADOC_S3_CREDENTIAL_FILE` | workload identity 없을 때만 | ✓ |
| `ADOC_UPLOAD_MAX_BYTES` | 100 MiB, 1 MiB~5 GiB | — |
| `ADOC_ALLOWED_MIME_FILE` | versioned allowlist path | — |
| `ADOC_AI_DRIVER` | `codex_cli|openai_responses` | — |
| `ADOC_CODEX_EXECUTABLE` | allowlisted absolute binary path | — |
| `ADOC_OPENAI_API_KEY_FILE` | Responses driver에서만 | ✓ |
| `ADOC_AI_REQUEST_TIMEOUT` | `180s`, 10~600s | — |
| `ADOC_AI_KILL_GRACE` | `5s`, 1~30s | — |
| `ADOC_AI_MAX_CONTEXT_TOKENS` | model capability 이하 | — |

개인 ChatGPT/Codex subscription credential은 shared server 설정으로 받지 않는다. AI model,
Workspace concurrency와 budget 정본은 DB configuration이며 environment는 bootstrap default만
제공한다.

## Retention·worker

| Key | 기본값·제약 | reload |
|---|---|---|---|
| `ADOC_TRASH_RETENTION` | `30d`, 현재 제품 정책상 고정 | 재시작 |
| `ADOC_WORKSPACE_RETENTION` | `30d`, 현재 제품 정책상 고정 | 재시작 |
| `ADOC_JOB_LEASE` | `60s` | 재시작 |
| `ADOC_JOB_MAX_ATTEMPTS` | `5`, kind override는 code-owned | 재시작 |
| `ADOC_OUTBOX_BATCH_SIZE` | `100`, 1~1000 | reload |
| `ADOC_RECONCILE_INTERVAL` | `30s`, 5s~10m | reload |

production에서 정책 고정값을 다른 값으로 설정하면 warning이 아니라 시작 실패다. 정책 변경은
Decision, lifecycle 문서, migration과 API를 먼저 변경한다.

## Preflight

각 binary는 `--check-config`로 secret 내용을 출력하지 않고 source, key ID, connectivity,
schema migration, contract version과 writable volume을 검사한다. readiness는 core dependency와
degraded dependency를 분리하며 AI·OpenSearch 장애가 Document core liveness를 실패시키지 않는다.
