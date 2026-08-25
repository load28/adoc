import { parseLocale } from "@adoc/i18n";
import { ApiClient, ApiProblemError, type SessionView } from "@adoc/ui-domain";
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

function anonymousBootstrap(): ShellBootstrap {
  return { authenticated: false, locale: "ko", theme: "SYSTEM" };
}

function parseTheme(value: unknown): ThemePreference {
  return value === "LIGHT" || value === "DARK" ? value : "SYSTEM";
}
