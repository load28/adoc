# Endpoint Coverage

- **문서 ID**: API-08
- **상태**: 동결
- **정본 목록**: [Command·Query Catalog](COMMAND-QUERY-CATALOG.md)

## Screen→API

| Screen | Query Operation | Command Operation |
|---|---|---|
| SCR-01~03 Identity | `getSession`, `listWorkspaces` | `beginGoogleLogin`, `completeGoogleLogin`, `logout`, `createWorkspace`, `acceptInvitation` |
| SCR-04 Workspace home | `getWorkspace`, `getDocumentTree` | `createDocument` |
| SCR-05 Published Document | `getDocument`, `getPublishPolicy`, `listPublicLinks` | `createOrGetDraft`, `publishDocument`, `createPublicLink`, `revokePublicLink`, `trashDocument` |
| SCR-06 Draft Editor | `getDocument`, `getDraft` | `acquireEditLease`, `renewEditLease`, `releaseEditLease`, `applyDraftOperations`, `createReference`, `deleteReference`, `createFileUpload`, `completeFileUpload` |
| SCR-07 Discussion | `listDiscussions`, `getDiscussion` | `createDiscussion`, `updateDiscussion`, `closeDiscussion`, `reopenDiscussion`, `addDiscussionTopic`, `removeDiscussionTopic`, `createMessage`, `updateMessage`, `deleteMessage` |
| SCR-08 Review | `getReview` | `requestReview`, `submitReviewDecision`, `cancelReview` |
| SCR-09 History | `listVersions`, `getVersion`, `compareVersions` | `createOrGetDraft` |
| SCR-10 References | `listBacklinks` | target navigation only |
| SCR-11 AI | `listAIJobs`, `getAIJob`, `getProposal` | `createAIJob`, `cancelAIJob`, `applyProposal`, `rejectProposal` |
| SCR-12 Search | `searchKnowledge` | target navigation only |
| SCR-13 Inbox | `listInbox` | `markInboxItemRead`, `markAllInboxRead`, `resolveInboxItem` |
| SCR-14 Vocabulary | `listVocabulary`, `getVocabularyConcept` | `createVocabularyConcept`, `updateVocabularyConcept`, `deprecateVocabularyConcept` |
| SCR-15 Trash | `listTrashedDocuments` | `restoreDocument`, `purgeDocument` |
| SCR-16 Members | `listMembers`, `listInvitations` | `inviteMember`, `revokeInvitation`, `updateMemberRole`, `removeMember` |
| SCR-17 Groups | `listGroups`, `getGroup` | `createGroup`, `updateGroup`, `deleteGroup`, `addGroupMember`, `removeGroupMember` |
| SCR-18 Permission | `getDocumentPermissions`, `explainEffectivePermission`, `getPublishPolicy` | `setDocumentPermission`, `deleteDocumentPermission`, `setPublishPolicy`, `previewDocumentMove`, `moveDocument` |
| SCR-19 Writing Settings | `getWritingConfiguration` | `updateWritingConfiguration` |
| SCR-20 AI Settings | `getAIConfiguration`, `getAIUsage`, `getAIProviderHealth` | `updateAIConfiguration` |
| SCR-21 Audit | `listAuditEvents` | 없음 |
| SCR-22 Public | `getPublicDocument` | 없음 |

Workspace settings의 `updateWorkspace`, `scheduleWorkspaceDeletion`, `cancelWorkspaceDeletion`과
File metadata의 `getFile`, `downloadFile`, `deleteFile`은 해당 settings·Document attachment UI에서
호출한다. `openWorkspaceStream`은 인증된 모든 Workspace shell에서 한 번 연결한다.

## Contract gate

| Gate | 검사 |
|---|---|
| Operation 완전성 | Catalog Operation ID 집합 = OpenAPI operationId 집합 |
| Route 완전성 | 모든 path variable에 required path parameter 존재 |
| Mutation 안전성 | 모든 C operation에 Idempotency Key, mutable target에는 If-Match, Draft에는 Lease token |
| Schema 완전성 | 모든 internal `$ref` 해소, Content·Operation·AI·Event 외부 schema 해소 |
| UI 완전성 | 모든 SCR primary/secondary action이 Operation ID 또는 local navigation으로 분류 |
| Test 완전성 | Operation마다 success, unauthorized, forbidden/not-found, validation, conflict/idempotency case |

예외는 `beginGoogleLogin`, `completeGoogleLogin`, `logout`, `acceptInvitation`, read marker처럼
자연적으로 revision target이 없는 command뿐이다. 예외도 idempotency 또는 OAuth state/token
single-use contract를 가진다. gate script는 catalog·OpenAPI·screen·contract coverage의 집합 차이를
0으로 검사한다.
