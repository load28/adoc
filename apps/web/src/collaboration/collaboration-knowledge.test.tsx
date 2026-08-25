import { describe, expect, test } from "bun:test";
import axe from "axe-core";
import { JSDOM } from "jsdom";
import { renderToStaticMarkup } from "react-dom/server";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";

import { ProductAppProvider } from "../shell/product-app-provider";
import { SearchScreen } from "./collaboration-knowledge-screen";

describe("collaboration and knowledge screens", () => {
  test("renders an accessible search command without submitting local input", async () => {
    const html = renderToStaticMarkup(
      <ProductAppProvider locale="ko" theme="LIGHT">
        <QueryClientProvider client={new QueryClient()}>
          <SearchScreen
            workspaceId="10000000-0000-7000-8000-000000000001"
            workspaceSlug="workspace"
          />
        </QueryClientProvider>
      </ProductAppProvider>,
    );
    expect(html).toContain("검색어");
    const dom = new JSDOM(
      `<!doctype html><html lang="ko"><head><title>검색</title></head><body>${html}</body></html>`,
      { runScripts: "outside-only" },
    );
    dom.window.eval(axe.source);
    const result = await (dom.window as unknown as { axe: typeof axe }).axe.run(
      dom.window.document,
      { resultTypes: ["violations"], rules: { "color-contrast": { enabled: false } } },
    );
    expect(
      result.violations.filter((item) => ["critical", "serious"].includes(item.impact ?? "")),
    ).toEqual([]);
  });
});
