import { expect, test } from "@playwright/test";
import manifest from "../../docs/design/quality/browser-acceptance-manifest.json";
import { authenticate, fixture, freezeVisuals, openAuthenticated } from "./helpers";

test.describe.configure({ mode: "serial" });

const engineEvidence = ["Chromium", "Firefox", "WebKit"] as const;
void engineEvidence;

for (const scenario of manifest.scenarios) {
  test(`${scenario.id} ${scenario.title}`, async ({ page, browser }, testInfo) => {
    const documentPath = `/w/${fixture.workspaceSlug}/docs/${fixture.documentId}`;
    switch (scenario.id) {
      case "ACC-01":
        await openAuthenticated(page, `${documentPath}?mode=published`);
        await expect(page.getByRole("heading", { name: "Authentication", level: 1 })).toBeVisible();
        await expect(page.getByText("발행된 지식을 기준으로 설명합니다.")).toBeVisible();
        await freezeVisuals(page);
        await expect(page).toHaveScreenshot("document-wide.png", { fullPage: true });
        break;
      case "ACC-02": {
        await authenticate(page.context(), fixture.memberSessionToken);
        const denied = await page.request.get(
          `/api/v1/workspaces/${fixture.workspaceId}/documents/${fixture.privateDocumentId}`,
        );
        expect(denied.status()).toBe(404);
        await page.goto(`/w/${fixture.workspaceSlug}/home`);
        await expect(page.getByText("Private Architecture")).toHaveCount(0);
        break;
      }
      case "ACC-03": {
        const member = await browser.newContext();
        const owner = page.context();
        const memberPage = await member.newPage();
        let acquiredLease:
          | { clientInstanceId: string; revision: number; token: string }
          | undefined;
        try {
          await authenticate(owner, fixture.ownerSessionToken);
          await authenticate(member, fixture.memberSessionToken);
          const ownerCsrf = (await owner.cookies()).find(
            (cookie) => cookie.name === "adoc_csrf",
          )?.value;
          if (!ownerCsrf) throw new Error("lease owner is missing the CSRF cookie");
          const leaseResponse = page.waitForResponse(
            (response) =>
              response.url().endsWith("/lease") && response.request().method() === "POST",
          );
          await page.goto(`${documentPath}?mode=draft`);
          acquiredLease = (await (await leaseResponse).json()) as typeof acquiredLease;
          await expect(page.getByRole("region", { name: "문서 초안 편집기" })).toBeVisible();
          await memberPage.goto(`${documentPath}?mode=draft`);
          await expect(memberPage.getByText(/편집 세션|editing session/)).toBeVisible();
        } finally {
          if (acquiredLease) {
            const csrf = (await owner.cookies()).find(
              (cookie) => cookie.name === "adoc_csrf",
            )?.value;
            if (csrf) {
              const release = await owner.request.delete(
                `/api/v1/workspaces/${fixture.workspaceId}/documents/${fixture.documentId}/lease`,
                {
                  headers: {
                    "if-match": `"${acquiredLease.revision}"`,
                    "x-edit-lease": acquiredLease.token,
                    "x-client-instance": acquiredLease.clientInstanceId,
                    "x-csrf-token": csrf,
                    "idempotency-key": crypto.randomUUID(),
                    origin: process.env.ADOC_BROWSER_BASE_URL ?? "http://localhost:8080",
                  },
                },
              );
              expect(release.status()).toBe(204);
            }
          }
          await memberPage.close();
          await member.close();
        }
        break;
      }
      case "ACC-04":
        await openAuthenticated(page, `${documentPath}?mode=published&panel=review`);
        await expect(page.getByRole("heading", { name: /검토/ }).first()).toBeVisible();
        break;
      case "ACC-05":
      case "ACC-06":
        await openAuthenticated(page, `${documentPath}?mode=published`);
        await expect(page.getByText(/v1 · Initial browser baseline/)).toBeVisible();
        await page.reload();
        await expect(page.getByText("발행된 지식을 기준으로 설명합니다.")).toBeVisible();
        break;
      case "ACC-07": {
        await openAuthenticated(page, `${documentPath}?mode=published&panel=discussion`);
        const discussionTitle = `브라우저 토론 ${testInfo.project.name}`;
        await page.getByLabel("제목").fill(discussionTitle);
        await page.getByLabel("첫 메시지").fill("메시지 이력을 보존합니다.");
        await page.getByRole("button", { name: "토론 만들기" }).click();
        await page.getByRole("link", { name: discussionTitle }).click();
        await expect(
          page.getByText("메시지 이력을 보존합니다.", { exact: true }).first(),
        ).toBeVisible();
        await page.getByLabel("종료 사유").fill("결론 반영");
        await page.getByRole("button", { name: "토론 닫기" }).click();
        await expect(page.getByRole("button", { name: "토론 다시 열기" })).toBeVisible();
        await page.getByLabel("다시 여는 이유").fill("추가 검토");
        await page.getByRole("button", { name: "토론 다시 열기" }).click();
        await expect(page.getByRole("button", { name: "토론 닫기" })).toBeVisible();
        await expect(
          page.getByText("메시지 이력을 보존합니다.", { exact: true }).first(),
        ).toBeVisible();
        break;
      }
      case "ACC-08":
        await openAuthenticated(
          page,
          `/w/${fixture.workspaceSlug}/search?q=Private%20Architecture`,
          fixture.memberSessionToken,
        );
        await expect(page.getByText("Private Architecture")).toHaveCount(0);
        break;
      case "ACC-09":
      case "ACC-10":
      case "ACC-11":
        await openAuthenticated(page, `${documentPath}?mode=published&panel=ai`);
        await expect(page.getByRole("heading", { name: "AI 도우미" })).toBeVisible();
        await expect(page.getByRole("button", { name: "작업 실행" })).toBeDisabled();
        break;
      case "ACC-12":
        await openAuthenticated(page, `${documentPath}?mode=published`);
        await expect(page.getByText("발행된 지식을 기준으로 설명합니다.")).toBeVisible();
        break;
      case "ACC-13":
        await page.goto(`/p/${fixture.publicToken}`);
        await expect(page.getByRole("heading", { name: "Authentication" })).toBeVisible();
        await expect(page.getByText("발행된 지식을 기준으로 설명합니다.")).toBeVisible();
        await expect(page.getByRole("navigation")).toHaveCount(0);
        break;
      case "ACC-14":
        await openAuthenticated(page, `/w/${fixture.workspaceSlug}/trash`);
        await expect(page.getByText("Trashed Decision")).toBeVisible();
        await expect(page.getByRole("button", { name: "복원" })).toBeVisible();
        break;
      case "ACC-15":
        await openAuthenticated(page, `/w/${fixture.workspaceSlug}/inbox`);
        await expect(page.getByRole("heading", { name: "받은 편지함" })).toBeVisible();
        await expect
          .poll(async () => (await page.request.get("/api/v1/session")).status())
          .toBe(200);
        break;
      default:
        throw new Error(`unhandled browser scenario ${scenario.id}`);
    }
  });
}
