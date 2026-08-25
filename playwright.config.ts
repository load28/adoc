import { defineConfig, devices } from "@playwright/test";

const baseURL = process.env.ADOC_BROWSER_BASE_URL ?? "http://localhost:8080";

export default defineConfig({
  testDir: "./tests/browser",
  outputDir: "artifacts/browser/test-results",
  fullyParallel: false,
  workers: 1,
  retries: 0,
  timeout: 45_000,
  expect: { timeout: 10_000, toHaveScreenshot: { maxDiffPixels: 0 } },
  reporter: [["list"], ["json", { outputFile: "artifacts/browser/results.json" }]],
  snapshotPathTemplate: "{testDir}/__screenshots__/{projectName}/{arg}{ext}",
  use: {
    baseURL,
    locale: "ko-KR",
    timezoneId: "Asia/Seoul",
    colorScheme: "light",
    reducedMotion: "reduce",
    deviceScaleFactor: 1,
    trace: "retain-on-failure",
    screenshot: "only-on-failure",
    video: "retain-on-failure",
  },
  projects: [
    {
      name: "Chromium-wide",
      use: { ...devices["Desktop Chrome"], viewport: { width: 1440, height: 1000 } },
      testMatch: /acceptance\.spec\.ts/,
    },
    {
      name: "Firefox-wide",
      use: { ...devices["Desktop Firefox"], viewport: { width: 1440, height: 1000 } },
      testMatch: /acceptance\.spec\.ts/,
    },
    {
      name: "WebKit-wide",
      use: { ...devices["Desktop Safari"], viewport: { width: 1440, height: 1000 } },
      testMatch: /acceptance\.spec\.ts/,
    },
    {
      name: "Chromium-compact",
      use: { browserName: "chromium", viewport: { width: 390, height: 844 } },
      testMatch: /browser-quality\.spec\.ts/,
    },
    {
      name: "Firefox-compact",
      use: { browserName: "firefox", viewport: { width: 390, height: 844 } },
      testMatch: /browser-quality\.spec\.ts/,
    },
    {
      name: "WebKit-compact",
      use: { browserName: "webkit", viewport: { width: 390, height: 844 } },
      testMatch: /browser-quality\.spec\.ts/,
    },
  ],
});
