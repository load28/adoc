# Command·Query Catalog

- **문서 ID**: API-06
- **상태**: 동결
- **기본 prefix**: `/api/v1`

이 문서의 모든 command는 cookie session, CSRF, `Idempotency-Key`를 요구한다. `R`은
`If-Match` expected revision, `L`은 Edit Lease token도 필요하다는 뜻이다. 세부 request·response
type은 [OpenAPI](openapi.yaml), 오류 의미는 [Error Catalog](ERROR-CATALOG.md)가 정본이다.

## Identity·Workspace

| Operation ID | Method·path | 종류 | 최소 권한 | 동시성 | 핵심 오류 |
|---|---|---|---|---|---|
| getSession | `GET /session` | Q | session | — | `AUTH_REQUIRED` |
| beginGoogleLogin | `GET /auth/google/start` | Q | public | state cookie | `AUTH_PROVIDER_UNAVAILABLE` |
| completeGoogleLogin | `GET /auth/google/callback` | C | public | OAuth state 1회 | `AUTH_CALLBACK_INVALID` |
| logout | `POST /session/logout` | C | session | idempotent | `AUTH_REQUIRED` |
| getUserPreferences | `GET /preferences` | Q | session | — | — |
| updateUserPreferences | `PUT /preferences` | C | session | R | `REVISION_CONFLICT` |
| listWorkspaces | `GET /workspaces` | Q | session | — | — |
| createWorkspace | `POST /workspaces` | C | session | key | `WORKSPACE_SLUG_TAKEN` |
| getWorkspace | `GET /workspaces/{workspaceId}` | Q | Member | — | `WORKSPACE_NOT_FOUND` |
| updateWorkspace | `PUT /workspaces/{workspaceId}` | C | Admin | R | `REVISION_CONFLICT` |
| scheduleWorkspaceDeletion | `POST /workspaces/{workspaceId}/deletion` | C | Owner | R | `LAST_OWNER`, `WORKSPACE_STATE_INVALID` |
| cancelWorkspaceDeletion | `DELETE /workspaces/{workspaceId}/deletion` | C | Owner | R | `WORKSPACE_STATE_INVALID` |

## Membership·Group

| Operation ID | Method·path | 종류 | 최소 권한 | 동시성 | 핵심 오류 |
|---|---|---|---|---|---|
| listMembers | `GET /workspaces/{workspaceId}/members` | Q | Member | cursor | — |
| inviteMember | `POST /workspaces/{workspaceId}/invitations` | C | Admin | key | `INVITATION_EXISTS` |
| listInvitations | `GET /workspaces/{workspaceId}/invitations` | Q | Admin | cursor | — |
| revokeInvitation | `DELETE /workspaces/{workspaceId}/invitations/{invitationId}` | C | Admin | R | `INVITATION_STATE_INVALID` |
| acceptInvitation | `POST /invitations/{token}/accept` | C | session | token 1회 | `INVITATION_INVALID` |
| updateMemberRole | `PUT /workspaces/{workspaceId}/members/{userId}/role` | C | Owner | R | `LAST_OWNER` |
| removeMember | `DELETE /workspaces/{workspaceId}/members/{userId}` | C | Admin | R | `LAST_OWNER` |
| listGroups | `GET /workspaces/{workspaceId}/groups` | Q | Member | cursor | — |
| createGroup | `POST /workspaces/{workspaceId}/groups` | C | Admin | key | `GROUP_NAME_TAKEN` |
| getGroup | `GET /workspaces/{workspaceId}/groups/{groupId}` | Q | Member | — | `GROUP_NOT_FOUND` |
| updateGroup | `PUT /workspaces/{workspaceId}/groups/{groupId}` | C | Admin | R | `GROUP_NAME_TAKEN` |
| deleteGroup | `DELETE /workspaces/{workspaceId}/groups/{groupId}` | C | Admin | R | `GROUP_IN_USE` |
| addGroupMember | `PUT /workspaces/{workspaceId}/groups/{groupId}/members/{userId}` | C | Admin | group R | `GROUP_MEMBER_INVALID` |
| removeGroupMember | `DELETE /workspaces/{workspaceId}/groups/{groupId}/members/{userId}` | C | Admin | group R | `GROUP_MEMBER_NOT_FOUND` |

## Document·Draft·Version

| Operation ID | Method·path | 종류 | 최소 권한 | 동시성 | 핵심 오류 |
|---|---|---|---|---|---|
| getDocumentTree | `GET /workspaces/{workspaceId}/documents/tree` | Q | Member | watermark | — |
| listTrashedDocuments | `GET /workspaces/{workspaceId}/documents/trash` | Q | Admin | cursor | — |
| createDocument | `POST /workspaces/{workspaceId}/documents` | C | parent Contributor | key | `DOCUMENT_PARENT_INVALID` |
| getDocument | `GET /workspaces/{workspaceId}/documents/{documentId}` | Q | Viewer | — | `DOCUMENT_NOT_FOUND` |
| updateDocumentMetadata | `PUT /workspaces/{workspaceId}/documents/{documentId}` | C | Contributor | R | `REVISION_CONFLICT` |
| previewDocumentMove | `POST /workspaces/{workspaceId}/documents/{documentId}/move-preview` | Q | Editor | R | `DOCUMENT_TREE_CYCLE` |
| moveDocument | `POST /workspaces/{workspaceId}/documents/{documentId}/move` | C | Editor | R | `DOCUMENT_RANK_CONFLICT` |
| trashDocument | `POST /workspaces/{workspaceId}/documents/{documentId}/trash` | C | Editor | R | `DOCUMENT_STATE_INVALID` |
| restoreDocument | `POST /workspaces/{workspaceId}/documents/{documentId}/restore` | C | Editor | R | `DOCUMENT_PARENT_INVALID` |
| purgeDocument | `DELETE /workspaces/{workspaceId}/documents/{documentId}` | C | Admin+Manage | R | `PURGE_NOT_ELIGIBLE` |
| getDraft | `GET /workspaces/{workspaceId}/documents/{documentId}/draft` | Q | Contributor | — | `DRAFT_NOT_FOUND` |
| createOrGetDraft | `POST /workspaces/{workspaceId}/documents/{documentId}/draft` | C | Contributor | key | `DOCUMENT_STATE_INVALID` |
| applyDraftOperations | `POST /workspaces/{workspaceId}/documents/{documentId}/draft/operations` | C | Contributor | R+L | `OPERATION_PRECONDITION_FAILED` |
| acquireEditLease | `POST /workspaces/{workspaceId}/documents/{documentId}/lease` | C | Contributor | R | `EDIT_LEASE_HELD` |
| renewEditLease | `POST /workspaces/{workspaceId}/documents/{documentId}/lease/renew` | C | lease holder | R+L | `EDIT_LEASE_INVALID` |
| releaseEditLease | `DELETE /workspaces/{workspaceId}/documents/{documentId}/lease` | C | lease holder | R+L | `EDIT_LEASE_INVALID` |
| listVersions | `GET /workspaces/{workspaceId}/documents/{documentId}/versions` | Q | Viewer | cursor | — |
| getVersion | `GET /workspaces/{workspaceId}/documents/{documentId}/versions/{versionId}` | Q | Viewer | — | `VERSION_NOT_FOUND` |
| compareVersions | `GET /workspaces/{workspaceId}/documents/{documentId}/version-diff` | Q | Viewer | from+to | `VERSION_NOT_FOUND` |
| restoreVersionToDraft | `POST /workspaces/{workspaceId}/documents/{documentId}/versions/{versionId}/restore` | C | Editor | R | `DRAFT_EXISTS` |
| publishDocument | `POST /workspaces/{workspaceId}/documents/{documentId}/publish` | C | Editor | R+L | `PUBLISH_REVIEW_REQUIRED` |

## Permission·Policy·Public Link

| Operation ID | Method·path | 종류 | 최소 권한 | 동시성 | 핵심 오류 |
|---|---|---|---|---|---|
| getDocumentPermissions | `GET /workspaces/{workspaceId}/documents/{documentId}/permissions` | Q | Manage | — | `PERMISSION_DENIED` |
| setDocumentPermission | `PUT /workspaces/{workspaceId}/documents/{documentId}/permissions/{grantId}` | C | Manage | R | `PERMISSION_SUBJECT_INVALID` |
| deleteDocumentPermission | `DELETE /workspaces/{workspaceId}/documents/{documentId}/permissions/{grantId}` | C | Manage | R | `PERMISSION_LAST_MANAGER` |
| explainEffectivePermission | `GET /workspaces/{workspaceId}/documents/{documentId}/permission-explanation` | Q | self or Manage | subject | `PERMISSION_DENIED` |
| getPublishPolicy | `GET /workspaces/{workspaceId}/documents/{documentId}/publish-policy` | Q | Viewer | — | — |
| setPublishPolicy | `PUT /workspaces/{workspaceId}/documents/{documentId}/publish-policy` | C | Manage | R | `PUBLISH_POLICY_INVALID` |
| listPublicLinks | `GET /workspaces/{workspaceId}/documents/{documentId}/public-links` | Q | Manage | — | — |
| createPublicLink | `POST /workspaces/{workspaceId}/documents/{documentId}/public-links` | C | Manage | R | `DOCUMENT_UNPUBLISHED` |
| revokePublicLink | `DELETE /workspaces/{workspaceId}/documents/{documentId}/public-links/{linkId}` | C | Manage | R | `PUBLIC_LINK_STATE_INVALID` |
| getPublicDocument | `GET /public/v1/documents/{token}` | Q | capability | — | `PUBLIC_LINK_INVALID` |

## Discussion·Review·Inbox

| Operation ID | Method·path | 종류 | 최소 권한 | 동시성 | 핵심 오류 |
|---|---|---|---|---|---|
| listDiscussions | `GET /workspaces/{workspaceId}/documents/{documentId}/discussions` | Q | Viewer | cursor | — |
| createDiscussion | `POST /workspaces/{workspaceId}/documents/{documentId}/discussions` | C | Contributor | key | `DISCUSSION_TARGET_INVALID` |
| getDiscussion | `GET /workspaces/{workspaceId}/discussions/{discussionId}` | Q | Viewer | — | `DISCUSSION_NOT_FOUND` |
| updateDiscussion | `PUT /workspaces/{workspaceId}/discussions/{discussionId}` | C | author or Editor | R | `REVISION_CONFLICT` |
| closeDiscussion | `POST /workspaces/{workspaceId}/discussions/{discussionId}/close` | C | Contributor | R | `DISCUSSION_STATE_INVALID` |
| reopenDiscussion | `POST /workspaces/{workspaceId}/discussions/{discussionId}/reopen` | C | Contributor | R | `DISCUSSION_STATE_INVALID` |
| addDiscussionTopic | `POST /workspaces/{workspaceId}/discussions/{discussionId}/topics` | C | Contributor | R | `DISCUSSION_TARGET_INVALID` |
| removeDiscussionTopic | `DELETE /workspaces/{workspaceId}/discussions/{discussionId}/topics/{topicId}` | C | author or Editor | R | `DISCUSSION_TOPIC_REQUIRED` |
| createMessage | `POST /workspaces/{workspaceId}/discussions/{discussionId}/messages` | C | Contributor | key | `DISCUSSION_CLOSED` |
| updateMessage | `PUT /workspaces/{workspaceId}/discussions/{discussionId}/messages/{messageId}` | C | author | R | `MESSAGE_EDIT_WINDOW_EXPIRED` |
| deleteMessage | `DELETE /workspaces/{workspaceId}/discussions/{discussionId}/messages/{messageId}` | C | author or Editor | R | `MESSAGE_STATE_INVALID` |
| requestReview | `POST /workspaces/{workspaceId}/documents/{documentId}/reviews` | C | Contributor | draft R | `REVIEW_ALREADY_OPEN` |
| getReview | `GET /workspaces/{workspaceId}/reviews/{reviewId}` | Q | Viewer | — | `REVIEW_NOT_FOUND` |
| submitReviewDecision | `POST /workspaces/{workspaceId}/reviews/{reviewId}/decisions` | C | assigned reviewer | review R | `REVIEW_STALE` |
| cancelReview | `POST /workspaces/{workspaceId}/reviews/{reviewId}/cancel` | C | requester or Editor | review R | `REVIEW_STATE_INVALID` |
| listInbox | `GET /workspaces/{workspaceId}/inbox` | Q | self | cursor | — |
| markInboxItemRead | `POST /workspaces/{workspaceId}/inbox/{itemId}/read` | C | self | idempotent | `INBOX_ITEM_NOT_FOUND` |
| markAllInboxRead | `POST /workspaces/{workspaceId}/inbox/read-all` | C | self | before cursor | — |
| resolveInboxItem | `POST /workspaces/{workspaceId}/inbox/{itemId}/resolve` | C | self | idempotent | `INBOX_ITEM_NOT_FOUND` |

## Knowledge·AI

| Operation ID | Method·path | 종류 | 최소 권한 | 동시성 | 핵심 오류 |
|---|---|---|---|---|---|
| searchKnowledge | `GET /workspaces/{workspaceId}/search` | Q | Member | cursor | `SEARCH_UNAVAILABLE` |
| listBacklinks | `GET /workspaces/{workspaceId}/documents/{documentId}/backlinks` | Q | Viewer | cursor | — |
| createReference | `POST /workspaces/{workspaceId}/documents/{documentId}/references` | C | Contributor | draft R+L | `REFERENCE_TARGET_INVALID` |
| deleteReference | `DELETE /workspaces/{workspaceId}/documents/{documentId}/references/{referenceId}` | C | Contributor | draft R+L | `REFERENCE_NOT_FOUND` |
| listVocabulary | `GET /workspaces/{workspaceId}/vocabulary` | Q | Member | cursor | — |
| createVocabularyConcept | `POST /workspaces/{workspaceId}/vocabulary` | C | Admin | key | `VOCABULARY_TERM_CONFLICT` |
| getVocabularyConcept | `GET /workspaces/{workspaceId}/vocabulary/{conceptId}` | Q | Member | — | `VOCABULARY_NOT_FOUND` |
| updateVocabularyConcept | `PUT /workspaces/{workspaceId}/vocabulary/{conceptId}` | C | Admin | R | `VOCABULARY_TERM_CONFLICT` |
| deprecateVocabularyConcept | `POST /workspaces/{workspaceId}/vocabulary/{conceptId}/deprecate` | C | Admin | R | `VOCABULARY_STATE_INVALID` |
| createAIJob | `POST /workspaces/{workspaceId}/ai/jobs` | C | target Contributor | key+target R | `AI_QUOTA_EXCEEDED` |
| listAIJobs | `GET /workspaces/{workspaceId}/ai/jobs` | Q | self | cursor | — |
| getAIJob | `GET /workspaces/{workspaceId}/ai/jobs/{jobId}` | Q | owner | — | `AI_JOB_NOT_FOUND` |
| cancelAIJob | `DELETE /workspaces/{workspaceId}/ai/jobs/{jobId}` | C | owner | job R | `AI_JOB_STATE_INVALID` |
| getProposal | `GET /workspaces/{workspaceId}/proposals/{proposalId}` | Q | target Contributor | — | `PROPOSAL_NOT_FOUND` |
| applyProposal | `POST /workspaces/{workspaceId}/proposals/{proposalId}/apply` | C | target Contributor | draft R+L | `PROPOSAL_STALE` |
| rejectProposal | `POST /workspaces/{workspaceId}/proposals/{proposalId}/reject` | C | target Contributor | proposal R | `PROPOSAL_STATE_INVALID` |

## File·Audit·Stream

| Operation ID | Method·path | 종류 | 최소 권한 | 동시성 | 핵심 오류 |
|---|---|---|---|---|---|
| createFileUpload | `POST /workspaces/{workspaceId}/files/uploads` | C | Member | key | `FILE_LIMIT_EXCEEDED` |
| completeFileUpload | `POST /workspaces/{workspaceId}/files/{assetId}/complete` | C | uploader | asset R | `FILE_CHECKSUM_MISMATCH` |
| getFile | `GET /workspaces/{workspaceId}/files/{assetId}` | Q | referenced-resource access | — | `FILE_NOT_FOUND` |
| downloadFile | `GET /workspaces/{workspaceId}/files/{assetId}/content` | Q | referenced-resource access | range | `FILE_NOT_READY` |
| deleteFile | `DELETE /workspaces/{workspaceId}/files/{assetId}` | C | uploader or Admin | asset R | `FILE_STILL_REFERENCED` |
| listAuditEvents | `GET /workspaces/{workspaceId}/audit-events` | Q | Admin | sequence cursor | — |
| openWorkspaceStream | `GET /stream?workspaceId=...` | Q/SSE | Member | event cursor | `STREAM_CURSOR_EXPIRED` |

## Workspace configuration

| Operation ID | Method·path | 종류 | 최소 권한 | 동시성 | 핵심 오류 |
|---|---|---|---|---|---|
| getWritingConfiguration | `GET /workspaces/{workspaceId}/writing-configuration` | Q | Member | — | — |
| updateWritingConfiguration | `PUT /workspaces/{workspaceId}/writing-configuration` | C | Admin | R | `WRITING_CONFIGURATION_INVALID` |
| getAIConfiguration | `GET /workspaces/{workspaceId}/ai/configuration` | Q | Admin | — | — |
| updateAIConfiguration | `PUT /workspaces/{workspaceId}/ai/configuration` | C | Admin | R | `AI_CONFIGURATION_INVALID` |
| getAIUsage | `GET /workspaces/{workspaceId}/ai/usage` | Q | Admin | period | `AI_USAGE_UNAVAILABLE` |
| getAIProviderHealth | `GET /workspaces/{workspaceId}/ai/provider-health` | Q | Admin | — | `AI_PROVIDER_UNAVAILABLE` |

## 완전성 규칙

Catalog의 Operation ID는 OpenAPI operationId와 정확히 1:1이어야 한다. command는
[Database Matrix](../data/DATABASE-INVARIANT-TRANSACTION-MATRIX.md)의 transaction 또는 명시적
read-only owner를 가진다. 화면에서 호출하지 않는 내부 endpoint도 contract test를 가지며,
화면 action은 [Endpoint Coverage](ENDPOINT-COVERAGE.md)에 역방향으로 연결한다.
