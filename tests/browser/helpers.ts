import { expect, type BrowserContext, type Page } from "@playwright/test";
import { readFileSync } from "node:fs";

export type BrowserFixture = {
  schemaVersion: number;
  runId: string;
  workspaceId: string;
  workspaceSlug: string;
  documentId: string;
  privateDocumentId: string;
  trashDocumentId: string;
  ownerUserId: string;
  ownerSessionToken: string;
  memberUserId: string;
  memberSessionToken: string;
  publicToken: string;
};

export const fixture: BrowserFixture = JSON.parse(
  readFileSync(process.env.ADOC_BROWSER_FIXTURE_FILE ?? "artifacts/browser/fixture.json", "utf8"),
);

export async function authenticate(context: BrowserContext, token: string) {
  await context.addCookies([
    {
      name: "adoc_session",
      value: token,
      url: process.env.ADOC_BROWSER_BASE_URL ?? "http://localhost:8080",
      httpOnly: true,
      secure: true,
      sameSite: "Lax",
    },
  ]);
  const response = await context.request.get("/api/v1/session");
  expect(response.status()).toBe(200);
  const csrfCookie = response
    .headersArray()
    .find(
      (header) =>
        header.name.toLowerCase() === "set-cookie" && header.value.startsWith("adoc_csrf="),
    )
    ?.value.split(";", 1)[0]
    ?.slice("adoc_csrf=".length);
  expect(csrfCookie).toBeTruthy();
  if (!csrfCookie) throw new Error("session response did not set the CSRF cookie");
  await context.addCookies([
    {
      name: "adoc_csrf",
      value: csrfCookie,
      url: process.env.ADOC_BROWSER_BASE_URL ?? "http://localhost:8080",
      httpOnly: false,
      secure: true,
      sameSite: "Strict",
    },
  ]);
}

export async function openAuthenticated(
  page: Page,
  path: string,
  token = fixture.ownerSessionToken,
) {
  await authenticate(page.context(), token);
  const errors: string[] = [];
  page.on("pageerror", (error) => errors.push(error.message));
  page.on("response", (response) => {
    if (response.url().startsWith(page.url().split("/w/")[0]) && response.status() >= 500) {
      errors.push(`${response.status()} ${response.url()}`);
    }
  });
  const response = await page.goto(path, { waitUntil: "domcontentloaded" });
  expect(response?.status()).toBeLessThan(500);
  await expect.poll(() => errors).toEqual([]);
}

export async function freezeVisuals(page: Page) {
  await page.emulateMedia({ reducedMotion: "reduce", colorScheme: "light" });
  await page.addStyleTag({
    content:
      "*,*::before,*::after{animation:none!important;transition:none!important;caret-color:transparent!important}",
  });
  await page.evaluate(() => document.fonts.ready);
}

export async function assertAxe(page: Page) {
  await page.addScriptTag({ path: "apps/web/node_modules/axe-core/axe.min.js" });
  const violations = await page.evaluate(async () => {
    const axe = (
      window as typeof window & {
        axe: {
          run: (
            root: Document,
            options: object,
          ) => Promise<{
            violations: Array<{ id: string; impact: string | null; nodes: unknown[] }>;
          }>;
        };
      }
    ).axe;
    return (
      await axe.run(document, {
        runOnly: { type: "tag", values: ["wcag2a", "wcag2aa", "wcag21a", "wcag21aa", "wcag22aa"] },
      })
    ).violations;
  });
  expect(violations, JSON.stringify(violations, null, 2)).toEqual([]);
}
