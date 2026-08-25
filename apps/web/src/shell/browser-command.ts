import type { CommandHeaders } from "@adoc/ui-domain";

export function browserCommand(idempotencyKey = crypto.randomUUID()): CommandHeaders {
  const csrfToken = document.cookie
    .split("; ")
    .find((item) => item.startsWith("adoc_csrf="))
    ?.slice("adoc_csrf=".length);
  if (!csrfToken) throw new Error("CSRF token is unavailable");
  return { csrfToken: decodeURIComponent(csrfToken), idempotencyKey };
}
