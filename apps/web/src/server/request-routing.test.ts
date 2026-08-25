import { describe, expect, test } from "bun:test";

import { isApiUpstreamPath } from "./request-routing";

describe("web request routing", () => {
  test("proxies authenticated and anonymous API namespaces only", () => {
    expect(isApiUpstreamPath("/api/v1/workspaces")).toBe(true);
    expect(isApiUpstreamPath("/public/v1/documents/token")).toBe(true);
    expect(isApiUpstreamPath("/api/v10/workspaces")).toBe(false);
    expect(isApiUpstreamPath("/public/v10/documents/token")).toBe(false);
    expect(isApiUpstreamPath("/p/token")).toBe(false);
  });
});
