import { parseLocale } from "@adoc/i18n";
import {
  ApiClient,
  ApiProblemError,
  type DocumentDetail,
  type InvitationPreview,
  type SessionView,
  type WorkspaceView,
} from "@adoc/ui-domain";
import { createServerFn } from "@tanstack/react-start";
import { getRequest, getRequestHeader } from "@tanstack/react-start/server";

import type { ThemePreference } from "./product-app-provider";

export type ShellBootstrap = {
  authenticated: boolean;
  locale: "ko" | "en";
  theme: ThemePreference;
  session?: SessionView;
};

export const loadShellBootstrap = createServerFn({ method: "GET" }).handler(
  async (): Promise<ShellBootstrap> => {
    const request = getRequest();
    const cookie = getRequestHeader("cookie") ?? "";
    if (!/(?:^|;\s*)adoc_session=/.test(cookie)) return anonymousBootstrap();

    const client = new ApiClient((input, init) => {
      const headers = new Headers(init?.headers);
      headers.set("cookie", cookie);
      return fetch(new URL(String(input), request.url), { ...init, headers });
    });

    try {
      const session = await client.session();
      const preferences = await client.preferences().catch(() => undefined);
      return {
        authenticated: true,
        locale: parseLocale(preferences?.locale ?? session.user.locale),
        theme: parseTheme(preferences?.theme),
        session,
      };
    } catch (error) {
      if (error instanceof ApiProblemError && error.problem.code === "SESSION_REQUIRED") {
        return anonymousBootstrap();
      }
      throw error;
    }
  },
);

export const loadInvitationPreview = createServerFn({ method: "GET" })
  .validator((input: { token: string }) => input)
  .handler(async ({ data }): Promise<InvitationPreview> => {
    const request = getRequest();
    const cookie = getRequestHeader("cookie") ?? "";
    const client = new ApiClient((input, init) => {
      const headers = new Headers(init?.headers);
      headers.set("cookie", cookie);
      return fetch(new URL(String(input), request.url), { ...init, headers });
    });
    return client.invitationPreview(data.token);
  });

export const loadWorkspaceList = createServerFn({ method: "GET" }).handler(
  async (): Promise<WorkspaceView[]> => serverApiClient().workspaces(),
);

export const loadWorkspaceBySlug = createServerFn({ method: "GET" })
  .validator((input: { workspaceSlug: string }) => input)
  .handler(async ({ data }): Promise<WorkspaceView> => {
    const client = serverApiClient();
    const summary = (await client.workspaces()).find(
      (candidate) => candidate.slug === data.workspaceSlug,
    );
    if (!summary) throw new ApiProblemError({ code: "WORKSPACE_NOT_FOUND", message: "Not found" });
    return client.workspace(summary.id);
  });

export const loadDocumentRoute = createServerFn({ method: "GET" })
  .validator((input: { workspaceSlug: string; documentId: string }) => input)
  .handler(async ({ data }): Promise<{ workspaceId: string; document: DocumentDetail }> => {
    const request = getRequest();
    const cookie = getRequestHeader("cookie") ?? "";
    const client = new ApiClient((input, init) => {
      const headers = new Headers(init?.headers);
      headers.set("cookie", cookie);
      return fetch(new URL(String(input), request.url), { ...init, headers });
    });
    const session = await client.session();
    const workspace = session.workspaces.find((candidate) => candidate.slug === data.workspaceSlug);
    if (!workspace)
      throw new ApiProblemError({ code: "WORKSPACE_NOT_FOUND", message: "Not found" });
    return {
      workspaceId: workspace.id,
      document: await client.document(workspace.id, data.documentId),
    };
  });

function anonymousBootstrap(): ShellBootstrap {
  return { authenticated: false, locale: "ko", theme: "SYSTEM" };
}

function serverApiClient(): ApiClient {
  const request = getRequest();
  const cookie = getRequestHeader("cookie") ?? "";
  return new ApiClient((input, init) => {
    const headers = new Headers(init?.headers);
    headers.set("cookie", cookie);
    return fetch(new URL(String(input), request.url), { ...init, headers });
  });
}

function parseTheme(value: unknown): ThemePreference {
  return value === "LIGHT" || value === "DARK" ? value : "SYSTEM";
}
