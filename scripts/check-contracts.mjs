import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import YAML from "yaml";

const root = resolve(import.meta.dirname, "..");
const generated = resolve(root, "packages/contracts/src/generated");
const bundle = JSON.parse(readFileSync(resolve(generated, "contract-bundle.schema.json"), "utf8"));
const manifest = JSON.parse(readFileSync(resolve(generated, "manifest.json"), "utf8"));
const openapi = YAML.parse(readFileSync(resolve(root, "docs/design/api/openapi.yaml"), "utf8"));

const httpMethods = new Set(["get", "put", "post", "delete", "options", "head", "patch", "trace"]);
const operationIds = Object.values(openapi.paths ?? {}).flatMap((path) =>
  Object.entries(path)
    .filter(([method]) => httpMethods.has(method))
    .map(([, operation]) => operation.operationId),
);
if (
  operationIds.some((id) => typeof id !== "string") ||
  new Set(operationIds).size !== operationIds.length
)
  throw new Error("OpenAPI operation IDs are missing or duplicated");
if (manifest.counts.openapiOperations !== operationIds.length)
  throw new Error(
    `generated OpenAPI operation count differs: source ${operationIds.length}, manifest ${manifest.counts.openapiOperations}`,
  );
if (manifest.counts.asyncapiOperations !== 3 || manifest.counts.asyncapiMessages !== 2)
  throw new Error("AsyncAPI operation or message coverage drifted");

function inspectReferences(value) {
  if (Array.isArray(value)) return value.forEach(inspectReferences);
  if (!value || typeof value !== "object") return;
  if (typeof value.$ref === "string" && !value.$ref.startsWith("#/$defs/"))
    throw new Error(`generated bundle contains non-local reference: ${value.$ref}`);
  Object.values(value).forEach(inspectReferences);
}
inspectReferences(bundle);

const rust = JSON.parse(
  execFileSync("cargo", ["run", "--quiet", "-p", "adoc-contracts", "--bin", "fixture_verdicts"], {
    cwd: root,
    encoding: "utf8",
  }),
);
const typescript = JSON.parse(
  execFileSync("bun", ["run", "packages/contracts/scripts/fixture-verdicts.ts"], {
    cwd: root,
    encoding: "utf8",
  }),
);
const ordered = (value) =>
  JSON.stringify(
    Object.fromEntries(Object.entries(value).sort(([left], [right]) => left.localeCompare(right))),
  );
if (ordered(rust) !== ordered(typescript))
  throw new Error(
    `Rust and TypeScript contract verdicts differ\nRust: ${ordered(rust)}\nTypeScript: ${ordered(typescript)}`,
  );

console.log(
  `contract coverage passed: ${manifest.counts.openapiOperations} HTTP operations, ${manifest.counts.asyncapiMessages} event messages, 8 bilingual fixtures`,
);
