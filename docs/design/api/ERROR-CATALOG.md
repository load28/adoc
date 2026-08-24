# Error Catalog

- **문서 ID**: API-07
- **상태**: 동결

## Envelope

모든 오류는 `application/problem+json`이다. 필수 field는 `type`, `title`, `status`, `code`,
`retryable`, `correlationId`, `fieldErrors`다. `details`에는 allowlist된 non-sensitive scalar만
들어간다. 존재를 숨겨야 하는 resource의 authorization 실패는 `404`와 동일 body를 반환한다.

## 공통 오류

| Code | HTTP | 재시도 | 의미·client 처리 |
|---|---:|---|---|
| `VALIDATION_FAILED` | 422 | 아니요 | field error 표시, 입력 유지 |
| `AUTH_REQUIRED` | 401 | 아니요 | 로그인 복귀 경로 저장 |
| `CSRF_INVALID` | 403 | 아니요 | session 새로고침 후 로그인 |
| `PERMISSION_DENIED` | 403 | 아니요 | action 제거, 현재 화면 재조회 |
| `RESOURCE_NOT_FOUND` | 404 | 아니요 | 존재 비노출, 안전한 상위 route 이동 |
| `REVISION_CONFLICT` | 409 | 아니요 | 최신 revision 조회 후 diff·재시도 선택 |
| `IDEMPOTENCY_KEY_REUSED` | 409 | 아니요 | 새 key로 같은 요청을 자동 재전송하지 않음 |
| `RATE_LIMITED` | 429 | 예 | `Retry-After` 뒤 사용자 동의 없이 동일 command 1회 |
| `DEPENDENCY_UNAVAILABLE` | 503 | 예 | degraded 상태 표시 |
| `INTERNAL_ERROR` | 500 | 조건부 | correlation ID 표시, command 자동 반복 금지 |

## 도메인 오류

| 영역 | Code | HTTP | client가 신뢰할 detail |
|---|---|---:|---|
| Auth | `AUTH_PROVIDER_UNAVAILABLE`, `AUTH_CALLBACK_INVALID` | 503, 400 | `provider` |
| Workspace | `WORKSPACE_NOT_FOUND`, `WORKSPACE_SLUG_TAKEN`, `WORKSPACE_STATE_INVALID`, `LAST_OWNER` | 404, 409, 409, 409 | `currentStatus` |
| Invitation | `INVITATION_EXISTS`, `INVITATION_INVALID`, `INVITATION_STATE_INVALID` | 409, 404, 409 | `expiresAt` |
| Group | `GROUP_NOT_FOUND`, `GROUP_NAME_TAKEN`, `GROUP_IN_USE`, `GROUP_MEMBER_INVALID`, `GROUP_MEMBER_NOT_FOUND` | 404, 409, 409, 422, 404 | `referenceCount` |
| Document | `DOCUMENT_NOT_FOUND`, `DOCUMENT_PARENT_INVALID`, `DOCUMENT_TREE_CYCLE`, `DOCUMENT_RANK_CONFLICT`, `DOCUMENT_STATE_INVALID`, `DOCUMENT_EFFECTIVELY_TRASHED`, `MOVE_PREVIEW_STALE`, `NO_EFFECT`, `PURGE_NOT_ELIGIBLE` | 404, 422, 409, 409, 409, 409, 409, 409, 409 | `currentStatus`, `purgeAfter` |
| Draft | `DRAFT_NOT_FOUND`, `DRAFT_EXISTS`, `OPERATION_PRECONDITION_FAILED`, `EDIT_LEASE_HELD`, `EDIT_LEASE_INVALID`, `EDIT_LEASE_EXPIRED` | 404, 409, 409, 423, 409, 409 | `currentRevision`, `leaseExpiresAt` |
| Version | `VERSION_NOT_FOUND`, `IMMUTABLE_RESOURCE`, `DOCUMENT_UNPUBLISHED` | 404, 409, 409 | `currentVersion` |
| Permission | `PERMISSION_SUBJECT_INVALID`, `PERMISSION_MANAGE_REQUIRES_EDITOR`, `PERMISSION_GRANT_CONFLICT`, `PERMISSION_LAST_MANAGER` | 422, 422, 409, 409 | `subjectKind` |
| Policy | `PUBLISH_POLICY_INVALID`, `PUBLISH_REVIEW_REQUIRED` | 422, 409 | `requiredApprovals`, `approvedCount` |
| Public | `PUBLIC_LINK_INVALID`, `PUBLIC_LINK_STATE_INVALID`, `PUBLIC_ASSET_NOT_EMBEDDED` | 404, 409, 404 | 없음 |
| Discussion | `DISCUSSION_NOT_FOUND`, `DISCUSSION_TARGET_INVALID`, `DISCUSSION_STATE_INVALID`, `DISCUSSION_CLOSED`, `DISCUSSION_TOPIC_REQUIRED` | 404, 422, 409, 409, 409 | `currentStatus` |
| Message | `MESSAGE_EDIT_WINDOW_EXPIRED`, `MESSAGE_STATE_INVALID` | 409, 409 | `editedAt` |
| Review | `REVIEW_NOT_FOUND`, `REVIEW_ALREADY_OPEN`, `REVIEW_STALE`, `REVIEW_STATE_INVALID` | 404, 409, 409, 409 | `draftRevision`, `currentStatus` |
| Inbox | `INBOX_ITEM_NOT_FOUND` | 404 | 없음 |
| Knowledge | `SEARCH_UNAVAILABLE`, `REFERENCE_TARGET_INVALID`, `REFERENCE_NOT_FOUND`, `VOCABULARY_TERM_CONFLICT`, `VOCABULARY_NOT_FOUND`, `VOCABULARY_STATE_INVALID` | 503, 422, 404, 409, 404, 409 | `term` |
| AI | `AI_QUOTA_EXCEEDED`, `AI_CONCURRENCY_LIMIT`, `AI_JOB_NOT_FOUND`, `AI_JOB_STATE_INVALID`, `PROPOSAL_NOT_FOUND`, `PROPOSAL_STALE`, `PROPOSAL_STATE_INVALID` | 429, 429, 404, 409, 404, 409, 409 | `limit`, `resetAt`, `baseRevision` |
| Configuration | `WRITING_CONFIGURATION_INVALID`, `AI_CONFIGURATION_INVALID`, `AI_USAGE_UNAVAILABLE`, `AI_PROVIDER_UNAVAILABLE` | 422, 422, 503, 503 | `configurationVersion`, `provider` |
| File | `FILE_LIMIT_EXCEEDED`, `FILE_CHECKSUM_MISMATCH`, `FILE_NOT_FOUND`, `FILE_NOT_READY`, `FILE_STILL_REFERENCED` | 413, 422, 404, 409, 409 | `limitBytes`, `status`, `referenceCount` |
| Stream | `STREAM_CURSOR_EXPIRED` | 409 | `minimumCursor` |

## Retry contract

`retryable=true`는 같은 idempotency key와 body로 재시도할 수 있다는 뜻이다. Browser는 network
timeout·503·429만 최대 2회 exponential backoff+jitter로 재시도한다. `409`, `422`, `401`,
`403`, `404`는 사용자의 새 입력이나 최신 state 없이 재시도하지 않는다. AI·Search provider
오류를 다른 모델이나 일반 지식 답변으로 조용히 대체하지 않는다.
