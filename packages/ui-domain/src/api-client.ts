import type { DocumentOperation } from "@adoc/contracts";
import type { components } from "@adoc/contracts/openapi";

export type SessionView = components["schemas"]["SessionView"];
export type UserPreferences = components["schemas"]["UserPreferences"];
export type WorkspaceView = components["schemas"]["Workspace"];
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

  async #json<T>(path: string, init: RequestInit): Promise<T> {
    if (!path.startsWith("/api/v1/") || path.startsWith("//")) throw new Error("unsafe API path");
    const response = await this.#fetch(path, {
      ...init,
      credentials: "same-origin",
      headers: { accept: "application/json", ...init.headers },
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
    const response = await this.#fetch(path, {
      ...init,
      credentials: "same-origin",
      headers: { accept: "application/json", ...init.headers },
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
