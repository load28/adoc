import assert from "node:assert/strict";
import test from "node:test";

import { isAllowedLicense } from "./check-cargo-licenses.mjs";

const allowed = new Set(["Apache-2.0", "ISC", "MIT", "Unicode-3.0"]);

test("SPDX evaluator requires every AND branch", () => {
  assert.equal(isAllowedLicense("Apache-2.0 AND ISC", allowed), true);
  assert.equal(isAllowedLicense("Apache-2.0 AND GPL-3.0", allowed), false);
});

test("SPDX evaluator respects grouped OR and AND precedence", () => {
  assert.equal(isAllowedLicense("MIT OR GPL-3.0", allowed), true);
  assert.equal(isAllowedLicense("(MIT OR Apache-2.0) AND Unicode-3.0", allowed), true);
  assert.equal(isAllowedLicense("(MIT OR Apache-2.0) AND GPL-3.0", allowed), false);
});
