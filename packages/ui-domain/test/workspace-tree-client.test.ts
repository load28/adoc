import { describe, expect, test } from "bun:test";

import { ApiClient } from "../src";

describe("workspace and tree API client", () => {
  test("maps invitation preview and acceptance to one token capability path", async () => {
    const requests: Array<{ input: string; init?: RequestInit }> = [];
    const client = new ApiClient(async (input, init) => {
      requests.push({ input: String(input), init });
      if (init?.method === "POST") {
        return Response.json({
          userId: crypto.randomUUID(),
          role: "MEMBER",
          status: "ACTIVE",
          revision: 0,
        });
      }
      return Response.json({
        workspaceId: crypto.randomUUID(),
        workspaceName: "Platform",
        workspaceSlug: "platform",
        role: "MEMBER",
        expiresAt: new Date().toISOString(),
      });
    });
    const token = "a/b";
    await client.invitationPreview(token);
    await client.acceptInvitation(token, {
      csrfToken: "x".repeat(43),
      idempotencyKey: "invite-command-01",
    });
    expect(requests.map(({ input }) => input)).toEqual([
      "/api/v1/invitations/a%2Fb/accept",
      "/api/v1/invitations/a%2Fb/accept",
    ]);
    expect(requests[1].init?.method).toBe("POST");
  });

  test("keeps move preview anchors identical in the commit body", async () => {
    const requests: RequestInit[] = [];
    const client = new ApiClient(async (_input, init) => {
      requests.push(init ?? {});
      return Response.json(
        requests.length === 1
          ? {
              previewToken: "p".repeat(43),
              permissionChanges: 1,
              policyChanges: 2,
              expiresAt: new Date().toISOString(),
            }
          : {
              id: crypto.randomUUID(),
              title: "Moved",
              parentId: null,
              status: "ACTIVE",
              currentVersionId: null,
              revision: 2,
            },
      );
    });
    const workspace = crypto.randomUUID();
    const document = crypto.randomUUID();
    const after = crypto.randomUUID();
    const csrfToken = "c".repeat(43);
    const preview = await client.previewDocumentMove(
      workspace,
      document,
      1,
      null,
      after,
      csrfToken,
    );
    await client.moveDocument(workspace, document, 1, null, after, preview.previewToken, {
      csrfToken,
      idempotencyKey: "move-command-0001",
    });
    expect(JSON.parse(String(requests[0].body))).toEqual({
      newParentId: null,
      afterDocumentId: after,
    });
    expect(JSON.parse(String(requests[1].body))).toEqual({
      newParentId: null,
      afterDocumentId: after,
      previewToken: preview.previewToken,
    });
  });
});
