import { describe, expect, test } from "bun:test";
import { hardenResponse } from "./response-policy";

describe("web response hardening", () => {
  test("applies one closed security policy without overwriting explicit asset caching", () => {
    const response = hardenResponse(
      new Response("x", {
        headers: { "Cache-Control": "public, immutable", "Set-Cookie": "session=opaque" },
      }),
      false,
    );
    expect(response.headers.get("cache-control")).toBe("public, immutable");
    expect(response.headers.get("set-cookie")).toBe("session=opaque");
    expect(response.headers.get("content-security-policy")).toContain("frame-ancestors 'none'");
    expect(response.headers.get("strict-transport-security")).toBeNull();
  });

  test("defaults dynamic responses to no-store and adds HSTS only in production", () => {
    const response = hardenResponse(Response.json({ ok: true }), true);
    expect(response.headers.get("cache-control")).toBe("no-store");
    expect(response.headers.get("strict-transport-security")).toContain("max-age=31536000");
    expect(response.headers.get("x-content-type-options")).toBe("nosniff");
  });
});
