import { expect, test } from "@playwright/test";
import { assertAxe, fixture, freezeVisuals, openAuthenticated } from "./helpers";

test("compact login has keyboard access, WCAG automation and visual baseline", async ({ page }) => {
  await page.goto("/login");
  await expect(page.getByRole("heading", { level: 1 })).toBeVisible();
  await page.keyboard.press("Tab");
  await expect(page.getByRole("link", { name: /Google/ })).toBeFocused();
  await assertAxe(page);
  await freezeVisuals(page);
  await expect(page).toHaveScreenshot("login-compact.png", { fullPage: true });
});

test("compact Workspace shell reflows without horizontal overflow", async ({ page }) => {
  await openAuthenticated(page, `/w/${fixture.workspaceSlug}/home`);
  const overflow = await page.evaluate(
    () => document.documentElement.scrollWidth - window.innerWidth,
  );
  expect(overflow).toBeLessThanOrEqual(0);
  await assertAxe(page);
  await freezeVisuals(page);
  await expect(page).toHaveScreenshot("workspace-compact.png", { fullPage: true });
});

test("compact public viewer remains isolated and accessible", async ({ page }) => {
  await page.goto(`/p/${fixture.publicToken}`);
  await expect(page.getByRole("heading", { name: "Authentication" })).toBeVisible();
  await assertAxe(page);
  await freezeVisuals(page);
  await expect(page).toHaveScreenshot("public-compact.png", { fullPage: true });
});
