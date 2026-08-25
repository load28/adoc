import { describe, expect, test } from "bun:test";

import {
  ApiClient,
  ApiProblemError,
  beginCommand,
  canonicalReturnTo,
  parseDocumentSearch,
  parseSettingsSearch,
} from "../src";

describe("frontend shell contracts", () => {
  test("canonicalizes document search and removes unknown values", () => {
    expect(
      parseDocumentSearch({ mode: "unsafe", panel: "history", from: " v1 ", token: "secret" }),
    ).toEqual({ mode: "published", panel: "history", from: "v1" });
  });

  test("keeps one idempotency key across command phases", () => {
    expect(beginCommand({ title: "A" }, "018f4f0c-8f4d-7cc8-9ca6-8e8b4c8a3451")).toMatchObject({
      phase: "VALIDATING",
      idempotencyKey: "018f4f0c-8f4d-7cc8-9ca6-8e8b4c8a3451",
    });
  });

  test("accepts only same-origin relative login return paths", () => {
    expect(canonicalReturnTo("/invites/token?from=email")).toBe("/invites/token?from=email");
    expect(canonicalReturnTo("https://attacker.test/invites/token")).toBe("/workspaces");
    expect(canonicalReturnTo("//attacker.test/invites/token")).toBe("/workspaces");
    expect(canonicalReturnTo("/\\attacker.test/invites/token")).toBe("/workspaces");
  });

  test("uses same-origin credentials and normalizes Problem responses", async () => {
    let request: RequestInit | undefined;
    const client = new ApiClient(async (_input, init) => {
      request = init;
      return new Response(JSON.stringify({ code: "SESSION_REQUIRED", message: "Sign in" }), {
        status: 401,
        headers: { "content-type": "application/problem+json" },
      });
    });
    const error = await client.session().catch((cause: unknown) => cause);
    expect(request).toMatchObject({ credentials: "same-origin", redirect: "manual" });
    expect(error).toBeInstanceOf(ApiProblemError);
    expect((error as ApiProblemError).problem.code).toBe("SESSION_REQUIRED");
  });

  test("bounds settings selection without accepting capability-like values", () => {
    expect(parseSettingsSearch({ document: " doc ", subject: "user", token: "secret" })).toEqual({
      document: "doc",
      subject: "user",
    });
  });

  test("public viewer omits credentials and makes every failure indistinguishable", async () => {
    let request: RequestInit | undefined;
    const client = new ApiClient(async (_input, init) => {
      request = init;
      return Response.json({ code: "PUBLIC_LINK_REVOKED" }, { status: 410 });
    });
    const token = "A".repeat(43);
    const error = await client.publicDocument(token).catch((cause: unknown) => cause);
    expect(request).toMatchObject({ credentials: "omit", redirect: "manual" });
    expect((error as ApiProblemError).problem.code).toBe("PUBLIC_DOCUMENT_NOT_FOUND");
  });
});
