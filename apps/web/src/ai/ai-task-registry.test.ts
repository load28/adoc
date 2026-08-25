import { describe, expect, test } from "bun:test";

import { aiTaskKinds, taskInputReady, taskTarget } from "./ai-inspector";

describe("AI task registry", () => {
  test("keeps all six product task kinds in one closed registry", () => {
    expect(aiTaskKinds).toEqual([
      "COMPOSE",
      "REWRITE",
      "REVIEW",
      "DISCUSSION_APPLY",
      "CONFLICT_MERGE",
      "KNOWLEDGE_QUERY",
    ]);
  });

  test("derives task-specific targets and readiness without changing revision semantics", () => {
    expect(taskTarget("REVIEW", "document", "", "")).toEqual({
      kind: "DOCUMENT",
      documentId: "document",
    });
    expect(taskTarget("DISCUSSION_APPLY", "document", "discussion", "")).toEqual({
      kind: "DISCUSSION",
      discussionId: "discussion",
    });
    expect(taskTarget("KNOWLEDGE_QUERY", "document", "", "질문")).toEqual({
      kind: "WORKSPACE_QUERY",
      question: "질문",
    });
    expect(taskInputReady("DISCUSSION_APPLY", "", "")).toBe(false);
    expect(taskInputReady("KNOWLEDGE_QUERY", "", "")).toBe(false);
  });
});
