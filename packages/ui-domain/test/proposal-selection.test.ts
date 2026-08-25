import { describe, expect, test } from "bun:test";
import type { DocumentOperation } from "@adoc/contracts";
import { selectProposalOperation } from "../src/proposal-selection";

const operation = (opId: string, dependsOn: string[] = []): DocumentOperation => ({
  opId,
  kind: "DELETE_BLOCK",
  scope: { kind: "BLOCK", blockId: crypto.randomUUID() },
  precondition: { draftRevision: 1 },
  blockId: crypto.randomUUID(),
  dependsOn,
});

describe("proposal selection", () => {
  test("selects transitive dependencies in proposal order", () => {
    const operations = [operation("a"), operation("b", ["a"]), operation("c", ["b"])];
    expect(selectProposalOperation(operations, [], "c")).toEqual(["a", "b", "c"]);
  });

  test("removes dependent operations when a prerequisite is cleared", () => {
    const operations = [operation("a"), operation("b", ["a"]), operation("c", ["b"])];
    expect(selectProposalOperation(operations, ["a", "b", "c"], "a")).toEqual([]);
  });
});
