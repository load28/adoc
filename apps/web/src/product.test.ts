import { describe, expect, test } from "bun:test";

import { PRODUCT_NAME } from "./product";

describe("web bootstrap", () => {
  test("keeps the canonical product name", () => {
    expect(PRODUCT_NAME).toBe("adoc");
  });
});
