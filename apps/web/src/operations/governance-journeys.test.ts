import { describe, expect, test } from "bun:test";
import { ApiClient } from "@adoc/ui-domain";

import {
  discussionTopic,
  inboxTargetHref,
  resolveInboxTarget,
} from "../collaboration/collaboration-knowledge-screen";
import { reviewerRule } from "./settings-audit-screen";

describe("governance journey state", () => {
  test("builds canonical document Inbox targets without leaking IDs into path syntax", () => {
    expect(
      inboxTargetHref("team docs", {
        kind: "DOCUMENT",
        id: "document/id",
      }),
    ).toBe("/w/team%20docs/docs/document%2Fid?mode=published");
  });

  test("constructs only the closed publish reviewer rule union", () => {
    expect(reviewerRule("ANY_EDITOR", "ignored")).toEqual({ kind: "ANY_EDITOR" });
    expect(reviewerRule("USERS", "a, b, a")).toEqual({ kind: "USERS", userIds: ["a", "b"] });
    expect(reviewerRule("GROUPS", "g1, g2")).toEqual({
      kind: "GROUPS",
      groupIds: ["g1", "g2"],
    });
  });

  test("constructs the closed Discussion topic union without ambiguous fields", () => {
    expect(discussionTopic("TEXT", "근거", "내용", "")).toEqual({
      kind: "TEXT",
      label: "근거",
      text: "내용",
    });
    expect(discussionTopic("REGION", "문단", "document", "block")).toEqual({
      kind: "REGION",
      label: "문단",
      targetId: "document",
      region: { kind: "BLOCK", blockId: "block" },
    });
    expect(() => discussionTopic("EXTERNAL", "링크", "http://unsafe", "")).toThrow();
  });

  test("resolves Review Inbox targets through permission-checked detail", async () => {
    const api = new ApiClient(async () => Response.json({ documentId: "document", id: "review" }));
    await expect(
      resolveInboxTarget(api, "workspace", "team", { kind: "REVIEW", id: "review" }),
    ).resolves.toBe("/w/team/docs/document?mode=published&panel=review&review=review");
  });
});
