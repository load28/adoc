import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import YAML from "yaml";

const root = resolve(import.meta.dirname, "..");
const coveragePath = resolve(root, "docs/design/quality/web-contract-coverage.json");
const openapiPath = resolve(root, "docs/design/api/openapi.yaml");
const clientPath = resolve(root, "packages/ui-domain/src/api-client.ts");
const uxPath = resolve(root, "docs/design/ux/SCREEN-BEHAVIOR-SPECS.md");
const allowedSurfaces = new Set([
  "browser-client",
  "browser-navigation",
  "server-callback",
  "stream",
  "binary",
]);
const httpMethods = new Set(["get", "put", "post", "delete", "options", "head", "patch", "trace"]);

function fail(message) {
  throw new Error(`web contract coverage: ${message}`);
}

function duplicates(values) {
  const seen = new Set();
  return [...new Set(values.filter((value) => (seen.has(value) ? true : !seen.add(value))))];
}

function exactDifference(expected, actual) {
  return {
    missing: [...expected].filter((value) => !actual.has(value)).sort(),
    orphan: [...actual].filter((value) => !expected.has(value)).sort(),
  };
}

function calls(source, owner) {
  return new RegExp(`(?:\\.|\\b)${owner}\\s*\\(`).test(source);
}

function validate({ coverage, openapi, clientSource, uxSource, readModule }) {
  if (coverage.schemaVersion !== 1) fail("unsupported schemaVersion");
  const operationIds = Object.values(openapi.paths ?? {}).flatMap((path) =>
    Object.entries(path)
      .filter(([method]) => httpMethods.has(method))
      .map(([, operation]) => operation.operationId),
  );
  if (operationIds.some((id) => typeof id !== "string")) fail("OpenAPI operationId is missing");
  const openapiDuplicates = duplicates(operationIds);
  if (openapiDuplicates.length > 0)
    fail(`duplicate OpenAPI operations: ${openapiDuplicates.join(", ")}`);

  const rows = coverage.operations ?? [];
  const rowIds = rows.map((row) => row.id);
  const rowDuplicates = duplicates(rowIds);
  if (rowDuplicates.length > 0) fail(`duplicate manifest operations: ${rowDuplicates.join(", ")}`);
  const operationDiff = exactDifference(new Set(operationIds), new Set(rowIds));
  if (operationDiff.missing.length > 0 || operationDiff.orphan.length > 0)
    fail(
      `operation diff is not empty; missing=[${operationDiff.missing.join(", ")}], orphan=[${operationDiff.orphan.join(", ")}]`,
    );

  const classBody = clientSource.slice(
    clientSource.indexOf("export class ApiClient"),
    clientSource.indexOf("export type CommandHeaders"),
  );
  const clientMethods = new Set(
    [...classBody.matchAll(/^ {2}(?:async )?([a-z][A-Za-z0-9]*)\(/gm)]
      .map((match) => match[1])
      .filter((name) => name !== "constructor"),
  );
  const clientOwners = new Set(
    rows.filter((row) => row.surface === "browser-client").map((row) => row.owner),
  );
  const clientDiff = exactDifference(clientMethods, clientOwners);
  if (clientDiff.missing.length > 0 || clientDiff.orphan.length > 0)
    fail(
      `ApiClient diff is not empty; missing manifest owners=[${clientDiff.missing.join(", ")}], nonexistent owners=[${clientDiff.orphan.join(", ")}]`,
    );

  for (const row of rows) {
    if (!allowedSurfaces.has(row.surface)) fail(`${row.id} has unknown surface ${row.surface}`);
    if (typeof row.owner !== "string" || row.owner.length === 0) fail(`${row.id} has no owner`);
    if (row.surface === "browser-client") continue;
    const source = row.module ? readModule(row.module) : clientSource;
    if (!calls(source, row.owner))
      fail(`${row.id} owner ${row.owner} is absent from ${row.module ?? "api-client.ts"}`);
  }

  const screens = coverage.screens ?? [];
  const expectedScreenIds = Array.from(
    { length: 22 },
    (_, index) => `SCR-${String(index + 1).padStart(2, "0")}`,
  );
  const screenIds = screens.map((screen) => screen.id);
  const screenDiff = exactDifference(new Set(expectedScreenIds), new Set(screenIds));
  if (
    duplicates(screenIds).length > 0 ||
    screenDiff.missing.length > 0 ||
    screenDiff.orphan.length > 0
  )
    fail("screen IDs must be the exact SCR-01..22 registry");

  const operationSet = new Set(operationIds);
  const coveredOperations = new Set();
  const bindingMissing = [];
  for (const screen of screens) {
    const loader = screen.loaderOperations ?? [];
    const actions = screen.actionOperations ?? [];
    const local = screen.localActions ?? [];
    const duplicateHttp = duplicates([...loader, ...actions]);
    if (duplicateHttp.length > 0)
      fail(`${screen.id} duplicates HTTP operations: ${duplicateHttp.join(", ")}`);
    for (const operation of [...loader, ...actions]) {
      if (!operationSet.has(operation))
        fail(`${screen.id} references unknown operation ${operation}`);
      coveredOperations.add(operation);
    }
    for (const action of local) {
      if (operationSet.has(action)) fail(`${screen.id} local action shadows operation ${action}`);
    }
    const moduleSource = readModule(screen.module);
    if (!new RegExp(`export\\s+(?:const|function|class)\\s+${screen.symbol}\\b`).test(moduleSource))
      fail(`${screen.id} runtime symbol ${screen.symbol} is absent from ${screen.module}`);
    const implementationModules = coverage.implementationModules?.[screen.id];
    if (!Array.isArray(implementationModules) || implementationModules.length === 0)
      fail(`${screen.id} has no implementation modules`);
    const implementationSource = implementationModules.map(readModule).join("\n");
    for (const operation of [...loader, ...actions]) {
      const row = rows.find((candidate) => candidate.id === operation);
      if (row.surface === "server-callback") continue;
      if (!calls(implementationSource, row.owner))
        bindingMissing.push(`${screen.id}:${operation}->${row.owner}`);
    }
  }
  if (bindingMissing.length > 0) fail(`runtime bindings are missing: ${bindingMissing.join(", ")}`);

  const nonScreenOperations = rows
    .filter((row) => !coveredOperations.has(row.id))
    .map((row) => row.id)
    .sort();
  if (nonScreenOperations.join(",") !== "completeGoogleLogin")
    fail(`unexpected non-screen operations: ${nonScreenOperations.join(", ")}`);

  const uxRows = uxSource
    .split("\n")
    .filter((line) => /^\| SCR-\d{2} \|/.test(line))
    .map((line) => line.split("|").map((cell) => cell.trim()));
  if (uxRows.length !== 22) fail("UX-13 must contain exactly 22 screen rows");
  for (const cells of uxRows) {
    const id = cells[1];
    const screen = screens.find((candidate) => candidate.id === id);
    const documentedOperations = [
      ...cells[3].matchAll(/`([A-Za-z][A-Za-z0-9]+)`/g),
      ...cells[4].matchAll(/`([A-Za-z][A-Za-z0-9]+)`/g),
    ]
      .map((match) => match[1])
      .filter((value) => operationSet.has(value));
    const declared = new Set([...screen.loaderOperations, ...screen.actionOperations]);
    const missing = documentedOperations.filter((operation) => !declared.has(operation));
    if (missing.length > 0) fail(`${id} omits UX-13 operations: ${missing.join(", ")}`);
  }

  return {
    operations: operationIds.length,
    clientMethods: clientMethods.size,
    screens: screens.length,
  };
}

const coverage = JSON.parse(readFileSync(coveragePath, "utf8"));
const openapi = YAML.parse(readFileSync(openapiPath, "utf8"));
const clientSource = readFileSync(clientPath, "utf8");
const uxSource = readFileSync(uxPath, "utf8");
const readModule = (module) => readFileSync(resolve(root, module), "utf8");
const result = validate({ coverage, openapi, clientSource, uxSource, readModule });

if (process.argv.includes("--self-test")) {
  const cases = [
    (value) => value.operations.pop(),
    (value) =>
      value.operations.push({ id: "orphanOperation", surface: "browser-client", owner: "orphan" }),
    (value) => value.operations.push({ ...value.operations[0] }),
    (value) => {
      value.operations.find((row) => row.surface === "browser-client").owner = "missingMethod";
    },
  ];
  for (const mutate of cases) {
    const invalid = structuredClone(coverage);
    mutate(invalid);
    let rejected = false;
    try {
      validate({ coverage: invalid, openapi, clientSource, uxSource, readModule });
    } catch {
      rejected = true;
    }
    if (!rejected) fail("negative self-test accepted invalid coverage");
  }
}

console.log(
  `web contract coverage passed: ${result.operations} operations, ${result.clientMethods} client methods, ${result.screens} screens`,
);
