import { validateSpdxDocument } from "./lib/spdx.mjs";

if (!process.argv.includes("--self-test"))
  throw new Error("SBOM contract check requires --self-test");
const fixture = {
  SPDXID: "SPDXRef-DOCUMENT",
  spdxVersion: "SPDX-2.2",
  documentNamespace: "https://example.invalid/spdx/test",
  packages: [{ SPDXID: "SPDXRef-Package" }],
};
if (validateSpdxDocument(fixture).version !== "SPDX-2.2")
  throw new Error("SPDX 2.2 capability was not preserved");
for (const invalid of [
  { ...fixture, spdxVersion: "SPDX-2.1" },
  { ...fixture, documentNamespace: "not-a-namespace" },
  { ...fixture, packages: [] },
])
  expectFailure(() => validateSpdxDocument(invalid));
console.log(JSON.stringify({ schemaVersion: 1, supportedVersions: ["SPDX-2.2", "SPDX-2.3"] }));

function expectFailure(callback) {
  try {
    callback();
  } catch {
    return;
  }
  throw new Error("SBOM negative self-test unexpectedly passed");
}
