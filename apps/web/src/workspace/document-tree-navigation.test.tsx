import { describe, expect, test } from "bun:test";

import { allowedTreeActions } from "./document-tree-navigation";

describe("permission-scoped document tree", () => {
  test("derives mutation visibility only from server-provided effective access", () => {
    expect(allowedTreeActions("NO_ACCESS")).toEqual([]);
    expect(allowedTreeActions("VIEWER")).toEqual([]);
    expect(allowedTreeActions("CONTRIBUTOR")).toEqual(["create"]);
    expect(allowedTreeActions("EDITOR")).toEqual(["create", "rename", "move", "trash"]);
  });
});
