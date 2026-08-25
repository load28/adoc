import { describe, expect, test } from "bun:test";

import { selectVersionIds } from "./version-history-panel";

describe("version comparison selection", () => {
  test("keeps an ordered exact pair and replaces the oldest selection", () => {
    expect(selectVersionIds([], "v1")).toEqual(["v1"]);
    expect(selectVersionIds(["v1"], "v2")).toEqual(["v1", "v2"]);
    expect(selectVersionIds(["v1", "v2"], "v3")).toEqual(["v2", "v3"]);
    expect(selectVersionIds(["v2", "v3"], "v2")).toEqual(["v3"]);
  });
});
