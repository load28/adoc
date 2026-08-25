import { describe, expect, test } from "bun:test";

import { safeLink } from "./editor-structural-commands";

describe("editor structural commands", () => {
  test("admits only explicit safe link schemes and same-origin paths", () => {
    expect(safeLink("/w/platform/docs/one")).toBe("/w/platform/docs/one");
    expect(safeLink("https://example.test/source")).toBe("https://example.test/source");
    expect(safeLink("javascript:alert(1)")).toBeUndefined();
    expect(safeLink("//attacker.test/path")).toBeUndefined();
    expect(safeLink("https:\\attacker.test")).toBeUndefined();
  });
});
