/* Generated from canonical OpenAPI and AsyncAPI. Do not edit. */
export const operations = [
  {
    "operationId": "getSession",
    "method": "GET",
    "path": "/session",
    "request": "Operation__GetSessionRequest",
    "response": "Operation__GetSessionResponse"
  },
  {
    "operationId": "beginGoogleLogin",
    "method": "GET",
    "path": "/auth/google/start",
    "request": "Operation__BeginGoogleLoginRequest",
    "response": "Operation__BeginGoogleLoginResponse"
  },
  {
    "operationId": "completeGoogleLogin",
    "method": "GET",
    "path": "/auth/google/callback",
    "request": "Operation__CompleteGoogleLoginRequest",
    "response": "Operation__CompleteGoogleLoginResponse"
  },
  {
    "operationId": "logout",
    "method": "POST",
    "path": "/session/logout",
    "request": "Operation__LogoutRequest",
    "response": "Operation__LogoutResponse"
  },
  {
    "operationId": "getUserPreferences",
    "method": "GET",
    "path": "/preferences",
    "request": "Operation__GetUserPreferencesRequest",
    "response": "Operation__GetUserPreferencesResponse"
  },
  {
    "operationId": "updateUserPreferences",
    "method": "PUT",
    "path": "/preferences",
    "request": "Operation__UpdateUserPreferencesRequest",
    "response": "Operation__UpdateUserPreferencesResponse"
  },
  {
    "operationId": "listWorkspaces",
    "method": "GET",
    "path": "/workspaces",
    "request": "Operation__ListWorkspacesRequest",
    "response": "Operation__ListWorkspacesResponse"
  },
  {
    "operationId": "createWorkspace",
    "method": "POST",
    "path": "/workspaces",
    "request": "Operation__CreateWorkspaceRequest",
    "response": "Operation__CreateWorkspaceResponse"
  },
  {
    "operationId": "getWorkspace",
    "method": "GET",
    "path": "/workspaces/{workspaceId}",
    "request": "Operation__GetWorkspaceRequest",
    "response": "Operation__GetWorkspaceResponse"
  },
  {
    "operationId": "updateWorkspace",
    "method": "PUT",
    "path": "/workspaces/{workspaceId}",
    "request": "Operation__UpdateWorkspaceRequest",
    "response": "Operation__UpdateWorkspaceResponse"
  },
  {
    "operationId": "scheduleWorkspaceDeletion",
    "method": "POST",
    "path": "/workspaces/{workspaceId}/deletion",
    "request": "Operation__ScheduleWorkspaceDeletionRequest",
    "response": "Operation__ScheduleWorkspaceDeletionResponse"
  },
  {
    "operationId": "cancelWorkspaceDeletion",
    "method": "DELETE",
    "path": "/workspaces/{workspaceId}/deletion",
    "request": "Operation__CancelWorkspaceDeletionRequest",
    "response": "Operation__CancelWorkspaceDeletionResponse"
  },
  {
    "operationId": "listMembers",
    "method": "GET",
    "path": "/workspaces/{workspaceId}/members",
    "request": "Operation__ListMembersRequest",
    "response": "Operation__ListMembersResponse"
  },
  {
    "operationId": "updateMemberRole",
    "method": "PUT",
    "path": "/workspaces/{workspaceId}/members/{userId}/role",
    "request": "Operation__UpdateMemberRoleRequest",
    "response": "Operation__UpdateMemberRoleResponse"
  },
  {
    "operationId": "removeMember",
    "method": "DELETE",
    "path": "/workspaces/{workspaceId}/members/{userId}",
    "request": "Operation__RemoveMemberRequest",
    "response": "Operation__RemoveMemberResponse"
  },
  {
    "operationId": "listInvitations",
    "method": "GET",
    "path": "/workspaces/{workspaceId}/invitations",
    "request": "Operation__ListInvitationsRequest",
    "response": "Operation__ListInvitationsResponse"
  },
  {
    "operationId": "inviteMember",
    "method": "POST",
    "path": "/workspaces/{workspaceId}/invitations",
    "request": "Operation__InviteMemberRequest",
    "response": "Operation__InviteMemberResponse"
  },
  {
    "operationId": "revokeInvitation",
    "method": "DELETE",
    "path": "/workspaces/{workspaceId}/invitations/{invitationId}",
    "request": "Operation__RevokeInvitationRequest",
    "response": "Operation__RevokeInvitationResponse"
  },
  {
    "operationId": "acceptInvitation",
    "method": "POST",
    "path": "/invitations/{token}/accept",
    "request": "Operation__AcceptInvitationRequest",
    "response": "Operation__AcceptInvitationResponse"
  },
  {
    "operationId": "listGroups",
    "method": "GET",
    "path": "/workspaces/{workspaceId}/groups",
    "request": "Operation__ListGroupsRequest",
    "response": "Operation__ListGroupsResponse"
  },
  {
    "operationId": "createGroup",
    "method": "POST",
    "path": "/workspaces/{workspaceId}/groups",
    "request": "Operation__CreateGroupRequest",
    "response": "Operation__CreateGroupResponse"
  },
  {
    "operationId": "getGroup",
    "method": "GET",
    "path": "/workspaces/{workspaceId}/groups/{groupId}",
    "request": "Operation__GetGroupRequest",
    "response": "Operation__GetGroupResponse"
  },
  {
    "operationId": "updateGroup",
    "method": "PUT",
    "path": "/workspaces/{workspaceId}/groups/{groupId}",
    "request": "Operation__UpdateGroupRequest",
    "response": "Operation__UpdateGroupResponse"
  },
  {
    "operationId": "deleteGroup",
    "method": "DELETE",
    "path": "/workspaces/{workspaceId}/groups/{groupId}",
    "request": "Operation__DeleteGroupRequest",
    "response": "Operation__DeleteGroupResponse"
  },
  {
    "operationId": "addGroupMember",
    "method": "PUT",
    "path": "/workspaces/{workspaceId}/groups/{groupId}/members/{userId}",
    "request": "Operation__AddGroupMemberRequest",
    "response": "Operation__AddGroupMemberResponse"
  },
  {
    "operationId": "removeGroupMember",
    "method": "DELETE",
    "path": "/workspaces/{workspaceId}/groups/{groupId}/members/{userId}",
    "request": "Operation__RemoveGroupMemberRequest",
    "response": "Operation__RemoveGroupMemberResponse"
  },
  {
    "operationId": "createDocument",
    "method": "POST",
    "path": "/workspaces/{workspaceId}/documents",
    "request": "Operation__CreateDocumentRequest",
    "response": "Operation__CreateDocumentResponse"
  },
  {
    "operationId": "getDocumentTree",
    "method": "GET",
    "path": "/workspaces/{workspaceId}/documents/tree",
    "request": "Operation__GetDocumentTreeRequest",
    "response": "Operation__GetDocumentTreeResponse"
  },
  {
    "operationId": "listTrashedDocuments",
    "method": "GET",
    "path": "/workspaces/{workspaceId}/documents/trash",
    "request": "Operation__ListTrashedDocumentsRequest",
    "response": "Operation__ListTrashedDocumentsResponse"
  },
  {
    "operationId": "getDocument",
    "method": "GET",
    "path": "/workspaces/{workspaceId}/documents/{documentId}",
    "request": "Operation__GetDocumentRequest",
    "response": "Operation__GetDocumentResponse"
  },
  {
    "operationId": "updateDocumentMetadata",
    "method": "PUT",
    "path": "/workspaces/{workspaceId}/documents/{documentId}",
    "request": "Operation__UpdateDocumentMetadataRequest",
    "response": "Operation__UpdateDocumentMetadataResponse"
  },
  {
    "operationId": "purgeDocument",
    "method": "DELETE",
    "path": "/workspaces/{workspaceId}/documents/{documentId}",
    "request": "Operation__PurgeDocumentRequest",
    "response": "Operation__PurgeDocumentResponse"
  },
  {
    "operationId": "trashDocument",
    "method": "POST",
    "path": "/workspaces/{workspaceId}/documents/{documentId}/trash",
    "request": "Operation__TrashDocumentRequest",
    "response": "Operation__TrashDocumentResponse"
  },
  {
    "operationId": "restoreDocument",
    "method": "POST",
    "path": "/workspaces/{workspaceId}/documents/{documentId}/restore",
    "request": "Operation__RestoreDocumentRequest",
    "response": "Operation__RestoreDocumentResponse"
  },
  {
    "operationId": "previewDocumentMove",
    "method": "POST",
    "path": "/workspaces/{workspaceId}/documents/{documentId}/move-preview",
    "request": "Operation__PreviewDocumentMoveRequest",
    "response": "Operation__PreviewDocumentMoveResponse"
  },
  {
    "operationId": "moveDocument",
    "method": "POST",
    "path": "/workspaces/{workspaceId}/documents/{documentId}/move",
    "request": "Operation__MoveDocumentRequest",
    "response": "Operation__MoveDocumentResponse"
  },
  {
    "operationId": "getDocumentPermissions",
    "method": "GET",
    "path": "/workspaces/{workspaceId}/documents/{documentId}/permissions",
    "request": "Operation__GetDocumentPermissionsRequest",
    "response": "Operation__GetDocumentPermissionsResponse"
  },
  {
    "operationId": "setDocumentPermission",
    "method": "PUT",
    "path": "/workspaces/{workspaceId}/documents/{documentId}/permissions/{grantId}",
    "request": "Operation__SetDocumentPermissionRequest",
    "response": "Operation__SetDocumentPermissionResponse"
  },
  {
    "operationId": "deleteDocumentPermission",
    "method": "DELETE",
    "path": "/workspaces/{workspaceId}/documents/{documentId}/permissions/{grantId}",
    "request": "Operation__DeleteDocumentPermissionRequest",
    "response": "Operation__DeleteDocumentPermissionResponse"
  },
  {
    "operationId": "explainEffectivePermission",
    "method": "GET",
    "path": "/workspaces/{workspaceId}/documents/{documentId}/permission-explanation",
    "request": "Operation__ExplainEffectivePermissionRequest",
    "response": "Operation__ExplainEffectivePermissionResponse"
  },
  {
    "operationId": "getPublishPolicy",
    "method": "GET",
    "path": "/workspaces/{workspaceId}/documents/{documentId}/publish-policy",
    "request": "Operation__GetPublishPolicyRequest",
    "response": "Operation__GetPublishPolicyResponse"
  },
  {
    "operationId": "setPublishPolicy",
    "method": "PUT",
    "path": "/workspaces/{workspaceId}/documents/{documentId}/publish-policy",
    "request": "Operation__SetPublishPolicyRequest",
    "response": "Operation__SetPublishPolicyResponse"
  },
  {
    "operationId": "getDraft",
    "method": "GET",
    "path": "/workspaces/{workspaceId}/documents/{documentId}/draft",
    "request": "Operation__GetDraftRequest",
    "response": "Operation__GetDraftResponse"
  },
  {
    "operationId": "createOrGetDraft",
    "method": "POST",
    "path": "/workspaces/{workspaceId}/documents/{documentId}/draft",
    "request": "Operation__CreateOrGetDraftRequest",
    "response": "Operation__CreateOrGetDraftResponse"
  },
  {
    "operationId": "acquireEditLease",
    "method": "POST",
    "path": "/workspaces/{workspaceId}/documents/{documentId}/lease",
    "request": "Operation__AcquireEditLeaseRequest",
    "response": "Operation__AcquireEditLeaseResponse"
  },
  {
    "operationId": "releaseEditLease",
    "method": "DELETE",
    "path": "/workspaces/{workspaceId}/documents/{documentId}/lease",
    "request": "Operation__ReleaseEditLeaseRequest",
    "response": "Operation__ReleaseEditLeaseResponse"
  },
  {
    "operationId": "renewEditLease",
    "method": "POST",
    "path": "/workspaces/{workspaceId}/documents/{documentId}/lease/renew",
    "request": "Operation__RenewEditLeaseRequest",
    "response": "Operation__RenewEditLeaseResponse"
  },
  {
    "operationId": "applyDraftOperations",
    "method": "POST",
    "path": "/workspaces/{workspaceId}/documents/{documentId}/draft/operations",
    "request": "Operation__ApplyDraftOperationsRequest",
    "response": "Operation__ApplyDraftOperationsResponse"
  },
  {
    "operationId": "requestReview",
    "method": "POST",
    "path": "/workspaces/{workspaceId}/documents/{documentId}/reviews",
    "request": "Operation__RequestReviewRequest",
    "response": "Operation__RequestReviewResponse"
  },
  {
    "operationId": "listVersions",
    "method": "GET",
    "path": "/workspaces/{workspaceId}/documents/{documentId}/versions",
    "request": "Operation__ListVersionsRequest",
    "response": "Operation__ListVersionsResponse"
  },
  {
    "operationId": "getVersion",
    "method": "GET",
    "path": "/workspaces/{workspaceId}/documents/{documentId}/versions/{versionId}",
    "request": "Operation__GetVersionRequest",
    "response": "Operation__GetVersionResponse"
  },
  {
    "operationId": "restoreVersionToDraft",
    "method": "POST",
    "path": "/workspaces/{workspaceId}/documents/{documentId}/versions/{versionId}/restore",
    "request": "Operation__RestoreVersionToDraftRequest",
    "response": "Operation__RestoreVersionToDraftResponse"
  },
  {
    "operationId": "compareVersions",
    "method": "GET",
    "path": "/workspaces/{workspaceId}/documents/{documentId}/version-diff",
    "request": "Operation__CompareVersionsRequest",
    "response": "Operation__CompareVersionsResponse"
  },
  {
    "operationId": "submitReviewDecision",
    "method": "POST",
    "path": "/workspaces/{workspaceId}/reviews/{reviewId}/decisions",
    "request": "Operation__SubmitReviewDecisionRequest",
    "response": "Operation__SubmitReviewDecisionResponse"
  },
  {
    "operationId": "publishDocument",
    "method": "POST",
    "path": "/workspaces/{workspaceId}/documents/{documentId}/publish",
    "request": "Operation__PublishDocumentRequest",
    "response": "Operation__PublishDocumentResponse"
  },
  {
    "operationId": "listDiscussions",
    "method": "GET",
    "path": "/workspaces/{workspaceId}/documents/{documentId}/discussions",
    "request": "Operation__ListDiscussionsRequest",
    "response": "Operation__ListDiscussionsResponse"
  },
  {
    "operationId": "createDiscussion",
    "method": "POST",
    "path": "/workspaces/{workspaceId}/documents/{documentId}/discussions",
    "request": "Operation__CreateDiscussionRequest",
    "response": "Operation__CreateDiscussionResponse"
  },
  {
    "operationId": "getDiscussion",
    "method": "GET",
    "path": "/workspaces/{workspaceId}/discussions/{discussionId}",
    "request": "Operation__GetDiscussionRequest",
    "response": "Operation__GetDiscussionResponse"
  },
  {
    "operationId": "updateDiscussion",
    "method": "PUT",
    "path": "/workspaces/{workspaceId}/discussions/{discussionId}",
    "request": "Operation__UpdateDiscussionRequest",
    "response": "Operation__UpdateDiscussionResponse"
  },
  {
    "operationId": "closeDiscussion",
    "method": "POST",
    "path": "/workspaces/{workspaceId}/discussions/{discussionId}/close",
    "request": "Operation__CloseDiscussionRequest",
    "response": "Operation__CloseDiscussionResponse"
  },
  {
    "operationId": "reopenDiscussion",
    "method": "POST",
    "path": "/workspaces/{workspaceId}/discussions/{discussionId}/reopen",
    "request": "Operation__ReopenDiscussionRequest",
    "response": "Operation__ReopenDiscussionResponse"
  },
  {
    "operationId": "addDiscussionTopic",
    "method": "POST",
    "path": "/workspaces/{workspaceId}/discussions/{discussionId}/topics",
    "request": "Operation__AddDiscussionTopicRequest",
    "response": "Operation__AddDiscussionTopicResponse"
  },
  {
    "operationId": "removeDiscussionTopic",
    "method": "DELETE",
    "path": "/workspaces/{workspaceId}/discussions/{discussionId}/topics/{topicId}",
    "request": "Operation__RemoveDiscussionTopicRequest",
    "response": "Operation__RemoveDiscussionTopicResponse"
  },
  {
    "operationId": "createMessage",
    "method": "POST",
    "path": "/workspaces/{workspaceId}/discussions/{discussionId}/messages",
    "request": "Operation__CreateMessageRequest",
    "response": "Operation__CreateMessageResponse"
  },
  {
    "operationId": "updateMessage",
    "method": "PUT",
    "path": "/workspaces/{workspaceId}/discussions/{discussionId}/messages/{messageId}",
    "request": "Operation__UpdateMessageRequest",
    "response": "Operation__UpdateMessageResponse"
  },
  {
    "operationId": "deleteMessage",
    "method": "DELETE",
    "path": "/workspaces/{workspaceId}/discussions/{discussionId}/messages/{messageId}",
    "request": "Operation__DeleteMessageRequest",
    "response": "Operation__DeleteMessageResponse"
  },
  {
    "operationId": "getReview",
    "method": "GET",
    "path": "/workspaces/{workspaceId}/reviews/{reviewId}",
    "request": "Operation__GetReviewRequest",
    "response": "Operation__GetReviewResponse"
  },
  {
    "operationId": "cancelReview",
    "method": "POST",
    "path": "/workspaces/{workspaceId}/reviews/{reviewId}/cancel",
    "request": "Operation__CancelReviewRequest",
    "response": "Operation__CancelReviewResponse"
  },
  {
    "operationId": "listInbox",
    "method": "GET",
    "path": "/workspaces/{workspaceId}/inbox",
    "request": "Operation__ListInboxRequest",
    "response": "Operation__ListInboxResponse"
  },
  {
    "operationId": "markInboxItemRead",
    "method": "POST",
    "path": "/workspaces/{workspaceId}/inbox/{itemId}/read",
    "request": "Operation__MarkInboxItemReadRequest",
    "response": "Operation__MarkInboxItemReadResponse"
  },
  {
    "operationId": "markAllInboxRead",
    "method": "POST",
    "path": "/workspaces/{workspaceId}/inbox/read-all",
    "request": "Operation__MarkAllInboxReadRequest",
    "response": "Operation__MarkAllInboxReadResponse"
  },
  {
    "operationId": "resolveInboxItem",
    "method": "POST",
    "path": "/workspaces/{workspaceId}/inbox/{itemId}/resolve",
    "request": "Operation__ResolveInboxItemRequest",
    "response": "Operation__ResolveInboxItemResponse"
  },
  {
    "operationId": "searchKnowledge",
    "method": "GET",
    "path": "/workspaces/{workspaceId}/search",
    "request": "Operation__SearchKnowledgeRequest",
    "response": "Operation__SearchKnowledgeResponse"
  },
  {
    "operationId": "listBacklinks",
    "method": "GET",
    "path": "/workspaces/{workspaceId}/documents/{documentId}/backlinks",
    "request": "Operation__ListBacklinksRequest",
    "response": "Operation__ListBacklinksResponse"
  },
  {
    "operationId": "createReference",
    "method": "POST",
    "path": "/workspaces/{workspaceId}/documents/{documentId}/references",
    "request": "Operation__CreateReferenceRequest",
    "response": "Operation__CreateReferenceResponse"
  },
  {
    "operationId": "deleteReference",
    "method": "DELETE",
    "path": "/workspaces/{workspaceId}/documents/{documentId}/references/{referenceId}",
    "request": "Operation__DeleteReferenceRequest",
    "response": "Operation__DeleteReferenceResponse"
  },
  {
    "operationId": "listVocabulary",
    "method": "GET",
    "path": "/workspaces/{workspaceId}/vocabulary",
    "request": "Operation__ListVocabularyRequest",
    "response": "Operation__ListVocabularyResponse"
  },
  {
    "operationId": "createVocabularyConcept",
    "method": "POST",
    "path": "/workspaces/{workspaceId}/vocabulary",
    "request": "Operation__CreateVocabularyConceptRequest",
    "response": "Operation__CreateVocabularyConceptResponse"
  },
  {
    "operationId": "getVocabularyConcept",
    "method": "GET",
    "path": "/workspaces/{workspaceId}/vocabulary/{conceptId}",
    "request": "Operation__GetVocabularyConceptRequest",
    "response": "Operation__GetVocabularyConceptResponse"
  },
  {
    "operationId": "updateVocabularyConcept",
    "method": "PUT",
    "path": "/workspaces/{workspaceId}/vocabulary/{conceptId}",
    "request": "Operation__UpdateVocabularyConceptRequest",
    "response": "Operation__UpdateVocabularyConceptResponse"
  },
  {
    "operationId": "deprecateVocabularyConcept",
    "method": "POST",
    "path": "/workspaces/{workspaceId}/vocabulary/{conceptId}/deprecate",
    "request": "Operation__DeprecateVocabularyConceptRequest",
    "response": "Operation__DeprecateVocabularyConceptResponse"
  },
  {
    "operationId": "listAIJobs",
    "method": "GET",
    "path": "/workspaces/{workspaceId}/ai/jobs",
    "request": "Operation__ListAIJobsRequest",
    "response": "Operation__ListAIJobsResponse"
  },
  {
    "operationId": "createAIJob",
    "method": "POST",
    "path": "/workspaces/{workspaceId}/ai/jobs",
    "request": "Operation__CreateAIJobRequest",
    "response": "Operation__CreateAIJobResponse"
  },
  {
    "operationId": "getAIJob",
    "method": "GET",
    "path": "/workspaces/{workspaceId}/ai/jobs/{jobId}",
    "request": "Operation__GetAIJobRequest",
    "response": "Operation__GetAIJobResponse"
  },
  {
    "operationId": "cancelAIJob",
    "method": "DELETE",
    "path": "/workspaces/{workspaceId}/ai/jobs/{jobId}",
    "request": "Operation__CancelAIJobRequest",
    "response": "Operation__CancelAIJobResponse"
  },
  {
    "operationId": "previewAIContext",
    "method": "POST",
    "path": "/workspaces/{workspaceId}/ai/context-preview",
    "request": "Operation__PreviewAIContextRequest",
    "response": "Operation__PreviewAIContextResponse"
  },
  {
    "operationId": "applyProposal",
    "method": "POST",
    "path": "/workspaces/{workspaceId}/proposals/{proposalId}/apply",
    "request": "Operation__ApplyProposalRequest",
    "response": "Operation__ApplyProposalResponse"
  },
  {
    "operationId": "getProposal",
    "method": "GET",
    "path": "/workspaces/{workspaceId}/proposals/{proposalId}",
    "request": "Operation__GetProposalRequest",
    "response": "Operation__GetProposalResponse"
  },
  {
    "operationId": "rejectProposal",
    "method": "POST",
    "path": "/workspaces/{workspaceId}/proposals/{proposalId}/reject",
    "request": "Operation__RejectProposalRequest",
    "response": "Operation__RejectProposalResponse"
  },
  {
    "operationId": "createFileUpload",
    "method": "POST",
    "path": "/workspaces/{workspaceId}/files/uploads",
    "request": "Operation__CreateFileUploadRequest",
    "response": "Operation__CreateFileUploadResponse"
  },
  {
    "operationId": "completeFileUpload",
    "method": "POST",
    "path": "/workspaces/{workspaceId}/files/{assetId}/complete",
    "request": "Operation__CompleteFileUploadRequest",
    "response": "Operation__CompleteFileUploadResponse"
  },
  {
    "operationId": "getFile",
    "method": "GET",
    "path": "/workspaces/{workspaceId}/files/{assetId}",
    "request": "Operation__GetFileRequest",
    "response": "Operation__GetFileResponse"
  },
  {
    "operationId": "deleteFile",
    "method": "DELETE",
    "path": "/workspaces/{workspaceId}/files/{assetId}",
    "request": "Operation__DeleteFileRequest",
    "response": "Operation__DeleteFileResponse"
  },
  {
    "operationId": "downloadFile",
    "method": "GET",
    "path": "/workspaces/{workspaceId}/files/{assetId}/content",
    "request": "Operation__DownloadFileRequest",
    "response": "Operation__DownloadFileResponse"
  },
  {
    "operationId": "uploadFileContent",
    "method": "PUT",
    "path": "/workspaces/{workspaceId}/files/{assetId}/content",
    "request": "Operation__UploadFileContentRequest",
    "response": "Operation__UploadFileContentResponse"
  },
  {
    "operationId": "downloadPublicFile",
    "method": "GET",
    "path": "/public/v1/documents/{publicToken}/files/{assetId}",
    "request": "Operation__DownloadPublicFileRequest",
    "response": "Operation__DownloadPublicFileResponse"
  },
  {
    "operationId": "listAuditEvents",
    "method": "GET",
    "path": "/workspaces/{workspaceId}/audit-events",
    "request": "Operation__ListAuditEventsRequest",
    "response": "Operation__ListAuditEventsResponse"
  },
  {
    "operationId": "getWritingConfiguration",
    "method": "GET",
    "path": "/workspaces/{workspaceId}/writing-configuration",
    "request": "Operation__GetWritingConfigurationRequest",
    "response": "Operation__GetWritingConfigurationResponse"
  },
  {
    "operationId": "updateWritingConfiguration",
    "method": "PUT",
    "path": "/workspaces/{workspaceId}/writing-configuration",
    "request": "Operation__UpdateWritingConfigurationRequest",
    "response": "Operation__UpdateWritingConfigurationResponse"
  },
  {
    "operationId": "getAIConfiguration",
    "method": "GET",
    "path": "/workspaces/{workspaceId}/ai/configuration",
    "request": "Operation__GetAIConfigurationRequest",
    "response": "Operation__GetAIConfigurationResponse"
  },
  {
    "operationId": "updateAIConfiguration",
    "method": "PUT",
    "path": "/workspaces/{workspaceId}/ai/configuration",
    "request": "Operation__UpdateAIConfigurationRequest",
    "response": "Operation__UpdateAIConfigurationResponse"
  },
  {
    "operationId": "getAIUsage",
    "method": "GET",
    "path": "/workspaces/{workspaceId}/ai/usage",
    "request": "Operation__GetAIUsageRequest",
    "response": "Operation__GetAIUsageResponse"
  },
  {
    "operationId": "getAIProviderHealth",
    "method": "GET",
    "path": "/workspaces/{workspaceId}/ai/provider-health",
    "request": "Operation__GetAIProviderHealthRequest",
    "response": "Operation__GetAIProviderHealthResponse"
  },
  {
    "operationId": "listPublicLinks",
    "method": "GET",
    "path": "/workspaces/{workspaceId}/documents/{documentId}/public-links",
    "request": "Operation__ListPublicLinksRequest",
    "response": "Operation__ListPublicLinksResponse"
  },
  {
    "operationId": "createPublicLink",
    "method": "POST",
    "path": "/workspaces/{workspaceId}/documents/{documentId}/public-links",
    "request": "Operation__CreatePublicLinkRequest",
    "response": "Operation__CreatePublicLinkResponse"
  },
  {
    "operationId": "revokePublicLink",
    "method": "DELETE",
    "path": "/workspaces/{workspaceId}/documents/{documentId}/public-links/{linkId}",
    "request": "Operation__RevokePublicLinkRequest",
    "response": "Operation__RevokePublicLinkResponse"
  },
  {
    "operationId": "openWorkspaceStream",
    "method": "GET",
    "path": "/stream",
    "request": "Operation__OpenWorkspaceStreamRequest",
    "response": "Operation__OpenWorkspaceStreamResponse"
  },
  {
    "operationId": "getPublicDocument",
    "method": "GET",
    "path": "/public/v1/documents/{token}",
    "request": "Operation__GetPublicDocumentRequest",
    "response": "Operation__GetPublicDocumentResponse"
  }
] as const;
export type OperationId = typeof operations[number]["operationId"];
export const asyncMessages = ["WorkspaceEvent","DomainEvent"] as const;
