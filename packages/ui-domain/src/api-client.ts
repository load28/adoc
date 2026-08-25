import type { components } from "@adoc/contracts/openapi";

export type SessionView = components["schemas"]["SessionView"];
export type UserPreferences = components["schemas"]["UserPreferences"];
export type WorkspaceView = components["schemas"]["Workspace"];

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
