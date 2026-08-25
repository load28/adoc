# Identity·Session Implementation Contract

- **문서 ID**: PLAN-12
- **상태**: 구현 기준
- **관련 package**: IMP-06

## 책임 경계

`identity` domain은 Google protocol이나 HTTP·SQL을 모른다. 검증된 외부 identity, User profile,
Preference와 session lifetime 불변식을 소유한다. application service는 OIDC·repository·clock·random
port를 조합한다. Google HTTP/JWT와 PostgreSQL은 adapter, cookie·header·problem response는 Axum
transport가 소유한다.

IMP-06은 인증된 `SessionPrincipal { user_id, session_hash, locale }`를 제공한다. Workspace 목록은
session query에서 read-only projection으로 포함할 수 있지만 Workspace 생성·Membership 상태 전이와
role 정책은 IMP-07이 소유한다. Permission과 UI는 각각 IMP-08·22까지 구현하지 않는다.

## Domain과 application contract

```text
VerifiedGoogleIdentity
  subject, verified_email, display_name

User
  id, google_subject, email, display_name
  preferences { locale, timezone, theme, revision }

Session
  token_hash, hash_key_id, user_id
  created_at, last_seen_at, idle_expires_at, absolute_expires_at
  rotated_from_hash?, revoked_at?

LoginFlow
  state_hash, nonce, pkce_verifier
  return_to, expires_at, consumed_at?
```

email은 Google의 `email_verified=true`일 때만 받으며 Unicode trim 뒤 ASCII domain을 lowercase한다.
identity key는 email이 아니라 `(issuer, subject)`다. display name은 빈 값·200자 초과를 거부한다.
Locale은 `ko|en`, theme은 `LIGHT|DARK|SYSTEM`, timezone은 IANA name allowlist로 검증한다.

application port는 다음 의미를 갖는다.

```rust
trait OidcProvider {
  start(nonce, code_challenge, redirect_uri) -> AuthorizationUrl;
  exchange(code, verifier, redirect_uri, expected_nonce) -> VerifiedExternalIdentity;
}
trait IdentityRepository {
  create_login_flow(flow) -> Result<(), IdentityStoreError>;
  consume_login_flow(state_hash, now) -> Result<LoginFlow, LoginFlowError>;
  establish_identity(identity, session, now) -> Result<UserSession, IdentityStoreError>;
  authenticate(candidate_hashes, now, idle_extension) -> Result<SessionPrincipal, SessionError>;
  revoke(session_hash, now) -> Result<RevokeOutcome, IdentityStoreError>;
  preferences(user_id) -> Result<UserPreferences, IdentityStoreError>;
  update_preferences(user_id, expected_revision, input, now) -> Result<UserPreferences, ...>;
}
```

`consume_login_flow`, `establish_identity`, `authenticate`의 expiry 연장, logout과 preference conditional
update는 각각 단일 DB transaction이다. provider network는 DB transaction 밖에서 실행한다.

## OIDC flow

`GET /api/v1/auth/google/start?returnTo=`는 return path를 검증하고 32-byte state, nonce와 PKCE verifier를
CSPRNG로 생성한다. verifier의 SHA-256 base64url challenge만 authorization URL에 보낸다. state는 token
pepper로 hash하고 LoginFlow 원문은 10분 뒤 만료된다. returnTo는 `/`로 시작하는 same-origin path만,
`//`, backslash, control character, embedded credential과 2048자 초과를 거부한다.

callback은 다음 순서를 유지한다.

```text
입력 길이 검사 → state hash → flow 원자 consume → provider token exchange
→ discovery/JWKS 기반 ID token 검증 → User upsert → Session 발급 → redirect
```

Google adapter는 allowlisted issuer `https://accounts.google.com`, configured client audience, RS256
signature, `exp`, `iat`, nonce, subject, `email_verified`를 검증한다. discovery의 authorization·token·
JWKS endpoint는 HTTPS이고 Google host allowlist 안에 있어야 한다. JWKS는 `Cache-Control`의 상한 1시간과
ETag로 cache하며 unknown `kid`에서 한 번 강제 refresh한다. access·refresh token과 authorization code는
보존하거나 log하지 않는다. discovery·JWKS body는 256 KiB, token body는 64 KiB를 상한으로 읽는다.
provider timeout·invalid body·key miss는 stable category로 변환한다.

flow는 provider 호출 전에 consume한다. provider가 실패해도 같은 state를 재사용하지 않으며 사용자는
새 login을 시작한다. 이는 외부 호출 중 DB lock을 유지하지 않으면서 callback replay를 막는다.

login abuse gate는 Redis의 atomic script를 사용한다. start는 peer IP hash당 10분 20회, callback은 peer
IP hash와 login marker hash 각각 10분 40회로 제한한다. Redis key에는 flow pepper로 HMAC한 signal만
포함하고 IP·marker 원문을 넣지 않는다. Redis 장애 시 신규 login은 fail closed로 503을 반환하지만 기존
session API는 영향받지 않는다. transport는 직접 peer address를 기준으로 하며 trusted proxy signal
해석은 configured CIDR 검증을 추가하는 ingress 태스크 전까지 수행하지 않는다.

## Session·token·rotation

신규 token은 CSPRNG 32 byte base64url이다. DB key는 `HMAC-SHA-256(session_hmac_key, token)`이며
`hash_key_id`를 함께 저장한다. request 검증은 current·previous pepper candidate를 constant-time
비교 가능한 고정 길이 hash로 계산한다. 신규 row는 current key만 사용한다.

login state와 browser marker는 범용 `token_hash_pepper`, session token은 `session_hmac_key`를 사용해
protocol 사이 key 재사용을 막는다. 두 key ring은 각각 current·previous만 읽는다.

idle lifetime은 12시간, absolute lifetime은 최초 login부터 30일이다. 인증 시 idle expiry를
`min(now+12h, absolute_expires_at)`로 연장하되 마지막 write 후 5분 이내에는 write amplification을
피한다. `now >= idle_expires_at`, `now >= absolute_expires_at`, revoked session은 모두 `AUTH_REQUIRED`다.
login은 기존 browser session을 revoke한 뒤 새 token을 발급해 fixation을 막는다. privilege change는
IMP-07에서 같은 repository의 no-grace rotate/revoke를 호출한다. rotation은 old revoke와 new insert를
한 transaction으로 수행하고 old token은 즉시 실패한다.

cookie 계약은 다음과 같다.

| Cookie | 값 | 속성 |
|---|---|---|
| `adoc_session` | opaque token | `Secure; HttpOnly; SameSite=Lax; Path=/; Max-Age=43200` |
| `adoc_csrf` | random token+MAC | `Secure; SameSite=Strict; Path=/; Max-Age=43200`, SPA가 double-submit header를 구성하도록 HttpOnly 미사용 |
| `adoc_login` | flow marker | `Secure; HttpOnly; SameSite=Lax; Path=/api/v1/auth/google; Max-Age=600` |

logout은 session row를 idempotent revoke하고 세 cookie를 `Max-Age=0`으로 제거한다. session 원문,
hash, CSRF token과 cookie header는 telemetry field에 넣지 않는다.

## CSRF·Origin·request extractor

unsafe method는 `Origin == ADOC_PUBLIC_ORIGIN`, session cookie, `X-CSRF-Token` 순서로 검증한다.
CSRF 값은 `key_id.random.mac` envelope다. MAC input은
`key_id || session_hash || random`이고 session hash 자체는 cookie에 포함하지 않는다. header와
`adoc_csrf` cookie의 bytes를 constant-time 비교한 뒤 MAC을 검증한다.
CSRF current·previous key를 읽되 response는 current key로 재발급한다.

`GET`, `HEAD`, `OPTIONS`는 CSRF를 요구하지 않지만 session이 필요한 route는 동일 session extractor를
거친다. CORS credential wildcard를 만들지 않고 API는 same-origin만 응답한다. proxy header는
configured trusted CIDR에서만 client signal로 사용하며 인증 판단은 proxy header에 의존하지 않는다.

## 저장 계약과 migration

canonical DDL은 다음을 강제한다.

- `users`: `(issuer, google_subject)` unique, verified email/profile과 preference revision 유지
- `login_flows`: 32-byte state hash PK, nonce·PKCE verifier·return path, created/expiry/consumed time,
  expiry·consumption 순서 check와 expiry index
- `sessions`: `hash_key_id`, `last_seen_at`, `idle_expires_at`, `absolute_expires_at`, revoke·rotation link,
  `created <= last_seen <= idle <= absolute` check와 active user/expiry index
- update preference: `WHERE id=$user AND revision=$expected`, 성공 시 revision `+1`
- `user_command_receipts`: Workspace 밖 사용자 command의 `(user, operation, key)` request hash·response를
  원자 보존하며 같은 key의 다른 body를 거부
- user upsert: subject row lock 뒤 verified email·display name만 갱신하고 preference는 덮지 않음

login flow와 session token hash는 만료+30일 뒤 retention worker가 제거한다. 보안 추적에 필요한
비민감 event는 Audit 구현 전까지 structured auth event로만 남기고 token/IP 원문을 저장하지 않는다.

## HTTP 계약

| Operation | 성공 | 인증·동시성 | stable 실패 |
|---|---|---|---|
| `beginGoogleLogin` | 302 Google URL+login cookie | public, rate-limit hook | `VALIDATION_FAILED`, `AUTH_PROVIDER_UNAVAILABLE` |
| `completeGoogleLogin` | 302 validated returnTo+session/CSRF cookie | state 1회 | `AUTH_CALLBACK_INVALID`, provider unavailable |
| `getSession` | `SessionView`+fresh CSRF cookie | session | `AUTH_REQUIRED` |
| `logout` | 204+cookie clear | session, Origin+CSRF, idempotent | `AUTH_REQUIRED`, `CSRF_INVALID` |
| `getUserPreferences` | `UserPreferences` | session | `AUTH_REQUIRED` |
| `updateUserPreferences` | updated resource | session, Origin+CSRF, expected revision | validation, `REVISION_CONFLICT` |

모든 route는 `/api/v1` 아래에 있고 OpenAPI operationId와 handler name을 맞춘다. error는
`application/problem+json`이고 provider·SQL·token detail 대신 correlation ID만 노출한다.
callback의 validation 실패 redirect는 query에 provider 원문을 넣지 않고 고정 error code만 전달한다.

## 실패·동시성·복구

- 동일 state callback 경쟁: conditional consume에서 한 요청만 성공하고 나머지는 invalid callback
- 동일 Google subject 최초 login 경쟁: unique constraint 뒤 subject row를 재조회해 User 하나만 유지
- session refresh/logout 경쟁: row lock에서 logout revoke가 우선하며 refresh가 session을 부활시키지 않음
- preference 동시 update: expected revision 하나만 성공하고 current revision을 conflict metadata로 반환
- preference network replay: 같은 key·request hash는 저장된 response를 반환하고 다른 hash는
  `IDEMPOTENCY_KEY_REUSED`
- key rotation: previous key 검증은 허용하되 성공한 session은 current key token으로 rotate할 수 있음
- provider/JWKS 장애: 기존 session API는 유지하고 login만 unavailable; cached key의 유효기간을 넘겨 사용하지 않음
- DB 장애: session-required route는 fail closed; anonymous이나 stale session으로 조용히 진행하지 않음

## Test gate

1. Domain: email/profile/preference/return path/lifetime negative corpus와 token redaction.
2. Service: deterministic clock/random, one-shot state, fixation revoke, expiry, no-grace rotation, revision race.
3. OIDC adapter: local fake discovery/token/JWKS로 PKCE 전송과 issuer·audience·signature·nonce·exp·kid 실패.
4. PostgreSQL 16: concurrent callback/upsert, candidate key authentication, revoke-refresh race, preference CAS.
5. HTTP: cookie attribute snapshot, no localStorage/token body, Origin·CSRF matrix, logout clear, open redirect,
   provider·SQL message 비노출.
6. Contract: OpenAPI operation/schema coverage와 configuration negative corpus.
7. Repository: root format, clippy, test, build, secret/license scan과 Compose core regression.

Compose `test` profile은 host port를 추가하지 않고 internal network에서 ignored PostgreSQL contract
test를 실행한다. migration gate는 별도 database에 sealed baseline을 적용한 뒤 forward migration과
identity column assertion을 수행해 fresh apply와 upgrade path를 모두 검증한다.
