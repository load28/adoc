import { describe, expect, test } from "bun:test";
import type { DocumentContent, DocumentOperation } from "@adoc/contracts";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { OperationError, applyOperations, createTextRegion, reanchorRegion } from "../src/reducer";

const empty: DocumentContent = { schemaVersion: 1, root: { type: "doc", children: [] } };
const operation = (opId: string, blockId: string, index: number): DocumentOperation => ({
  opId,
  kind: "INSERT_BLOCK",
  scope: { kind: "DOCUMENT" },
  precondition: { draftRevision: 0, targetHash: null },
  dependsOn: [],
  parentId: null,
  index,
  block: { id: blockId, type: "paragraph", children: [{ type: "text", text: "가" }] },
});

describe("content operation reducer", () => {
  test("matches the shared Rust and TypeScript fixture", async () => {
    const fixture = JSON.parse(
      readFileSync(
        resolve(
          import.meta.dir,
          "../../../docs/design/quality/fixtures/operation-reducer.valid.json",
        ),
        "utf8",
      ),
    );
    const result = await applyOperations(fixture);
    expect(result.content).toEqual(fixture.expected.content);
    expect(result.contentFingerprint).toBe(fixture.expected.contentFingerprint);
    expect(result.appliedOperationIds).toEqual(fixture.expected.appliedOperationIds);
    expect(result.inverseOperations.map((operation) => operation.opId)).toEqual(
      fixture.expected.inverseOperationIds,
    );
  });

  test("orders independent operations and applies inverse deterministically", async () => {
    const first = "00000000-0000-0000-0000-000000000001";
    const second = "00000000-0000-0000-0000-000000000002";
    const result = await applyOperations({
      content: empty,
      baseRevision: 0,
      operations: [
        operation(second, "00000000-0000-0000-0000-000000000012", 1),
        operation(first, "00000000-0000-0000-0000-000000000011", 0),
      ],
      references: [],
    });
    expect(result.appliedOperationIds).toEqual([first, second]);
    const restored = await applyOperations({
      content: result.content,
      baseRevision: 1,
      operations: result.inverseOperations,
      references: [],
    });
    expect(restored.content).toEqual(empty);
  });

  test("rejects a dependency outside the batch", async () => {
    const invalid = operation(
      "00000000-0000-0000-0000-000000000001",
      "00000000-0000-0000-0000-000000000011",
      0,
    );
    invalid.dependsOn = ["00000000-0000-0000-0000-000000000099"];
    try {
      await applyOperations({
        content: empty,
        baseRevision: 0,
        operations: [invalid],
        references: [],
      });
      throw new Error("expected failure");
    } catch (error) {
      expect(error).toBeInstanceOf(OperationError);
      expect((error as OperationError).code).toBe("DEPENDENCY_INVALID");
    }
  });

  test("reanchors one exact text candidate after nearby content moves", async () => {
    const blockId = "00000000-0000-0000-0000-000000000040";
    const originalText = `${"x".repeat(40)}target${"y".repeat(40)}`;
    const original = {
      schemaVersion: 1,
      root: {
        type: "doc",
        children: [
          { id: blockId, type: "paragraph", children: [{ type: "text", text: originalText }] },
        ],
      },
    } as DocumentContent;
    const region = await createTextRegion(original, blockId, 40, 46);
    const moved = structuredClone(original);
    const paragraph = moved.root.children[0] as { children: Array<{ text: string }> };
    const text = paragraph.children[0];
    if (!text) throw new Error("fixture text is missing");
    text.text = `z${originalText}`;
    const resolution = await reanchorRegion(moved, region);
    expect(resolution.status).toBe("MOVED");
    expect(resolution.region).toMatchObject({
      kind: "TEXT_RANGE",
      from: { offset: 41 },
      to: { offset: 47 },
    });
  });

  test("rejects duplicate IDs, unsafe links, and invalid table grids", async () => {
    const id = "00000000-0000-0000-0000-000000000001";
    const invalid = [
      {
        schemaVersion: 1,
        root: {
          type: "doc",
          children: [
            { id, type: "divider" },
            { id, type: "divider" },
          ],
        },
      },
      {
        schemaVersion: 1,
        root: {
          type: "doc",
          children: [
            {
              id,
              type: "paragraph",
              children: [
                { type: "text", text: "x", marks: [{ type: "link", href: "javascript:alert(1)" }] },
              ],
            },
          ],
        },
      },
      {
        schemaVersion: 1,
        root: {
          type: "doc",
          children: [
            {
              id,
              type: "table",
              rows: [
                {
                  id: "00000000-0000-0000-0000-000000000002",
                  type: "tableRow",
                  cells: [
                    {
                      id: "00000000-0000-0000-0000-000000000003",
                      type: "tableCell",
                      colspan: 2,
                      children: [
                        {
                          id: "00000000-0000-0000-0000-000000000004",
                          type: "paragraph",
                          children: [],
                        },
                      ],
                    },
                  ],
                },
                {
                  id: "00000000-0000-0000-0000-000000000005",
                  type: "tableRow",
                  cells: [
                    {
                      id: "00000000-0000-0000-0000-000000000006",
                      type: "tableCell",
                      children: [
                        {
                          id: "00000000-0000-0000-0000-000000000007",
                          type: "paragraph",
                          children: [],
                        },
                      ],
                    },
                  ],
                },
              ],
            },
          ],
        },
      },
    ] as unknown as DocumentContent[];
    for (const content of invalid)
      await expect(createTextRegion(content, id, 0, 0)).rejects.toBeInstanceOf(OperationError);
  });
});
