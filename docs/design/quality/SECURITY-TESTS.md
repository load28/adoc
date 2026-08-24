# Security Tests

- **문서 ID**: TEST-04
- **상태**: 동결

## Authorization matrix

User·Group precedence, Admin content deny, removed Member, trashed Document, stale cache와
cross-workspace ID를 모든 query·command·Search·Reference·File·AI endpoint에 적용한다.

## Public link

token brute force rate limit, hash-only DB, revoke·expiry, no current Version, trash, asset ID
substitution, route discovery, cache key confusion과 Referer leakage를 검사한다.

## Web

OIDC state·nonce·PKCE, session fixation·rotation, CSRF, CORS, CSP, stored/reflected XSS, open
redirect, request smuggling boundary와 security header를 검사한다.

## AI·File

prompt injection Source, oversized output, invalid schema, tool request, private URL SSRF, redirect
rebind, zip bomb, MIME polyglot, SVG script와 path traversal fixture를 사용한다.

## Supply chain

lockfile integrity, license allowlist, SBOM, secret scan, Rust advisory·npm audit와 container image
scan을 수행한다. `@atlaskit` package는 Apache-2.0과 official source를 검증한다.

## 결과

authorization·tenant leak는 severity와 무관하게 release blocker다. penetration finding은
재현 test 없이는 close하지 않는다.
