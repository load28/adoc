import { describe, expect, test } from "bun:test";
import type { DocumentContent } from "@adoc/contracts";
import { renderToStaticMarkup } from "react-dom/server";

import { ContentRenderer } from "./content-renderer";

describe("immutable content renderer", () => {
  test("renders semantic content and resolves only provided asset boundaries", () => {
    const content: DocumentContent = {
      schemaVersion: 1,
      root: {
        type: "doc",
        children: [
          {
            id: "00000000-0000-4000-8000-000000000001",
            type: "heading",
            level: 2,
            children: [{ type: "text", text: "제목", marks: [{ type: "bold" }] }],
          },
          {
            id: "00000000-0000-4000-8000-000000000002",
            type: "image",
            assetId: "00000000-0000-4000-8000-000000000003",
            alt: "구조도",
          },
        ],
      },
    };
    const markup = renderToStaticMarkup(
      <ContentRenderer content={content} assetUrl={(id) => `/scoped/${id}`} />,
    );
    expect(markup).toContain("<h2><span><strong>제목</strong></span></h2>");
    expect(markup).toContain('alt="구조도"');
    expect(markup).toContain("/scoped/00000000-0000-4000-8000-000000000003");
  });
});
