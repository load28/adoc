import assert from "node:assert/strict";
import test from "node:test";

import { generateBaseline } from "./generate-migrations.mjs";

test("baseline transform removes only the outer transaction", () => {
  const generated = generateBaseline("-- canonical\n\nBEGIN;\nSELECT 'BEGIN;';\n\nCOMMIT;\n");
  assert.match(generated, /SQLx owns the transaction/);
  assert.match(generated, /SELECT 'BEGIN;';/);
  assert.doesNotMatch(generated, /^BEGIN;/m);
  assert.doesNotMatch(generated, /^COMMIT;/m);
});

test("baseline transform rejects non-canonical boundaries", () => {
  assert.throws(() => generateBaseline("SELECT 1;\n"), /one outer BEGIN/);
  assert.throws(() => generateBaseline("BEGIN;\r\nCOMMIT;\r\n"), /LF line endings/);
  assert.throws(() => generateBaseline("BEGIN;\nCOMMIT;"), /end with a newline/);
});
