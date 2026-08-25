import { readFileSync } from "node:fs";

const manifestPath = "docs/design/quality/browser-acceptance-manifest.json";
const manifest = JSON.parse(readFileSync(manifestPath, "utf8"));
const feature = readFileSync(manifest.featureDocument, "utf8");
const suite = readFileSync(manifest.suite, "utf8");
const qualitySuite = readFileSync("tests/browser/browser-quality.spec.ts", "utf8");
const featureTitles = [...feature.matchAll(/^\s*시나리오:\s*(.+)$/gm)].map((match) =>
  match[1].trim(),
);
const ids = manifest.scenarios.map((scenario) => scenario.id);
const titles = manifest.scenarios.map((scenario) => scenario.title);

assertExactSequence(
  ids,
  ids.map((_, index) => `ACC-${String(index + 1).padStart(2, "0")}`),
  "scenario IDs",
);
assertExactSet(titles, featureTitles, "feature titles");
if (!suite.includes("manifest.scenarios"))
  throw new Error("browser suite must generate tests from the exact manifest");
for (const marker of [
  "Chromium",
  "Firefox",
  "WebKit",
  "browser.newContext",
  "fixture.publicToken",
]) {
  if (!suite.includes(marker)) throw new Error(`browser suite is missing ${marker} evidence`);
}
for (const marker of ["assertAxe", "toHaveScreenshot", "keyboard.press"]) {
  if (!qualitySuite.includes(marker))
    throw new Error(`browser quality suite is missing ${marker} evidence`);
}

if (process.argv.includes("--self-test")) {
  expectFailure(() => assertExactSet(titles.slice(1), featureTitles, "missing"));
  expectFailure(() => assertExactSet([...titles, titles[0]], featureTitles, "duplicate"));
  expectFailure(() => assertExactSequence(ids.slice().reverse(), ids, "order"));
}

console.log(`browser acceptance contract passed: ${ids.length} exact scenarios`);

function assertExactSequence(actual, expected, label) {
  if (JSON.stringify(actual) !== JSON.stringify(expected)) {
    throw new Error(`${label} differ: ${JSON.stringify({ actual, expected })}`);
  }
  if (new Set(actual).size !== actual.length) throw new Error(`${label} contain duplicates`);
}

function assertExactSet(actual, expected, label) {
  const duplicates = actual.filter((value, index) => actual.indexOf(value) !== index);
  const missing = expected.filter((value) => !actual.includes(value));
  const extra = actual.filter((value) => !expected.includes(value));
  if (duplicates.length || missing.length || extra.length) {
    throw new Error(`${label} differ: ${JSON.stringify({ duplicates, missing, extra })}`);
  }
}

function expectFailure(callback) {
  try {
    callback();
  } catch {
    return;
  }
  throw new Error("negative self-test unexpectedly passed");
}
