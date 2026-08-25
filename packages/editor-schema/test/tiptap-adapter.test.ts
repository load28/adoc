import { describe, expect, test } from "bun:test";
import type { DocumentContent } from "@adoc/contracts";

import {
  UnsupportedEditorContentError,
  createEditorOperationBatch,
  editorJsonToProductContent,
  productContentToEditorJson,
} from "../src";

const firstId = "00000000-0000-4000-8000-000000000001";
const secondId = "00000000-0000-4000-8000-000000000002";

describe("Tiptap product adapter", () => {
  test("preserves stable ids, UTF-16 text and marks", () => {
    const content: DocumentContent = {
      schemaVersion: 1,
      root: {
        type: "doc",
        children: [
          {
            id: firstId,
            type: "heading",
            level: 2,
            children: [{ type: "text", text: "한글😀", marks: [{ type: "bold" }] }],
          },
        ],
      },
    };
    expect(editorJsonToProductContent(productContentToEditorJson(content))).toEqual(content);
  });

  test("creates the smallest changed block region", () => {
    const before = paragraphs("before", "stable");
    const after = paragraphs("after", "stable");
    const operations = createEditorOperationBatch(before, after, 7, () => firstId);
    expect(operations).toHaveLength(1);
    expect(operations[0]).toMatchObject({
      kind: "REPLACE_REGION",
      scope: { kind: "BLOCK", blockId: firstId },
      precondition: { draftRevision: 7 },
    });
  });

  test("rejects unknown nodes without silent conversion", () => {
    expect(() =>
      editorJsonToProductContent({
        type: "doc",
        content: [{ type: "foreignNode", attrs: { blockId: firstId } }],
      }),
    ).toThrow(UnsupportedEditorContentError);
  });
});

function paragraphs(first: string, second: string): DocumentContent {
  return {
    schemaVersion: 1,
    root: {
      type: "doc",
      children: [
        { id: firstId, type: "paragraph", children: [{ type: "text", text: first }] },
        { id: secondId, type: "paragraph", children: [{ type: "text", text: second }] },
      ],
    },
  };
}
