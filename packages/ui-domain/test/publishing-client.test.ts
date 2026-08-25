import { describe, expect, test } from "bun:test";

import { ApiClient } from "../src";

describe("publishing API client", () => {
  test("binds publish to the exact draft revision and current lease", async () => {
    const requests: Array<{ input: string; init?: RequestInit }> = [];
    const api = new ApiClient(async (input, init) => {
      requests.push({ input: String(input), init });
      return Response.json(version());
    });
    await api.publishDocument(
      "11111111-1111-4111-8111-111111111111",
      "22222222-2222-4222-8222-222222222222",
      7,
      {
        summary: "요약",
        clientInstanceId: "33333333-3333-4333-8333-333333333333",
        leaseToken: "x".repeat(43),
      },
      { csrfToken: "csrf", idempotencyKey: "44444444-4444-4444-8444-444444444444" },
    );
    expect(new Headers(requests[0]?.init?.headers).get("if-match")).toBe('"7"');
    expect(JSON.parse(String(requests[0]?.init?.body))).toEqual({
      summary: "요약",
      clientInstanceId: "33333333-3333-4333-8333-333333333333",
      leaseToken: "x".repeat(43),
    });
  });

  test("uses immutable version resource for detail and restore", async () => {
    const requests: Array<{ input: string; init?: RequestInit }> = [];
    const api = new ApiClient(async (input, init) => {
      requests.push({ input: String(input), init });
      return Response.json(
        requests.length === 1
          ? version()
          : { id: "draft", revision: 0, content: version().content },
      );
    });
    await api.version("workspace", "document", "version");
    await api.restoreVersion("workspace", "document", "version", 9, {
      csrfToken: "csrf",
      idempotencyKey: "key",
    });
    expect(requests.map((request) => [request.init?.method ?? "GET", request.input])).toEqual([
      ["GET", "/api/v1/workspaces/workspace/documents/document/versions/version"],
      ["POST", "/api/v1/workspaces/workspace/documents/document/versions/version/restore"],
    ]);
    expect(new Headers(requests[1]?.init?.headers).get("if-match")).toBe('"9"');
  });
});

function version() {
  return {
    id: "version",
    documentId: "document",
    number: 1,
    publishedAt: "2026-08-25T00:00:00Z",
    publisherId: "user",
    schemaVersion: 1,
    contentFingerprint: "a".repeat(64),
    basedOnVersionId: null,
    sourceDraftRevision: 7,
    content: { schemaVersion: 1, root: { type: "doc", children: [] } },
    summary: "요약",
    reviewSnapshot: {},
    discussionIds: [],
  };
}
