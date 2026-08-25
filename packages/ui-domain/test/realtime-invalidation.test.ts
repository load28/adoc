import { describe, expect, test } from "bun:test";

import { invalidationRoots, workspaceRealtimeEvents } from "../src/realtime-invalidation";

describe("workspace realtime invalidation", () => {
  test("uses events only as bounded query invalidation signals", () => {
    expect(workspaceRealtimeEvents).toContain("DISCUSSION_CHANGED");
    expect(invalidationRoots("DISCUSSION_CHANGED")).toEqual([
      "discussion",
      "discussion-detail",
      "inbox",
    ]);
    expect(invalidationRoots("VOCABULARY_CHANGED")).toEqual(["vocabulary", "search"]);
  });
});
