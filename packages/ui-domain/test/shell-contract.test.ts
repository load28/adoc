import { describe, expect, test } from "bun:test";

import { ApiClient, ApiProblemError, beginCommand, parseDocumentSearch } from "../src";

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

  test("uses same-origin credentials and normalizes Problem responses", async () => {
    let request: RequestInit | undefined;
    const client = new ApiClient(async (_input, init) => {
      request = init;
      return Response.json({ code: "SESSION_REQUIRED", message: "Sign in" }, { status: 401 });
    });
    const error = await client.session().catch((cause: unknown) => cause);
    expect(request).toMatchObject({ credentials: "same-origin", redirect: "manual" });
    expect(error).toBeInstanceOf(ApiProblemError);
    expect((error as ApiProblemError).problem.code).toBe("SESSION_REQUIRED");
  });
});
