/* Generated from canonical OpenAPI. Do not edit. */
export interface paths {
    "/session": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get: operations["getSession"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/auth/google/start": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get: operations["beginGoogleLogin"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/auth/google/callback": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get: operations["completeGoogleLogin"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/session/logout": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        post: operations["logout"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/preferences": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get: operations["getUserPreferences"];
        put: operations["updateUserPreferences"];
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/workspaces": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get: operations["listWorkspaces"];
        put?: never;
        post: operations["createWorkspace"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/workspaces/{workspaceId}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get: operations["getWorkspace"];
        put: operations["updateWorkspace"];
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/workspaces/{workspaceId}/deletion": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        post: operations["scheduleWorkspaceDeletion"];
        delete: operations["cancelWorkspaceDeletion"];
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/workspaces/{workspaceId}/members": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get: operations["listMembers"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/workspaces/{workspaceId}/members/{userId}/role": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put: operations["updateMemberRole"];
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/workspaces/{workspaceId}/members/{userId}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        post?: never;
        delete: operations["removeMember"];
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/workspaces/{workspaceId}/invitations": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get: operations["listInvitations"];
        put?: never;
        post: operations["inviteMember"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/workspaces/{workspaceId}/invitations/{invitationId}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        post?: never;
        delete: operations["revokeInvitation"];
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/invitations/{token}/accept": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        post: operations["acceptInvitation"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/workspaces/{workspaceId}/groups": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get: operations["listGroups"];
        put?: never;
        post: operations["createGroup"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/workspaces/{workspaceId}/groups/{groupId}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get: operations["getGroup"];
        put: operations["updateGroup"];
        post?: never;
        delete: operations["deleteGroup"];
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/workspaces/{workspaceId}/groups/{groupId}/members/{userId}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put: operations["addGroupMember"];
        post?: never;
        delete: operations["removeGroupMember"];
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/workspaces/{workspaceId}/documents": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        post: operations["createDocument"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/workspaces/{workspaceId}/documents/tree": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get: operations["getDocumentTree"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/workspaces/{workspaceId}/documents/trash": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get: operations["listTrashedDocuments"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/workspaces/{workspaceId}/documents/{documentId}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get: operations["getDocument"];
        put: operations["updateDocumentMetadata"];
        post?: never;
        delete: operations["purgeDocument"];
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/workspaces/{workspaceId}/documents/{documentId}/trash": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        post: operations["trashDocument"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/workspaces/{workspaceId}/documents/{documentId}/restore": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        post: operations["restoreDocument"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/workspaces/{workspaceId}/documents/{documentId}/move-preview": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        post: operations["previewDocumentMove"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/workspaces/{workspaceId}/documents/{documentId}/move": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        post: operations["moveDocument"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/workspaces/{workspaceId}/documents/{documentId}/permissions": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get: operations["getDocumentPermissions"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/workspaces/{workspaceId}/documents/{documentId}/permissions/{grantId}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put: operations["setDocumentPermission"];
        post?: never;
        delete: operations["deleteDocumentPermission"];
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/workspaces/{workspaceId}/documents/{documentId}/permission-explanation": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get: operations["explainEffectivePermission"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/workspaces/{workspaceId}/documents/{documentId}/publish-policy": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get: operations["getPublishPolicy"];
        put: operations["setPublishPolicy"];
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/workspaces/{workspaceId}/documents/{documentId}/draft": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get: operations["getDraft"];
        put?: never;
        post: operations["createOrGetDraft"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/workspaces/{workspaceId}/documents/{documentId}/lease": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        post: operations["acquireEditLease"];
        delete: operations["releaseEditLease"];
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/workspaces/{workspaceId}/documents/{documentId}/lease/renew": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        post: operations["renewEditLease"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/workspaces/{workspaceId}/documents/{documentId}/draft/operations": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        post: operations["applyDraftOperations"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/workspaces/{workspaceId}/documents/{documentId}/reviews": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        post: operations["requestReview"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/workspaces/{workspaceId}/documents/{documentId}/versions": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get: operations["listVersions"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/workspaces/{workspaceId}/documents/{documentId}/versions/{versionId}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get: operations["getVersion"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/workspaces/{workspaceId}/documents/{documentId}/versions/{versionId}/restore": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        post: operations["restoreVersionToDraft"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/workspaces/{workspaceId}/documents/{documentId}/version-diff": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get: operations["compareVersions"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/workspaces/{workspaceId}/reviews/{reviewId}/decisions": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        post: operations["submitReviewDecision"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/workspaces/{workspaceId}/documents/{documentId}/publish": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        post: operations["publishDocument"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/workspaces/{workspaceId}/documents/{documentId}/discussions": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get: operations["listDiscussions"];
        put?: never;
        post: operations["createDiscussion"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/workspaces/{workspaceId}/discussions/{discussionId}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get: operations["getDiscussion"];
        put: operations["updateDiscussion"];
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/workspaces/{workspaceId}/discussions/{discussionId}/close": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        post: operations["closeDiscussion"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/workspaces/{workspaceId}/discussions/{discussionId}/reopen": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        post: operations["reopenDiscussion"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/workspaces/{workspaceId}/discussions/{discussionId}/topics": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        post: operations["addDiscussionTopic"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/workspaces/{workspaceId}/discussions/{discussionId}/topics/{topicId}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        post?: never;
        delete: operations["removeDiscussionTopic"];
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/workspaces/{workspaceId}/discussions/{discussionId}/messages": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        post: operations["createMessage"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/workspaces/{workspaceId}/discussions/{discussionId}/messages/{messageId}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put: operations["updateMessage"];
        post?: never;
        delete: operations["deleteMessage"];
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/workspaces/{workspaceId}/reviews/{reviewId}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get: operations["getReview"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/workspaces/{workspaceId}/reviews/{reviewId}/cancel": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        post: operations["cancelReview"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/workspaces/{workspaceId}/inbox": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get: operations["listInbox"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/workspaces/{workspaceId}/inbox/{itemId}/read": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        post: operations["markInboxItemRead"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/workspaces/{workspaceId}/inbox/read-all": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        post: operations["markAllInboxRead"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/workspaces/{workspaceId}/inbox/{itemId}/resolve": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        post: operations["resolveInboxItem"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/workspaces/{workspaceId}/search": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get: operations["searchKnowledge"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/workspaces/{workspaceId}/documents/{documentId}/backlinks": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get: operations["listBacklinks"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/workspaces/{workspaceId}/documents/{documentId}/references": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        post: operations["createReference"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/workspaces/{workspaceId}/documents/{documentId}/references/{referenceId}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        post?: never;
        delete: operations["deleteReference"];
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/workspaces/{workspaceId}/vocabulary": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get: operations["listVocabulary"];
        put?: never;
        post: operations["createVocabularyConcept"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/workspaces/{workspaceId}/vocabulary/{conceptId}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get: operations["getVocabularyConcept"];
        put: operations["updateVocabularyConcept"];
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/workspaces/{workspaceId}/vocabulary/{conceptId}/deprecate": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        post: operations["deprecateVocabularyConcept"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/workspaces/{workspaceId}/ai/jobs": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get: operations["listAIJobs"];
        put?: never;
        post: operations["createAIJob"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/workspaces/{workspaceId}/ai/jobs/{jobId}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get: operations["getAIJob"];
        put?: never;
        post?: never;
        delete: operations["cancelAIJob"];
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/workspaces/{workspaceId}/ai/context-preview": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        post: operations["previewAIContext"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/workspaces/{workspaceId}/proposals/{proposalId}/apply": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        post: operations["applyProposal"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/workspaces/{workspaceId}/proposals/{proposalId}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get: operations["getProposal"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/workspaces/{workspaceId}/proposals/{proposalId}/reject": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        post: operations["rejectProposal"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/workspaces/{workspaceId}/files/uploads": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        post: operations["createFileUpload"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/workspaces/{workspaceId}/files/{assetId}/complete": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        post: operations["completeFileUpload"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/workspaces/{workspaceId}/files/{assetId}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get: operations["getFile"];
        put?: never;
        post?: never;
        delete: operations["deleteFile"];
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/workspaces/{workspaceId}/files/{assetId}/content": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get: operations["downloadFile"];
        put: operations["uploadFileContent"];
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/public/v1/documents/{publicToken}/files/{assetId}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get: operations["downloadPublicFile"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/workspaces/{workspaceId}/audit-events": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get: operations["listAuditEvents"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/workspaces/{workspaceId}/writing-configuration": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get: operations["getWritingConfiguration"];
        put: operations["updateWritingConfiguration"];
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/workspaces/{workspaceId}/ai/configuration": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get: operations["getAIConfiguration"];
        put: operations["updateAIConfiguration"];
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/workspaces/{workspaceId}/ai/usage": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get: operations["getAIUsage"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/workspaces/{workspaceId}/ai/provider-health": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get: operations["getAIProviderHealth"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/workspaces/{workspaceId}/documents/{documentId}/public-links": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get: operations["listPublicLinks"];
        put?: never;
        post: operations["createPublicLink"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/workspaces/{workspaceId}/documents/{documentId}/public-links/{linkId}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        post?: never;
        delete: operations["revokePublicLink"];
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/stream": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get: operations["openWorkspaceStream"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/public/v1/documents/{token}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get: operations["getPublicDocument"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
}
export type webhooks = Record<string, never>;
export interface components {
    schemas: {
        /** Format: uuid */
        Id: string;
        NullableId: components["schemas"]["Id"] | null;
        SessionView: {
            user: components["schemas"]["UserSummary"];
            workspaces: components["schemas"]["Workspace"][];
        };
        UserSummary: {
            id: components["schemas"]["Id"];
            /** Format: email */
            email: string;
            displayName: string;
            /** @enum {string} */
            locale: "ko" | "en";
            timezone: string;
        };
        UserPreferences: {
            /** @enum {string} */
            locale: "ko" | "en";
            timezone: string;
            /** @enum {string} */
            theme: "LIGHT" | "DARK" | "SYSTEM";
            revision: number;
        };
        Workspace: {
            id: components["schemas"]["Id"];
            name: string;
            slug: string;
            /** @enum {string} */
            status: "ACTIVE" | "DELETION_SCHEDULED" | "PURGING" | "DELETED";
            revision: number;
        };
        Membership: {
            userId: components["schemas"]["Id"];
            /** @enum {string} */
            role: "MEMBER" | "ADMIN" | "OWNER";
            /** @enum {string} */
            status: "ACTIVE" | "SUSPENDED" | "REMOVED";
            revision: number;
        };
        Invitation: {
            id: components["schemas"]["Id"];
            /** Format: email */
            email: string;
            /** @enum {string} */
            role: "MEMBER" | "ADMIN";
            /** @enum {string} */
            status: "PENDING" | "ACCEPTED" | "REVOKED" | "EXPIRED";
            /** Format: date-time */
            expiresAt: string;
            revision: number;
        };
        InvitationPage: {
            items: components["schemas"]["Invitation"][];
            nextCursor?: string | null;
        };
        Group: {
            id: components["schemas"]["Id"];
            name: string;
            memberIds: components["schemas"]["Id"][];
            revision: number;
        };
        /** @enum {string} */
        Access: "NO_ACCESS" | "VIEWER" | "CONTRIBUTOR" | "EDITOR";
        PermissionGrantInput: {
            /** @enum {string} */
            subjectKind: "USER" | "GROUP";
            subjectId: components["schemas"]["Id"];
            access: components["schemas"]["Access"];
            manage: boolean;
        };
        PermissionGrant: {
            id: components["schemas"]["Id"];
            /** @enum {string} */
            subjectKind: "USER" | "GROUP";
            subjectId: components["schemas"]["Id"];
            access: components["schemas"]["Access"];
            manage: boolean;
            revision: number;
        };
        PermissionView: {
            effective: components["schemas"]["EffectivePermission"];
            explicitGrants: components["schemas"]["PermissionGrant"][];
            revision: number;
        };
        EffectivePermission: {
            access: components["schemas"]["Access"];
            manage: boolean;
            sourceDocumentId: components["schemas"]["NullableId"];
            evidenceGrantIds: components["schemas"]["Id"][];
        };
        PermissionExplanation: {
            effective: components["schemas"]["EffectivePermission"];
            steps: {
                documentId: components["schemas"]["Id"];
                /** @enum {string} */
                decision: "NO_GRANT" | "USER_GRANT" | "GROUP_DENY" | "GROUP_MAX" | "INHERITED";
            }[];
            fingerprint: string;
        };
        ReviewerRule: {
            /** @constant */
            kind: "ANY_EDITOR";
        } | {
            /** @constant */
            kind: "USERS";
            userIds: components["schemas"]["Id"][];
        } | {
            /** @constant */
            kind: "GROUPS";
            groupIds: components["schemas"]["Id"][];
        };
        PublishPolicy: {
            documentId: components["schemas"]["Id"];
            /** @enum {string} */
            mode: "DIRECT" | "REVIEW_REQUIRED";
            requiredApprovals: number;
            reviewerRule: components["schemas"]["ReviewerRule"];
            inheritedFromDocumentId: components["schemas"]["NullableId"];
            revision: number;
        };
        Document: {
            id: components["schemas"]["Id"];
            title: string;
            parentId: components["schemas"]["NullableId"];
            /** @enum {string} */
            status: "ACTIVE" | "TRASHED" | "PURGING";
            currentVersionId: components["schemas"]["NullableId"];
            revision: number;
        };
        DocumentPage: {
            items: components["schemas"]["Document"][];
            nextCursor?: string | null;
        };
        DocumentTree: {
            nodes: components["schemas"]["DocumentTreeNode"][];
            watermark: number;
        };
        DocumentTreeNode: {
            document: components["schemas"]["Document"];
            children: components["schemas"]["DocumentTreeNode"][];
        };
        JobReference: {
            jobId: components["schemas"]["Id"];
            /** @enum {string} */
            status: "QUEUED";
        };
        DocumentDetail: components["schemas"]["Document"] & {
            draft?: components["schemas"]["Draft"] | null;
            publishedVersion?: components["schemas"]["PublishedVersion"] | null;
        };
        Draft: {
            id: components["schemas"]["Id"];
            documentId: components["schemas"]["Id"];
            baseVersionId: components["schemas"]["NullableId"];
            revision: number;
            schemaVersion: number;
            contentFingerprint: string;
            content: components["schemas"]["document-content.schema"];
        };
        EditLease: {
            holderUserId: components["schemas"]["Id"];
            clientInstanceId: components["schemas"]["Id"];
            token?: string;
            /** Format: date-time */
            expiresAt: string;
            revision: number;
        };
        DocumentOperation: components["schemas"]["document-operation.schema"];
        MutationResult: {
            revision: number;
            contentFingerprint: string;
            appliedOperationIds: components["schemas"]["Id"][];
            inverseOperations: components["schemas"]["document-operation.schema"][];
        };
        PublishedVersion: {
            id: components["schemas"]["Id"];
            documentId: components["schemas"]["Id"];
            number: number;
            /** Format: date-time */
            publishedAt: string;
            publisherId: components["schemas"]["Id"];
            schemaVersion: number;
            contentFingerprint: string;
            basedOnVersionId: components["schemas"]["NullableId"];
            sourceDraftRevision: number;
            content: components["schemas"]["document-content.schema"];
            summary: string;
            reviewSnapshot: Record<string, never>;
            discussionIds: components["schemas"]["Id"][];
        };
        VersionPage: {
            items: components["schemas"]["PublishedVersion"][];
            nextCursor?: string | null;
        };
        DocumentDiff: {
            fromVersionId: components["schemas"]["Id"];
            toVersionId: components["schemas"]["Id"];
            operations: components["schemas"]["document-operation.schema"][];
        };
        MoveDocumentInput: {
            newParentId: components["schemas"]["NullableId"];
            afterDocumentId: components["schemas"]["NullableId"];
        };
        ImpactPreview: {
            previewToken: string;
            permissionChanges: number;
            policyChanges: number;
            /** Format: date-time */
            expiresAt: string;
        };
        TopicInput: {
            /** @enum {string} */
            kind: "TEXT" | "DOCUMENT" | "REGION" | "EXTERNAL";
            label: string;
            text?: string;
            targetId?: components["schemas"]["NullableId"];
            region?: components["schemas"]["region"];
            /** Format: uri */
            url?: string;
        };
        Topic: {
            id: components["schemas"]["Id"];
            /** @enum {string} */
            kind: "TEXT" | "DOCUMENT" | "REGION" | "EXTERNAL";
            label: string;
            rank: string;
            text?: string;
            targetId?: components["schemas"]["NullableId"];
            region?: components["schemas"]["region"];
            /** Format: uri */
            url?: string;
        };
        RichMessage: {
            body: components["schemas"]["document-content.schema"];
            mentionUserIds?: components["schemas"]["Id"][];
            attachmentIds?: components["schemas"]["Id"][];
        };
        Discussion: {
            id: components["schemas"]["Id"];
            documentId: components["schemas"]["Id"];
            title: string;
            /** @enum {string} */
            status: "OPEN" | "CLOSED";
            topics?: components["schemas"]["Topic"][];
            revision: number;
        };
        Message: {
            id: components["schemas"]["Id"];
            authorId: components["schemas"]["Id"];
            body: components["schemas"]["document-content.schema"];
            mentionUserIds: components["schemas"]["Id"][];
            revision: number;
            /** Format: date-time */
            createdAt: string;
            /** Format: date-time */
            editedAt?: string | null;
            /** Format: date-time */
            deletedAt?: string | null;
        };
        DiscussionPage: {
            items: components["schemas"]["Discussion"][];
            nextCursor?: string | null;
        };
        DiscussionDetail: {
            discussion: components["schemas"]["Discussion"];
            messages: components["schemas"]["Message"][];
            nextCursor?: string | null;
        };
        ReviewDecisionInput: {
            /** @constant */
            decision: "APPROVE";
            discussionId?: null;
        } | {
            /** @constant */
            decision: "REQUEST_CHANGES";
            discussionId: components["schemas"]["Id"];
        };
        ReviewAssignment: {
            reviewerId: components["schemas"]["Id"];
            /** @enum {string} */
            decision: "PENDING" | "APPROVED" | "CHANGES_REQUESTED";
            discussionId: components["schemas"]["NullableId"];
            /** Format: date-time */
            decidedAt: string | null;
            revision: number;
        };
        Review: {
            id: components["schemas"]["Id"];
            documentId: components["schemas"]["Id"];
            draftId: components["schemas"]["Id"];
            draftRevision: number;
            requestedBy: components["schemas"]["Id"];
            policySnapshot: Record<string, never>;
            policyOutdated: boolean;
            /** @enum {string} */
            status: "REQUESTED" | "APPROVED" | "CHANGES_REQUESTED" | "CANCELLED" | "INVALIDATED";
            assignments: components["schemas"]["ReviewAssignment"][];
            /** Format: date-time */
            requestedAt: string;
            /** Format: date-time */
            resolvedAt: string | null;
            revision: number;
        };
        InboxItem: {
            id: components["schemas"]["Id"];
            /** @enum {string} */
            kind: "REVIEW_REQUESTED" | "REVIEW_DECIDED" | "MENTIONED" | "DISCUSSION_CHANGED" | "PERMISSION_CHANGED" | "AI_JOB_COMPLETED";
            target: components["schemas"]["ResourceTarget"];
            revision: number;
            /** Format: date-time */
            createdAt: string;
            /** Format: date-time */
            readAt: string | null;
            /** Format: date-time */
            resolvedAt: string | null;
        };
        InboxPage: {
            items: components["schemas"]["InboxItem"][];
            nextCursor?: string | null;
        };
        AffectedCount: {
            count: number;
        };
        ResourceTarget: {
            /** @enum {string} */
            kind: "WORKSPACE" | "DOCUMENT" | "DISCUSSION" | "REVIEW" | "AI_JOB" | "FILE";
            id: components["schemas"]["Id"];
        };
        SearchPage: {
            items: {
                source: components["schemas"]["Source"];
                score: number;
            }[];
            nextCursor?: string | null;
            indexWatermark: number;
            /** @constant */
            configurationVersion: "search-ranking-v1";
        };
        Source: {
            /** @enum {string} */
            kind: "PUBLISHED" | "DRAFT";
            stableId: string;
            documentId: components["schemas"]["Id"];
            regionId: components["schemas"]["Id"];
            version?: number | null;
            draftRevision?: number | null;
            /** @enum {string} */
            authority: "OFFICIAL" | "WORKING";
            snapshotHash: string;
            displaySnapshot: {
                title: string;
                excerpt: string;
                /** Format: date-time */
                updatedAt: string;
            };
        };
        Reference: {
            id: components["schemas"]["Id"];
            sourceDocumentId: components["schemas"]["Id"];
            sourceRegion: components["schemas"]["region"];
            target: components["schemas"]["referenceTarget"];
            snapshot: {
                title: string;
                snapshotHash: string;
            };
            /** Format: date-time */
            createdAt: string;
        };
        ReferencePage: {
            items: components["schemas"]["Reference"][];
            nextCursor?: string | null;
        };
        VocabularyTerm: {
            term: string;
            /** @enum {string} */
            kind: "CANONICAL" | "SYNONYM" | "PROHIBITED";
        };
        VocabularyConcept: {
            id: components["schemas"]["Id"];
            canonicalTerm: string;
            definition: string;
            terms: components["schemas"]["VocabularyTerm"][];
            /** @enum {string} */
            status: "ACTIVE" | "DEPRECATED";
            replacementConceptId: components["schemas"]["NullableId"];
            revision: number;
        };
        VocabularyPage: {
            items: components["schemas"]["VocabularyConcept"][];
            nextCursor?: string | null;
        };
        AIContextRequest: {
            /** @enum {string} */
            kind: "COMPOSE" | "REWRITE" | "REVIEW" | "DISCUSSION_APPLY" | "CONFLICT_MERGE" | "KNOWLEDGE_QUERY";
            target: components["schemas"]["target"];
            expectedRevision: number;
            externalWebEnabled: boolean;
            instruction?: string;
            includeSourceIds?: components["schemas"]["Id"][];
            excludeSourceIds?: components["schemas"]["Id"][];
        };
        CreateAIJob: {
            /** @enum {string} */
            kind: "COMPOSE" | "REWRITE" | "REVIEW" | "DISCUSSION_APPLY" | "CONFLICT_MERGE" | "KNOWLEDGE_QUERY";
            target: components["schemas"]["target"];
            expectedRevision: number;
            externalWebEnabled: boolean;
            contextFingerprint: string;
            instruction?: string;
            includeSourceIds?: components["schemas"]["Id"][];
            excludeSourceIds?: components["schemas"]["Id"][];
        };
        AIContextSourcePreview: {
            sourceId: components["schemas"]["Id"];
            /** @enum {string} */
            kind: "DRAFT" | "PUBLISHED_REGION" | "DISCUSSION" | "VOCABULARY" | "USER_INPUT";
            stableId: string;
            /** @enum {string} */
            authority: "USER_EXPLICIT" | "OFFICIAL" | "VOCABULARY" | "DISCUSSION_CONFIRMED" | "RELATED_INTERNAL";
            /** @enum {string} */
            includeReason: "CURRENT_TARGET" | "EXPLICIT_REFERENCE" | "DISCUSSION_CONTEXT" | "VOCABULARY_POLICY" | "RETRIEVED_RELATED" | "USER_PROVIDED";
            snapshotHash: string;
            included: boolean;
            title?: string | null;
            /** Format: date-time */
            updatedAt?: string | null;
        };
        AIContextPreview: {
            artifactFingerprint: string;
            /** Format: date-time */
            expiresAt: string;
            sources: components["schemas"]["AIContextSourcePreview"][];
            omissions: ("SOURCE_UNAVAILABLE" | "SOURCE_EXCLUDED" | "CONTEXT_BUDGET")[];
            estimatedInputUnits: number;
        };
        AIJob: {
            id: components["schemas"]["Id"];
            /** @enum {string} */
            kind: "COMPOSE" | "REWRITE" | "REVIEW" | "DISCUSSION_APPLY" | "CONFLICT_MERGE" | "KNOWLEDGE_QUERY";
            /** @enum {string} */
            status: "QUEUED" | "RUNNING" | "CANCEL_REQUESTED" | "SUCCEEDED" | "FAILED" | "CANCELLED" | "TIMED_OUT";
            sequence: number;
            revision: number;
            result?: components["schemas"]["result"] | null;
            proposalId?: components["schemas"]["NullableId"];
            errorCode?: string | null;
        };
        AIJobPage: {
            items: components["schemas"]["AIJob"][];
            nextCursor?: string | null;
        };
        Proposal: components["schemas"]["proposal"];
        FileUpload: {
            assetId: components["schemas"]["Id"];
            /** Format: uri */
            uploadUrl: string;
            uploadToken: string;
            /** Format: date-time */
            expiresAt: string;
        };
        FileAsset: {
            id: components["schemas"]["Id"];
            originalName: string;
            mimeType: string;
            sizeBytes: number;
            checksumSha256: string;
            /** @enum {string} */
            status: "UPLOADING" | "VALIDATING" | "READY" | "FAILED" | "DELETED";
            failureCode?: string | null;
            /** Format: date-time */
            readyAt?: string | null;
            revision: number;
        };
        AuditEvent: {
            id: components["schemas"]["Id"];
            sequence: number;
            actor: {
                /** @enum {string} */
                kind: "USER" | "SYSTEM";
                userId?: components["schemas"]["NullableId"];
            };
            action: string;
            target: components["schemas"]["ResourceTarget"];
            before?: {
                [key: string]: string | number | boolean | null;
            } | null;
            after?: {
                [key: string]: string | number | boolean | null;
            } | null;
            metadata: {
                [key: string]: string | number | boolean | null;
            };
            correlationId: string;
            /** Format: date-time */
            occurredAt: string;
            /** Format: date-time */
            redactedAt?: string | null;
        };
        AuditPage: {
            items: components["schemas"]["AuditEvent"][];
            nextCursor?: string | null;
        };
        PublicLink: {
            id: components["schemas"]["Id"];
            /** Format: date-time */
            expiresAt: string | null;
            /** Format: date-time */
            revokedAt: string | null;
            /** Format: date-time */
            createdAt: string;
            revision: number;
        };
        WritingRuleOverride: {
            ruleId: string;
            enabled: boolean;
            /** @enum {string} */
            severity: "BLOCKING" | "WARNING" | "ADVISORY";
            values: string[];
        };
        WritingConfiguration: {
            /** @constant */
            baselineVersion: "writing-rules-v1";
            overrides: components["schemas"]["WritingRuleOverride"][];
            revision: number;
        };
        AIConfiguration: {
            /** @enum {string} */
            provider: "CODEX_CLI" | "OPENAI_RESPONSES";
            model: string;
            userConcurrencyLimit: number;
            workspaceConcurrencyLimit: number;
            monthlyBudgetMicrounits: number;
            revision: number;
        };
        AIUsage: {
            /** Format: date */
            from: string;
            /** Format: date */
            to: string;
            inputTokens: number;
            outputTokens: number;
            estimatedMicrounits: number;
            jobCount: number;
        };
        AIProviderHealth: {
            /** @enum {string} */
            provider: "CODEX_CLI" | "OPENAI_RESPONSES";
            /** @enum {string} */
            status: "HEALTHY" | "DEGRADED" | "UNAVAILABLE" | "UNCONFIGURED";
            /** Format: date-time */
            checkedAt: string;
            code?: string | null;
        };
        PublicLinkCreated: {
            id: components["schemas"]["Id"];
            token: string;
            /** Format: uri */
            url: string;
        };
        PublicDocument: {
            title: string;
            versionNumber: number;
            /** Format: date-time */
            publishedAt: string;
            schemaVersion: number;
            content: components["schemas"]["document-content.schema"];
        };
        Problem: {
            /** Format: uri */
            type: string;
            title: string;
            status: number;
            code: string;
            retryable: boolean;
            correlationId: string;
            currentRevision?: number;
            baseVersionId?: components["schemas"]["NullableId"];
            currentVersionId?: components["schemas"]["NullableId"];
            draftId?: components["schemas"]["NullableId"];
            fieldErrors?: {
                field: string;
                code: string;
            }[];
        };
        /** Format: uuid */
        id: string;
        mark: {
            /** @enum {unknown} */
            type: "bold" | "italic" | "underline" | "strike" | "code" | "subscript" | "superscript";
        } | {
            /** @constant */
            type: "link";
            /** Format: uri */
            href: string;
            title?: string | null;
        } | {
            /** @enum {unknown} */
            type: "highlight" | "textColor";
            token: string;
        };
        inline: {
            /** @constant */
            type: "text";
            text: string;
            marks?: components["schemas"]["mark"][];
        } | {
            /** @constant */
            type: "hardBreak";
        };
        textChildren: components["schemas"]["inline"][];
        paragraph: {
            id: components["schemas"]["id"];
            /** @constant */
            type: "paragraph";
            children: components["schemas"]["textChildren"];
        };
        heading: {
            id: components["schemas"]["id"];
            /** @constant */
            type: "heading";
            level: number;
            children: components["schemas"]["textChildren"];
        };
        list: {
            id: components["schemas"]["id"];
            /** @enum {unknown} */
            type: "bulletList" | "orderedList" | "taskList";
            start?: number;
            items: components["schemas"]["listItem"][];
        };
        listItem: {
            id: components["schemas"]["id"];
            /** @constant */
            type: "listItem";
            checked?: boolean | null;
            children: (components["schemas"]["paragraph"] | components["schemas"]["list"])[];
        };
        quote: {
            id: components["schemas"]["id"];
            /** @constant */
            type: "quote";
            children: (components["schemas"]["paragraph"] | components["schemas"]["list"])[];
        };
        callout: {
            id: components["schemas"]["id"];
            /** @constant */
            type: "callout";
            /** @enum {unknown} */
            tone: "info" | "success" | "warning" | "danger" | "note";
            icon?: string | null;
            children: (components["schemas"]["paragraph"] | components["schemas"]["list"])[];
        };
        codeBlock: {
            id: components["schemas"]["id"];
            /** @constant */
            type: "codeBlock";
            language?: string | null;
            text: string;
        };
        tableCell: {
            id: components["schemas"]["id"];
            /** @enum {unknown} */
            type: "tableCell" | "tableHeader";
            colspan?: number;
            rowspan?: number;
            children: (components["schemas"]["paragraph"] | components["schemas"]["list"] | components["schemas"]["codeBlock"])[];
        };
        tableRow: {
            id: components["schemas"]["id"];
            /** @constant */
            type: "tableRow";
            cells: components["schemas"]["tableCell"][];
        };
        table: {
            id: components["schemas"]["id"];
            /** @constant */
            type: "table";
            rows: components["schemas"]["tableRow"][];
        };
        block: components["schemas"]["paragraph"] | components["schemas"]["heading"] | components["schemas"]["quote"] | components["schemas"]["callout"] | components["schemas"]["list"] | components["schemas"]["codeBlock"] | components["schemas"]["table"] | components["schemas"]["toggle"] | components["schemas"]["divider"] | components["schemas"]["image"] | components["schemas"]["file"];
        toggle: {
            id: components["schemas"]["id"];
            /** @constant */
            type: "toggle";
            summary: components["schemas"]["textChildren"];
            children: components["schemas"]["block"][];
        };
        divider: {
            id: components["schemas"]["id"];
            /** @constant */
            type: "divider";
        };
        image: {
            id: components["schemas"]["id"];
            /** @constant */
            type: "image";
            assetId: components["schemas"]["id"];
            alt: string;
            caption?: string | null;
            width?: number | null;
        };
        file: {
            id: components["schemas"]["id"];
            /** @constant */
            type: "file";
            assetId: components["schemas"]["id"];
            caption?: string | null;
        };
        doc: {
            /** @constant */
            type: "doc";
            children: components["schemas"]["block"][];
        };
        /** DocumentContent */
        "document-content.schema": {
            /** @constant */
            schemaVersion: 1;
            root: components["schemas"]["doc"];
            $defs: {
                /** Format: uuid */
                id: string;
                doc: {
                    /** @constant */
                    type: "doc";
                    children: components["schemas"]["block"][];
                };
                block: components["schemas"]["paragraph"] | components["schemas"]["heading"] | components["schemas"]["quote"] | components["schemas"]["callout"] | components["schemas"]["list"] | components["schemas"]["codeBlock"] | components["schemas"]["table"] | components["schemas"]["toggle"] | components["schemas"]["divider"] | components["schemas"]["image"] | components["schemas"]["file"];
                inline: {
                    /** @constant */
                    type: "text";
                    text: string;
                    marks?: components["schemas"]["mark"][];
                } | {
                    /** @constant */
                    type: "hardBreak";
                };
                mark: {
                    /** @enum {unknown} */
                    type: "bold" | "italic" | "underline" | "strike" | "code" | "subscript" | "superscript";
                } | {
                    /** @constant */
                    type: "link";
                    /** Format: uri */
                    href: string;
                    title?: string | null;
                } | {
                    /** @enum {unknown} */
                    type: "highlight" | "textColor";
                    token: string;
                };
                textChildren: components["schemas"]["inline"][];
                paragraph: {
                    id: components["schemas"]["id"];
                    /** @constant */
                    type: "paragraph";
                    children: components["schemas"]["textChildren"];
                };
                heading: {
                    id: components["schemas"]["id"];
                    /** @constant */
                    type: "heading";
                    level: number;
                    children: components["schemas"]["textChildren"];
                };
                quote: {
                    id: components["schemas"]["id"];
                    /** @constant */
                    type: "quote";
                    children: (components["schemas"]["paragraph"] | components["schemas"]["list"])[];
                };
                callout: {
                    id: components["schemas"]["id"];
                    /** @constant */
                    type: "callout";
                    /** @enum {unknown} */
                    tone: "info" | "success" | "warning" | "danger" | "note";
                    icon?: string | null;
                    children: (components["schemas"]["paragraph"] | components["schemas"]["list"])[];
                };
                list: {
                    id: components["schemas"]["id"];
                    /** @enum {unknown} */
                    type: "bulletList" | "orderedList" | "taskList";
                    start?: number;
                    items: components["schemas"]["listItem"][];
                };
                listItem: {
                    id: components["schemas"]["id"];
                    /** @constant */
                    type: "listItem";
                    checked?: boolean | null;
                    children: (components["schemas"]["paragraph"] | components["schemas"]["list"])[];
                };
                codeBlock: {
                    id: components["schemas"]["id"];
                    /** @constant */
                    type: "codeBlock";
                    language?: string | null;
                    text: string;
                };
                table: {
                    id: components["schemas"]["id"];
                    /** @constant */
                    type: "table";
                    rows: components["schemas"]["tableRow"][];
                };
                tableRow: {
                    id: components["schemas"]["id"];
                    /** @constant */
                    type: "tableRow";
                    cells: components["schemas"]["tableCell"][];
                };
                tableCell: {
                    id: components["schemas"]["id"];
                    /** @enum {unknown} */
                    type: "tableCell" | "tableHeader";
                    colspan?: number;
                    rowspan?: number;
                    children: (components["schemas"]["paragraph"] | components["schemas"]["list"] | components["schemas"]["codeBlock"])[];
                };
                toggle: {
                    id: components["schemas"]["id"];
                    /** @constant */
                    type: "toggle";
                    summary: components["schemas"]["textChildren"];
                    children: components["schemas"]["block"][];
                };
                divider: {
                    id: components["schemas"]["id"];
                    /** @constant */
                    type: "divider";
                };
                image: {
                    id: components["schemas"]["id"];
                    /** @constant */
                    type: "image";
                    assetId: components["schemas"]["id"];
                    alt: string;
                    caption?: string | null;
                    width?: number | null;
                };
                file: {
                    id: components["schemas"]["id"];
                    /** @constant */
                    type: "file";
                    assetId: components["schemas"]["id"];
                    caption?: string | null;
                };
            };
        };
        textAnchor: {
            offset: number;
            /** @enum {unknown} */
            affinity: "BEFORE" | "AFTER";
            contextHash: string;
        };
        region: {
            /** @constant */
            kind: "DOCUMENT";
        } | {
            /** @constant */
            kind: "BLOCK";
            blockId: components["schemas"]["id"];
        } | {
            /** @constant */
            kind: "BLOCK_RANGE";
            startBlockId: components["schemas"]["id"];
            endBlockId: components["schemas"]["id"];
        } | {
            /** @constant */
            kind: "SECTION";
            headingId: components["schemas"]["id"];
        } | {
            /** @constant */
            kind: "TEXT_RANGE";
            blockId: components["schemas"]["id"];
            from: components["schemas"]["textAnchor"];
            to: components["schemas"]["textAnchor"];
            quoteHash: string;
        };
        precondition: {
            draftRevision: number;
            targetHash?: string | null;
        };
        base: {
            opId: components["schemas"]["id"];
            kind: string;
            scope: components["schemas"]["region"];
            precondition: components["schemas"]["precondition"];
            dependsOn?: components["schemas"]["id"][];
        };
        contentNode: components["schemas"]["block"] | components["schemas"]["listItem"] | components["schemas"]["tableRow"] | components["schemas"]["tableCell"];
        attrPatch: {
            /** @constant */
            action: "SET";
            value: string | number | boolean | null;
        } | {
            /** @constant */
            action: "REMOVE";
        };
        referenceTarget: {
            /** @constant */
            kind: "REGION";
            id: components["schemas"]["id"];
            region: components["schemas"]["region"];
        } | {
            /** @enum {unknown} */
            kind: "DOCUMENT" | "DISCUSSION" | "VOCABULARY";
            id: components["schemas"]["id"];
        } | {
            /** @constant */
            kind: "EXTERNAL";
            /** Format: uri */
            id: string;
        };
        insertBlock: components["schemas"]["base"] & {
            /** @constant */
            kind: "INSERT_BLOCK";
            parentId: components["schemas"]["id"] | null;
            index: number;
            block: components["schemas"]["contentNode"];
        };
        deleteBlock: components["schemas"]["base"] & {
            /** @constant */
            kind: "DELETE_BLOCK";
            blockId: components["schemas"]["id"];
        };
        moveBlock: components["schemas"]["base"] & {
            /** @constant */
            kind: "MOVE_BLOCK";
            blockId: components["schemas"]["id"];
            newParentId: components["schemas"]["id"] | null;
            newIndex: number;
        };
        replaceText: components["schemas"]["base"] & {
            /** @constant */
            kind: "REPLACE_TEXT";
            range: components["schemas"]["region"];
            content: components["schemas"]["textChildren"];
        };
        setBlockAttrs: components["schemas"]["base"] & {
            /** @constant */
            kind: "SET_BLOCK_ATTRS";
            blockId: components["schemas"]["id"];
            attrs: {
                [key: string]: components["schemas"]["attrPatch"];
            };
        };
        setMarks: components["schemas"]["base"] & {
            /** @constant */
            kind: "SET_MARKS";
            range: components["schemas"]["region"];
            /** @enum {unknown} */
            mode: "ADD" | "REMOVE" | "REPLACE";
            marks: components["schemas"]["mark"][];
        };
        replaceRegion: components["schemas"]["base"] & {
            /** @constant */
            kind: "REPLACE_REGION";
            region: components["schemas"]["region"];
            blocks: components["schemas"]["contentNode"][];
        };
        addReference: components["schemas"]["base"] & {
            /** @constant */
            kind: "ADD_REFERENCE";
            referenceId: components["schemas"]["id"];
            sourceRegion: components["schemas"]["region"];
            target: components["schemas"]["referenceTarget"];
        };
        removeReference: components["schemas"]["base"] & {
            /** @constant */
            kind: "REMOVE_REFERENCE";
            referenceId: components["schemas"]["id"];
            sourceRegion: components["schemas"]["region"];
            target: components["schemas"]["referenceTarget"];
        };
        /** DocumentOperation */
        "document-operation.schema": {
            $defs: {
                /** Format: uuid */
                id: string;
                base: {
                    opId: components["schemas"]["id"];
                    kind: string;
                    scope: components["schemas"]["region"];
                    precondition: components["schemas"]["precondition"];
                    dependsOn?: components["schemas"]["id"][];
                };
                precondition: {
                    draftRevision: number;
                    targetHash?: string | null;
                };
                region: {
                    /** @constant */
                    kind: "DOCUMENT";
                } | {
                    /** @constant */
                    kind: "BLOCK";
                    blockId: components["schemas"]["id"];
                } | {
                    /** @constant */
                    kind: "BLOCK_RANGE";
                    startBlockId: components["schemas"]["id"];
                    endBlockId: components["schemas"]["id"];
                } | {
                    /** @constant */
                    kind: "SECTION";
                    headingId: components["schemas"]["id"];
                } | {
                    /** @constant */
                    kind: "TEXT_RANGE";
                    blockId: components["schemas"]["id"];
                    from: components["schemas"]["textAnchor"];
                    to: components["schemas"]["textAnchor"];
                    quoteHash: string;
                };
                textAnchor: {
                    offset: number;
                    /** @enum {unknown} */
                    affinity: "BEFORE" | "AFTER";
                    contextHash: string;
                };
                contentNode: components["schemas"]["block"] | components["schemas"]["listItem"] | components["schemas"]["tableRow"] | components["schemas"]["tableCell"];
                insertBlock: components["schemas"]["base"] & {
                    /** @constant */
                    kind: "INSERT_BLOCK";
                    parentId: components["schemas"]["id"] | null;
                    index: number;
                    block: components["schemas"]["contentNode"];
                };
                deleteBlock: components["schemas"]["base"] & {
                    /** @constant */
                    kind: "DELETE_BLOCK";
                    blockId: components["schemas"]["id"];
                };
                moveBlock: components["schemas"]["base"] & {
                    /** @constant */
                    kind: "MOVE_BLOCK";
                    blockId: components["schemas"]["id"];
                    newParentId: components["schemas"]["id"] | null;
                    newIndex: number;
                };
                replaceText: components["schemas"]["base"] & {
                    /** @constant */
                    kind: "REPLACE_TEXT";
                    range: components["schemas"]["region"];
                    content: components["schemas"]["textChildren"];
                };
                setBlockAttrs: components["schemas"]["base"] & {
                    /** @constant */
                    kind: "SET_BLOCK_ATTRS";
                    blockId: components["schemas"]["id"];
                    attrs: {
                        [key: string]: components["schemas"]["attrPatch"];
                    };
                };
                setMarks: components["schemas"]["base"] & {
                    /** @constant */
                    kind: "SET_MARKS";
                    range: components["schemas"]["region"];
                    /** @enum {unknown} */
                    mode: "ADD" | "REMOVE" | "REPLACE";
                    marks: components["schemas"]["mark"][];
                };
                replaceRegion: components["schemas"]["base"] & {
                    /** @constant */
                    kind: "REPLACE_REGION";
                    region: components["schemas"]["region"];
                    blocks: components["schemas"]["contentNode"][];
                };
                addReference: components["schemas"]["base"] & {
                    /** @constant */
                    kind: "ADD_REFERENCE";
                    referenceId: components["schemas"]["id"];
                    sourceRegion: components["schemas"]["region"];
                    target: components["schemas"]["referenceTarget"];
                };
                removeReference: components["schemas"]["base"] & {
                    /** @constant */
                    kind: "REMOVE_REFERENCE";
                    referenceId: components["schemas"]["id"];
                    sourceRegion: components["schemas"]["region"];
                    target: components["schemas"]["referenceTarget"];
                };
                attrPatch: {
                    /** @constant */
                    action: "SET";
                    value: string | number | boolean | null;
                } | {
                    /** @constant */
                    action: "REMOVE";
                };
                referenceTarget: {
                    /** @constant */
                    kind: "REGION";
                    id: components["schemas"]["id"];
                    region: components["schemas"]["region"];
                } | {
                    /** @enum {unknown} */
                    kind: "DOCUMENT" | "DISCUSSION" | "VOCABULARY";
                    id: components["schemas"]["id"];
                } | {
                    /** @constant */
                    kind: "EXTERNAL";
                    /** Format: uri */
                    id: string;
                };
            };
        } & (components["schemas"]["insertBlock"] | components["schemas"]["deleteBlock"] | components["schemas"]["moveBlock"] | components["schemas"]["replaceText"] | components["schemas"]["setBlockAttrs"] | components["schemas"]["setMarks"] | components["schemas"]["replaceRegion"] | components["schemas"]["addReference"] | components["schemas"]["removeReference"]);
        finding: {
            findingId: components["schemas"]["id"];
            ruleId: string;
            /** @enum {unknown} */
            severity: "BLOCKING" | "WARNING" | "ADVISORY";
            region: components["schemas"]["region"];
            reason: string;
            suggestion: string | null;
            sourceIds: components["schemas"]["id"][];
        };
        claim: {
            text: string;
            sourceIds: components["schemas"]["id"][];
            /** @enum {unknown} */
            certainty: "SUPPORTED" | "CONFLICTING" | "INSUFFICIENT";
        };
        conflict: {
            description: string;
            sourceIds: components["schemas"]["id"][];
        };
        result: {
            /** @constant */
            schemaVersion: 1;
            /** @enum {unknown} */
            taskKind: "COMPOSE" | "REWRITE" | "REVIEW" | "DISCUSSION_APPLY" | "CONFLICT_MERGE" | "KNOWLEDGE_QUERY";
            /** @enum {unknown} */
            status: "READY" | "INSUFFICIENT_CONTEXT" | "CONFLICTING_CONTEXT" | "NO_CHANGE";
            operations: components["schemas"]["document-operation.schema"][];
            findings: components["schemas"]["finding"][];
            claims: components["schemas"]["claim"][];
            uncertainties: string[];
            conflicts: components["schemas"]["conflict"][];
            usedSourceIds: components["schemas"]["id"][];
        };
        target: {
            /** @constant */
            kind: "DOCUMENT";
            documentId: components["schemas"]["id"];
        } | {
            /** @constant */
            kind: "REGION";
            documentId: components["schemas"]["id"];
            region: components["schemas"]["region"];
        } | {
            /** @constant */
            kind: "DISCUSSION";
            discussionId: components["schemas"]["id"];
        } | {
            /** @constant */
            kind: "WORKSPACE_QUERY";
            question: string;
        };
        proposal: {
            proposalId: components["schemas"]["id"];
            jobId: components["schemas"]["id"];
            documentId: components["schemas"]["id"];
            baseRevision: number;
            operations: components["schemas"]["document-operation.schema"][];
            /** @enum {unknown} */
            status: "OPEN" | "APPLIED" | "REJECTED" | "STALE" | "CANCELLED";
            revision: number;
            appliedRevision?: number | null;
            appliedOperationIds?: components["schemas"]["id"][];
            /** Format: date-time */
            createdAt?: string;
            /** Format: date-time */
            resolvedAt?: string | null;
        };
    };
    responses: {
        /** @description Domain or transport problem */
        Problem: {
            headers: {
                [name: string]: unknown;
            };
            content: {
                "application/problem+json": components["schemas"]["Problem"];
            };
        };
    };
    parameters: {
        WorkspaceId: components["schemas"]["Id"];
        DocumentId: components["schemas"]["Id"];
        ReviewId: components["schemas"]["Id"];
        JobId: components["schemas"]["Id"];
        ProposalId: components["schemas"]["Id"];
        UserId: components["schemas"]["Id"];
        InvitationId: components["schemas"]["Id"];
        GroupId: components["schemas"]["Id"];
        GrantId: components["schemas"]["Id"];
        VersionId: components["schemas"]["Id"];
        DiscussionId: components["schemas"]["Id"];
        TopicId: components["schemas"]["Id"];
        MessageId: components["schemas"]["Id"];
        ItemId: components["schemas"]["Id"];
        ReferenceId: components["schemas"]["Id"];
        ConceptId: components["schemas"]["Id"];
        AssetId: components["schemas"]["Id"];
        LinkId: components["schemas"]["Id"];
        Cursor: string;
        StatusFilter: "UNREAD" | "ACTIONABLE" | "RESOLVED" | "ALL";
        Range: string;
        UploadToken: string;
        IfMatch: string;
        IdempotencyKey: string;
        CsrfToken: string;
        LeaseToken: string;
        ClientInstance: components["schemas"]["Id"];
    };
    requestBodies: {
        CreateWorkspace: {
            content: {
                "application/json": {
                    name: string;
                };
            };
        };
        MoveDocument: {
            content: {
                "application/json": components["schemas"]["MoveDocumentInput"];
            };
        };
        UpdateWorkspace: {
            content: {
                "application/json": {
                    name?: string;
                    /** @enum {string} */
                    defaultPublishMode?: "DIRECT" | "REVIEW_REQUIRED";
                };
            };
        };
        Reason: {
            content: {
                "application/json": {
                    reason: string;
                };
            };
        };
        InviteMember: {
            content: {
                "application/json": {
                    /** Format: email */
                    email: string;
                    /** @enum {string} */
                    role: "MEMBER" | "ADMIN";
                };
            };
        };
        UpdateMemberRole: {
            content: {
                "application/json": {
                    /** @enum {string} */
                    role: "MEMBER" | "ADMIN" | "OWNER";
                };
            };
        };
        UpdateGroup: {
            content: {
                "application/json": {
                    name: string;
                };
            };
        };
        UpdateDocumentMetadata: {
            content: {
                "application/json": {
                    title: string;
                };
            };
        };
        RestoreDocument: {
            content: {
                "application/json": {
                    parentId: components["schemas"]["NullableId"];
                    afterDocumentId?: components["schemas"]["NullableId"];
                };
            };
        };
        SetPublishPolicy: {
            content: {
                "application/json": {
                    /** @enum {string} */
                    mode: "DIRECT" | "REVIEW_REQUIRED";
                    requiredApprovals: number;
                    reviewerRule: components["schemas"]["ReviewerRule"];
                };
            };
        };
        UpdateDiscussion: {
            content: {
                "application/json": {
                    title: string;
                };
            };
        };
        CreateTopic: {
            content: {
                "application/json": components["schemas"]["TopicInput"];
            };
        };
        CreateMessage: {
            content: {
                "application/json": components["schemas"]["RichMessage"];
            };
        };
        ReadAllInbox: {
            content: {
                "application/json": {
                    /** Format: date-time */
                    before: string;
                };
            };
        };
        CreateReference: {
            content: {
                "application/json": {
                    sourceRegion: components["schemas"]["region"];
                    target: components["schemas"]["referenceTarget"];
                };
            };
        };
        WriteVocabularyConcept: {
            content: {
                "application/json": {
                    canonicalTerm: string;
                    definition: string;
                    terms: components["schemas"]["VocabularyTerm"][];
                };
            };
        };
        DeprecateVocabularyConcept: {
            content: {
                "application/json": {
                    reason: string;
                    replacementConceptId?: components["schemas"]["NullableId"];
                };
            };
        };
        CompleteUpload: {
            content: {
                "application/json": {
                    checksumSha256: string;
                    sizeBytes: number;
                };
            };
        };
        UpdateUserPreferences: {
            content: {
                "application/json": {
                    /** @enum {string} */
                    locale: "ko" | "en";
                    timezone: string;
                    /** @enum {string} */
                    theme: "LIGHT" | "DARK" | "SYSTEM";
                };
            };
        };
        UpdateWritingConfiguration: {
            content: {
                "application/json": {
                    /** @constant */
                    baselineVersion: "writing-rules-v1";
                    overrides: components["schemas"]["WritingRuleOverride"][];
                };
            };
        };
        UpdateAIConfiguration: {
            content: {
                "application/json": {
                    /** @enum {string} */
                    provider: "CODEX_CLI" | "OPENAI_RESPONSES";
                    model: string;
                    userConcurrencyLimit: number;
                    workspaceConcurrencyLimit: number;
                    monthlyBudgetMicrounits: number;
                };
            };
        };
    };
    headers: never;
    pathItems: never;
}
export type $defs = Record<string, never>;
export interface operations {
    getSession: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Session */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["SessionView"];
                };
            };
            default: components["responses"]["Problem"];
        };
    };
    beginGoogleLogin: {
        parameters: {
            query?: {
                returnTo?: string;
            };
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Google authorization redirect */
            302: {
                headers: {
                    Location?: string;
                    [name: string]: unknown;
                };
                content?: never;
            };
            default: components["responses"]["Problem"];
        };
    };
    completeGoogleLogin: {
        parameters: {
            query: {
                code: string;
                state: string;
            };
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Validated application redirect */
            302: {
                headers: {
                    Location?: string;
                    [name: string]: unknown;
                };
                content?: never;
            };
            default: components["responses"]["Problem"];
        };
    };
    logout: {
        parameters: {
            query?: never;
            header: {
                "Idempotency-Key": components["parameters"]["IdempotencyKey"];
                "X-CSRF-Token": components["parameters"]["CsrfToken"];
            };
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Session revoked */
            204: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            default: components["responses"]["Problem"];
        };
    };
    getUserPreferences: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Preferences */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["UserPreferences"];
                };
            };
            default: components["responses"]["Problem"];
        };
    };
    updateUserPreferences: {
        parameters: {
            query?: never;
            header: {
                "If-Match": components["parameters"]["IfMatch"];
                "Idempotency-Key": components["parameters"]["IdempotencyKey"];
                "X-CSRF-Token": components["parameters"]["CsrfToken"];
            };
            path?: never;
            cookie?: never;
        };
        requestBody: components["requestBodies"]["UpdateUserPreferences"];
        responses: {
            /** @description Preferences */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["UserPreferences"];
                };
            };
            default: components["responses"]["Problem"];
        };
    };
    listWorkspaces: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description OK */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Workspace"][];
                };
            };
        };
    };
    createWorkspace: {
        parameters: {
            query?: never;
            header: {
                "Idempotency-Key": components["parameters"]["IdempotencyKey"];
                "X-CSRF-Token": components["parameters"]["CsrfToken"];
            };
            path?: never;
            cookie?: never;
        };
        requestBody: components["requestBodies"]["CreateWorkspace"];
        responses: {
            /** @description Created */
            201: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Workspace"];
                };
            };
            default: components["responses"]["Problem"];
        };
    };
    getWorkspace: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Workspace */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Workspace"];
                };
            };
            default: components["responses"]["Problem"];
        };
    };
    updateWorkspace: {
        parameters: {
            query?: never;
            header: {
                "If-Match": components["parameters"]["IfMatch"];
                "Idempotency-Key": components["parameters"]["IdempotencyKey"];
                "X-CSRF-Token": components["parameters"]["CsrfToken"];
            };
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
            };
            cookie?: never;
        };
        requestBody: components["requestBodies"]["UpdateWorkspace"];
        responses: {
            /** @description Updated */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Workspace"];
                };
            };
            default: components["responses"]["Problem"];
        };
    };
    scheduleWorkspaceDeletion: {
        parameters: {
            query?: never;
            header: {
                "If-Match": components["parameters"]["IfMatch"];
                "Idempotency-Key": components["parameters"]["IdempotencyKey"];
                "X-CSRF-Token": components["parameters"]["CsrfToken"];
            };
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
            };
            cookie?: never;
        };
        requestBody: components["requestBodies"]["Reason"];
        responses: {
            /** @description Scheduled */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Workspace"];
                };
            };
            default: components["responses"]["Problem"];
        };
    };
    cancelWorkspaceDeletion: {
        parameters: {
            query?: never;
            header: {
                "If-Match": components["parameters"]["IfMatch"];
                "Idempotency-Key": components["parameters"]["IdempotencyKey"];
                "X-CSRF-Token": components["parameters"]["CsrfToken"];
            };
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Cancelled */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Workspace"];
                };
            };
            default: components["responses"]["Problem"];
        };
    };
    listMembers: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description OK */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Membership"][];
                };
            };
        };
    };
    updateMemberRole: {
        parameters: {
            query?: never;
            header: {
                "If-Match": components["parameters"]["IfMatch"];
                "Idempotency-Key": components["parameters"]["IdempotencyKey"];
                "X-CSRF-Token": components["parameters"]["CsrfToken"];
            };
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
                userId: components["parameters"]["UserId"];
            };
            cookie?: never;
        };
        requestBody: components["requestBodies"]["UpdateMemberRole"];
        responses: {
            /** @description Updated */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Membership"];
                };
            };
            default: components["responses"]["Problem"];
        };
    };
    removeMember: {
        parameters: {
            query?: never;
            header: {
                "If-Match": components["parameters"]["IfMatch"];
                "Idempotency-Key": components["parameters"]["IdempotencyKey"];
                "X-CSRF-Token": components["parameters"]["CsrfToken"];
            };
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
                userId: components["parameters"]["UserId"];
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Removed */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Membership"];
                };
            };
            default: components["responses"]["Problem"];
        };
    };
    listInvitations: {
        parameters: {
            query?: {
                cursor?: components["parameters"]["Cursor"];
            };
            header?: never;
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Invitations */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["InvitationPage"];
                };
            };
            default: components["responses"]["Problem"];
        };
    };
    inviteMember: {
        parameters: {
            query?: never;
            header: {
                "Idempotency-Key": components["parameters"]["IdempotencyKey"];
                "X-CSRF-Token": components["parameters"]["CsrfToken"];
            };
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
            };
            cookie?: never;
        };
        requestBody: components["requestBodies"]["InviteMember"];
        responses: {
            /** @description Invitation created */
            201: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Invitation"];
                };
            };
            default: components["responses"]["Problem"];
        };
    };
    revokeInvitation: {
        parameters: {
            query?: never;
            header: {
                "If-Match": components["parameters"]["IfMatch"];
                "Idempotency-Key": components["parameters"]["IdempotencyKey"];
                "X-CSRF-Token": components["parameters"]["CsrfToken"];
            };
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
                invitationId: components["parameters"]["InvitationId"];
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Revoked */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Invitation"];
                };
            };
            default: components["responses"]["Problem"];
        };
    };
    acceptInvitation: {
        parameters: {
            query?: never;
            header: {
                "Idempotency-Key": components["parameters"]["IdempotencyKey"];
                "X-CSRF-Token": components["parameters"]["CsrfToken"];
            };
            path: {
                token: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Membership */
            201: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Membership"];
                };
            };
            default: components["responses"]["Problem"];
        };
    };
    listGroups: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description OK */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Group"][];
                };
            };
        };
    };
    createGroup: {
        parameters: {
            query?: never;
            header: {
                "Idempotency-Key": components["parameters"]["IdempotencyKey"];
                "X-CSRF-Token": components["parameters"]["CsrfToken"];
            };
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    name: string;
                    memberIds?: components["schemas"]["Id"][];
                };
            };
        };
        responses: {
            /** @description Created */
            201: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Group"];
                };
            };
            default: components["responses"]["Problem"];
        };
    };
    getGroup: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
                groupId: components["parameters"]["GroupId"];
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Group */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Group"];
                };
            };
            default: components["responses"]["Problem"];
        };
    };
    updateGroup: {
        parameters: {
            query?: never;
            header: {
                "If-Match": components["parameters"]["IfMatch"];
                "Idempotency-Key": components["parameters"]["IdempotencyKey"];
                "X-CSRF-Token": components["parameters"]["CsrfToken"];
            };
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
                groupId: components["parameters"]["GroupId"];
            };
            cookie?: never;
        };
        requestBody: components["requestBodies"]["UpdateGroup"];
        responses: {
            /** @description Updated */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Group"];
                };
            };
            default: components["responses"]["Problem"];
        };
    };
    deleteGroup: {
        parameters: {
            query?: never;
            header: {
                "If-Match": components["parameters"]["IfMatch"];
                "Idempotency-Key": components["parameters"]["IdempotencyKey"];
                "X-CSRF-Token": components["parameters"]["CsrfToken"];
            };
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
                groupId: components["parameters"]["GroupId"];
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Deleted */
            204: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            default: components["responses"]["Problem"];
        };
    };
    addGroupMember: {
        parameters: {
            query?: never;
            header: {
                "If-Match": components["parameters"]["IfMatch"];
                "Idempotency-Key": components["parameters"]["IdempotencyKey"];
                "X-CSRF-Token": components["parameters"]["CsrfToken"];
            };
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
                groupId: components["parameters"]["GroupId"];
                userId: components["parameters"]["UserId"];
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Updated */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Group"];
                };
            };
            default: components["responses"]["Problem"];
        };
    };
    removeGroupMember: {
        parameters: {
            query?: never;
            header: {
                "If-Match": components["parameters"]["IfMatch"];
                "Idempotency-Key": components["parameters"]["IdempotencyKey"];
                "X-CSRF-Token": components["parameters"]["CsrfToken"];
            };
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
                groupId: components["parameters"]["GroupId"];
                userId: components["parameters"]["UserId"];
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Updated */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Group"];
                };
            };
            default: components["responses"]["Problem"];
        };
    };
    createDocument: {
        parameters: {
            query?: never;
            header: {
                "Idempotency-Key": components["parameters"]["IdempotencyKey"];
                "X-CSRF-Token": components["parameters"]["CsrfToken"];
            };
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    title: string;
                    parentId?: components["schemas"]["NullableId"];
                    afterDocumentId?: components["schemas"]["NullableId"];
                };
            };
        };
        responses: {
            /** @description Created */
            201: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Document"];
                };
            };
            default: components["responses"]["Problem"];
        };
    };
    getDocumentTree: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Permission-filtered tree */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["DocumentTree"];
                };
            };
            default: components["responses"]["Problem"];
        };
    };
    listTrashedDocuments: {
        parameters: {
            query?: {
                cursor?: components["parameters"]["Cursor"];
            };
            header?: never;
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Trashed documents */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["DocumentPage"];
                };
            };
            default: components["responses"]["Problem"];
        };
    };
    getDocument: {
        parameters: {
            query?: {
                cursor?: components["parameters"]["Cursor"];
            };
            header?: never;
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
                documentId: components["parameters"]["DocumentId"];
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description OK */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["DocumentDetail"];
                };
            };
            default: components["responses"]["Problem"];
        };
    };
    updateDocumentMetadata: {
        parameters: {
            query?: never;
            header: {
                "If-Match": components["parameters"]["IfMatch"];
                "Idempotency-Key": components["parameters"]["IdempotencyKey"];
                "X-CSRF-Token": components["parameters"]["CsrfToken"];
            };
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
                documentId: components["parameters"]["DocumentId"];
            };
            cookie?: never;
        };
        requestBody: components["requestBodies"]["UpdateDocumentMetadata"];
        responses: {
            /** @description Updated */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Document"];
                };
            };
            default: components["responses"]["Problem"];
        };
    };
    purgeDocument: {
        parameters: {
            query?: never;
            header: {
                "If-Match": components["parameters"]["IfMatch"];
                "Idempotency-Key": components["parameters"]["IdempotencyKey"];
                "X-CSRF-Token": components["parameters"]["CsrfToken"];
            };
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
                documentId: components["parameters"]["DocumentId"];
            };
            cookie?: never;
        };
        requestBody: components["requestBodies"]["Reason"];
        responses: {
            /** @description Purge accepted */
            202: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["JobReference"];
                };
            };
            default: components["responses"]["Problem"];
        };
    };
    trashDocument: {
        parameters: {
            query?: never;
            header: {
                "If-Match": components["parameters"]["IfMatch"];
                "Idempotency-Key": components["parameters"]["IdempotencyKey"];
                "X-CSRF-Token": components["parameters"]["CsrfToken"];
            };
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
                documentId: components["parameters"]["DocumentId"];
            };
            cookie?: never;
        };
        requestBody: components["requestBodies"]["Reason"];
        responses: {
            /** @description Trashed */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Document"];
                };
            };
            default: components["responses"]["Problem"];
        };
    };
    restoreDocument: {
        parameters: {
            query?: never;
            header: {
                "If-Match": components["parameters"]["IfMatch"];
                "Idempotency-Key": components["parameters"]["IdempotencyKey"];
                "X-CSRF-Token": components["parameters"]["CsrfToken"];
            };
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
                documentId: components["parameters"]["DocumentId"];
            };
            cookie?: never;
        };
        requestBody: components["requestBodies"]["RestoreDocument"];
        responses: {
            /** @description Restored */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Document"];
                };
            };
            default: components["responses"]["Problem"];
        };
    };
    previewDocumentMove: {
        parameters: {
            query?: never;
            header: {
                "If-Match": components["parameters"]["IfMatch"];
                "X-CSRF-Token": components["parameters"]["CsrfToken"];
            };
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
                documentId: components["parameters"]["DocumentId"];
            };
            cookie?: never;
        };
        requestBody: components["requestBodies"]["MoveDocument"];
        responses: {
            /** @description Impact */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ImpactPreview"];
                };
            };
            default: components["responses"]["Problem"];
        };
    };
    moveDocument: {
        parameters: {
            query?: never;
            header: {
                "If-Match": components["parameters"]["IfMatch"];
                "Idempotency-Key": components["parameters"]["IdempotencyKey"];
                "X-CSRF-Token": components["parameters"]["CsrfToken"];
            };
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
                documentId: components["parameters"]["DocumentId"];
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": components["schemas"]["MoveDocumentInput"] & {
                    previewToken: string;
                };
            };
        };
        responses: {
            /** @description Moved */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Document"];
                };
            };
            default: components["responses"]["Problem"];
        };
    };
    getDocumentPermissions: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
                documentId: components["parameters"]["DocumentId"];
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description OK */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["PermissionView"];
                };
            };
            default: components["responses"]["Problem"];
        };
    };
    setDocumentPermission: {
        parameters: {
            query?: never;
            header: {
                "If-Match": components["parameters"]["IfMatch"];
                "Idempotency-Key": components["parameters"]["IdempotencyKey"];
            };
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
                documentId: components["parameters"]["DocumentId"];
                grantId: components["parameters"]["GrantId"];
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": components["schemas"]["PermissionGrantInput"];
            };
        };
        responses: {
            /** @description Updated */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["PermissionGrant"];
                };
            };
            default: components["responses"]["Problem"];
        };
    };
    deleteDocumentPermission: {
        parameters: {
            query?: never;
            header: {
                "If-Match": components["parameters"]["IfMatch"];
                "Idempotency-Key": components["parameters"]["IdempotencyKey"];
            };
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
                documentId: components["parameters"]["DocumentId"];
                grantId: components["parameters"]["GrantId"];
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Deleted */
            204: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            default: components["responses"]["Problem"];
        };
    };
    explainEffectivePermission: {
        parameters: {
            query: {
                subjectKind: "USER" | "GROUP";
                subjectId: components["schemas"]["Id"];
            };
            header?: never;
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
                documentId: components["parameters"]["DocumentId"];
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Explanation */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["PermissionExplanation"];
                };
            };
            default: components["responses"]["Problem"];
        };
    };
    getPublishPolicy: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
                documentId: components["parameters"]["DocumentId"];
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Effective policy */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["PublishPolicy"];
                };
            };
            default: components["responses"]["Problem"];
        };
    };
    setPublishPolicy: {
        parameters: {
            query?: never;
            header: {
                "If-Match": components["parameters"]["IfMatch"];
                "Idempotency-Key": components["parameters"]["IdempotencyKey"];
                "X-CSRF-Token": components["parameters"]["CsrfToken"];
            };
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
                documentId: components["parameters"]["DocumentId"];
            };
            cookie?: never;
        };
        requestBody: components["requestBodies"]["SetPublishPolicy"];
        responses: {
            /** @description Updated */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["PublishPolicy"];
                };
            };
            default: components["responses"]["Problem"];
        };
    };
    getDraft: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
                documentId: components["parameters"]["DocumentId"];
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Draft */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Draft"];
                };
            };
            default: components["responses"]["Problem"];
        };
    };
    createOrGetDraft: {
        parameters: {
            query?: never;
            header: {
                "Idempotency-Key": components["parameters"]["IdempotencyKey"];
                "X-CSRF-Token": components["parameters"]["CsrfToken"];
            };
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
                documentId: components["parameters"]["DocumentId"];
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Draft */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Draft"];
                };
            };
            default: components["responses"]["Problem"];
        };
    };
    acquireEditLease: {
        parameters: {
            query?: never;
            header: {
                "If-Match": components["parameters"]["IfMatch"];
                "Idempotency-Key": components["parameters"]["IdempotencyKey"];
                "X-CSRF-Token": components["parameters"]["CsrfToken"];
            };
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
                documentId: components["parameters"]["DocumentId"];
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    clientInstanceId: components["schemas"]["Id"];
                    /** @default false */
                    force?: boolean;
                    reason?: string;
                };
            };
        };
        responses: {
            /** @description Lease */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["EditLease"];
                };
            };
            default: components["responses"]["Problem"];
        };
    };
    releaseEditLease: {
        parameters: {
            query?: never;
            header: {
                "If-Match": components["parameters"]["IfMatch"];
                "Idempotency-Key": components["parameters"]["IdempotencyKey"];
                "X-Edit-Lease": components["parameters"]["LeaseToken"];
                "X-Client-Instance": components["parameters"]["ClientInstance"];
                "X-CSRF-Token": components["parameters"]["CsrfToken"];
            };
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
                documentId: components["parameters"]["DocumentId"];
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Released */
            204: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            default: components["responses"]["Problem"];
        };
    };
    renewEditLease: {
        parameters: {
            query?: never;
            header: {
                "If-Match": components["parameters"]["IfMatch"];
                "Idempotency-Key": components["parameters"]["IdempotencyKey"];
                "X-Edit-Lease": components["parameters"]["LeaseToken"];
                "X-Client-Instance": components["parameters"]["ClientInstance"];
                "X-CSRF-Token": components["parameters"]["CsrfToken"];
            };
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
                documentId: components["parameters"]["DocumentId"];
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Renewed */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["EditLease"];
                };
            };
            default: components["responses"]["Problem"];
        };
    };
    applyDraftOperations: {
        parameters: {
            query?: never;
            header: {
                "If-Match": components["parameters"]["IfMatch"];
                "Idempotency-Key": components["parameters"]["IdempotencyKey"];
                "X-Edit-Lease": components["parameters"]["LeaseToken"];
                "X-Client-Instance": components["parameters"]["ClientInstance"];
                "X-CSRF-Token": components["parameters"]["CsrfToken"];
            };
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
                documentId: components["parameters"]["DocumentId"];
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    operations: components["schemas"]["document-operation.schema"][];
                };
            };
        };
        responses: {
            /** @description Applied */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["MutationResult"];
                };
            };
            default: components["responses"]["Problem"];
        };
    };
    requestReview: {
        parameters: {
            query?: never;
            header: {
                "If-Match": components["parameters"]["IfMatch"];
                "Idempotency-Key": components["parameters"]["IdempotencyKey"];
                "X-CSRF-Token": components["parameters"]["CsrfToken"];
            };
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
                documentId: components["parameters"]["DocumentId"];
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Requested */
            201: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Review"];
                };
            };
            default: components["responses"]["Problem"];
        };
    };
    listVersions: {
        parameters: {
            query?: {
                cursor?: components["parameters"]["Cursor"];
            };
            header?: never;
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
                documentId: components["parameters"]["DocumentId"];
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Versions */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["VersionPage"];
                };
            };
            default: components["responses"]["Problem"];
        };
    };
    getVersion: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
                documentId: components["parameters"]["DocumentId"];
                versionId: components["parameters"]["VersionId"];
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Version */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["PublishedVersion"];
                };
            };
            default: components["responses"]["Problem"];
        };
    };
    restoreVersionToDraft: {
        parameters: {
            query?: never;
            header: {
                "If-Match": components["parameters"]["IfMatch"];
                "Idempotency-Key": components["parameters"]["IdempotencyKey"];
                "X-CSRF-Token": components["parameters"]["CsrfToken"];
            };
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
                documentId: components["parameters"]["DocumentId"];
                versionId: components["parameters"]["VersionId"];
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description New draft copied from the selected immutable version */
            201: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Draft"];
                };
            };
            default: components["responses"]["Problem"];
        };
    };
    compareVersions: {
        parameters: {
            query: {
                from: components["schemas"]["Id"];
                to: components["schemas"]["Id"];
            };
            header?: never;
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
                documentId: components["parameters"]["DocumentId"];
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Diff */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["DocumentDiff"];
                };
            };
            default: components["responses"]["Problem"];
        };
    };
    submitReviewDecision: {
        parameters: {
            query?: never;
            header: {
                "If-Match": components["parameters"]["IfMatch"];
                "Idempotency-Key": components["parameters"]["IdempotencyKey"];
                "X-CSRF-Token": components["parameters"]["CsrfToken"];
            };
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
                reviewId: components["parameters"]["ReviewId"];
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": components["schemas"]["ReviewDecisionInput"];
            };
        };
        responses: {
            /** @description Recorded */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Review"];
                };
            };
            default: components["responses"]["Problem"];
        };
    };
    publishDocument: {
        parameters: {
            query?: never;
            header: {
                "If-Match": components["parameters"]["IfMatch"];
                "Idempotency-Key": components["parameters"]["IdempotencyKey"];
                "X-CSRF-Token": components["parameters"]["CsrfToken"];
            };
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
                documentId: components["parameters"]["DocumentId"];
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    summary: string;
                    clientInstanceId?: components["schemas"]["NullableId"];
                    leaseToken?: string | null;
                };
            };
        };
        responses: {
            /** @description Published */
            201: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["PublishedVersion"];
                };
            };
            default: components["responses"]["Problem"];
        };
    };
    listDiscussions: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
                documentId: components["parameters"]["DocumentId"];
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description OK */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["DiscussionPage"];
                };
            };
        };
    };
    createDiscussion: {
        parameters: {
            query?: never;
            header: {
                "Idempotency-Key": components["parameters"]["IdempotencyKey"];
                "X-CSRF-Token": components["parameters"]["CsrfToken"];
            };
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
                documentId: components["parameters"]["DocumentId"];
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    title: string;
                    message: components["schemas"]["RichMessage"];
                    topics: components["schemas"]["TopicInput"][];
                };
            };
        };
        responses: {
            /** @description Created */
            201: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Discussion"];
                };
            };
            default: components["responses"]["Problem"];
        };
    };
    getDiscussion: {
        parameters: {
            query?: {
                cursor?: components["parameters"]["Cursor"];
            };
            header?: never;
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
                discussionId: components["parameters"]["DiscussionId"];
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Discussion detail */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["DiscussionDetail"];
                };
            };
            default: components["responses"]["Problem"];
        };
    };
    updateDiscussion: {
        parameters: {
            query?: never;
            header: {
                "If-Match": components["parameters"]["IfMatch"];
                "Idempotency-Key": components["parameters"]["IdempotencyKey"];
                "X-CSRF-Token": components["parameters"]["CsrfToken"];
            };
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
                discussionId: components["parameters"]["DiscussionId"];
            };
            cookie?: never;
        };
        requestBody: components["requestBodies"]["UpdateDiscussion"];
        responses: {
            /** @description Updated */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Discussion"];
                };
            };
            default: components["responses"]["Problem"];
        };
    };
    closeDiscussion: {
        parameters: {
            query?: never;
            header: {
                "If-Match": components["parameters"]["IfMatch"];
                "Idempotency-Key": components["parameters"]["IdempotencyKey"];
                "X-CSRF-Token": components["parameters"]["CsrfToken"];
            };
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
                discussionId: components["parameters"]["DiscussionId"];
            };
            cookie?: never;
        };
        requestBody: components["requestBodies"]["Reason"];
        responses: {
            /** @description Closed */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Discussion"];
                };
            };
            default: components["responses"]["Problem"];
        };
    };
    reopenDiscussion: {
        parameters: {
            query?: never;
            header: {
                "If-Match": components["parameters"]["IfMatch"];
                "Idempotency-Key": components["parameters"]["IdempotencyKey"];
                "X-CSRF-Token": components["parameters"]["CsrfToken"];
            };
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
                discussionId: components["parameters"]["DiscussionId"];
            };
            cookie?: never;
        };
        requestBody: components["requestBodies"]["Reason"];
        responses: {
            /** @description Reopened */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Discussion"];
                };
            };
            default: components["responses"]["Problem"];
        };
    };
    addDiscussionTopic: {
        parameters: {
            query?: never;
            header: {
                "If-Match": components["parameters"]["IfMatch"];
                "Idempotency-Key": components["parameters"]["IdempotencyKey"];
                "X-CSRF-Token": components["parameters"]["CsrfToken"];
            };
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
                discussionId: components["parameters"]["DiscussionId"];
            };
            cookie?: never;
        };
        requestBody: components["requestBodies"]["CreateTopic"];
        responses: {
            /** @description Updated discussion */
            201: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Discussion"];
                };
            };
            default: components["responses"]["Problem"];
        };
    };
    removeDiscussionTopic: {
        parameters: {
            query?: never;
            header: {
                "If-Match": components["parameters"]["IfMatch"];
                "Idempotency-Key": components["parameters"]["IdempotencyKey"];
                "X-CSRF-Token": components["parameters"]["CsrfToken"];
            };
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
                discussionId: components["parameters"]["DiscussionId"];
                topicId: components["parameters"]["TopicId"];
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Removed */
            204: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            default: components["responses"]["Problem"];
        };
    };
    createMessage: {
        parameters: {
            query?: never;
            header: {
                "Idempotency-Key": components["parameters"]["IdempotencyKey"];
                "X-CSRF-Token": components["parameters"]["CsrfToken"];
            };
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
                discussionId: components["parameters"]["DiscussionId"];
            };
            cookie?: never;
        };
        requestBody: components["requestBodies"]["CreateMessage"];
        responses: {
            /** @description Message */
            201: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Message"];
                };
            };
            default: components["responses"]["Problem"];
        };
    };
    updateMessage: {
        parameters: {
            query?: never;
            header: {
                "If-Match": components["parameters"]["IfMatch"];
                "Idempotency-Key": components["parameters"]["IdempotencyKey"];
                "X-CSRF-Token": components["parameters"]["CsrfToken"];
            };
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
                discussionId: components["parameters"]["DiscussionId"];
                messageId: components["parameters"]["MessageId"];
            };
            cookie?: never;
        };
        requestBody: components["requestBodies"]["CreateMessage"];
        responses: {
            /** @description Updated */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Message"];
                };
            };
            default: components["responses"]["Problem"];
        };
    };
    deleteMessage: {
        parameters: {
            query?: never;
            header: {
                "If-Match": components["parameters"]["IfMatch"];
                "Idempotency-Key": components["parameters"]["IdempotencyKey"];
                "X-CSRF-Token": components["parameters"]["CsrfToken"];
            };
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
                discussionId: components["parameters"]["DiscussionId"];
                messageId: components["parameters"]["MessageId"];
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Redacted */
            204: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            default: components["responses"]["Problem"];
        };
    };
    getReview: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
                reviewId: components["parameters"]["ReviewId"];
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Review */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Review"];
                };
            };
            default: components["responses"]["Problem"];
        };
    };
    cancelReview: {
        parameters: {
            query?: never;
            header: {
                "If-Match": components["parameters"]["IfMatch"];
                "Idempotency-Key": components["parameters"]["IdempotencyKey"];
                "X-CSRF-Token": components["parameters"]["CsrfToken"];
            };
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
                reviewId: components["parameters"]["ReviewId"];
            };
            cookie?: never;
        };
        requestBody: components["requestBodies"]["Reason"];
        responses: {
            /** @description Cancelled */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Review"];
                };
            };
            default: components["responses"]["Problem"];
        };
    };
    listInbox: {
        parameters: {
            query?: {
                cursor?: components["parameters"]["Cursor"];
                status?: components["parameters"]["StatusFilter"];
            };
            header?: never;
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Inbox */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["InboxPage"];
                };
            };
            default: components["responses"]["Problem"];
        };
    };
    markInboxItemRead: {
        parameters: {
            query?: never;
            header: {
                "Idempotency-Key": components["parameters"]["IdempotencyKey"];
                "X-CSRF-Token": components["parameters"]["CsrfToken"];
            };
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
                itemId: components["parameters"]["ItemId"];
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Item */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["InboxItem"];
                };
            };
            default: components["responses"]["Problem"];
        };
    };
    markAllInboxRead: {
        parameters: {
            query?: never;
            header: {
                "Idempotency-Key": components["parameters"]["IdempotencyKey"];
                "X-CSRF-Token": components["parameters"]["CsrfToken"];
            };
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
            };
            cookie?: never;
        };
        requestBody: components["requestBodies"]["ReadAllInbox"];
        responses: {
            /** @description Count */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["AffectedCount"];
                };
            };
            default: components["responses"]["Problem"];
        };
    };
    resolveInboxItem: {
        parameters: {
            query?: never;
            header: {
                "Idempotency-Key": components["parameters"]["IdempotencyKey"];
                "X-CSRF-Token": components["parameters"]["CsrfToken"];
            };
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
                itemId: components["parameters"]["ItemId"];
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Item */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["InboxItem"];
                };
            };
            default: components["responses"]["Problem"];
        };
    };
    searchKnowledge: {
        parameters: {
            query: {
                q: string;
                includeDrafts?: boolean;
                limit?: number;
                cursor?: string;
            };
            header?: never;
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Results */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["SearchPage"];
                };
            };
            default: components["responses"]["Problem"];
        };
    };
    listBacklinks: {
        parameters: {
            query?: {
                cursor?: components["parameters"]["Cursor"];
            };
            header?: never;
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
                documentId: components["parameters"]["DocumentId"];
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Backlinks */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ReferencePage"];
                };
            };
            default: components["responses"]["Problem"];
        };
    };
    createReference: {
        parameters: {
            query?: never;
            header: {
                "If-Match": components["parameters"]["IfMatch"];
                "Idempotency-Key": components["parameters"]["IdempotencyKey"];
                "X-Edit-Lease": components["parameters"]["LeaseToken"];
                "X-Client-Instance": components["parameters"]["ClientInstance"];
                "X-CSRF-Token": components["parameters"]["CsrfToken"];
            };
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
                documentId: components["parameters"]["DocumentId"];
            };
            cookie?: never;
        };
        requestBody: components["requestBodies"]["CreateReference"];
        responses: {
            /** @description Reference */
            201: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Reference"];
                };
            };
            default: components["responses"]["Problem"];
        };
    };
    deleteReference: {
        parameters: {
            query?: never;
            header: {
                "If-Match": components["parameters"]["IfMatch"];
                "Idempotency-Key": components["parameters"]["IdempotencyKey"];
                "X-Edit-Lease": components["parameters"]["LeaseToken"];
                "X-Client-Instance": components["parameters"]["ClientInstance"];
                "X-CSRF-Token": components["parameters"]["CsrfToken"];
            };
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
                documentId: components["parameters"]["DocumentId"];
                referenceId: components["parameters"]["ReferenceId"];
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Deleted */
            204: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            default: components["responses"]["Problem"];
        };
    };
    listVocabulary: {
        parameters: {
            query?: {
                cursor?: components["parameters"]["Cursor"];
            };
            header?: never;
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Vocabulary */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["VocabularyPage"];
                };
            };
            default: components["responses"]["Problem"];
        };
    };
    createVocabularyConcept: {
        parameters: {
            query?: never;
            header: {
                "Idempotency-Key": components["parameters"]["IdempotencyKey"];
                "X-CSRF-Token": components["parameters"]["CsrfToken"];
            };
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
            };
            cookie?: never;
        };
        requestBody: components["requestBodies"]["WriteVocabularyConcept"];
        responses: {
            /** @description Concept */
            201: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["VocabularyConcept"];
                };
            };
            default: components["responses"]["Problem"];
        };
    };
    getVocabularyConcept: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
                conceptId: components["parameters"]["ConceptId"];
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Concept */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["VocabularyConcept"];
                };
            };
            default: components["responses"]["Problem"];
        };
    };
    updateVocabularyConcept: {
        parameters: {
            query?: never;
            header: {
                "If-Match": components["parameters"]["IfMatch"];
                "Idempotency-Key": components["parameters"]["IdempotencyKey"];
                "X-CSRF-Token": components["parameters"]["CsrfToken"];
            };
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
                conceptId: components["parameters"]["ConceptId"];
            };
            cookie?: never;
        };
        requestBody: components["requestBodies"]["WriteVocabularyConcept"];
        responses: {
            /** @description Updated */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["VocabularyConcept"];
                };
            };
            default: components["responses"]["Problem"];
        };
    };
    deprecateVocabularyConcept: {
        parameters: {
            query?: never;
            header: {
                "If-Match": components["parameters"]["IfMatch"];
                "Idempotency-Key": components["parameters"]["IdempotencyKey"];
                "X-CSRF-Token": components["parameters"]["CsrfToken"];
            };
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
                conceptId: components["parameters"]["ConceptId"];
            };
            cookie?: never;
        };
        requestBody: components["requestBodies"]["DeprecateVocabularyConcept"];
        responses: {
            /** @description Deprecated */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["VocabularyConcept"];
                };
            };
            default: components["responses"]["Problem"];
        };
    };
    listAIJobs: {
        parameters: {
            query?: {
                cursor?: components["parameters"]["Cursor"];
            };
            header?: never;
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Jobs */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["AIJobPage"];
                };
            };
            default: components["responses"]["Problem"];
        };
    };
    createAIJob: {
        parameters: {
            query?: never;
            header: {
                "If-Match": components["parameters"]["IfMatch"];
                "Idempotency-Key": components["parameters"]["IdempotencyKey"];
                "X-CSRF-Token": components["parameters"]["CsrfToken"];
            };
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": components["schemas"]["CreateAIJob"];
            };
        };
        responses: {
            /** @description Queued */
            202: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["AIJob"];
                };
            };
            default: components["responses"]["Problem"];
        };
    };
    getAIJob: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
                jobId: components["parameters"]["JobId"];
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Job */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["AIJob"];
                };
            };
        };
    };
    cancelAIJob: {
        parameters: {
            query?: never;
            header: {
                "If-Match": components["parameters"]["IfMatch"];
                "Idempotency-Key": components["parameters"]["IdempotencyKey"];
                "X-CSRF-Token": components["parameters"]["CsrfToken"];
            };
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
                jobId: components["parameters"]["JobId"];
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Cancellation requested */
            202: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            default: components["responses"]["Problem"];
        };
    };
    previewAIContext: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": components["schemas"]["AIContextRequest"];
            };
        };
        responses: {
            /** @description Permission-safe context preview */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["AIContextPreview"];
                };
            };
            default: components["responses"]["Problem"];
        };
    };
    applyProposal: {
        parameters: {
            query?: never;
            header: {
                "If-Match": components["parameters"]["IfMatch"];
                "Idempotency-Key": components["parameters"]["IdempotencyKey"];
                "X-Edit-Lease": components["parameters"]["LeaseToken"];
                "X-Client-Instance": components["parameters"]["ClientInstance"];
                "X-CSRF-Token": components["parameters"]["CsrfToken"];
            };
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
                proposalId: components["parameters"]["ProposalId"];
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    operationIds?: components["schemas"]["Id"][];
                };
            };
        };
        responses: {
            /** @description Applied */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["MutationResult"];
                };
            };
            default: components["responses"]["Problem"];
        };
    };
    getProposal: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
                proposalId: components["parameters"]["ProposalId"];
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Proposal */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["proposal"];
                };
            };
            default: components["responses"]["Problem"];
        };
    };
    rejectProposal: {
        parameters: {
            query?: never;
            header: {
                "If-Match": components["parameters"]["IfMatch"];
                "Idempotency-Key": components["parameters"]["IdempotencyKey"];
                "X-CSRF-Token": components["parameters"]["CsrfToken"];
            };
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
                proposalId: components["parameters"]["ProposalId"];
            };
            cookie?: never;
        };
        requestBody: components["requestBodies"]["Reason"];
        responses: {
            /** @description Rejected */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["proposal"];
                };
            };
            default: components["responses"]["Problem"];
        };
    };
    createFileUpload: {
        parameters: {
            query?: never;
            header: {
                "Idempotency-Key": components["parameters"]["IdempotencyKey"];
                "X-CSRF-Token": components["parameters"]["CsrfToken"];
            };
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    name: string;
                    mimeType: string;
                    size: number;
                    checksum: string;
                };
            };
        };
        responses: {
            /** @description Upload created */
            201: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["FileUpload"];
                };
            };
            default: components["responses"]["Problem"];
        };
    };
    completeFileUpload: {
        parameters: {
            query?: never;
            header: {
                "If-Match": components["parameters"]["IfMatch"];
                "Idempotency-Key": components["parameters"]["IdempotencyKey"];
                "X-CSRF-Token": components["parameters"]["CsrfToken"];
            };
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
                assetId: components["parameters"]["AssetId"];
            };
            cookie?: never;
        };
        requestBody: components["requestBodies"]["CompleteUpload"];
        responses: {
            /** @description Asset */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["FileAsset"];
                };
            };
            default: components["responses"]["Problem"];
        };
    };
    getFile: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
                assetId: components["parameters"]["AssetId"];
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Metadata */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["FileAsset"];
                };
            };
            default: components["responses"]["Problem"];
        };
    };
    deleteFile: {
        parameters: {
            query?: never;
            header: {
                "If-Match": components["parameters"]["IfMatch"];
                "Idempotency-Key": components["parameters"]["IdempotencyKey"];
                "X-CSRF-Token": components["parameters"]["CsrfToken"];
            };
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
                assetId: components["parameters"]["AssetId"];
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Deleted */
            204: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            default: components["responses"]["Problem"];
        };
    };
    downloadFile: {
        parameters: {
            query?: never;
            header?: {
                Range?: components["parameters"]["Range"];
            };
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
                assetId: components["parameters"]["AssetId"];
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description File bytes */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/octet-stream": string;
                };
            };
            /** @description Partial file bytes */
            206: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/octet-stream": string;
                };
            };
            default: components["responses"]["Problem"];
        };
    };
    uploadFileContent: {
        parameters: {
            query?: never;
            header: {
                "X-Upload-Token": components["parameters"]["UploadToken"];
                "X-CSRF-Token": components["parameters"]["CsrfToken"];
            };
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
                assetId: components["parameters"]["AssetId"];
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/octet-stream": string;
            };
        };
        responses: {
            /** @description Uploaded */
            204: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            default: components["responses"]["Problem"];
        };
    };
    downloadPublicFile: {
        parameters: {
            query?: never;
            header?: {
                Range?: components["parameters"]["Range"];
            };
            path: {
                publicToken: string;
                assetId: components["parameters"]["AssetId"];
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description File bytes */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/octet-stream": string;
                };
            };
            /** @description Partial file bytes */
            206: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/octet-stream": string;
                };
            };
            default: components["responses"]["Problem"];
        };
    };
    listAuditEvents: {
        parameters: {
            query?: {
                cursor?: components["parameters"]["Cursor"];
            };
            header?: never;
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Events */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["AuditPage"];
                };
            };
            default: components["responses"]["Problem"];
        };
    };
    getWritingConfiguration: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Configuration */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["WritingConfiguration"];
                };
            };
            default: components["responses"]["Problem"];
        };
    };
    updateWritingConfiguration: {
        parameters: {
            query?: never;
            header: {
                "If-Match": components["parameters"]["IfMatch"];
                "Idempotency-Key": components["parameters"]["IdempotencyKey"];
                "X-CSRF-Token": components["parameters"]["CsrfToken"];
            };
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
            };
            cookie?: never;
        };
        requestBody: components["requestBodies"]["UpdateWritingConfiguration"];
        responses: {
            /** @description Updated */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["WritingConfiguration"];
                };
            };
            default: components["responses"]["Problem"];
        };
    };
    getAIConfiguration: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Configuration */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["AIConfiguration"];
                };
            };
            default: components["responses"]["Problem"];
        };
    };
    updateAIConfiguration: {
        parameters: {
            query?: never;
            header: {
                "If-Match": components["parameters"]["IfMatch"];
                "Idempotency-Key": components["parameters"]["IdempotencyKey"];
            };
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
            };
            cookie?: never;
        };
        requestBody: components["requestBodies"]["UpdateAIConfiguration"];
        responses: {
            /** @description Updated */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["AIConfiguration"];
                };
            };
            default: components["responses"]["Problem"];
        };
    };
    getAIUsage: {
        parameters: {
            query: {
                from: string;
                to: string;
            };
            header?: never;
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Usage */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["AIUsage"];
                };
            };
            default: components["responses"]["Problem"];
        };
    };
    getAIProviderHealth: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Health without credentials */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["AIProviderHealth"];
                };
            };
            default: components["responses"]["Problem"];
        };
    };
    listPublicLinks: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
                documentId: components["parameters"]["DocumentId"];
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Links without secret tokens */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["PublicLink"][];
                };
            };
            default: components["responses"]["Problem"];
        };
    };
    createPublicLink: {
        parameters: {
            query?: never;
            header: {
                "If-Match": components["parameters"]["IfMatch"];
                "Idempotency-Key": components["parameters"]["IdempotencyKey"];
                "X-CSRF-Token": components["parameters"]["CsrfToken"];
            };
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
                documentId: components["parameters"]["DocumentId"];
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /** Format: date-time */
                    expiresAt?: string | null;
                };
            };
        };
        responses: {
            /** @description Token shown once */
            201: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["PublicLinkCreated"];
                };
            };
            default: components["responses"]["Problem"];
        };
    };
    revokePublicLink: {
        parameters: {
            query?: never;
            header: {
                "If-Match": components["parameters"]["IfMatch"];
                "Idempotency-Key": components["parameters"]["IdempotencyKey"];
                "X-CSRF-Token": components["parameters"]["CsrfToken"];
            };
            path: {
                workspaceId: components["parameters"]["WorkspaceId"];
                documentId: components["parameters"]["DocumentId"];
                linkId: components["parameters"]["LinkId"];
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Revoked */
            204: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            default: components["responses"]["Problem"];
        };
    };
    openWorkspaceStream: {
        parameters: {
            query: {
                workspaceId: components["schemas"]["Id"];
                cursor?: string;
            };
            header?: {
                "Last-Event-ID"?: string;
            };
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Permission-filtered resumable SSE stream */
            200: {
                headers: {
                    "Cache-Control"?: "no-cache, no-store";
                    "X-Accel-Buffering"?: "no";
                    [name: string]: unknown;
                };
                content: {
                    "text/event-stream": string;
                };
            };
            default: components["responses"]["Problem"];
        };
    };
    getPublicDocument: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                token: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Latest published document only */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["PublicDocument"];
                };
            };
            /** @description Unknown, revoked, expired, trashed or unpublished */
            404: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
        };
    };
}
