import { describe, expect, test } from "bun:test";

import {
  ApiClient,
  beginGoogleLoginUrl,
  fileContentUrl,
  publicFileContentUrl,
  workspaceStreamUrl,
} from "../src";

const command = { csrfToken: "csrf", idempotencyKey: "command-key" };

describe("complete Web operation client", () => {
  test("binds preference and Workspace lifecycle commands to exact revisions", async () => {
    const requests: RecordedRequest[] = [];
    const api = recorder(requests);
    await api.updatePreferences(
      { revision: 3 },
      { locale: "ko", timezone: "Asia/Seoul", theme: "SYSTEM" },
      command,
    );
    const workspace = { id: "workspace", revision: 4 };
    await api.updateWorkspace(workspace, { name: "Platform" }, command);
    await api.scheduleWorkspaceDeletion(workspace, "통합", command);
    await api.cancelWorkspaceDeletion(workspace, command);

    expect(requests.map(shape)).toEqual([
      ["PUT", "/api/v1/preferences", '"3"'],
      ["PUT", "/api/v1/workspaces/workspace", '"4"'],
      ["POST", "/api/v1/workspaces/workspace/deletion", '"4"'],
      ["DELETE", "/api/v1/workspaces/workspace/deletion", '"4"'],
    ]);
  });

  test("binds File and public-link lifecycle without exposing a public token in list calls", async () => {
    const requests: RecordedRequest[] = [];
    const api = recorder(requests);
    await api.file("workspace", "asset");
    await api.deleteFile("workspace", "asset", 2, command);
    await api.publicLinks("workspace", "document");
    await api.createPublicLink("workspace", "document", null, command);
    await api.revokePublicLink("workspace", "document", { id: "link", revision: 5 }, command);

    expect(requests.map(shape)).toEqual([
      ["GET", "/api/v1/workspaces/workspace/files/asset", null],
      ["DELETE", "/api/v1/workspaces/workspace/files/asset", '"2"'],
      ["GET", "/api/v1/workspaces/workspace/documents/document/public-links", null],
      ["POST", "/api/v1/workspaces/workspace/documents/document/public-links", null],
      ["DELETE", "/api/v1/workspaces/workspace/documents/document/public-links/link", '"5"'],
    ]);
  });

  test("centralizes navigation, stream and binary operation URLs", () => {
    expect(beginGoogleLoginUrl("/workspaces?a=1")).toBe(
      "/api/v1/auth/google/start?returnTo=%2Fworkspaces%3Fa%3D1",
    );
    expect(() => beginGoogleLoginUrl("//host/path")).toThrow("unsafe return path");
    expect(workspaceStreamUrl("space/id")).toBe("/api/v1/stream?workspaceId=space%2Fid");
    expect(fileContentUrl("space/id", "asset/id")).toBe(
      "/api/v1/workspaces/space%2Fid/files/asset%2Fid/content",
    );
    expect(publicFileContentUrl("token/value", "asset/id")).toBe(
      "/public/v1/documents/token%2Fvalue/files/asset%2Fid",
    );
  });
});

type RecordedRequest = { input: string; init?: RequestInit };

function recorder(requests: RecordedRequest[]): ApiClient {
  return new ApiClient(async (input, init) => {
    requests.push({ input: String(input), init });
    if (
      init?.method === "DELETE" &&
      (String(input).includes("/files/") || String(input).includes("/public-links/"))
    )
      return new Response(null, { status: 204 });
    return Response.json({});
  });
}

function shape(request: RecordedRequest): [string, string, string | null] {
  return [
    request.init?.method ?? "GET",
    request.input,
    new Headers(request.init?.headers).get("if-match"),
  ];
}
