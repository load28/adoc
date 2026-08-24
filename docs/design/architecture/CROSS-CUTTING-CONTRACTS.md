# Cross-cutting Contracts

- **문서 ID**: ARCH-05
- **상태**: 동결

## Identity

외부에는 UUIDv7 기반 opaque typed ID를 string으로 전달한다. workspaceId는 모든 tenant row와
cache·event key에 포함한다. title, slug와 storage key를 identity로 사용하지 않는다.

## 시간과 순서

서버가 생성한 UTC instant만 정본이다. 사용자 locale은 표시 계층에서 적용한다. causal
ordering은 aggregate revision 또는 event sequence로 표현하며 timestamp로 충돌을 해결하지
않는다.

## Revision과 Version

- Revision: mutable aggregate의 monotonic concurrency token
- Published Version number: Document 안에서 1부터 증가하는 immutable ordinal
- Schema version: serialized payload 형식 version
- Event version: integration payload 호환 version

## Request context

모든 command/query는 requestId, actor, workspaceId, sessionId, locale과 permission evaluation
context를 가진다. 비동기 job은 causationId·correlationId를 이어받되 session credential을
복사하지 않는다.

## Error

`code`, `category`, `retryable`, `fieldErrors?`, `currentRevision?`, `correlationId` 계약을 쓴다.
category는 validation, authentication, authorization, not_found, conflict, quota,
dependency, unavailable, internal이다. 내부 stack·SQL·provider message는 외부로 보내지 않는다.

## Idempotency

모든 side-effect HTTP command는 Workspace·actor·route 범위의 key를 요구한다. 같은 key와 같은
request hash는 기존 result를 반환하고 다른 hash는 conflict다.
