import { describe, expect, test } from "bun:test";

import { ApiClient } from "../src";

const command = { csrfToken: "csrf", idempotencyKey: "command-key" };

describe("collaboration and governance API client", () => {
  test("binds nested collaboration mutations to exact aggregate revisions", async () => {
    const requests: Array<{ input: string; init?: RequestInit }> = [];
    const api = recorder(requests);

    await api.addDiscussionTopic(
      "workspace",
      { id: "discussion", revision: 4 },
      { kind: "TEXT", label: "결정", text: "근거" },
      command,
    );
    await api.updateMessage(
      "workspace",
      "discussion",
      "message",
      9,
      { body: content(), mentionUserIds: [], attachmentIds: [] },
      command,
    );
    await api.cancelReview("workspace", { id: "review", revision: 3 }, "다시 작성", command);

    expect(requests.map(requestShape)).toEqual([
      ["POST", "/api/v1/workspaces/workspace/discussions/discussion/topics", "4"],
      ["PUT", "/api/v1/workspaces/workspace/discussions/discussion/messages/message", "9"],
      ["POST", "/api/v1/workspaces/workspace/reviews/review/cancel", "3"],
    ]);
  });

  test("binds reference mutations to one draft lease boundary", async () => {
    const requests: Array<{ input: string; init?: RequestInit }> = [];
    const api = recorder(requests, true);
    const region = { kind: "BLOCK" as const, blockId: "block" };
    const target = { kind: "DOCUMENT" as const, id: "target-document" };

    await api.createReference(
      "workspace",
      "document",
      7,
      { sourceRegion: region, target },
      "lease-token",
      "client-id",
      command,
    );
    await api.deleteReference(
      "workspace",
      "document",
      "reference",
      8,
      "lease-token",
      "client-id",
      command,
    );

    expect(requests.map(requestShape)).toEqual([
      ["POST", "/api/v1/workspaces/workspace/documents/document/references", "7"],
      ["DELETE", "/api/v1/workspaces/workspace/documents/document/references/reference", "8"],
    ]);
    for (const request of requests) {
      const headers = new Headers(request.init?.headers);
      expect(headers.get("x-edit-lease")).toBe("lease-token");
      expect(headers.get("x-client-instance")).toBe("client-id");
    }
  });

  test("keeps governance revisions and audit filters in the transport contract", async () => {
    const requests: Array<{ input: string; init?: RequestInit }> = [];
    const api = recorder(requests, true);

    await api.changeGroupMember("workspace", { id: "group", revision: 2 }, "user", "add", command);
    await api.deleteDocumentPermission("workspace", "document", "grant", 5, command);
    await api.setPublishPolicy(
      "workspace",
      "document",
      { revision: 6 },
      { mode: "REVIEW_REQUIRED", requiredApprovals: 2, reviewerRule: { kind: "ANY_EDITOR" } },
      command,
    );
    await api.auditEvents("workspace", "cursor", undefined, {
      action: "DOCUMENT_MOVED",
      actorUserId: "actor",
      targetKind: "DOCUMENT",
      from: "2026-08-01T00:00:00Z",
      to: "2026-08-25T00:00:00Z",
    });

    expect(requests.slice(0, 3).map(requestShape)).toEqual([
      ["PUT", "/api/v1/workspaces/workspace/groups/group/members/user", "2"],
      ["DELETE", "/api/v1/workspaces/workspace/documents/document/permissions/grant", "5"],
      ["PUT", "/api/v1/workspaces/workspace/documents/document/publish-policy", "6"],
    ]);
    expect(requests[3]?.input).toBe(
      "/api/v1/workspaces/workspace/audit-events?cursor=cursor&action=DOCUMENT_MOVED&actorUserId=actor&targetKind=DOCUMENT&from=2026-08-01T00%3A00%3A00Z&to=2026-08-25T00%3A00%3A00Z",
    );
  });
});

function recorder(
  requests: Array<{ input: string; init?: RequestInit }>,
  emptyDelete = false,
): ApiClient {
  return new ApiClient(async (input, init) => {
    requests.push({ input: String(input), init });
    if (emptyDelete && init?.method === "DELETE") return new Response(null, { status: 204 });
    return Response.json({});
  });
}

function requestShape(request: {
  input: string;
  init?: RequestInit;
}): [string, string, string | null] {
  return [
    request.init?.method ?? "GET",
    request.input,
    new Headers(request.init?.headers).get("if-match"),
  ];
}

function content() {
  return {
    schemaVersion: 1 as const,
    root: {
      type: "doc" as const,
      children: [
        {
          id: "00000000-0000-4000-8000-000000000001",
          type: "paragraph" as const,
          children: [{ type: "text" as const, text: "메시지" }],
        },
      ],
    },
  };
}
