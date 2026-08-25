import { describe, expect, test } from "bun:test";

import { resolveClientAsset } from "./static-assets";

describe("web static asset boundary", () => {
  test("maps only immutable client assets with explicit content types", () => {
    expect(resolveClientAsset("/assets/theme-bootstrap-Ab12.js")).toEqual({
      relativePath: "client/assets/theme-bootstrap-Ab12.js",
      contentType: "text/javascript; charset=utf-8",
    });
    expect(resolveClientAsset("/assets/app.css")?.contentType).toBe("text/css; charset=utf-8");
  });

  test("rejects traversal, malformed encoding and non-asset paths", () => {
    expect(resolveClientAsset("/assets/../server/server.js")).toBeUndefined();
    expect(resolveClientAsset("/assets/%2e%2e/server/server.js")).toBeUndefined();
    expect(resolveClientAsset("/assets/%ZZ.js")).toBeUndefined();
    expect(resolveClientAsset("/login")).toBeUndefined();
  });
});
