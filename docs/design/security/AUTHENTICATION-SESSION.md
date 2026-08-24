# Authentication과 Session Security

- **문서 ID**: SEC-02
- **상태**: 동결

## Google OIDC

Authorization Code+PKCE, state와 nonce를 사용한다. issuer·audience·signature·exp·nonce를
검증하고 redirect URI는 exact allowlist다. identity는 Google `sub`, 초대 수락은 verified
email normalized match를 사용한다.

## Session token

256-bit random opaque token, Argon2id 또는 keyed SHA-256 hash 저장, constant-time compare를
사용한다. cookie는 `Secure; HttpOnly; SameSite=Lax; Path=/`이고 production HTTPS만 허용한다.

## Lifetime

idle 12시간, absolute 30일. privilege change·login에서 rotate하며 이전 token에 짧은 replay
window를 주지 않는다. revoke는 user·Workspace·session 단위로 가능하다.

## CSRF·CORS

same-origin browser API만 허용한다. unsafe method는 session-bound CSRF token과 Origin 검사를
모두 수행한다. permissive CORS와 wildcard credential을 금지한다.

## Login abuse

state start·callback을 IP·browser signal로 rate limit한다. account existence를 email 기반
message로 노출하지 않는다. auth log는 content log와 분리하고 token·code를 기록하지 않는다.

## Frontend

session을 localStorage에 넣지 않는다. SSR loader가 auth context를 검증하고 client hydration
payload에는 필요한 user profile과 active Workspace ID만 포함한다.
