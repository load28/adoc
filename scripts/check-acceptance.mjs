import { readFile } from "node:fs/promises";
import { resolve } from "node:path";

const root = resolve(new URL("..", import.meta.url).pathname);
const read = (path) => readFile(resolve(root, path), "utf8");

function featureScenarios(feature) {
  return feature
    .split(/\r?\n/u)
    .map((line) => line.match(/^\s*시나리오:\s*(.+?)\s*$/u)?.[1])
    .filter(Boolean);
}

async function validate(manifest, feature, load = read) {
  if (manifest.schemaVersion !== 1) throw new Error("unsupported acceptance manifest version");
  const scenarios = featureScenarios(feature);
  if (scenarios.length === 0) throw new Error("acceptance feature has no scenarios");
  const ids = manifest.scenarios?.map(({ id }) => id) ?? [];
  const titles = manifest.scenarios?.map(({ title }) => title) ?? [];
  if (new Set(ids).size !== ids.length) throw new Error("duplicate acceptance id");
  if (new Set(titles).size !== titles.length) throw new Error("duplicate acceptance title");
  if (JSON.stringify(titles) !== JSON.stringify(scenarios))
    throw new Error("feature and manifest scenarios are not an ordered 1:1 mapping");
  const workstreams = manifest.workstreams?.map(({ id }) => id) ?? [];
  const expectedWorkstreams = Array.from(
    { length: 9 },
    (_, index) => `W-${String(index + 1).padStart(2, "0")}`,
  );
  if (JSON.stringify(workstreams) !== JSON.stringify(expectedWorkstreams))
    throw new Error("W-01~09 gate mapping is incomplete");
  const compose = await load("scripts/check-compose-integration.sh");
  if (!compose.includes("--ignored"))
    throw new Error("compose acceptance does not run ignored suites");
  for (const scenario of manifest.scenarios) {
    if (!Array.isArray(scenario.evidence) || scenario.evidence.length === 0)
      throw new Error(`${scenario.id} has no executable evidence`);
    for (const evidence of scenario.evidence) {
      if (!new Set(["root", "compose"]).has(evidence.execution))
        throw new Error(`${scenario.id} has invalid execution boundary`);
      const source = await load(evidence.path);
      const marker = `fn ${evidence.test}`;
      const markerIndex = source.indexOf(marker);
      if (markerIndex < 0) throw new Error(`${scenario.id} evidence test does not exist`);
      const nextTest = source.indexOf("\n#[", markerIndex + marker.length);
      const body = source.slice(markerIndex, nextTest < 0 ? undefined : nextTest);
      if (!/\bassert(?:_eq|_ne)?!/u.test(body))
        throw new Error(`${scenario.id} evidence has no assertion`);
      const attributes = source.slice(Math.max(0, markerIndex - 300), markerIndex);
      if (evidence.execution === "compose") {
        if (!attributes.includes("#[ignore"))
          throw new Error(`${scenario.id} compose evidence is not dependency-isolated`);
        const suite = evidence.path.split("/").at(-1).replace(/\.rs$/u, "");
        if (!compose.includes(`--test ${suite}`) && !compose.includes(`--test ${suite} \\`))
          throw new Error(`${scenario.id} suite is absent from compose acceptance`);
      } else if (attributes.includes("#[ignore")) {
        throw new Error(`${scenario.id} root evidence is skipped by the root gate`);
      }
    }
  }
  return { schemaVersion: 1, scenarios: scenarios.length, workstreams: workstreams.length };
}

const manifest = JSON.parse(await read("docs/design/quality/acceptance-manifest.json"));
const feature = await read(manifest.featureDocument);
if (process.argv.includes("--self-test")) {
  const invalid = [
    { ...manifest, scenarios: manifest.scenarios.slice(1) },
    { ...manifest, scenarios: [...manifest.scenarios, manifest.scenarios[0]] },
    {
      ...manifest,
      scenarios: manifest.scenarios.map((scenario, index) =>
        index === 0
          ? { ...scenario, evidence: [{ ...scenario.evidence[0], test: "missing_test" }] }
          : scenario,
      ),
    },
  ];
  for (const candidate of invalid) {
    let rejected = false;
    try {
      await validate(candidate, feature);
    } catch {
      rejected = true;
    }
    if (!rejected) throw new Error("acceptance negative self-test was not rejected");
  }
}
console.log(JSON.stringify(await validate(manifest, feature)));
