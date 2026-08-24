# API Conventions

- **문서 ID**: API-01
- **상태**: 동결

## 경계

`/api/v1`은 웹 애플리케이션 내부 HTTP 계약이다. 고객용 Public API가 아니다. Browser와
server는 cookie session을 사용하고 Public Viewer는 `/public/v1` capability endpoint만 쓴다.

## Command·query

query는 GET, create command는 POST, state transition은 명시적 action POST를 사용한다.
generic PATCH로 domain transition을 우회하지 않는다. side effect command는
`Idempotency-Key`와 mutable target의 `If-Match: "revision"`을 요구한다.

## Response

성공은 typed resource 또는 `{data,meta}`다. cursor pagination은 opaque `nextCursor`를 쓴다.
오류는 `application/problem+json`의 다음 shape다.

```json
{"type":"urn:adoc:error:STALE_REVISION","title":"Conflict","status":409,
 "code":"STALE_REVISION","retryable":false,"correlationId":"...",
 "currentRevision":14,"fieldErrors":[]}
```

## 보안

state-changing cookie request는 CSRF header를 요구한다. Workspace route의 workspaceId와
session-selected Workspace를 모두 검사한다. IDOR 방지를 handler 뒤가 아니라 application
command 입구에서 수행한다.

## Versioning

additive field는 같은 v1에서 가능하다. enum 추가는 client unknown handling을 요구한다.
의미·required field·transition 변화는 새 endpoint/version과 compatibility window를 둔다.

## Streaming

SSE는 `Last-Event-ID` 또는 `cursor`로 resume한다. event payload는 [Streaming Jobs](STREAMING-JOBS.md)와
[AsyncAPI](asyncapi.yaml)를 따른다.
