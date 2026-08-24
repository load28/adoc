# System Context

- **문서 ID**: ARCH-01
- **상태**: 동결

## Actors와 외부 시스템

```text
Member/Admin/Public Viewer
        │ HTTPS
        ▼
Team Document System
 ├─ Google OIDC
 ├─ OpenAI Responses API (managed production)
 ├─ Codex CLI (local/self-hosted)
 └─ optional external web sources
```

## Trust boundary

- Browser ↔ Edge/Web: untrusted input, session·CSRF boundary
- Web ↔ Rust API: internal network여도 authenticated request context 요구
- API/Worker ↔ PostgreSQL: tenant predicate와 transaction boundary
- API/Worker ↔ OpenSearch·Redis·ObjectStorage: projection·delivery system, 정본 아님
- AI Runtime: 최소 Context만 받는 별도 process·credential boundary
- Public Viewer: Workspace principal이 아닌 capability-token boundary

## Data ownership

PostgreSQL만 domain state의 진실 소스다. OpenSearch는 rebuild 가능, Redis는 queue·ephemeral
coordination, ObjectStorage는 File bytes를 소유한다. Browser local storage는 unsynced Draft
recovery만 보유하며 server 정본을 대체하지 않는다.

## 외부 경계

결제, 고객용 Public API·Webhook과 익명 편집은 없다. 외부 web은 AI task마다 opt-in하며
retrieved Source만 결과 근거가 된다.
