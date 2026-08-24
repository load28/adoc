/* Generated from canonical schemas. Do not edit. */

export type AdocContractBundle =
  | AIContracts
  | AiContracts_Id
  | AiContracts_Source
  | AiContracts_Task
  | AiContracts_Target
  | AiContracts_Context
  | AiContracts_Finding
  | AiContracts_Claim
  | AiContracts_Conflict
  | AiContracts_Result
  | AiContracts_Proposal
  | DocumentContent
  | DocumentContent_Id
  | DocumentContent_Doc
  | DocumentContent_Block
  | DocumentContent_Inline
  | DocumentContent_Mark
  | DocumentContent_TextChildren
  | DocumentContent_Paragraph
  | DocumentContent_Heading
  | DocumentContent_Quote
  | DocumentContent_Callout
  | DocumentContent_List
  | DocumentContent_ListItem
  | DocumentContent_CodeBlock
  | DocumentContent_Table
  | DocumentContent_TableRow
  | DocumentContent_TableCell
  | DocumentContent_Toggle
  | DocumentContent_Divider
  | DocumentContent_Image
  | DocumentContent_File
  | DocumentOperation
  | DocumentOperation_Id
  | DocumentOperation_Base
  | DocumentOperation_Precondition
  | DocumentOperation_Region
  | DocumentOperation_TextAnchor
  | DocumentOperation_ContentNode
  | DocumentOperation_InsertBlock
  | DocumentOperation_DeleteBlock
  | DocumentOperation_MoveBlock
  | DocumentOperation_ReplaceText
  | DocumentOperation_SetBlockAttrs
  | DocumentOperation_SetMarks
  | DocumentOperation_ReplaceRegion
  | DocumentOperation_AddReference
  | DocumentOperation_RemoveReference
  | DocumentOperation_AttrPatch
  | DocumentOperation_ReferenceTarget
  | EventEnvelope
  | EventPayloads_Id
  | EventPayloads_Revision
  | EventPayloads_Payload
  | EventPayloads_EntityChanged
  | EventPayloads_DocumentMoved
  | EventPayloads_DraftChanged
  | EventPayloads_LeaseChanged
  | EventPayloads_VersionPublished
  | EventPayloads_JobChanged
  | EventPayloads_ProposalApplied
  | EventPayloads_PurgeChanged
  | OpenApi__Id
  | OpenApi__NullableId
  | OpenApi__SessionView
  | OpenApi__UserSummary
  | OpenApi__UserPreferences
  | OpenApi__Workspace
  | OpenApi__Membership
  | OpenApi__Invitation
  | OpenApi__InvitationPage
  | OpenApi__Group
  | OpenApi__Access
  | OpenApi__PermissionGrantInput
  | OpenApi__PermissionGrant
  | OpenApi__PermissionView
  | OpenApi__EffectivePermission
  | OpenApi__PermissionExplanation
  | OpenApi__ReviewerRule
  | OpenApi__PublishPolicy
  | OpenApi__Document
  | OpenApi__DocumentPage
  | OpenApi__DocumentTree
  | OpenApi__DocumentTreeNode
  | OpenApi__JobReference
  | OpenApi__DocumentDetail
  | OpenApi__Draft
  | OpenApi__EditLease
  | OpenApi__MutationResult
  | OpenApi__PublishedVersion
  | OpenApi__VersionPage
  | OpenApi__DocumentDiff
  | OpenApi__MoveDocumentInput
  | OpenApi__ImpactPreview
  | OpenApi__TopicInput
  | OpenApi__Topic
  | OpenApi__RichMessage
  | OpenApi__Discussion
  | OpenApi__Message
  | OpenApi__DiscussionDetail
  | OpenApi__Review
  | OpenApi__InboxItem
  | OpenApi__InboxPage
  | OpenApi__AffectedCount
  | OpenApi__ResourceTarget
  | OpenApi__SearchPage
  | OpenApi__Source
  | OpenApi__Reference
  | OpenApi__ReferencePage
  | OpenApi__VocabularyTerm
  | OpenApi__VocabularyConcept
  | OpenApi__VocabularyPage
  | OpenApi__CreateAIJob
  | OpenApi__AIJob
  | OpenApi__AIJobPage
  | OpenApi__Proposal
  | OpenApi__FileUpload
  | OpenApi__FileAsset
  | OpenApi__AuditEvent
  | OpenApi__AuditPage
  | OpenApi__PublicLink
  | OpenApi__WritingRuleOverride
  | OpenApi__WritingConfiguration
  | OpenApi__AIConfiguration
  | OpenApi__AIUsage
  | OpenApi__AIProviderHealth
  | OpenApi__PublicLinkCreated
  | OpenApi__PublicDocument
  | OpenApi__Problem
  | Operation__GetSessionRequest
  | Operation__GetSessionResponse
  | Operation__BeginGoogleLoginRequest
  | Operation__BeginGoogleLoginResponse
  | Operation__CompleteGoogleLoginRequest
  | Operation__CompleteGoogleLoginResponse
  | Operation__LogoutRequest
  | Operation__LogoutResponse
  | Operation__GetUserPreferencesRequest
  | Operation__GetUserPreferencesResponse
  | Operation__UpdateUserPreferencesRequest
  | Operation__UpdateUserPreferencesResponse
  | Operation__ListWorkspacesRequest
  | Operation__ListWorkspacesResponse
  | Operation__CreateWorkspaceRequest
  | Operation__CreateWorkspaceResponse
  | Operation__GetWorkspaceRequest
  | Operation__GetWorkspaceResponse
  | Operation__UpdateWorkspaceRequest
  | Operation__UpdateWorkspaceResponse
  | Operation__ScheduleWorkspaceDeletionRequest
  | Operation__ScheduleWorkspaceDeletionResponse
  | Operation__CancelWorkspaceDeletionRequest
  | Operation__CancelWorkspaceDeletionResponse
  | Operation__ListMembersRequest
  | Operation__ListMembersResponse
  | Operation__UpdateMemberRoleRequest
  | Operation__UpdateMemberRoleResponse
  | Operation__RemoveMemberRequest
  | Operation__RemoveMemberResponse
  | Operation__ListInvitationsRequest
  | Operation__ListInvitationsResponse
  | Operation__InviteMemberRequest
  | Operation__InviteMemberResponse
  | Operation__RevokeInvitationRequest
  | Operation__RevokeInvitationResponse
  | Operation__AcceptInvitationRequest
  | Operation__AcceptInvitationResponse
  | Operation__ListGroupsRequest
  | Operation__ListGroupsResponse
  | Operation__CreateGroupRequest
  | Operation__CreateGroupResponse
  | Operation__GetGroupRequest
  | Operation__GetGroupResponse
  | Operation__UpdateGroupRequest
  | Operation__UpdateGroupResponse
  | Operation__DeleteGroupRequest
  | Operation__DeleteGroupResponse
  | Operation__AddGroupMemberRequest
  | Operation__AddGroupMemberResponse
  | Operation__RemoveGroupMemberRequest
  | Operation__RemoveGroupMemberResponse
  | Operation__CreateDocumentRequest
  | Operation__CreateDocumentResponse
  | Operation__GetDocumentTreeRequest
  | Operation__GetDocumentTreeResponse
  | Operation__ListTrashedDocumentsRequest
  | Operation__ListTrashedDocumentsResponse
  | Operation__GetDocumentRequest
  | Operation__GetDocumentResponse
  | Operation__UpdateDocumentMetadataRequest
  | Operation__UpdateDocumentMetadataResponse
  | Operation__PurgeDocumentRequest
  | Operation__PurgeDocumentResponse
  | Operation__TrashDocumentRequest
  | Operation__TrashDocumentResponse
  | Operation__RestoreDocumentRequest
  | Operation__RestoreDocumentResponse
  | Operation__PreviewDocumentMoveRequest
  | Operation__PreviewDocumentMoveResponse
  | Operation__MoveDocumentRequest
  | Operation__MoveDocumentResponse
  | Operation__GetDocumentPermissionsRequest
  | Operation__GetDocumentPermissionsResponse
  | Operation__SetDocumentPermissionRequest
  | Operation__SetDocumentPermissionResponse
  | Operation__DeleteDocumentPermissionRequest
  | Operation__DeleteDocumentPermissionResponse
  | Operation__ExplainEffectivePermissionRequest
  | Operation__ExplainEffectivePermissionResponse
  | Operation__GetPublishPolicyRequest
  | Operation__GetPublishPolicyResponse
  | Operation__SetPublishPolicyRequest
  | Operation__SetPublishPolicyResponse
  | Operation__GetDraftRequest
  | Operation__GetDraftResponse
  | Operation__CreateOrGetDraftRequest
  | Operation__CreateOrGetDraftResponse
  | Operation__AcquireEditLeaseRequest
  | Operation__AcquireEditLeaseResponse
  | Operation__ReleaseEditLeaseRequest
  | Operation__ReleaseEditLeaseResponse
  | Operation__RenewEditLeaseRequest
  | Operation__RenewEditLeaseResponse
  | Operation__ApplyDraftOperationsRequest
  | Operation__ApplyDraftOperationsResponse
  | Operation__RequestReviewRequest
  | Operation__RequestReviewResponse
  | Operation__ListVersionsRequest
  | Operation__ListVersionsResponse
  | Operation__GetVersionRequest
  | Operation__GetVersionResponse
  | Operation__CompareVersionsRequest
  | Operation__CompareVersionsResponse
  | Operation__SubmitReviewDecisionRequest
  | Operation__SubmitReviewDecisionResponse
  | Operation__PublishDocumentRequest
  | Operation__PublishDocumentResponse
  | Operation__ListDiscussionsRequest
  | Operation__ListDiscussionsResponse
  | Operation__CreateDiscussionRequest
  | Operation__CreateDiscussionResponse
  | Operation__GetDiscussionRequest
  | Operation__GetDiscussionResponse
  | Operation__UpdateDiscussionRequest
  | Operation__UpdateDiscussionResponse
  | Operation__CloseDiscussionRequest
  | Operation__CloseDiscussionResponse
  | Operation__ReopenDiscussionRequest
  | Operation__ReopenDiscussionResponse
  | Operation__AddDiscussionTopicRequest
  | Operation__AddDiscussionTopicResponse
  | Operation__RemoveDiscussionTopicRequest
  | Operation__RemoveDiscussionTopicResponse
  | Operation__CreateMessageRequest
  | Operation__CreateMessageResponse
  | Operation__UpdateMessageRequest
  | Operation__UpdateMessageResponse
  | Operation__DeleteMessageRequest
  | Operation__DeleteMessageResponse
  | Operation__GetReviewRequest
  | Operation__GetReviewResponse
  | Operation__CancelReviewRequest
  | Operation__CancelReviewResponse
  | Operation__ListInboxRequest
  | Operation__ListInboxResponse
  | Operation__MarkInboxItemReadRequest
  | Operation__MarkInboxItemReadResponse
  | Operation__MarkAllInboxReadRequest
  | Operation__MarkAllInboxReadResponse
  | Operation__ResolveInboxItemRequest
  | Operation__ResolveInboxItemResponse
  | Operation__SearchKnowledgeRequest
  | Operation__SearchKnowledgeResponse
  | Operation__ListBacklinksRequest
  | Operation__ListBacklinksResponse
  | Operation__CreateReferenceRequest
  | Operation__CreateReferenceResponse
  | Operation__DeleteReferenceRequest
  | Operation__DeleteReferenceResponse
  | Operation__ListVocabularyRequest
  | Operation__ListVocabularyResponse
  | Operation__CreateVocabularyConceptRequest
  | Operation__CreateVocabularyConceptResponse
  | Operation__GetVocabularyConceptRequest
  | Operation__GetVocabularyConceptResponse
  | Operation__UpdateVocabularyConceptRequest
  | Operation__UpdateVocabularyConceptResponse
  | Operation__DeprecateVocabularyConceptRequest
  | Operation__DeprecateVocabularyConceptResponse
  | Operation__ListAIJobsRequest
  | Operation__ListAIJobsResponse
  | Operation__CreateAIJobRequest
  | Operation__CreateAIJobResponse
  | Operation__GetAIJobRequest
  | Operation__GetAIJobResponse
  | Operation__CancelAIJobRequest
  | Operation__CancelAIJobResponse
  | Operation__ApplyProposalRequest
  | Operation__ApplyProposalResponse
  | Operation__GetProposalRequest
  | Operation__GetProposalResponse
  | Operation__RejectProposalRequest
  | Operation__RejectProposalResponse
  | Operation__CreateFileUploadRequest
  | Operation__CreateFileUploadResponse
  | Operation__CompleteFileUploadRequest
  | Operation__CompleteFileUploadResponse
  | Operation__GetFileRequest
  | Operation__GetFileResponse
  | Operation__DeleteFileRequest
  | Operation__DeleteFileResponse
  | Operation__DownloadFileRequest
  | Operation__DownloadFileResponse
  | Operation__ListAuditEventsRequest
  | Operation__ListAuditEventsResponse
  | Operation__GetWritingConfigurationRequest
  | Operation__GetWritingConfigurationResponse
  | Operation__UpdateWritingConfigurationRequest
  | Operation__UpdateWritingConfigurationResponse
  | Operation__GetAIConfigurationRequest
  | Operation__GetAIConfigurationResponse
  | Operation__UpdateAIConfigurationRequest
  | Operation__UpdateAIConfigurationResponse
  | Operation__GetAIUsageRequest
  | Operation__GetAIUsageResponse
  | Operation__GetAIProviderHealthRequest
  | Operation__GetAIProviderHealthResponse
  | Operation__ListPublicLinksRequest
  | Operation__ListPublicLinksResponse
  | Operation__CreatePublicLinkRequest
  | Operation__CreatePublicLinkResponse
  | Operation__RevokePublicLinkRequest
  | Operation__RevokePublicLinkResponse
  | Operation__OpenWorkspaceStreamRequest
  | Operation__OpenWorkspaceStreamResponse
  | Operation__GetPublicDocumentRequest
  | Operation__GetPublicDocumentResponse
  | Operation__GetPublicDocumentAssetRequest
  | Operation__GetPublicDocumentAssetResponse
  | AsyncApi__StreamHeaders
  | AsyncApi__OutboxHeaders
  | AsyncApi__WorkspaceEvent
  | AsyncApi__DomainEvent;
export type AIContracts = AiContracts_Task | AiContracts_Context | AiContracts_Result | AiContracts_Proposal;
export type AiContracts_Id = string;
export type AiContracts_Target =
  | {
      kind: "DOCUMENT";
      documentId: AiContracts_Id;
    }
  | {
      kind: "REGION";
      documentId: AiContracts_Id;
      region: DocumentOperation_Region;
    }
  | {
      kind: "DISCUSSION";
      discussionId: AiContracts_Id;
    }
  | {
      kind: "WORKSPACE_QUERY";
      question: string;
    };
export type DocumentOperation_Region =
  | {
      kind: "DOCUMENT";
    }
  | {
      kind: "BLOCK";
      blockId: DocumentOperation_Id;
    }
  | {
      kind: "BLOCK_RANGE";
      startBlockId: DocumentOperation_Id;
      endBlockId: DocumentOperation_Id;
    }
  | {
      kind: "SECTION";
      headingId: DocumentOperation_Id;
    }
  | {
      kind: "TEXT_RANGE";
      blockId: DocumentOperation_Id;
      from: DocumentOperation_TextAnchor;
      to: DocumentOperation_TextAnchor;
      quoteHash: string;
    };
export type DocumentOperation_Id = string;
export type DocumentOperation =
  | DocumentOperation_InsertBlock
  | DocumentOperation_DeleteBlock
  | DocumentOperation_MoveBlock
  | DocumentOperation_ReplaceText
  | DocumentOperation_SetBlockAttrs
  | DocumentOperation_SetMarks
  | DocumentOperation_ReplaceRegion
  | DocumentOperation_AddReference
  | DocumentOperation_RemoveReference;
export type DocumentOperation_InsertBlock = DocumentOperation_Base & {
  kind: "INSERT_BLOCK";
  parentId: DocumentOperation_Id | null;
  index: number;
  block: DocumentOperation_ContentNode;
  [k: string]: unknown;
};
export type DocumentOperation_ContentNode =
  DocumentContent_Block | DocumentContent_ListItem | DocumentContent_TableRow | DocumentContent_TableCell;
export type DocumentContent_Block =
  | DocumentContent_Paragraph
  | DocumentContent_Heading
  | DocumentContent_Quote
  | DocumentContent_Callout
  | DocumentContent_List
  | DocumentContent_CodeBlock
  | DocumentContent_Table
  | DocumentContent_Toggle
  | DocumentContent_Divider
  | DocumentContent_Image
  | DocumentContent_File;
export type DocumentContent_Id = string;
export type DocumentContent_Inline =
  | {
      type: "text";
      text: string;
      marks?: DocumentContent_Mark[];
    }
  | {
      type: "hardBreak";
    };
export type DocumentContent_Mark =
  | {
      type: "bold" | "italic" | "underline" | "strike" | "code" | "subscript" | "superscript";
    }
  | {
      type: "link";
      href: string;
      title?: string | null;
    }
  | {
      type: "highlight" | "textColor";
      token: string;
    };
/**
 * @maxItems 100000
 */
export type DocumentContent_TextChildren = DocumentContent_Inline[];
export type DocumentOperation_DeleteBlock = DocumentOperation_Base & {
  kind: "DELETE_BLOCK";
  blockId: DocumentOperation_Id;
  [k: string]: unknown;
};
export type DocumentOperation_MoveBlock = DocumentOperation_Base & {
  kind: "MOVE_BLOCK";
  blockId: DocumentOperation_Id;
  newParentId: DocumentOperation_Id | null;
  newIndex: number;
  [k: string]: unknown;
};
export type DocumentOperation_ReplaceText = DocumentOperation_Base & {
  kind: "REPLACE_TEXT";
  range: DocumentOperation_Region;
  content: DocumentContent_TextChildren;
  [k: string]: unknown;
};
export type DocumentOperation_SetBlockAttrs = DocumentOperation_Base & {
  kind: "SET_BLOCK_ATTRS";
  blockId: DocumentOperation_Id;
  attrs: {
    [k: string]: DocumentOperation_AttrPatch;
  };
  [k: string]: unknown;
};
export type DocumentOperation_AttrPatch =
  | {
      action: "SET";
      value: string | number | boolean | null;
    }
  | {
      action: "REMOVE";
    };
export type DocumentOperation_SetMarks = DocumentOperation_Base & {
  kind: "SET_MARKS";
  range: DocumentOperation_Region;
  mode: "ADD" | "REMOVE" | "REPLACE";
  marks: DocumentContent_Mark[];
  [k: string]: unknown;
};
export type DocumentOperation_ReplaceRegion = DocumentOperation_Base & {
  kind: "REPLACE_REGION";
  region: DocumentOperation_Region;
  blocks: DocumentOperation_ContentNode[];
  [k: string]: unknown;
};
export type DocumentOperation_AddReference = DocumentOperation_Base & {
  kind: "ADD_REFERENCE";
  referenceId: DocumentOperation_Id;
  sourceRegion: DocumentOperation_Region;
  target: DocumentOperation_ReferenceTarget;
  [k: string]: unknown;
};
export type DocumentOperation_RemoveReference = DocumentOperation_Base & {
  kind: "REMOVE_REFERENCE";
  referenceId: DocumentOperation_Id;
  sourceRegion: DocumentOperation_Region;
  target: DocumentOperation_ReferenceTarget;
  [k: string]: unknown;
};
export type EventEnvelope = {
  [k: string]: unknown;
};
export type EventPayloads_Id = string;
export type EventPayloads_Revision = number;
export type EventPayloads_Payload =
  | EventPayloads_EntityChanged
  | EventPayloads_DocumentMoved
  | EventPayloads_DraftChanged
  | EventPayloads_LeaseChanged
  | EventPayloads_VersionPublished
  | EventPayloads_JobChanged
  | EventPayloads_ProposalApplied
  | EventPayloads_PurgeChanged;
export type OpenApi__Id = string;
export type OpenApi__NullableId = OpenApi__Id | null;
export type OpenApi__Access = "NO_ACCESS" | "VIEWER" | "CONTRIBUTOR" | "EDITOR";
export type OpenApi__ReviewerRule =
  | {
      kind: "ANY_EDITOR";
    }
  | {
      kind: "USERS";
      /**
       * @minItems 1
       */
      userIds: [OpenApi__Id, ...OpenApi__Id[]];
    }
  | {
      kind: "GROUPS";
      /**
       * @minItems 1
       */
      groupIds: [OpenApi__Id, ...OpenApi__Id[]];
    };
export type OpenApi__DocumentDetail = OpenApi__Document & {
  draft?: OpenApi__Draft | null;
  publishedVersion?: OpenApi__PublishedVersion | null;
  [k: string]: unknown;
};
export type OpenApi__Proposal = AiContracts_Proposal & {
  revision?: number;
  [k: string]: unknown;
};
export type Operation__GetSessionResponse =
  | {
      status: "200";
      body: OpenApi__SessionView;
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__BeginGoogleLoginResponse =
  | {
      status: "302";
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__CompleteGoogleLoginResponse =
  | {
      status: "302";
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__LogoutResponse =
  | {
      status: "204";
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__GetUserPreferencesResponse =
  | {
      status: "200";
      body: OpenApi__UserPreferences;
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__UpdateUserPreferencesResponse =
  | {
      status: "200";
      body: OpenApi__UserPreferences;
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__ListWorkspacesResponse = {
  status: "200";
  body: OpenApi__Workspace[];
};
export type Operation__CreateWorkspaceResponse =
  | {
      status: "201";
      body: OpenApi__Workspace;
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__GetWorkspaceResponse =
  | {
      status: "200";
      body: OpenApi__Workspace;
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__UpdateWorkspaceResponse =
  | {
      status: "200";
      body: OpenApi__Workspace;
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__ScheduleWorkspaceDeletionResponse =
  | {
      status: "200";
      body: OpenApi__Workspace;
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__CancelWorkspaceDeletionResponse =
  | {
      status: "200";
      body: OpenApi__Workspace;
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__ListMembersResponse = {
  status: "200";
  body: OpenApi__Membership[];
};
export type Operation__UpdateMemberRoleResponse =
  | {
      status: "200";
      body: OpenApi__Membership;
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__RemoveMemberResponse =
  | {
      status: "200";
      body: OpenApi__Membership;
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__ListInvitationsResponse =
  | {
      status: "200";
      body: OpenApi__InvitationPage;
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__InviteMemberResponse =
  | {
      status: "201";
      body: OpenApi__Invitation;
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__RevokeInvitationResponse =
  | {
      status: "200";
      body: OpenApi__Invitation;
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__AcceptInvitationResponse =
  | {
      status: "201";
      body: OpenApi__Membership;
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__ListGroupsResponse = {
  status: "200";
  body: OpenApi__Group[];
};
export type Operation__CreateGroupResponse =
  | {
      status: "201";
      body: OpenApi__Group;
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__GetGroupResponse =
  | {
      status: "200";
      body: OpenApi__Group;
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__UpdateGroupResponse =
  | {
      status: "200";
      body: OpenApi__Group;
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__DeleteGroupResponse =
  | {
      status: "204";
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__AddGroupMemberResponse =
  | {
      status: "200";
      body: OpenApi__Group;
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__RemoveGroupMemberResponse =
  | {
      status: "200";
      body: OpenApi__Group;
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__CreateDocumentResponse =
  | {
      status: "201";
      body: OpenApi__Document;
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__GetDocumentTreeResponse =
  | {
      status: "200";
      body: OpenApi__DocumentTree;
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__ListTrashedDocumentsResponse =
  | {
      status: "200";
      body: OpenApi__DocumentPage;
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__GetDocumentResponse =
  | {
      status: "200";
      body: OpenApi__DocumentDetail;
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__UpdateDocumentMetadataResponse =
  | {
      status: "200";
      body: OpenApi__Document;
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__PurgeDocumentResponse =
  | {
      status: "202";
      body: OpenApi__JobReference;
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__TrashDocumentResponse =
  | {
      status: "200";
      body: OpenApi__Document;
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__RestoreDocumentResponse =
  | {
      status: "200";
      body: OpenApi__Document;
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__PreviewDocumentMoveResponse =
  | {
      status: "200";
      body: OpenApi__ImpactPreview;
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__MoveDocumentResponse =
  | {
      status: "200";
      body: OpenApi__Document;
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__GetDocumentPermissionsResponse =
  | {
      status: "200";
      body: OpenApi__PermissionView;
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__SetDocumentPermissionResponse =
  | {
      status: "200";
      body: OpenApi__PermissionGrant;
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__DeleteDocumentPermissionResponse =
  | {
      status: "204";
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__ExplainEffectivePermissionResponse =
  | {
      status: "200";
      body: OpenApi__PermissionExplanation;
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__GetPublishPolicyResponse =
  | {
      status: "200";
      body: OpenApi__PublishPolicy;
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__SetPublishPolicyResponse =
  | {
      status: "200";
      body: OpenApi__PublishPolicy;
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__GetDraftResponse =
  | {
      status: "200";
      body: OpenApi__Draft;
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__CreateOrGetDraftResponse =
  | {
      status: "200";
      body: OpenApi__Draft;
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__AcquireEditLeaseResponse =
  | {
      status: "200";
      body: OpenApi__EditLease;
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__ReleaseEditLeaseResponse =
  | {
      status: "204";
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__RenewEditLeaseResponse =
  | {
      status: "200";
      body: OpenApi__EditLease;
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__ApplyDraftOperationsResponse =
  | {
      status: "200";
      body: OpenApi__MutationResult;
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__RequestReviewResponse =
  | {
      status: "201";
      body: OpenApi__Review;
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__ListVersionsResponse =
  | {
      status: "200";
      body: OpenApi__VersionPage;
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__GetVersionResponse =
  | {
      status: "200";
      body: OpenApi__PublishedVersion;
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__CompareVersionsResponse =
  | {
      status: "200";
      body: OpenApi__DocumentDiff;
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__SubmitReviewDecisionResponse =
  | {
      status: "200";
      body: OpenApi__Review;
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__PublishDocumentResponse =
  | {
      status: "201";
      body: OpenApi__PublishedVersion;
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__ListDiscussionsResponse = {
  status: "200";
  body: OpenApi__Discussion[];
};
export type Operation__CreateDiscussionResponse =
  | {
      status: "201";
      body: OpenApi__Discussion;
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__GetDiscussionResponse =
  | {
      status: "200";
      body: OpenApi__DiscussionDetail;
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__UpdateDiscussionResponse =
  | {
      status: "200";
      body: OpenApi__Discussion;
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__CloseDiscussionResponse =
  | {
      status: "200";
      body: OpenApi__Discussion;
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__ReopenDiscussionResponse =
  | {
      status: "200";
      body: OpenApi__Discussion;
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__AddDiscussionTopicResponse =
  | {
      status: "201";
      body: OpenApi__Topic;
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__RemoveDiscussionTopicResponse =
  | {
      status: "204";
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__CreateMessageResponse =
  | {
      status: "201";
      body: OpenApi__Message;
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__UpdateMessageResponse =
  | {
      status: "200";
      body: OpenApi__Message;
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__DeleteMessageResponse =
  | {
      status: "204";
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__GetReviewResponse =
  | {
      status: "200";
      body: OpenApi__Review;
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__CancelReviewResponse =
  | {
      status: "200";
      body: OpenApi__Review;
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__ListInboxResponse =
  | {
      status: "200";
      body: OpenApi__InboxPage;
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__MarkInboxItemReadResponse =
  | {
      status: "200";
      body: OpenApi__InboxItem;
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__MarkAllInboxReadResponse =
  | {
      status: "200";
      body: OpenApi__AffectedCount;
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__ResolveInboxItemResponse =
  | {
      status: "200";
      body: OpenApi__InboxItem;
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__SearchKnowledgeResponse =
  | {
      status: "200";
      body: OpenApi__SearchPage;
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__ListBacklinksResponse =
  | {
      status: "200";
      body: OpenApi__ReferencePage;
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__CreateReferenceResponse =
  | {
      status: "201";
      body: OpenApi__Reference;
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__DeleteReferenceResponse =
  | {
      status: "204";
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__ListVocabularyResponse =
  | {
      status: "200";
      body: OpenApi__VocabularyPage;
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__CreateVocabularyConceptResponse =
  | {
      status: "201";
      body: OpenApi__VocabularyConcept;
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__GetVocabularyConceptResponse =
  | {
      status: "200";
      body: OpenApi__VocabularyConcept;
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__UpdateVocabularyConceptResponse =
  | {
      status: "200";
      body: OpenApi__VocabularyConcept;
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__DeprecateVocabularyConceptResponse =
  | {
      status: "200";
      body: OpenApi__VocabularyConcept;
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__ListAIJobsResponse =
  | {
      status: "200";
      body: OpenApi__AIJobPage;
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__CreateAIJobResponse =
  | {
      status: "202";
      body: OpenApi__AIJob;
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__GetAIJobResponse = {
  status: "200";
  body: OpenApi__AIJob;
};
export type Operation__CancelAIJobResponse =
  | {
      status: "202";
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__ApplyProposalResponse =
  | {
      status: "200";
      body: OpenApi__MutationResult;
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__GetProposalResponse =
  | {
      status: "200";
      body: OpenApi__Proposal;
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__RejectProposalResponse =
  | {
      status: "200";
      body: OpenApi__Proposal;
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__CreateFileUploadResponse =
  | {
      status: "201";
      body: OpenApi__FileUpload;
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__CompleteFileUploadResponse =
  | {
      status: "200";
      body: OpenApi__FileAsset;
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__GetFileResponse =
  | {
      status: "200";
      body: OpenApi__FileAsset;
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__DeleteFileResponse =
  | {
      status: "204";
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__DownloadFileResponse =
  | {
      status: "200";
      body: string;
    }
  | {
      status: "206";
      body: string;
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__ListAuditEventsResponse =
  | {
      status: "200";
      body: OpenApi__AuditPage;
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__GetWritingConfigurationResponse =
  | {
      status: "200";
      body: OpenApi__WritingConfiguration;
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__UpdateWritingConfigurationResponse =
  | {
      status: "200";
      body: OpenApi__WritingConfiguration;
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__GetAIConfigurationResponse =
  | {
      status: "200";
      body: OpenApi__AIConfiguration;
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__UpdateAIConfigurationResponse =
  | {
      status: "200";
      body: OpenApi__AIConfiguration;
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__GetAIUsageResponse =
  | {
      status: "200";
      body: OpenApi__AIUsage;
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__GetAIProviderHealthResponse =
  | {
      status: "200";
      body: OpenApi__AIProviderHealth;
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__ListPublicLinksResponse =
  | {
      status: "200";
      body: OpenApi__PublicLink[];
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__CreatePublicLinkResponse =
  | {
      status: "201";
      body: OpenApi__PublicLinkCreated;
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__RevokePublicLinkResponse =
  | {
      status: "204";
    }
  | {
      status: "default";
      body: OpenApi__Problem;
    };
export type Operation__OpenWorkspaceStreamResponse = {
  status: "200";
  body: string;
};
export type Operation__GetPublicDocumentResponse =
  | {
      status: "200";
      body: OpenApi__PublicDocument;
    }
  | {
      status: "404";
    };
export type Operation__GetPublicDocumentAssetResponse =
  | {
      status: "200";
      body: string;
    }
  | {
      status: "404";
    };

export interface AiContracts_Task {
  kind: "COMPOSE" | "REWRITE" | "REVIEW" | "DISCUSSION_APPLY" | "CONFLICT_MERGE" | "KNOWLEDGE_QUERY";
  workspaceId: AiContracts_Id;
  actorId: AiContracts_Id;
  target: AiContracts_Target;
  expectedRevision: number;
  externalWebEnabled: boolean;
  instruction?: string;
}
export interface DocumentOperation_TextAnchor {
  offset: number;
  affinity: "BEFORE" | "AFTER";
  contextHash: string;
}
export interface AiContracts_Context {
  task: AiContracts_Task;
  /**
   * @maxItems 200
   */
  sources: AiContracts_Source[];
  writingRuleVersion: string;
  vocabularyRevision?: number;
}
export interface AiContracts_Source {
  sourceId: AiContracts_Id;
  kind: "DRAFT" | "PUBLISHED_REGION" | "DISCUSSION" | "VOCABULARY" | "EXTERNAL_WEB" | "USER_INPUT";
  stableId: string;
  authority: "USER_EXPLICIT" | "OFFICIAL" | "VOCABULARY" | "DISCUSSION_CONFIRMED" | "RELATED_INTERNAL" | "EXTERNAL";
  snapshotHash: string;
  retrievedAt?: string | null;
}
export interface AiContracts_Result {
  schemaVersion: 1;
  taskKind: "COMPOSE" | "REWRITE" | "REVIEW" | "DISCUSSION_APPLY" | "CONFLICT_MERGE" | "KNOWLEDGE_QUERY";
  status: "READY" | "INSUFFICIENT_CONTEXT" | "CONFLICTING_CONTEXT" | "NO_CHANGE";
  operations: DocumentOperation[];
  findings: AiContracts_Finding[];
  claims: AiContracts_Claim[];
  uncertainties: string[];
  conflicts: AiContracts_Conflict[];
  usedSourceIds: AiContracts_Id[];
}
export interface DocumentOperation_Base {
  opId: DocumentOperation_Id;
  kind: string;
  scope: DocumentOperation_Region;
  precondition: DocumentOperation_Precondition;
  dependsOn?: DocumentOperation_Id[];
  [k: string]: unknown;
}
export interface DocumentOperation_Precondition {
  draftRevision: number;
  targetHash?: string | null;
}
export interface DocumentContent_Paragraph {
  id: DocumentContent_Id;
  type: "paragraph";
  children: DocumentContent_TextChildren;
}
export interface DocumentContent_Heading {
  id: DocumentContent_Id;
  type: "heading";
  level: number;
  children: DocumentContent_TextChildren;
}
export interface DocumentContent_Quote {
  id: DocumentContent_Id;
  type: "quote";
  /**
   * @minItems 1
   */
  children: [DocumentContent_Paragraph | DocumentContent_List, ...(DocumentContent_Paragraph | DocumentContent_List)[]];
}
export interface DocumentContent_List {
  id: DocumentContent_Id;
  type: "bulletList" | "orderedList" | "taskList";
  start?: number;
  /**
   * @minItems 1
   */
  items: [DocumentContent_ListItem, ...DocumentContent_ListItem[]];
}
export interface DocumentContent_ListItem {
  id: DocumentContent_Id;
  type: "listItem";
  checked?: boolean | null;
  /**
   * @minItems 1
   */
  children: [DocumentContent_Paragraph | DocumentContent_List, ...(DocumentContent_Paragraph | DocumentContent_List)[]];
}
export interface DocumentContent_Callout {
  id: DocumentContent_Id;
  type: "callout";
  tone: "info" | "success" | "warning" | "danger" | "note";
  icon?: string | null;
  /**
   * @minItems 1
   */
  children: [DocumentContent_Paragraph | DocumentContent_List, ...(DocumentContent_Paragraph | DocumentContent_List)[]];
}
export interface DocumentContent_CodeBlock {
  id: DocumentContent_Id;
  type: "codeBlock";
  language?: string | null;
  text: string;
}
export interface DocumentContent_Table {
  id: DocumentContent_Id;
  type: "table";
  /**
   * @minItems 1
   * @maxItems 1000
   */
  rows: [DocumentContent_TableRow, ...DocumentContent_TableRow[]];
}
export interface DocumentContent_TableRow {
  id: DocumentContent_Id;
  type: "tableRow";
  /**
   * @minItems 1
   * @maxItems 100
   */
  cells: [DocumentContent_TableCell, ...DocumentContent_TableCell[]];
}
export interface DocumentContent_TableCell {
  id: DocumentContent_Id;
  type: "tableCell" | "tableHeader";
  colspan?: number;
  rowspan?: number;
  /**
   * @minItems 1
   */
  children: [
    DocumentContent_Paragraph | DocumentContent_List | DocumentContent_CodeBlock,
    ...(DocumentContent_Paragraph | DocumentContent_List | DocumentContent_CodeBlock)[]
  ];
}
export interface DocumentContent_Toggle {
  id: DocumentContent_Id;
  type: "toggle";
  summary: DocumentContent_TextChildren;
  children: DocumentContent_Block[];
}
export interface DocumentContent_Divider {
  id: DocumentContent_Id;
  type: "divider";
}
export interface DocumentContent_Image {
  id: DocumentContent_Id;
  type: "image";
  assetId: DocumentContent_Id;
  alt: string;
  caption?: string | null;
  width?: number | null;
}
export interface DocumentContent_File {
  id: DocumentContent_Id;
  type: "file";
  assetId: DocumentContent_Id;
  caption?: string | null;
}
export interface DocumentOperation_ReferenceTarget {
  kind: "DOCUMENT" | "REGION" | "DISCUSSION" | "VOCABULARY" | "EXTERNAL";
  id: string;
}
export interface AiContracts_Finding {
  findingId: AiContracts_Id;
  ruleId: string;
  severity: "BLOCKING" | "WARNING" | "ADVISORY";
  region: DocumentOperation_Region;
  reason: string;
  suggestion: string | null;
  sourceIds: AiContracts_Id[];
}
export interface AiContracts_Claim {
  text: string;
  sourceIds: AiContracts_Id[];
  certainty: "SUPPORTED" | "CONFLICTING" | "INSUFFICIENT";
}
export interface AiContracts_Conflict {
  description: string;
  /**
   * @minItems 2
   */
  sourceIds: [AiContracts_Id, AiContracts_Id, ...AiContracts_Id[]];
}
export interface AiContracts_Proposal {
  proposalId: AiContracts_Id;
  jobId: AiContracts_Id;
  documentId: AiContracts_Id;
  baseRevision: number;
  operations: DocumentOperation[];
  status: "OPEN" | "APPLIED" | "REJECTED" | "STALE" | "CANCELLED";
}
export interface DocumentContent {
  schemaVersion: 1;
  root: DocumentContent_Doc;
}
export interface DocumentContent_Doc {
  type: "doc";
  /**
   * @maxItems 50000
   */
  children: DocumentContent_Block[];
}
export interface EventPayloads_EntityChanged {
  entityId: EventPayloads_Id;
  revision: EventPayloads_Revision;
  action: "CREATED" | "UPDATED" | "DELETED" | "INVALIDATED" | "RESTORED" | "CLOSED" | "REOPENED";
}
export interface EventPayloads_DocumentMoved {
  documentId: EventPayloads_Id;
  beforeParentId: string | null;
  afterParentId: string | null;
  revision: EventPayloads_Revision;
}
export interface EventPayloads_DraftChanged {
  documentId: EventPayloads_Id;
  draftId: EventPayloads_Id;
  revision: EventPayloads_Revision;
  operationIds: EventPayloads_Id[];
}
export interface EventPayloads_LeaseChanged {
  documentId: EventPayloads_Id;
  holderUserId: string | null;
  expiresAt: string | null;
  revision: EventPayloads_Revision;
}
export interface EventPayloads_VersionPublished {
  documentId: EventPayloads_Id;
  versionId: EventPayloads_Id;
  number: number;
  sourceDraftRevision: EventPayloads_Revision;
}
export interface EventPayloads_JobChanged {
  jobId: EventPayloads_Id;
  status: "QUEUED" | "RUNNING" | "CANCEL_REQUESTED" | "SUCCEEDED" | "FAILED" | "CANCELLED" | "TIMED_OUT";
  jobSequence: number;
  phase?: string | null;
  queuePosition?: number | null;
}
export interface EventPayloads_ProposalApplied {
  proposalId: EventPayloads_Id;
  documentId: EventPayloads_Id;
  appliedOperationIds: EventPayloads_Id[];
  resultRevision: EventPayloads_Revision;
}
export interface EventPayloads_PurgeChanged {
  targetKind: "DOCUMENT" | "WORKSPACE" | "FILE";
  targetId: EventPayloads_Id;
  step: string;
  status: "STARTED" | "COMPLETED" | "FAILED";
}
export interface OpenApi__SessionView {
  user: OpenApi__UserSummary;
  workspaces: OpenApi__Workspace[];
}
export interface OpenApi__UserSummary {
  id: OpenApi__Id;
  email: string;
  displayName: string;
  locale: "ko" | "en";
  timezone: string;
}
export interface OpenApi__Workspace {
  id: OpenApi__Id;
  name: string;
  slug: string;
  status: "ACTIVE" | "DELETION_SCHEDULED" | "PURGING" | "DELETED";
  revision: number;
}
export interface OpenApi__UserPreferences {
  locale: "ko" | "en";
  timezone: string;
  theme: "LIGHT" | "DARK" | "SYSTEM";
  revision: number;
}
export interface OpenApi__Membership {
  userId: OpenApi__Id;
  role: "MEMBER" | "ADMIN" | "OWNER";
  status: "ACTIVE" | "SUSPENDED" | "REMOVED";
  revision: number;
}
export interface OpenApi__Invitation {
  id: OpenApi__Id;
  email: string;
  role: "MEMBER" | "ADMIN";
  status: "PENDING" | "ACCEPTED" | "REVOKED" | "EXPIRED";
  expiresAt: string;
  revision: number;
}
export interface OpenApi__InvitationPage {
  items: OpenApi__Invitation[];
  nextCursor?: string | null;
}
export interface OpenApi__Group {
  id: OpenApi__Id;
  name: string;
  memberIds: OpenApi__Id[];
  revision: number;
}
export interface OpenApi__PermissionGrantInput {
  subjectKind: "USER" | "GROUP";
  subjectId: OpenApi__Id;
  access: OpenApi__Access;
  manage: boolean;
}
export interface OpenApi__PermissionGrant {
  id: OpenApi__Id;
  subjectKind: "USER" | "GROUP";
  subjectId: OpenApi__Id;
  access: OpenApi__Access;
  manage: boolean;
  revision: number;
}
export interface OpenApi__PermissionView {
  effective: OpenApi__EffectivePermission;
  explicitGrants: OpenApi__PermissionGrant[];
  revision: number;
}
export interface OpenApi__EffectivePermission {
  access: OpenApi__Access;
  manage: boolean;
  sourceDocumentId: OpenApi__NullableId;
  evidenceGrantIds: OpenApi__Id[];
}
export interface OpenApi__PermissionExplanation {
  effective: OpenApi__EffectivePermission;
  steps: {
    documentId: OpenApi__Id;
    decision: "NO_GRANT" | "USER_GRANT" | "GROUP_DENY" | "GROUP_MAX" | "INHERITED";
  }[];
  fingerprint: string;
}
export interface OpenApi__PublishPolicy {
  documentId: OpenApi__Id;
  mode: "DIRECT" | "REVIEW_REQUIRED";
  requiredApprovals: number;
  reviewerRule: OpenApi__ReviewerRule;
  inheritedFromDocumentId: OpenApi__NullableId;
  revision: number;
}
export interface OpenApi__Document {
  id: OpenApi__Id;
  title: string;
  parentId?: OpenApi__NullableId;
  status: "ACTIVE" | "TRASHED" | "PURGING";
  currentVersionId?: OpenApi__NullableId;
  revision: number;
  [k: string]: unknown;
}
export interface OpenApi__DocumentPage {
  items: OpenApi__Document[];
  nextCursor?: string | null;
}
export interface OpenApi__DocumentTree {
  nodes: OpenApi__DocumentTreeNode[];
  watermark: number;
}
export interface OpenApi__DocumentTreeNode {
  document: OpenApi__Document;
  children: OpenApi__DocumentTreeNode[];
}
export interface OpenApi__JobReference {
  jobId: OpenApi__Id;
  status: "QUEUED";
}
export interface OpenApi__Draft {
  id: OpenApi__Id;
  documentId: OpenApi__Id;
  baseVersionId?: OpenApi__NullableId;
  revision: number;
  schemaVersion: number;
  content: DocumentContent;
}
export interface OpenApi__PublishedVersion {
  id: OpenApi__Id;
  documentId: OpenApi__Id;
  number: number;
  publishedAt: string;
  publisherId: OpenApi__Id;
  schemaVersion: number;
  content: DocumentContent;
  summary?: string;
}
export interface OpenApi__EditLease {
  holderUserId: OpenApi__Id;
  token: string;
  expiresAt: string;
  revision: number;
  [k: string]: unknown;
}
export interface OpenApi__MutationResult {
  revision: number;
  appliedOperationIds: OpenApi__Id[];
  [k: string]: unknown;
}
export interface OpenApi__VersionPage {
  items: OpenApi__PublishedVersion[];
  nextCursor?: string | null;
}
export interface OpenApi__DocumentDiff {
  fromVersionId: OpenApi__Id;
  toVersionId: OpenApi__Id;
  operations: DocumentOperation[];
}
export interface OpenApi__MoveDocumentInput {
  newParentId: OpenApi__NullableId;
  afterDocumentId?: OpenApi__NullableId;
  [k: string]: unknown;
}
export interface OpenApi__ImpactPreview {
  previewToken: string;
  permissionChanges: number;
  policyChanges: number;
  expiresAt: string;
  [k: string]: unknown;
}
export interface OpenApi__TopicInput {
  kind: "TEXT" | "DOCUMENT" | "REGION" | "EXTERNAL";
  label: string;
  text?: string;
  targetId?: OpenApi__NullableId;
  region?: DocumentOperation_Region;
  url?: string;
}
export interface OpenApi__Topic {
  id: OpenApi__Id;
  kind: "TEXT" | "DOCUMENT" | "REGION" | "EXTERNAL";
  label: string;
  rank: string;
  text?: string;
  targetId?: OpenApi__NullableId;
  region?: DocumentOperation_Region;
  url?: string;
}
export interface OpenApi__RichMessage {
  body: DocumentContent;
  mentionUserIds?: OpenApi__Id[];
  attachmentIds?: OpenApi__Id[];
}
export interface OpenApi__Discussion {
  id: OpenApi__Id;
  documentId: OpenApi__Id;
  title: string;
  status: "OPEN" | "CLOSED";
  topics?: OpenApi__Topic[];
  revision: number;
  [k: string]: unknown;
}
export interface OpenApi__Message {
  id: OpenApi__Id;
  authorId: OpenApi__Id;
  body: DocumentContent;
  revision: number;
  createdAt: string;
  editedAt?: string | null;
  deletedAt?: string | null;
}
export interface OpenApi__DiscussionDetail {
  discussion: OpenApi__Discussion;
  messages: OpenApi__Message[];
  nextCursor?: string | null;
}
export interface OpenApi__Review {
  id: OpenApi__Id;
  documentId: OpenApi__Id;
  draftRevision: number;
  status: "REQUESTED" | "APPROVED" | "CHANGES_REQUESTED" | "CANCELLED" | "INVALIDATED";
  revision: number;
  [k: string]: unknown;
}
export interface OpenApi__InboxItem {
  id: OpenApi__Id;
  kind:
    | "REVIEW_REQUESTED"
    | "REVIEW_DECIDED"
    | "MENTIONED"
    | "DISCUSSION_CHANGED"
    | "PERMISSION_CHANGED"
    | "AI_JOB_COMPLETED";
  target: OpenApi__ResourceTarget;
  createdAt: string;
  readAt: string | null;
  resolvedAt: string | null;
}
export interface OpenApi__ResourceTarget {
  kind: "WORKSPACE" | "DOCUMENT" | "DISCUSSION" | "REVIEW" | "AI_JOB" | "FILE";
  id: OpenApi__Id;
}
export interface OpenApi__InboxPage {
  items: OpenApi__InboxItem[];
  nextCursor?: string | null;
}
export interface OpenApi__AffectedCount {
  count: number;
}
export interface OpenApi__SearchPage {
  items: {
    source: OpenApi__Source;
    score: number;
    [k: string]: unknown;
  }[];
  nextCursor?: string | null;
  indexWatermark: number;
  [k: string]: unknown;
}
export interface OpenApi__Source {
  kind: string;
  stableId: OpenApi__Id;
  documentId?: OpenApi__NullableId;
  regionId?: OpenApi__NullableId;
  authority: string;
  snapshotHash: string;
  [k: string]: unknown;
}
export interface OpenApi__Reference {
  id: OpenApi__Id;
  sourceDocumentId: OpenApi__Id;
  sourceRegion: DocumentOperation_Region;
  target: DocumentOperation_ReferenceTarget;
  snapshot: {
    title: string;
    snapshotHash: string;
  };
}
export interface OpenApi__ReferencePage {
  items: OpenApi__Reference[];
  nextCursor?: string | null;
}
export interface OpenApi__VocabularyTerm {
  term: string;
  kind: "CANONICAL" | "SYNONYM" | "PROHIBITED";
}
export interface OpenApi__VocabularyConcept {
  id: OpenApi__Id;
  canonicalTerm: string;
  definition: string;
  terms: OpenApi__VocabularyTerm[];
  status: "ACTIVE" | "DEPRECATED";
  revision: number;
}
export interface OpenApi__VocabularyPage {
  items: OpenApi__VocabularyConcept[];
  nextCursor?: string | null;
}
export interface OpenApi__CreateAIJob {
  kind: "COMPOSE" | "REWRITE" | "REVIEW" | "DISCUSSION_APPLY" | "CONFLICT_MERGE" | "KNOWLEDGE_QUERY";
  target: AiContracts_Target;
  expectedRevision: number;
  externalWebEnabled: boolean;
  instruction?: string;
  /**
   * @maxItems 200
   */
  includeSourceIds?: OpenApi__Id[];
  /**
   * @maxItems 200
   */
  excludeSourceIds?: OpenApi__Id[];
}
export interface OpenApi__AIJob {
  id: OpenApi__Id;
  kind: "COMPOSE" | "REWRITE" | "REVIEW" | "DISCUSSION_APPLY" | "CONFLICT_MERGE" | "KNOWLEDGE_QUERY";
  status: "QUEUED" | "RUNNING" | "CANCEL_REQUESTED" | "SUCCEEDED" | "FAILED" | "CANCELLED" | "TIMED_OUT";
  sequence: number;
  revision: number;
  result?: AiContracts_Result | null;
  errorCode?: string | null;
}
export interface OpenApi__AIJobPage {
  items: OpenApi__AIJob[];
  nextCursor?: string | null;
}
export interface OpenApi__FileUpload {
  assetId: OpenApi__Id;
  uploadUrl: string;
  expiresAt: string;
  [k: string]: unknown;
}
export interface OpenApi__FileAsset {
  id: OpenApi__Id;
  originalName: string;
  mimeType: string;
  sizeBytes: number;
  checksumSha256: string;
  status: "UPLOADING" | "READY" | "FAILED" | "DELETED";
  revision: number;
}
export interface OpenApi__AuditEvent {
  id: OpenApi__Id;
  sequence: number;
  actor: {
    kind: "USER" | "SYSTEM";
    userId?: OpenApi__NullableId;
  };
  action: string;
  target: OpenApi__ResourceTarget;
  metadata: {
    [k: string]: string | number | boolean | null;
  };
  occurredAt: string;
}
export interface OpenApi__AuditPage {
  items: OpenApi__AuditEvent[];
  nextCursor?: string | null;
}
export interface OpenApi__PublicLink {
  id: OpenApi__Id;
  expiresAt: string | null;
  revokedAt: string | null;
  createdAt: string;
  revision: number;
}
export interface OpenApi__WritingRuleOverride {
  ruleId: string;
  enabled: boolean;
  severity: "BLOCKING" | "WARNING" | "ADVISORY";
  /**
   * @maxItems 1000
   */
  values: string[];
}
export interface OpenApi__WritingConfiguration {
  baselineVersion: string;
  overrides: OpenApi__WritingRuleOverride[];
  revision: number;
}
export interface OpenApi__AIConfiguration {
  provider: "CODEX_CLI" | "OPENAI_RESPONSES";
  model: string;
  userConcurrencyLimit: number;
  workspaceConcurrencyLimit: number;
  monthlyBudgetMicrounits: number;
  revision: number;
}
export interface OpenApi__AIUsage {
  from: string;
  to: string;
  inputTokens: number;
  outputTokens: number;
  estimatedMicrounits: number;
  jobCount: number;
}
export interface OpenApi__AIProviderHealth {
  provider: "CODEX_CLI" | "OPENAI_RESPONSES";
  status: "HEALTHY" | "DEGRADED" | "UNAVAILABLE" | "UNCONFIGURED";
  checkedAt: string;
  code?: string | null;
}
export interface OpenApi__PublicLinkCreated {
  id: OpenApi__Id;
  token: string;
  url: string;
  [k: string]: unknown;
}
export interface OpenApi__PublicDocument {
  title: string;
  versionNumber: number;
  publishedAt: string;
  schemaVersion: number;
  content: DocumentContent;
  allowedAssetIds: OpenApi__Id[];
}
export interface OpenApi__Problem {
  type: string;
  title: string;
  status: number;
  code: string;
  retryable: boolean;
  correlationId: string;
  currentRevision?: number;
  fieldErrors?: {
    field: string;
    code: string;
    [k: string]: unknown;
  }[];
  [k: string]: unknown;
}
export interface Operation__GetSessionRequest {}
export interface Operation__BeginGoogleLoginRequest {
  query?: {
    returnTo?: string;
  };
}
export interface Operation__CompleteGoogleLoginRequest {
  query: {
    code: string;
    state: string;
  };
}
export interface Operation__LogoutRequest {
  header: {
    "Idempotency-Key": string;
    "X-CSRF-Token": string;
  };
}
export interface Operation__GetUserPreferencesRequest {}
export interface Operation__UpdateUserPreferencesRequest {
  header: {
    "If-Match": string;
    "Idempotency-Key": string;
    "X-CSRF-Token": string;
  };
  body: {
    locale: "ko" | "en";
    timezone: string;
    theme: "LIGHT" | "DARK" | "SYSTEM";
  };
}
export interface Operation__ListWorkspacesRequest {}
export interface Operation__CreateWorkspaceRequest {
  header: {
    "Idempotency-Key": string;
    "X-CSRF-Token": string;
  };
  body: {
    name: string;
  };
}
export interface Operation__GetWorkspaceRequest {
  path: {
    workspaceId: OpenApi__Id;
  };
}
export interface Operation__UpdateWorkspaceRequest {
  path: {
    workspaceId: OpenApi__Id;
  };
  header: {
    "If-Match": string;
    "Idempotency-Key": string;
    "X-CSRF-Token": string;
  };
  body: {
    name?: string;
    defaultPublishMode?: "DIRECT" | "REVIEW_REQUIRED";
  };
}
export interface Operation__ScheduleWorkspaceDeletionRequest {
  path: {
    workspaceId: OpenApi__Id;
  };
  header: {
    "If-Match": string;
    "Idempotency-Key": string;
    "X-CSRF-Token": string;
  };
  body: {
    reason: string;
  };
}
export interface Operation__CancelWorkspaceDeletionRequest {
  path: {
    workspaceId: OpenApi__Id;
  };
  header: {
    "If-Match": string;
    "Idempotency-Key": string;
    "X-CSRF-Token": string;
  };
}
export interface Operation__ListMembersRequest {
  path: {
    workspaceId: OpenApi__Id;
  };
}
export interface Operation__UpdateMemberRoleRequest {
  path: {
    workspaceId: OpenApi__Id;
    userId: OpenApi__Id;
  };
  header: {
    "If-Match": string;
    "Idempotency-Key": string;
    "X-CSRF-Token": string;
  };
  body: {
    role: "MEMBER" | "ADMIN" | "OWNER";
  };
}
export interface Operation__RemoveMemberRequest {
  path: {
    workspaceId: OpenApi__Id;
    userId: OpenApi__Id;
  };
  header: {
    "If-Match": string;
    "Idempotency-Key": string;
    "X-CSRF-Token": string;
  };
}
export interface Operation__ListInvitationsRequest {
  path: {
    workspaceId: OpenApi__Id;
  };
  query?: {
    cursor?: string;
  };
}
export interface Operation__InviteMemberRequest {
  path: {
    workspaceId: OpenApi__Id;
  };
  header: {
    "Idempotency-Key": string;
    "X-CSRF-Token": string;
  };
  body: {
    email: string;
    role: "MEMBER" | "ADMIN";
  };
}
export interface Operation__RevokeInvitationRequest {
  path: {
    workspaceId: OpenApi__Id;
    invitationId: OpenApi__Id;
  };
  header: {
    "If-Match": string;
    "Idempotency-Key": string;
    "X-CSRF-Token": string;
  };
}
export interface Operation__AcceptInvitationRequest {
  path: {
    token: string;
  };
  header: {
    "Idempotency-Key": string;
    "X-CSRF-Token": string;
  };
}
export interface Operation__ListGroupsRequest {
  path: {
    workspaceId: OpenApi__Id;
  };
}
export interface Operation__CreateGroupRequest {
  path: {
    workspaceId: OpenApi__Id;
  };
  header: {
    "Idempotency-Key": string;
    "X-CSRF-Token": string;
  };
  body: {
    name: string;
    /**
     * @maxItems 1000
     */
    memberIds?: OpenApi__Id[];
  };
}
export interface Operation__GetGroupRequest {
  path: {
    workspaceId: OpenApi__Id;
    groupId: OpenApi__Id;
  };
}
export interface Operation__UpdateGroupRequest {
  path: {
    workspaceId: OpenApi__Id;
    groupId: OpenApi__Id;
  };
  header: {
    "If-Match": string;
    "Idempotency-Key": string;
    "X-CSRF-Token": string;
  };
  body: {
    name: string;
  };
}
export interface Operation__DeleteGroupRequest {
  path: {
    workspaceId: OpenApi__Id;
    groupId: OpenApi__Id;
  };
  header: {
    "If-Match": string;
    "Idempotency-Key": string;
    "X-CSRF-Token": string;
  };
}
export interface Operation__AddGroupMemberRequest {
  path: {
    workspaceId: OpenApi__Id;
    groupId: OpenApi__Id;
    userId: OpenApi__Id;
  };
  header: {
    "If-Match": string;
    "Idempotency-Key": string;
    "X-CSRF-Token": string;
  };
}
export interface Operation__RemoveGroupMemberRequest {
  path: {
    workspaceId: OpenApi__Id;
    groupId: OpenApi__Id;
    userId: OpenApi__Id;
  };
  header: {
    "If-Match": string;
    "Idempotency-Key": string;
    "X-CSRF-Token": string;
  };
}
export interface Operation__CreateDocumentRequest {
  path: {
    workspaceId: OpenApi__Id;
  };
  header: {
    "Idempotency-Key": string;
  };
  body: {
    title: string;
    parentId?: OpenApi__NullableId;
    afterDocumentId?: OpenApi__NullableId;
    [k: string]: unknown;
  };
}
export interface Operation__GetDocumentTreeRequest {
  path: {
    workspaceId: OpenApi__Id;
  };
}
export interface Operation__ListTrashedDocumentsRequest {
  path: {
    workspaceId: OpenApi__Id;
  };
  query?: {
    cursor?: string;
  };
}
export interface Operation__GetDocumentRequest {
  path: {
    workspaceId: OpenApi__Id;
    documentId: OpenApi__Id;
  };
}
export interface Operation__UpdateDocumentMetadataRequest {
  path: {
    workspaceId: OpenApi__Id;
    documentId: OpenApi__Id;
  };
  header: {
    "If-Match": string;
    "Idempotency-Key": string;
  };
  body: {
    title: string;
  };
}
export interface Operation__PurgeDocumentRequest {
  path: {
    workspaceId: OpenApi__Id;
    documentId: OpenApi__Id;
  };
  header: {
    "If-Match": string;
    "Idempotency-Key": string;
  };
  body: {
    reason: string;
  };
}
export interface Operation__TrashDocumentRequest {
  path: {
    workspaceId: OpenApi__Id;
    documentId: OpenApi__Id;
  };
  header: {
    "If-Match": string;
    "Idempotency-Key": string;
  };
  body: {
    reason: string;
  };
}
export interface Operation__RestoreDocumentRequest {
  path: {
    workspaceId: OpenApi__Id;
    documentId: OpenApi__Id;
  };
  header: {
    "If-Match": string;
    "Idempotency-Key": string;
  };
  body: {
    parentId: OpenApi__NullableId;
    afterDocumentId?: OpenApi__NullableId;
  };
}
export interface Operation__PreviewDocumentMoveRequest {
  path: {
    workspaceId: OpenApi__Id;
    documentId: OpenApi__Id;
  };
  header: {
    "If-Match": string;
  };
  body: OpenApi__MoveDocumentInput;
}
export interface Operation__MoveDocumentRequest {
  path: {
    workspaceId: OpenApi__Id;
    documentId: OpenApi__Id;
  };
  header: {
    "If-Match": string;
    "Idempotency-Key": string;
  };
  body: OpenApi__MoveDocumentInput & {
    previewToken: string;
    [k: string]: unknown;
  };
}
export interface Operation__GetDocumentPermissionsRequest {
  path: {
    workspaceId: OpenApi__Id;
    documentId: OpenApi__Id;
  };
}
export interface Operation__SetDocumentPermissionRequest {
  path: {
    workspaceId: OpenApi__Id;
    documentId: OpenApi__Id;
    grantId: OpenApi__Id;
  };
  header: {
    "If-Match": string;
    "Idempotency-Key": string;
  };
  body: OpenApi__PermissionGrantInput;
}
export interface Operation__DeleteDocumentPermissionRequest {
  path: {
    workspaceId: OpenApi__Id;
    documentId: OpenApi__Id;
    grantId: OpenApi__Id;
  };
  header: {
    "If-Match": string;
    "Idempotency-Key": string;
  };
}
export interface Operation__ExplainEffectivePermissionRequest {
  path: {
    workspaceId: OpenApi__Id;
    documentId: OpenApi__Id;
  };
  query: {
    subjectKind: "USER" | "GROUP";
    subjectId: OpenApi__Id;
  };
}
export interface Operation__GetPublishPolicyRequest {
  path: {
    workspaceId: OpenApi__Id;
    documentId: OpenApi__Id;
  };
}
export interface Operation__SetPublishPolicyRequest {
  path: {
    workspaceId: OpenApi__Id;
    documentId: OpenApi__Id;
  };
  header: {
    "If-Match": string;
    "Idempotency-Key": string;
  };
  body: {
    mode: "DIRECT" | "REVIEW_REQUIRED";
    requiredApprovals: number;
    reviewerRule: OpenApi__ReviewerRule;
  };
}
export interface Operation__GetDraftRequest {
  path: {
    workspaceId: OpenApi__Id;
    documentId: OpenApi__Id;
  };
}
export interface Operation__CreateOrGetDraftRequest {
  path: {
    workspaceId: OpenApi__Id;
    documentId: OpenApi__Id;
  };
  header: {
    "Idempotency-Key": string;
  };
}
export interface Operation__AcquireEditLeaseRequest {
  path: {
    workspaceId: OpenApi__Id;
    documentId: OpenApi__Id;
  };
  header: {
    "If-Match": string;
    "Idempotency-Key": string;
  };
  body: {
    clientInstanceId: OpenApi__Id;
    force?: boolean;
    reason?: string;
    [k: string]: unknown;
  };
}
export interface Operation__ReleaseEditLeaseRequest {
  path: {
    workspaceId: OpenApi__Id;
    documentId: OpenApi__Id;
  };
  header: {
    "If-Match": string;
    "Idempotency-Key": string;
    "X-Edit-Lease": string;
  };
}
export interface Operation__RenewEditLeaseRequest {
  path: {
    workspaceId: OpenApi__Id;
    documentId: OpenApi__Id;
  };
  header: {
    "If-Match": string;
    "Idempotency-Key": string;
    "X-Edit-Lease": string;
  };
}
export interface Operation__ApplyDraftOperationsRequest {
  path: {
    workspaceId: OpenApi__Id;
    documentId: OpenApi__Id;
  };
  header: {
    "If-Match": string;
    "Idempotency-Key": string;
    "X-Edit-Lease": string;
  };
  body: {
    /**
     * @minItems 1
     */
    operations: [DocumentOperation, ...DocumentOperation[]];
    [k: string]: unknown;
  };
}
export interface Operation__RequestReviewRequest {
  path: {
    workspaceId: OpenApi__Id;
    documentId: OpenApi__Id;
  };
  header: {
    "If-Match": string;
    "Idempotency-Key": string;
  };
}
export interface Operation__ListVersionsRequest {
  path: {
    workspaceId: OpenApi__Id;
    documentId: OpenApi__Id;
  };
  query?: {
    cursor?: string;
  };
}
export interface Operation__GetVersionRequest {
  path: {
    workspaceId: OpenApi__Id;
    documentId: OpenApi__Id;
    versionId: OpenApi__Id;
  };
}
export interface Operation__CompareVersionsRequest {
  path: {
    workspaceId: OpenApi__Id;
    documentId: OpenApi__Id;
  };
  query: {
    from: OpenApi__Id;
    to: OpenApi__Id;
  };
}
export interface Operation__SubmitReviewDecisionRequest {
  path: {
    workspaceId: OpenApi__Id;
    reviewId: OpenApi__Id;
  };
  header: {
    "If-Match": string;
    "Idempotency-Key": string;
  };
  body: {
    decision: "APPROVE" | "REQUEST_CHANGES";
    discussionId?: OpenApi__NullableId;
    [k: string]: unknown;
  };
}
export interface Operation__PublishDocumentRequest {
  path: {
    workspaceId: OpenApi__Id;
    documentId: OpenApi__Id;
  };
  header: {
    "If-Match": string;
    "Idempotency-Key": string;
    "X-Edit-Lease": string;
  };
  body: {
    summary: string;
    [k: string]: unknown;
  };
}
export interface Operation__ListDiscussionsRequest {
  path: {
    workspaceId: OpenApi__Id;
    documentId: OpenApi__Id;
  };
}
export interface Operation__CreateDiscussionRequest {
  path: {
    workspaceId: OpenApi__Id;
    documentId: OpenApi__Id;
  };
  header: {
    "Idempotency-Key": string;
  };
  body: {
    title: string;
    message: OpenApi__RichMessage;
    /**
     * @minItems 1
     */
    topics: [OpenApi__TopicInput, ...OpenApi__TopicInput[]];
  };
}
export interface Operation__GetDiscussionRequest {
  path: {
    workspaceId: OpenApi__Id;
    discussionId: OpenApi__Id;
  };
}
export interface Operation__UpdateDiscussionRequest {
  path: {
    workspaceId: OpenApi__Id;
    discussionId: OpenApi__Id;
  };
  header: {
    "If-Match": string;
    "Idempotency-Key": string;
  };
  body: {
    title: string;
  };
}
export interface Operation__CloseDiscussionRequest {
  path: {
    workspaceId: OpenApi__Id;
    discussionId: OpenApi__Id;
  };
  header: {
    "If-Match": string;
    "Idempotency-Key": string;
  };
  body: {
    reason: string;
  };
}
export interface Operation__ReopenDiscussionRequest {
  path: {
    workspaceId: OpenApi__Id;
    discussionId: OpenApi__Id;
  };
  header: {
    "If-Match": string;
    "Idempotency-Key": string;
  };
  body: {
    reason: string;
  };
}
export interface Operation__AddDiscussionTopicRequest {
  path: {
    workspaceId: OpenApi__Id;
    discussionId: OpenApi__Id;
  };
  header: {
    "If-Match": string;
    "Idempotency-Key": string;
  };
  body: OpenApi__TopicInput;
}
export interface Operation__RemoveDiscussionTopicRequest {
  path: {
    workspaceId: OpenApi__Id;
    discussionId: OpenApi__Id;
    topicId: OpenApi__Id;
  };
  header: {
    "If-Match": string;
    "Idempotency-Key": string;
  };
}
export interface Operation__CreateMessageRequest {
  path: {
    workspaceId: OpenApi__Id;
    discussionId: OpenApi__Id;
  };
  header: {
    "Idempotency-Key": string;
  };
  body: OpenApi__RichMessage;
}
export interface Operation__UpdateMessageRequest {
  path: {
    workspaceId: OpenApi__Id;
    discussionId: OpenApi__Id;
    messageId: OpenApi__Id;
  };
  header: {
    "If-Match": string;
    "Idempotency-Key": string;
  };
  body: OpenApi__RichMessage;
}
export interface Operation__DeleteMessageRequest {
  path: {
    workspaceId: OpenApi__Id;
    discussionId: OpenApi__Id;
    messageId: OpenApi__Id;
  };
  header: {
    "If-Match": string;
    "Idempotency-Key": string;
  };
}
export interface Operation__GetReviewRequest {
  path: {
    workspaceId: OpenApi__Id;
    reviewId: OpenApi__Id;
  };
}
export interface Operation__CancelReviewRequest {
  path: {
    workspaceId: OpenApi__Id;
    reviewId: OpenApi__Id;
  };
  header: {
    "If-Match": string;
    "Idempotency-Key": string;
  };
  body: {
    reason: string;
  };
}
export interface Operation__ListInboxRequest {
  path: {
    workspaceId: OpenApi__Id;
  };
  query?: {
    cursor?: string;
    status?: "UNREAD" | "ACTIONABLE" | "RESOLVED" | "ALL";
  };
}
export interface Operation__MarkInboxItemReadRequest {
  path: {
    workspaceId: OpenApi__Id;
    itemId: OpenApi__Id;
  };
  header: {
    "Idempotency-Key": string;
  };
}
export interface Operation__MarkAllInboxReadRequest {
  path: {
    workspaceId: OpenApi__Id;
  };
  header: {
    "Idempotency-Key": string;
  };
  body: {
    before: string;
  };
}
export interface Operation__ResolveInboxItemRequest {
  path: {
    workspaceId: OpenApi__Id;
    itemId: OpenApi__Id;
  };
  header: {
    "Idempotency-Key": string;
  };
}
export interface Operation__SearchKnowledgeRequest {
  path: {
    workspaceId: OpenApi__Id;
  };
  query: {
    q: string;
    cursor?: string;
  };
}
export interface Operation__ListBacklinksRequest {
  path: {
    workspaceId: OpenApi__Id;
    documentId: OpenApi__Id;
  };
  query?: {
    cursor?: string;
  };
}
export interface Operation__CreateReferenceRequest {
  path: {
    workspaceId: OpenApi__Id;
    documentId: OpenApi__Id;
  };
  header: {
    "If-Match": string;
    "Idempotency-Key": string;
    "X-Edit-Lease": string;
  };
  body: {
    sourceRegion: DocumentOperation_Region;
    target: DocumentOperation_ReferenceTarget;
  };
}
export interface Operation__DeleteReferenceRequest {
  path: {
    workspaceId: OpenApi__Id;
    documentId: OpenApi__Id;
    referenceId: OpenApi__Id;
  };
  header: {
    "If-Match": string;
    "Idempotency-Key": string;
    "X-Edit-Lease": string;
  };
}
export interface Operation__ListVocabularyRequest {
  path: {
    workspaceId: OpenApi__Id;
  };
  query?: {
    cursor?: string;
  };
}
export interface Operation__CreateVocabularyConceptRequest {
  path: {
    workspaceId: OpenApi__Id;
  };
  header: {
    "Idempotency-Key": string;
  };
  body: {
    canonicalTerm: string;
    definition: string;
    /**
     * @minItems 1
     * @maxItems 100
     */
    terms: [OpenApi__VocabularyTerm, ...OpenApi__VocabularyTerm[]];
  };
}
export interface Operation__GetVocabularyConceptRequest {
  path: {
    workspaceId: OpenApi__Id;
    conceptId: OpenApi__Id;
  };
}
export interface Operation__UpdateVocabularyConceptRequest {
  path: {
    workspaceId: OpenApi__Id;
    conceptId: OpenApi__Id;
  };
  header: {
    "If-Match": string;
    "Idempotency-Key": string;
  };
  body: {
    canonicalTerm: string;
    definition: string;
    /**
     * @minItems 1
     * @maxItems 100
     */
    terms: [OpenApi__VocabularyTerm, ...OpenApi__VocabularyTerm[]];
  };
}
export interface Operation__DeprecateVocabularyConceptRequest {
  path: {
    workspaceId: OpenApi__Id;
    conceptId: OpenApi__Id;
  };
  header: {
    "If-Match": string;
    "Idempotency-Key": string;
  };
  body: {
    reason: string;
  };
}
export interface Operation__ListAIJobsRequest {
  path: {
    workspaceId: OpenApi__Id;
  };
  query?: {
    cursor?: string;
  };
}
export interface Operation__CreateAIJobRequest {
  path: {
    workspaceId: OpenApi__Id;
  };
  header: {
    "If-Match": string;
    "Idempotency-Key": string;
  };
  body: OpenApi__CreateAIJob;
}
export interface Operation__GetAIJobRequest {
  path: {
    workspaceId: OpenApi__Id;
    jobId: OpenApi__Id;
  };
}
export interface Operation__CancelAIJobRequest {
  path: {
    workspaceId: OpenApi__Id;
    jobId: OpenApi__Id;
  };
  header: {
    "If-Match": string;
    "Idempotency-Key": string;
  };
}
export interface Operation__ApplyProposalRequest {
  path: {
    workspaceId: OpenApi__Id;
    proposalId: OpenApi__Id;
  };
  header: {
    "If-Match": string;
    "Idempotency-Key": string;
    "X-Edit-Lease": string;
  };
  body: {
    operationIds?: OpenApi__Id[];
    [k: string]: unknown;
  };
}
export interface Operation__GetProposalRequest {
  path: {
    workspaceId: OpenApi__Id;
    proposalId: OpenApi__Id;
  };
}
export interface Operation__RejectProposalRequest {
  path: {
    workspaceId: OpenApi__Id;
    proposalId: OpenApi__Id;
  };
  header: {
    "If-Match": string;
    "Idempotency-Key": string;
  };
  body: {
    reason: string;
  };
}
export interface Operation__CreateFileUploadRequest {
  path: {
    workspaceId: OpenApi__Id;
  };
  header: {
    "Idempotency-Key": string;
  };
  body: {
    name: string;
    mimeType: string;
    size: number;
    checksum: string;
    [k: string]: unknown;
  };
}
export interface Operation__CompleteFileUploadRequest {
  path: {
    workspaceId: OpenApi__Id;
    assetId: OpenApi__Id;
  };
  header: {
    "If-Match": string;
    "Idempotency-Key": string;
  };
  body: {
    checksumSha256: string;
    sizeBytes: number;
  };
}
export interface Operation__GetFileRequest {
  path: {
    workspaceId: OpenApi__Id;
    assetId: OpenApi__Id;
  };
}
export interface Operation__DeleteFileRequest {
  path: {
    workspaceId: OpenApi__Id;
    assetId: OpenApi__Id;
  };
  header: {
    "If-Match": string;
    "Idempotency-Key": string;
  };
}
export interface Operation__DownloadFileRequest {
  path: {
    workspaceId: OpenApi__Id;
    assetId: OpenApi__Id;
  };
  header?: {
    Range?: string;
  };
}
export interface Operation__ListAuditEventsRequest {
  path: {
    workspaceId: OpenApi__Id;
  };
  query?: {
    cursor?: string;
  };
}
export interface Operation__GetWritingConfigurationRequest {
  path: {
    workspaceId: OpenApi__Id;
  };
}
export interface Operation__UpdateWritingConfigurationRequest {
  path: {
    workspaceId: OpenApi__Id;
  };
  header: {
    "If-Match": string;
    "Idempotency-Key": string;
  };
  body: {
    baselineVersion: string;
    /**
     * @maxItems 500
     */
    overrides: OpenApi__WritingRuleOverride[];
  };
}
export interface Operation__GetAIConfigurationRequest {
  path: {
    workspaceId: OpenApi__Id;
  };
}
export interface Operation__UpdateAIConfigurationRequest {
  path: {
    workspaceId: OpenApi__Id;
  };
  header: {
    "If-Match": string;
    "Idempotency-Key": string;
  };
  body: {
    provider: "CODEX_CLI" | "OPENAI_RESPONSES";
    model: string;
    userConcurrencyLimit: number;
    workspaceConcurrencyLimit: number;
    monthlyBudgetMicrounits: number;
  };
}
export interface Operation__GetAIUsageRequest {
  path: {
    workspaceId: OpenApi__Id;
  };
  query: {
    from: string;
    to: string;
  };
}
export interface Operation__GetAIProviderHealthRequest {
  path: {
    workspaceId: OpenApi__Id;
  };
}
export interface Operation__ListPublicLinksRequest {
  path: {
    workspaceId: OpenApi__Id;
    documentId: OpenApi__Id;
  };
}
export interface Operation__CreatePublicLinkRequest {
  path: {
    workspaceId: OpenApi__Id;
    documentId: OpenApi__Id;
  };
  header: {
    "If-Match": string;
    "Idempotency-Key": string;
  };
  body: {
    expiresAt?: string | null;
    [k: string]: unknown;
  };
}
export interface Operation__RevokePublicLinkRequest {
  path: {
    workspaceId: OpenApi__Id;
    documentId: OpenApi__Id;
    linkId: OpenApi__Id;
  };
  header: {
    "If-Match": string;
    "Idempotency-Key": string;
  };
}
export interface Operation__OpenWorkspaceStreamRequest {
  query: {
    workspaceId: OpenApi__Id;
    cursor?: string;
  };
}
export interface Operation__GetPublicDocumentRequest {
  path: {
    token: string;
  };
}
export interface Operation__GetPublicDocumentAssetRequest {
  path: {
    token: string;
    assetId: OpenApi__Id;
  };
}
export interface AsyncApi__StreamHeaders {
  /**
   * Opaque SSE resume cursor
   */
  eventId: string;
  retryMilliseconds: number;
}
export interface AsyncApi__OutboxHeaders {
  eventId: string;
  aggregateKind: string;
  aggregateId: string;
  aggregateSequence: number;
  attempt: number;
}
export interface AsyncApi__WorkspaceEvent {
  headers: AsyncApi__StreamHeaders;
  payload: EventEnvelope;
}
export interface AsyncApi__DomainEvent {
  headers: AsyncApi__OutboxHeaders;
  payload: EventEnvelope;
}
