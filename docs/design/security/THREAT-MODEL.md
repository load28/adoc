# Threat Model

- **문서 ID**: SEC-01
- **상태**: 동결

## 보호 asset

Document content·title, Discussion, Reference graph, Vocabulary, File bytes, Google identity,
session, AI Context·Result, Permission policy, Audit와 backup encryption key.

## 공격자

미인증 인터넷 사용자, 악의적 Public link 보유자, 권한이 낮은 Member, 악의적 Admin,
탈취 session, prompt injection 문서·외부 web, compromised dependency와 운영 credential 보유자.

## 주요 위협과 완화

| 위협 | 구조적 완화 |
|---|---|
| cross-tenant IDOR | 모든 port에 workspaceId, DB composite key, authorization test matrix |
| search·AI existence leak | query 전 PermissionScope, result count·timing redaction |
| public link scope 확장 | random token hash, exact document/latest version/asset set, 별도 route |
| CSRF·session theft | state-changing CSRF token, Secure HttpOnly cookie, rotate·revoke |
| stored XSS | content schema allowlist, URL sanitize, CSP, safe renderer |
| SSRF | external fetch isolation, DNS/IP 재검사, redirect·size·MIME limit |
| prompt injection | Source를 instruction과 분리, no tools/DB, structured validation |
| malicious upload | streaming limit, detected MIME, malware scan, sandbox preview |
| queue replay | idempotency ledger, expected revision, terminal state guard |
| backup exposure | encryption, separated key, access audit, restore environment isolation |

## Admin 경계

Admin은 Membership·설정을 관리하지만 Document content access를 우회하지 않는다. 운영 break
glass는 제품 role이 아니며 별도 audited infrastructure procedure와 사용자 승인 근거를
요구한다.

## 검증

위협별 abuse case는 [Security Tests](../quality/SECURITY-TESTS.md)에 추적한다. 새로운 외부
integration·parser·renderer는 이 문서 갱신 없이는 추가하지 않는다.
