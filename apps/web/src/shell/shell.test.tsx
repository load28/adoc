import { describe, expect, test } from "bun:test";
import axe from "axe-core";
import { readFileSync } from "node:fs";
import { JSDOM } from "jsdom";
import { renderToStaticMarkup } from "react-dom/server";

import { RouteEmpty } from "./common-states";
import { ProductAppProvider, routerTarget, toColorMode } from "./product-app-provider";

describe("TanStack shadcn shell", () => {
  test("separates app route search and leaves API links native", () => {
    expect(routerTarget("/w/alpha/docs/doc?mode=published&discussion=item#messages")).toEqual({
      pathname: "/w/alpha/docs/doc",
      search: { mode: "published", discussion: "item" },
      hash: "messages",
    });
    expect(routerTarget("/api/v1/files/id/content")).toBeUndefined();
    expect(routerTarget("https://example.test/path")).toBeUndefined();
  });

  test("renders the same locale and theme inputs on the server", () => {
    const html = renderToStaticMarkup(
      <ProductAppProvider locale="ko" theme="DARK">
        <RouteEmpty />
      </ProductAppProvider>,
    );
    expect(html).toContain("참여 중인 워크스페이스가 없습니다.");
    expect(toColorMode("LIGHT")).toBe("light");
    expect(toColorMode("DARK")).toBe("dark");
    expect(toColorMode("SYSTEM")).toBe("auto");
  });

  test("keeps the pre-hydration theme bootstrap closed to fixed preferences", () => {
    const themeBootstrapScript = readFileSync(
      new URL("./theme-bootstrap.js", import.meta.url),
      "utf8",
    );
    expect(themeBootstrapScript).not.toContain("innerHTML");
    expect(themeBootstrapScript).toContain("themePreference");
    expect(themeBootstrapScript).toContain("prefers-color-scheme");
  });

  test("has no critical or serious automated accessibility violations", async () => {
    const body = renderToStaticMarkup(
      <ProductAppProvider locale="en" theme="LIGHT">
        <RouteEmpty />
      </ProductAppProvider>,
    );
    const dom = new JSDOM(
      `<!doctype html><html lang="en"><head><title>Team Documents</title></head><body>${body}</body></html>`,
      {
        runScripts: "outside-only",
      },
    );
    dom.window.eval(axe.source);
    const browserAxe = (dom.window as unknown as { axe: typeof axe }).axe;
    const result = await browserAxe.run(dom.window.document, {
      resultTypes: ["violations"],
      rules: { "color-contrast": { enabled: false } },
    });
    expect(
      result.violations.filter((item) => ["critical", "serious"].includes(item.impact ?? "")),
    ).toEqual([]);
  });
});
