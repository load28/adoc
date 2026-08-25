import type { DocumentOperation } from "@adoc/contracts";
import type { components } from "@adoc/contracts/openapi";

export type SessionView = components["schemas"]["SessionView"];
export type UserPreferences = components["schemas"]["UserPreferences"];
export type WorkspaceView = components["schemas"]["Workspace"];
export type InvitationPreview = components["schemas"]["InvitationPreview"];
export type DocumentView = components["schemas"]["Document"];
export type DocumentTree = components["schemas"]["DocumentTree"];
export type DocumentTreeNode = components["schemas"]["DocumentTreeNode"];
export type ImpactPreview = components["schemas"]["ImpactPreview"];
export type DocumentDetail = components["schemas"]["DocumentDetail"];
export type DraftView = components["schemas"]["Draft"];
export type EditLeaseView = components["schemas"]["EditLease"];
export type MutationResult = {
  revision: number;
  contentFingerprint: string;
  appliedOperationIds: string[];
  inverseOperations: DocumentOperation[];
};
export type FileUploadView = components["schemas"]["FileUpload"];
export type FileAssetView = components["schemas"]["FileAsset"];
export type DiscussionView = components["schemas"]["Discussion"];
export type DiscussionPage = components["schemas"]["DiscussionPage"];
export type DiscussionDetail = components["schemas"]["DiscussionDetail"];
export type RichMessage = components["schemas"]["RichMessage"];
export type ReviewView = components["schemas"]["Review"];
export type ReviewDecision = components["schemas"]["ReviewDecisionInput"];
export type VersionPage = components["schemas"]["VersionPage"];
export type PublishedVersion = components["schemas"]["PublishedVersion"];
export type DocumentDiff = components["schemas"]["DocumentDiff"];
export type InboxPage = components["schemas"]["InboxPage"];
export type InboxItem = components["schemas"]["InboxItem"];
export type SearchPage = components["schemas"]["SearchPage"];
export type ReferencePage = components["schemas"]["ReferencePage"];
export type ReferenceView = components["schemas"]["Reference"];
export type VocabularyPage = components["schemas"]["VocabularyPage"];
export type VocabularyConcept = components["schemas"]["VocabularyConcept"];
export type VocabularyTerm = components["schemas"]["VocabularyTerm"];
export type AIContextRequest = components["schemas"]["AIContextRequest"];
export type AIContextPreview = components["schemas"]["AIContextPreview"];
export type CreateAIJob = components["schemas"]["CreateAIJob"];
export type AIJob = components["schemas"]["AIJob"];
export type AIJobPage = components["schemas"]["AIJobPage"];
export type Proposal = components["schemas"]["Proposal"];
export type Membership = components["schemas"]["Membership"];
export type Invitation = components["schemas"]["Invitation"];
export type InvitationPage = components["schemas"]["InvitationPage"];
export type Group = components["schemas"]["Group"];
export type PermissionView = components["schemas"]["PermissionView"];
export type PublishPolicy = components["schemas"]["PublishPolicy"];
export type PermissionGrant = components["schemas"]["PermissionGrant"];
export type PermissionExplanation = components["schemas"]["PermissionExplanation"];
export type WritingConfiguration = components["schemas"]["WritingConfiguration"];
export type AIConfiguration = components["schemas"]["AIConfiguration"];
export type AIUsage = components["schemas"]["AIUsage"];
export type AIProviderHealth = components["schemas"]["AIProviderHealth"];
export type AuditPage = components["schemas"]["AuditPage"];
export type AuditFilter = {
  action?: string;
  actorUserId?: string;
  targetKind?: string;
  from?: string;
  to?: string;
};
export type DocumentPage = components["schemas"]["DocumentPage"];
export type PublicDocument = components["schemas"]["PublicDocument"];
export type JobReference = components["schemas"]["JobReference"];

export type ApiProblem = {
  code: string;
  message: string;
  correlationId?: string;
  fieldErrors?: Record<string, string>;
  meta?: Record<string, unknown>;
};

export class ApiProblemError extends Error {
  readonly problem: ApiProblem;

  constructor(problem: ApiProblem) {
    super(problem.message);
    this.name = "ApiProblemError";
    this.problem = problem;
  }
}

type FetchLike = (input: RequestInfo | URL, init?: RequestInit) => Promise<Response>;

export class ApiClient {
  readonly #fetch: FetchLike;

  constructor(fetcher: FetchLike = globalThis.fetch.bind(globalThis)) {
    this.#fetch = fetcher;
  }

  session(signal?: AbortSignal): Promise<SessionView> {
    return this.#json<SessionView>("/api/v1/session", { signal });
  }

  preferences(signal?: AbortSignal): Promise<UserPreferences> {
    return this.#json<UserPreferences>("/api/v1/preferences", { signal });
  }

  workspaces(signal?: AbortSignal): Promise<WorkspaceView[]> {
    return this.#json<WorkspaceView[]>("/api/v1/workspaces", { signal });
  }

  createWorkspace(name: string, command: CommandHeaders): Promise<WorkspaceView> {
    return this.#json<WorkspaceView>("/api/v1/workspaces", {
      method: "POST",
      headers: commandHeaders(command),
      body: JSON.stringify({ name }),
    });
  }

  invitationPreview(token: string, signal?: AbortSignal): Promise<InvitationPreview> {
    return this.#json<InvitationPreview>(
      `/api/v1/invitations/${encodeURIComponent(token)}/accept`,
      { signal },
    );
  }

  acceptInvitation(token: string, command: CommandHeaders): Promise<Membership> {
    return this.#json<Membership>(`/api/v1/invitations/${encodeURIComponent(token)}/accept`, {
      method: "POST",
      headers: commandHeaders(command),
    });
  }

  documentTree(workspaceId: string, signal?: AbortSignal): Promise<DocumentTree> {
    return this.#json<DocumentTree>(
      `/api/v1/workspaces/${encodeURIComponent(workspaceId)}/documents/tree`,
      { signal },
    );
  }

  createDocument(
    workspaceId: string,
    title: string,
    parentId: string | null,
    afterDocumentId: string | null,
    command: CommandHeaders,
  ): Promise<DocumentView> {
    return this.#json<DocumentView>(
      `/api/v1/workspaces/${encodeURIComponent(workspaceId)}/documents`,
      {
        method: "POST",
        headers: commandHeaders(command),
        body: JSON.stringify({ title, parentId, afterDocumentId }),
      },
    );
  }

  renameDocument(
    workspaceId: string,
    documentId: string,
    revision: number,
    title: string,
    command: CommandHeaders,
  ): Promise<DocumentView> {
    return this.#json<DocumentView>(resourcePath(workspaceId, "documents", documentId), {
      method: "PUT",
      headers: commandHeaders(command, revision),
      body: JSON.stringify({ title }),
    });
  }

  previewDocumentMove(
    workspaceId: string,
    documentId: string,
    revision: number,
    newParentId: string | null,
    afterDocumentId: string | null,
    csrfToken: string,
  ): Promise<ImpactPreview> {
    return this.#json<ImpactPreview>(
      `${resourcePath(workspaceId, "documents", documentId)}/move-preview`,
      {
        method: "POST",
        headers: previewHeaders(csrfToken, revision),
        body: JSON.stringify({ newParentId, afterDocumentId }),
      },
    );
  }

  moveDocument(
    workspaceId: string,
    documentId: string,
    revision: number,
    newParentId: string | null,
    afterDocumentId: string | null,
    previewToken: string,
    command: CommandHeaders,
  ): Promise<DocumentView> {
    return this.#json<DocumentView>(`${resourcePath(workspaceId, "documents", documentId)}/move`, {
      method: "POST",
      headers: commandHeaders(command, revision),
      body: JSON.stringify({ newParentId, afterDocumentId, previewToken }),
    });
  }

  trashDocument(
    workspaceId: string,
    documentId: string,
    revision: number,
    reason: string,
    command: CommandHeaders,
  ): Promise<DocumentView> {
    return this.#json<DocumentView>(`${resourcePath(workspaceId, "documents", documentId)}/trash`, {
      method: "POST",
      headers: commandHeaders(command, revision),
      body: JSON.stringify({ reason }),
    });
  }

  document(workspaceId: string, documentId: string, signal?: AbortSignal): Promise<DocumentDetail> {
    return this.#json<DocumentDetail>(resourcePath(workspaceId, "documents", documentId), {
      signal,
    });
  }

  draft(workspaceId: string, documentId: string, signal?: AbortSignal): Promise<DraftView> {
    return this.#json<DraftView>(`${resourcePath(workspaceId, "documents", documentId)}/draft`, {
      signal,
    });
  }

  createDraft(
    workspaceId: string,
    documentId: string,
    command: CommandHeaders,
  ): Promise<DraftView> {
    return this.#json<DraftView>(`${resourcePath(workspaceId, "documents", documentId)}/draft`, {
      method: "POST",
      headers: commandHeaders(command),
    });
  }

  acquireLease(
    workspaceId: string,
    documentId: string,
    documentRevision: number,
    clientInstanceId: string,
    command: CommandHeaders,
  ): Promise<EditLeaseView> {
    return this.#json<EditLeaseView>(
      `${resourcePath(workspaceId, "documents", documentId)}/lease`,
      {
        method: "POST",
        headers: commandHeaders(command, documentRevision),
        body: JSON.stringify({ clientInstanceId }),
      },
    );
  }

  renewLease(
    workspaceId: string,
    documentId: string,
    leaseRevision: number,
    leaseToken: string,
    clientInstanceId: string,
    command: CommandHeaders,
  ): Promise<EditLeaseView> {
    return this.#json<EditLeaseView>(
      `${resourcePath(workspaceId, "documents", documentId)}/lease/renew`,
      {
        method: "POST",
        headers: leaseHeaders(command, leaseRevision, leaseToken, clientInstanceId),
      },
    );
  }

  releaseLease(
    workspaceId: string,
    documentId: string,
    leaseRevision: number,
    leaseToken: string,
    clientInstanceId: string,
    command: CommandHeaders,
  ): Promise<void> {
    return this.#empty(`${resourcePath(workspaceId, "documents", documentId)}/lease`, {
      method: "DELETE",
      headers: leaseHeaders(command, leaseRevision, leaseToken, clientInstanceId),
      keepalive: true,
    });
  }

  applyDraftOperations(
    workspaceId: string,
    documentId: string,
    draftRevision: number,
    leaseToken: string,
    clientInstanceId: string,
    operations: DocumentOperation[],
    command: CommandHeaders,
  ): Promise<MutationResult> {
    return this.#json<MutationResult>(
      `${resourcePath(workspaceId, "documents", documentId)}/draft/operations`,
      {
        method: "POST",
        headers: leaseHeaders(command, draftRevision, leaseToken, clientInstanceId),
        body: JSON.stringify({ operations }),
      },
    );
  }

  createFileUpload(
    workspaceId: string,
    input: { name: string; mimeType: string; size: number; checksum: string },
    command: CommandHeaders,
  ): Promise<FileUploadView> {
    return this.#json<FileUploadView>(
      `/api/v1/workspaces/${encodeURIComponent(workspaceId)}/files/uploads`,
      {
        method: "POST",
        headers: commandHeaders(command),
        body: JSON.stringify(input),
      },
    );
  }

  async uploadFileBytes(
    uploadUrl: string,
    uploadToken: string,
    csrfToken: string,
    file: Blob,
  ): Promise<void> {
    const target = new URL(uploadUrl, globalThis.location?.origin ?? "http://localhost");
    if (globalThis.location && target.origin !== globalThis.location.origin)
      throw new Error("cross-origin upload target");
    const response = await this.#fetch(target.pathname + target.search, {
      method: "PUT",
      credentials: "same-origin",
      redirect: "manual",
      headers: {
        "content-type": "application/octet-stream",
        "x-upload-token": uploadToken,
        "x-csrf-token": csrfToken,
      },
      body: file,
    });
    if (response.ok) return;
    const payload: unknown = await response.json().catch(() => undefined);
    throw new ApiProblemError(parseProblem(payload));
  }

  completeFileUpload(
    workspaceId: string,
    assetId: string,
    checksumSha256: string,
    sizeBytes: number,
    command: CommandHeaders,
  ): Promise<FileAssetView> {
    return this.#json<FileAssetView>(
      `/api/v1/workspaces/${encodeURIComponent(workspaceId)}/files/${encodeURIComponent(assetId)}/complete`,
      {
        method: "POST",
        headers: commandHeaders(command, 0),
        body: JSON.stringify({ checksumSha256, sizeBytes }),
      },
    );
  }

  discussions(
    workspaceId: string,
    documentId: string,
    cursor?: string,
    signal?: AbortSignal,
  ): Promise<DiscussionPage> {
    return this.#json<DiscussionPage>(
      withQuery(`${resourcePath(workspaceId, "documents", documentId)}/discussions`, { cursor }),
      { signal },
    );
  }

  createDiscussion(
    workspaceId: string,
    documentId: string,
    input: { title: string; message: RichMessage; topics: components["schemas"]["TopicInput"][] },
    command: CommandHeaders,
  ): Promise<DiscussionView> {
    return this.#json(`${resourcePath(workspaceId, "documents", documentId)}/discussions`, {
      method: "POST",
      headers: commandHeaders(command),
      body: JSON.stringify(input),
    });
  }

  discussion(
    workspaceId: string,
    discussionId: string,
    cursor?: string,
    signal?: AbortSignal,
  ): Promise<DiscussionDetail> {
    return this.#json<DiscussionDetail>(
      withQuery(resourcePath(workspaceId, "discussions", discussionId), { cursor }),
      { signal },
    );
  }

  createMessage(
    workspaceId: string,
    discussionId: string,
    message: RichMessage,
    command: CommandHeaders,
  ): Promise<components["schemas"]["Message"]> {
    return this.#json(`${resourcePath(workspaceId, "discussions", discussionId)}/messages`, {
      method: "POST",
      headers: commandHeaders(command),
      body: JSON.stringify(message),
    });
  }

  changeDiscussionStatus(
    workspaceId: string,
    discussionId: string,
    revision: number,
    action: "close" | "reopen",
    reason: string,
    command: CommandHeaders,
  ): Promise<DiscussionView> {
    return this.#json(`${resourcePath(workspaceId, "discussions", discussionId)}/${action}`, {
      method: "POST",
      headers: commandHeaders(command, revision),
      body: JSON.stringify({ reason }),
    });
  }

  updateDiscussion(
    workspaceId: string,
    discussion: Pick<DiscussionView, "id" | "revision">,
    title: string,
    command: CommandHeaders,
  ): Promise<DiscussionView> {
    return this.#json(resourcePath(workspaceId, "discussions", discussion.id), {
      method: "PUT",
      headers: commandHeaders(command, discussion.revision),
      body: JSON.stringify({ title }),
    });
  }

  addDiscussionTopic(
    workspaceId: string,
    discussion: Pick<DiscussionView, "id" | "revision">,
    topic: components["schemas"]["TopicInput"],
    command: CommandHeaders,
  ): Promise<DiscussionView> {
    return this.#json(`${resourcePath(workspaceId, "discussions", discussion.id)}/topics`, {
      method: "POST",
      headers: commandHeaders(command, discussion.revision),
      body: JSON.stringify(topic),
    });
  }

  removeDiscussionTopic(
    workspaceId: string,
    discussion: Pick<DiscussionView, "id" | "revision">,
    topicId: string,
    command: CommandHeaders,
  ): Promise<void> {
    return this.#empty(
      `${resourcePath(workspaceId, "discussions", discussion.id)}/topics/${encodeURIComponent(topicId)}`,
      { method: "DELETE", headers: commandHeaders(command, discussion.revision) },
    );
  }

  updateMessage(
    workspaceId: string,
    discussionId: string,
    messageId: string,
    revision: number,
    message: RichMessage,
    command: CommandHeaders,
  ): Promise<components["schemas"]["Message"]> {
    return this.#json(
      `${resourcePath(workspaceId, "discussions", discussionId)}/messages/${encodeURIComponent(messageId)}`,
      {
        method: "PUT",
        headers: commandHeaders(command, revision),
        body: JSON.stringify(message),
      },
    );
  }

  deleteMessage(
    workspaceId: string,
    discussionId: string,
    messageId: string,
    revision: number,
    command: CommandHeaders,
  ): Promise<void> {
    return this.#empty(
      `${resourcePath(workspaceId, "discussions", discussionId)}/messages/${encodeURIComponent(messageId)}`,
      { method: "DELETE", headers: commandHeaders(command, revision) },
    );
  }

  requestReview(
    workspaceId: string,
    documentId: string,
    draftRevision: number,
    command: CommandHeaders,
  ): Promise<ReviewView> {
    return this.#json(`${resourcePath(workspaceId, "documents", documentId)}/reviews`, {
      method: "POST",
      headers: commandHeaders(command, draftRevision),
    });
  }

  review(workspaceId: string, reviewId: string, signal?: AbortSignal): Promise<ReviewView> {
    return this.#json<ReviewView>(resourcePath(workspaceId, "reviews", reviewId), { signal });
  }

  submitReviewDecision(
    workspaceId: string,
    reviewId: string,
    revision: number,
    decision: ReviewDecision,
    command: CommandHeaders,
  ): Promise<ReviewView> {
    return this.#json(`${resourcePath(workspaceId, "reviews", reviewId)}/decisions`, {
      method: "POST",
      headers: commandHeaders(command, revision),
      body: JSON.stringify(decision),
    });
  }

  cancelReview(
    workspaceId: string,
    review: Pick<ReviewView, "id" | "revision">,
    reason: string,
    command: CommandHeaders,
  ): Promise<ReviewView> {
    return this.#json(`${resourcePath(workspaceId, "reviews", review.id)}/cancel`, {
      method: "POST",
      headers: commandHeaders(command, review.revision),
      body: JSON.stringify({ reason }),
    });
  }

  versions(
    workspaceId: string,
    documentId: string,
    cursor?: string,
    signal?: AbortSignal,
  ): Promise<VersionPage> {
    return this.#json<VersionPage>(
      withQuery(`${resourcePath(workspaceId, "documents", documentId)}/versions`, { cursor }),
      { signal },
    );
  }

  version(
    workspaceId: string,
    documentId: string,
    versionId: string,
    signal?: AbortSignal,
  ): Promise<PublishedVersion> {
    return this.#json<PublishedVersion>(
      `${resourcePath(workspaceId, "documents", documentId)}/versions/${encodeURIComponent(versionId)}`,
      { signal },
    );
  }

  publishDocument(
    workspaceId: string,
    documentId: string,
    draftRevision: number,
    input: {
      summary: string;
      clientInstanceId?: string | null;
      leaseToken?: string | null;
    },
    command: CommandHeaders,
  ): Promise<PublishedVersion> {
    return this.#json<PublishedVersion>(
      `${resourcePath(workspaceId, "documents", documentId)}/publish`,
      {
        method: "POST",
        headers: commandHeaders(command, draftRevision),
        body: JSON.stringify(input),
      },
    );
  }

  restoreVersion(
    workspaceId: string,
    documentId: string,
    versionId: string,
    documentRevision: number,
    command: CommandHeaders,
  ): Promise<DraftView> {
    return this.#json<DraftView>(
      `${resourcePath(workspaceId, "documents", documentId)}/versions/${encodeURIComponent(versionId)}/restore`,
      { method: "POST", headers: commandHeaders(command, documentRevision) },
    );
  }

  versionDiff(
    workspaceId: string,
    documentId: string,
    from: string,
    to: string,
    signal?: AbortSignal,
  ): Promise<DocumentDiff> {
    return this.#json<DocumentDiff>(
      withQuery(`${resourcePath(workspaceId, "documents", documentId)}/version-diff`, { from, to }),
      { signal },
    );
  }

  backlinks(
    workspaceId: string,
    documentId: string,
    cursor?: string,
    signal?: AbortSignal,
  ): Promise<ReferencePage> {
    return this.#json<ReferencePage>(
      withQuery(`${resourcePath(workspaceId, "documents", documentId)}/backlinks`, { cursor }),
      { signal },
    );
  }

  createReference(
    workspaceId: string,
    documentId: string,
    draftRevision: number,
    input: {
      sourceRegion: components["schemas"]["Reference"]["sourceRegion"];
      target: components["schemas"]["Reference"]["target"];
    },
    leaseToken: string,
    clientInstanceId: string,
    command: CommandHeaders,
  ): Promise<ReferenceView> {
    return this.#json(`${resourcePath(workspaceId, "documents", documentId)}/references`, {
      method: "POST",
      headers: leaseHeaders(command, draftRevision, leaseToken, clientInstanceId),
      body: JSON.stringify(input),
    });
  }

  deleteReference(
    workspaceId: string,
    documentId: string,
    referenceId: string,
    draftRevision: number,
    leaseToken: string,
    clientInstanceId: string,
    command: CommandHeaders,
  ): Promise<void> {
    return this.#empty(
      `${resourcePath(workspaceId, "documents", documentId)}/references/${encodeURIComponent(referenceId)}`,
      {
        method: "DELETE",
        headers: leaseHeaders(command, draftRevision, leaseToken, clientInstanceId),
      },
    );
  }

  inbox(
    workspaceId: string,
    status: "UNREAD" | "ACTIONABLE" | "RESOLVED" | "ALL",
    cursor?: string,
    signal?: AbortSignal,
  ): Promise<InboxPage> {
    return this.#json<InboxPage>(
      withQuery(`/api/v1/workspaces/${encodeURIComponent(workspaceId)}/inbox`, { status, cursor }),
      { signal },
    );
  }

  updateInboxItem(
    workspaceId: string,
    itemId: string,
    action: "read" | "resolve",
    command: CommandHeaders,
  ): Promise<InboxItem> {
    return this.#json(
      `/api/v1/workspaces/${encodeURIComponent(workspaceId)}/inbox/${encodeURIComponent(itemId)}/${action}`,
      { method: "POST", headers: commandHeaders(command) },
    );
  }

  markAllInboxRead(
    workspaceId: string,
    before: string,
    command: CommandHeaders,
  ): Promise<components["schemas"]["AffectedCount"]> {
    return this.#json(`/api/v1/workspaces/${encodeURIComponent(workspaceId)}/inbox/read-all`, {
      method: "POST",
      headers: commandHeaders(command),
      body: JSON.stringify({ before }),
    });
  }

  search(
    workspaceId: string,
    query: string,
    cursor?: string,
    signal?: AbortSignal,
  ): Promise<SearchPage> {
    return this.#json<SearchPage>(
      withQuery(`/api/v1/workspaces/${encodeURIComponent(workspaceId)}/search`, {
        q: query,
        includeDrafts: "true",
        limit: "20",
        cursor,
      }),
      { signal },
    );
  }

  vocabulary(workspaceId: string, cursor?: string, signal?: AbortSignal): Promise<VocabularyPage> {
    return this.#json<VocabularyPage>(
      withQuery(`/api/v1/workspaces/${encodeURIComponent(workspaceId)}/vocabulary`, { cursor }),
      { signal },
    );
  }

  writeVocabulary(
    workspaceId: string,
    input: { canonicalTerm: string; definition: string; terms: VocabularyTerm[] },
    command: CommandHeaders,
    current?: Pick<VocabularyConcept, "id" | "revision">,
  ): Promise<VocabularyConcept> {
    const base = `/api/v1/workspaces/${encodeURIComponent(workspaceId)}/vocabulary`;
    return this.#json(current ? `${base}/${encodeURIComponent(current.id)}` : base, {
      method: current ? "PUT" : "POST",
      headers: commandHeaders(command, current?.revision),
      body: JSON.stringify(input),
    });
  }

  deprecateVocabulary(
    workspaceId: string,
    concept: Pick<VocabularyConcept, "id" | "revision">,
    replacementConceptId: string | null,
    reason: string,
    command: CommandHeaders,
  ): Promise<VocabularyConcept> {
    return this.#json(
      `/api/v1/workspaces/${encodeURIComponent(workspaceId)}/vocabulary/${encodeURIComponent(concept.id)}/deprecate`,
      {
        method: "POST",
        headers: commandHeaders(command, concept.revision),
        body: JSON.stringify({ replacementConceptId, reason }),
      },
    );
  }

  previewAIContext(
    workspaceId: string,
    input: AIContextRequest,
    signal?: AbortSignal,
  ): Promise<AIContextPreview> {
    return this.#json(`/api/v1/workspaces/${encodeURIComponent(workspaceId)}/ai/context-preview`, {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify(input),
      signal,
    });
  }

  aiJobs(workspaceId: string, cursor?: string, signal?: AbortSignal): Promise<AIJobPage> {
    return this.#json(
      withQuery(`/api/v1/workspaces/${encodeURIComponent(workspaceId)}/ai/jobs`, { cursor }),
      { signal },
    );
  }

  createAIJob(
    workspaceId: string,
    input: CreateAIJob,
    revision: number,
    command: CommandHeaders,
  ): Promise<AIJob> {
    return this.#json(`/api/v1/workspaces/${encodeURIComponent(workspaceId)}/ai/jobs`, {
      method: "POST",
      headers: commandHeaders(command, revision),
      body: JSON.stringify(input),
    });
  }

  aiJob(workspaceId: string, jobId: string, signal?: AbortSignal): Promise<AIJob> {
    return this.#json(resourcePath(workspaceId, "ai/jobs", jobId), { signal });
  }

  cancelAIJob(
    workspaceId: string,
    jobId: string,
    revision: number,
    command: CommandHeaders,
  ): Promise<void> {
    return this.#empty(resourcePath(workspaceId, "ai/jobs", jobId), {
      method: "DELETE",
      headers: commandHeaders(command, revision),
    });
  }

  proposal(workspaceId: string, proposalId: string, signal?: AbortSignal): Promise<Proposal> {
    return this.#json(resourcePath(workspaceId, "proposals", proposalId), { signal });
  }

  applyProposal(
    workspaceId: string,
    proposalId: string,
    draftRevision: number,
    leaseToken: string,
    clientInstanceId: string,
    operationIds: string[],
    command: CommandHeaders,
  ): Promise<MutationResult> {
    return this.#json(`${resourcePath(workspaceId, "proposals", proposalId)}/apply`, {
      method: "POST",
      headers: leaseHeaders(command, draftRevision, leaseToken, clientInstanceId),
      body: JSON.stringify({ operationIds }),
    });
  }

  rejectProposal(
    workspaceId: string,
    proposalId: string,
    proposalRevision: number,
    reason: string,
    command: CommandHeaders,
  ): Promise<Proposal> {
    return this.#json(`${resourcePath(workspaceId, "proposals", proposalId)}/reject`, {
      method: "POST",
      headers: commandHeaders(command, proposalRevision),
      body: JSON.stringify({ reason }),
    });
  }

  members(workspaceId: string, signal?: AbortSignal): Promise<Membership[]> {
    return this.#json(`/api/v1/workspaces/${encodeURIComponent(workspaceId)}/members`, { signal });
  }

  invitations(workspaceId: string, cursor?: string, signal?: AbortSignal): Promise<InvitationPage> {
    return this.#json(
      withQuery(`/api/v1/workspaces/${encodeURIComponent(workspaceId)}/invitations`, { cursor }),
      { signal },
    );
  }

  inviteMember(
    workspaceId: string,
    email: string,
    role: "MEMBER" | "ADMIN",
    command: CommandHeaders,
  ): Promise<Invitation> {
    return this.#json(`/api/v1/workspaces/${encodeURIComponent(workspaceId)}/invitations`, {
      method: "POST",
      headers: commandHeaders(command),
      body: JSON.stringify({ email, role }),
    });
  }

  revokeInvitation(
    workspaceId: string,
    invitation: Pick<Invitation, "id" | "revision">,
    command: CommandHeaders,
  ): Promise<Invitation> {
    return this.#json(
      `/api/v1/workspaces/${encodeURIComponent(workspaceId)}/invitations/${encodeURIComponent(invitation.id)}`,
      { method: "DELETE", headers: commandHeaders(command, invitation.revision) },
    );
  }

  updateMemberRole(
    workspaceId: string,
    member: Pick<Membership, "userId" | "revision">,
    role: Membership["role"],
    command: CommandHeaders,
  ): Promise<Membership> {
    return this.#json(`${resourcePath(workspaceId, "members", member.userId)}/role`, {
      method: "PUT",
      headers: commandHeaders(command, member.revision),
      body: JSON.stringify({ role }),
    });
  }

  removeMember(
    workspaceId: string,
    member: Pick<Membership, "userId" | "revision">,
    command: CommandHeaders,
  ): Promise<Membership> {
    return this.#json(resourcePath(workspaceId, "members", member.userId), {
      method: "DELETE",
      headers: commandHeaders(command, member.revision),
    });
  }

  groups(workspaceId: string, signal?: AbortSignal): Promise<Group[]> {
    return this.#json(`/api/v1/workspaces/${encodeURIComponent(workspaceId)}/groups`, { signal });
  }

  createGroup(workspaceId: string, name: string, command: CommandHeaders): Promise<Group> {
    return this.#json(`/api/v1/workspaces/${encodeURIComponent(workspaceId)}/groups`, {
      method: "POST",
      headers: commandHeaders(command),
      body: JSON.stringify({ name, memberIds: [] }),
    });
  }

  updateGroup(
    workspaceId: string,
    group: Pick<Group, "id" | "revision">,
    name: string,
    command: CommandHeaders,
  ): Promise<Group> {
    return this.#json(resourcePath(workspaceId, "groups", group.id), {
      method: "PUT",
      headers: commandHeaders(command, group.revision),
      body: JSON.stringify({ name }),
    });
  }

  deleteGroup(
    workspaceId: string,
    group: Pick<Group, "id" | "revision">,
    command: CommandHeaders,
  ): Promise<void> {
    return this.#empty(resourcePath(workspaceId, "groups", group.id), {
      method: "DELETE",
      headers: commandHeaders(command, group.revision),
    });
  }

  changeGroupMember(
    workspaceId: string,
    group: Pick<Group, "id" | "revision">,
    userId: string,
    action: "add" | "remove",
    command: CommandHeaders,
  ): Promise<Group> {
    return this.#json(
      `${resourcePath(workspaceId, "groups", group.id)}/members/${encodeURIComponent(userId)}`,
      {
        method: action === "add" ? "PUT" : "DELETE",
        headers: commandHeaders(command, group.revision),
      },
    );
  }

  documentPermissions(
    workspaceId: string,
    documentId: string,
    signal?: AbortSignal,
  ): Promise<PermissionView> {
    return this.#json(`${resourcePath(workspaceId, "documents", documentId)}/permissions`, {
      signal,
    });
  }

  publishPolicy(
    workspaceId: string,
    documentId: string,
    signal?: AbortSignal,
  ): Promise<PublishPolicy> {
    return this.#json(`${resourcePath(workspaceId, "documents", documentId)}/publish-policy`, {
      signal,
    });
  }

  setDocumentPermission(
    workspaceId: string,
    documentId: string,
    grantId: string,
    revision: number,
    input: components["schemas"]["PermissionGrantInput"],
    command: CommandHeaders,
  ): Promise<PermissionGrant> {
    return this.#json(
      `${resourcePath(workspaceId, "documents", documentId)}/permissions/${encodeURIComponent(grantId)}`,
      {
        method: "PUT",
        headers: commandHeaders(command, revision),
        body: JSON.stringify(input),
      },
    );
  }

  deleteDocumentPermission(
    workspaceId: string,
    documentId: string,
    grantId: string,
    revision: number,
    command: CommandHeaders,
  ): Promise<void> {
    return this.#empty(
      `${resourcePath(workspaceId, "documents", documentId)}/permissions/${encodeURIComponent(grantId)}`,
      { method: "DELETE", headers: commandHeaders(command, revision) },
    );
  }

  explainDocumentPermission(
    workspaceId: string,
    documentId: string,
    subjectKind: "USER" | "GROUP",
    subjectId: string,
    signal?: AbortSignal,
  ): Promise<PermissionExplanation> {
    return this.#json(
      withQuery(`${resourcePath(workspaceId, "documents", documentId)}/permission-explanation`, {
        subjectKind,
        subjectId,
      }),
      { signal },
    );
  }

  setPublishPolicy(
    workspaceId: string,
    documentId: string,
    current: Pick<PublishPolicy, "revision">,
    input: Pick<PublishPolicy, "mode" | "requiredApprovals" | "reviewerRule">,
    command: CommandHeaders,
  ): Promise<PublishPolicy> {
    return this.#json(`${resourcePath(workspaceId, "documents", documentId)}/publish-policy`, {
      method: "PUT",
      headers: commandHeaders(command, current.revision),
      body: JSON.stringify(input),
    });
  }

  writingConfiguration(workspaceId: string, signal?: AbortSignal): Promise<WritingConfiguration> {
    return this.#json(
      `/api/v1/workspaces/${encodeURIComponent(workspaceId)}/writing-configuration`,
      { signal },
    );
  }

  updateWritingConfiguration(
    workspaceId: string,
    revision: number,
    command: CommandHeaders,
  ): Promise<WritingConfiguration> {
    return this.#json(
      `/api/v1/workspaces/${encodeURIComponent(workspaceId)}/writing-configuration`,
      {
        method: "PUT",
        headers: commandHeaders(command, revision),
        body: JSON.stringify({ baselineVersion: "writing-rules-v1", overrides: [] }),
      },
    );
  }

  aiConfiguration(workspaceId: string, signal?: AbortSignal): Promise<AIConfiguration> {
    return this.#json(`/api/v1/workspaces/${encodeURIComponent(workspaceId)}/ai/configuration`, {
      signal,
    });
  }

  updateAIConfiguration(
    workspaceId: string,
    input: Omit<AIConfiguration, "revision">,
    revision: number,
    command: CommandHeaders,
  ): Promise<AIConfiguration> {
    return this.#json(`/api/v1/workspaces/${encodeURIComponent(workspaceId)}/ai/configuration`, {
      method: "PUT",
      headers: commandHeaders(command, revision),
      body: JSON.stringify(input),
    });
  }

  aiUsage(workspaceId: string, from: string, to: string, signal?: AbortSignal): Promise<AIUsage> {
    return this.#json(
      withQuery(`/api/v1/workspaces/${encodeURIComponent(workspaceId)}/ai/usage`, { from, to }),
      { signal },
    );
  }

  aiProviderHealth(workspaceId: string, signal?: AbortSignal): Promise<AIProviderHealth> {
    return this.#json(`/api/v1/workspaces/${encodeURIComponent(workspaceId)}/ai/provider-health`, {
      signal,
    });
  }

  auditEvents(
    workspaceId: string,
    cursor?: string,
    signal?: AbortSignal,
    filter: AuditFilter = {},
  ): Promise<AuditPage> {
    return this.#json(
      withQuery(`/api/v1/workspaces/${encodeURIComponent(workspaceId)}/audit-events`, {
        cursor,
        action: filter.action,
        actorUserId: filter.actorUserId,
        targetKind: filter.targetKind,
        from: filter.from,
        to: filter.to,
      }),
      { signal },
    );
  }

  trashedDocuments(
    workspaceId: string,
    cursor?: string,
    signal?: AbortSignal,
  ): Promise<DocumentPage> {
    return this.#json(
      withQuery(`/api/v1/workspaces/${encodeURIComponent(workspaceId)}/documents/trash`, {
        cursor,
      }),
      { signal },
    );
  }

  restoreDocument(
    workspaceId: string,
    documentId: string,
    revision: number,
    command: CommandHeaders,
  ): Promise<components["schemas"]["Document"]> {
    return this.#json(`${resourcePath(workspaceId, "documents", documentId)}/restore`, {
      method: "POST",
      headers: commandHeaders(command, revision),
      body: JSON.stringify({ parentId: null, afterDocumentId: null }),
    });
  }

  purgeDocument(
    workspaceId: string,
    documentId: string,
    revision: number,
    reason: string,
    command: CommandHeaders,
  ): Promise<JobReference> {
    return this.#json(resourcePath(workspaceId, "documents", documentId), {
      method: "DELETE",
      headers: commandHeaders(command, revision),
      body: JSON.stringify({ reason }),
    });
  }

  async publicDocument(token: string, signal?: AbortSignal): Promise<PublicDocument> {
    if (token.length !== 43 || !/^[A-Za-z0-9_-]+$/.test(token))
      throw new ApiProblemError({
        code: "PUBLIC_DOCUMENT_NOT_FOUND",
        message: "Document not found",
      });
    const response = await this.#fetch(`/public/v1/documents/${encodeURIComponent(token)}`, {
      signal,
      credentials: "omit",
      redirect: "manual",
      headers: { accept: "application/json" },
    });
    if (!response.ok)
      throw new ApiProblemError({
        code: "PUBLIC_DOCUMENT_NOT_FOUND",
        message: "Document not found",
      });
    const contentType = response.headers.get("content-type") ?? "";
    if (!contentType.toLowerCase().startsWith("application/json"))
      throw new ApiProblemError({
        code: "PUBLIC_DOCUMENT_NOT_FOUND",
        message: "Document not found",
      });
    return (await response.json()) as PublicDocument;
  }

  async #json<T>(path: string, init: RequestInit): Promise<T> {
    if (!path.startsWith("/api/v1/") || path.startsWith("//")) throw new Error("unsafe API path");
    const headers = new Headers(init.headers);
    if (!headers.has("accept")) headers.set("accept", "application/json");
    const response = await this.#fetch(path, {
      ...init,
      credentials: "same-origin",
      headers,
      redirect: "manual",
    });
    const contentType = response.headers.get("content-type") ?? "";
    if (!contentType.toLowerCase().startsWith("application/json")) {
      throw new ApiProblemError({
        code: "DEPENDENCY_UNAVAILABLE",
        message: "API returned an unsupported response",
      });
    }
    const payload: unknown = await response.json();
    if (!response.ok) throw new ApiProblemError(parseProblem(payload));
    return payload as T;
  }

  async #empty(path: string, init: RequestInit): Promise<void> {
    if (!path.startsWith("/api/v1/") || path.startsWith("//")) throw new Error("unsafe API path");
    const headers = new Headers(init.headers);
    if (!headers.has("accept")) headers.set("accept", "application/json");
    const response = await this.#fetch(path, {
      ...init,
      credentials: "same-origin",
      headers,
      redirect: "manual",
    });
    if (response.ok) return;
    const payload: unknown = await response.json().catch(() => undefined);
    throw new ApiProblemError(parseProblem(payload));
  }
}

export type CommandHeaders = { csrfToken: string; idempotencyKey: string };

function commandHeaders(command: CommandHeaders, revision?: number): Headers {
  const headers = new Headers({
    "content-type": "application/json",
    "x-csrf-token": command.csrfToken,
    "idempotency-key": command.idempotencyKey,
  });
  if (revision !== undefined) headers.set("if-match", String(revision));
  return headers;
}

function previewHeaders(csrfToken: string, revision: number): Headers {
  return new Headers({
    "content-type": "application/json",
    "x-csrf-token": csrfToken,
    "if-match": String(revision),
  });
}

function leaseHeaders(
  command: CommandHeaders,
  revision: number,
  leaseToken: string,
  clientInstanceId: string,
): Headers {
  const headers = commandHeaders(command, revision);
  headers.set("x-edit-lease", leaseToken);
  headers.set("x-client-instance", clientInstanceId);
  return headers;
}

function resourcePath(workspaceId: string, kind: string, resourceId: string): string {
  return `/api/v1/workspaces/${encodeURIComponent(workspaceId)}/${kind}/${encodeURIComponent(resourceId)}`;
}

function withQuery(path: string, values: Record<string, string | undefined>): string {
  const query = new URLSearchParams();
  for (const [key, value] of Object.entries(values)) if (value !== undefined) query.set(key, value);
  const encoded = query.toString();
  return encoded ? `${path}?${encoded}` : path;
}

function parseProblem(value: unknown): ApiProblem {
  if (!value || typeof value !== "object") {
    return { code: "INTERNAL", message: "Unexpected API error" };
  }
  const input = value as Record<string, unknown>;
  return {
    code: typeof input.code === "string" ? input.code : "INTERNAL",
    message: typeof input.message === "string" ? input.message : "Unexpected API error",
    ...(typeof input.correlationId === "string" ? { correlationId: input.correlationId } : {}),
    ...(isStringRecord(input.fieldErrors) ? { fieldErrors: input.fieldErrors } : {}),
    ...(isUnknownRecord(input.meta) ? { meta: input.meta } : {}),
  };
}

function isUnknownRecord(value: unknown): value is Record<string, unknown> {
  return Boolean(value) && typeof value === "object" && !Array.isArray(value);
}

function isStringRecord(value: unknown): value is Record<string, string> {
  return isUnknownRecord(value) && Object.values(value).every((item) => typeof item === "string");
}
