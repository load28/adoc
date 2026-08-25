import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import YAML from "yaml";

const root = resolve(import.meta.dirname, "..");
const httpMethods = new Set(["get", "put", "post", "delete", "options", "head", "patch", "trace"]);
const allowedProfiles = new Set([
  "query",
  "command",
  "lease-command",
  "async-command",
  "navigation",
  "callback",
  "stream",
  "binary",
]);
const profileCases = {
  query: ["success", "auth", "tenant", "permission", "validation", "pagination-or-filter"],
  command: [
    "success",
    "auth",
    "tenant",
    "permission",
    "validation",
    "idempotency-replay",
    "idempotency-conflict",
    "stale",
    "rollback",
    "declared-audit",
    "declared-outbox",
  ],
  "lease-command": ["expired-lease", "other-holder", "token-client-mismatch"],
  "async-command": ["cancel", "deadline", "redelivery", "terminal-immutable"],
  navigation: ["success", "malformed-input", "same-origin-return"],
  callback: ["success", "state-nonce", "replay", "disclosure"],
  stream: ["success", "auth", "tenant", "resume", "slow-consumer"],
  binary: ["success", "auth-or-capability", "range", "disclosure"],
};

function fail(message) {
  throw new Error(`exact contract coverage: ${message}`);
}

function duplicates(values) {
  const seen = new Set();
  return [...new Set(values.filter((value) => (seen.has(value) ? true : !seen.add(value))))];
}

function exactDiff(expected, actual) {
  return {
    missing: [...expected].filter((value) => !actual.has(value)).sort(),
    orphan: [...actual].filter((value) => !expected.has(value)).sort(),
  };
}

function sourceOperationMap(openapi) {
  return new Map(
    Object.values(openapi.paths ?? {}).flatMap((path) =>
      Object.entries(path)
        .filter(([method]) => httpMethods.has(method))
        .map(([method, operation]) => [operation.operationId, method]),
    ),
  );
}

function eventTypes(schema) {
  const enums = [];
  const visit = (value) => {
    if (Array.isArray(value)) return value.forEach(visit);
    if (!value || typeof value !== "object") return;
    if (Array.isArray(value.enum) && value.enum.every((item) => typeof item === "string"))
      enums.push(value.enum);
    Object.values(value).forEach(visit);
  };
  visit(schema);
  return new Set(enums.sort((left, right) => right.length - left.length)[0] ?? []);
}

function catalogEventTypes(source) {
  return new Set(
    source
      .split("\n")
      .filter((line) => /^\| [A-Za-z]/.test(line) && !line.startsWith("| Event |"))
      .map((line) => line.split("|")[1].trim())
      .map((value) =>
        value
          .replace(/([A-Z]+)([A-Z][a-z])/g, "$1_$2")
          .replace(/([a-z0-9])([A-Z])/g, "$1_$2")
          .toUpperCase(),
      ),
  );
}

function stateTransitions(source) {
  const transitions = [];
  for (const line of source.split("\n")) {
    if (!line.startsWith("|")) continue;
    const cells = line.split("|").map((cell) => cell.trim());
    const [aggregate, from, trigger, to] = cells.slice(1, 5);
    if (!aggregate || aggregate === "Aggregate" || aggregate.startsWith("---")) continue;
    const fromStates = from.split("/").map((value) => value.trim());
    const toStates = to.split("/").map((value) => value.trim());
    for (const fromState of fromStates)
      for (const toState of toStates)
        transitions.push({
          aggregate,
          id: [aggregate, fromState, trigger, toState]
            .map((value) => value.normalize("NFC"))
            .join("|"),
        });
  }
  return transitions;
}

function testExists(readSource, file, test) {
  if (typeof file !== "string" || typeof test !== "string" || /[*?]/.test(test)) return false;
  return new RegExp(`\\bfn\\s+${test}\\b`).test(readSource(file));
}

function eventWireName(id) {
  return `${id
    .toLowerCase()
    .split("_")
    .map((part) => part[0].toUpperCase() + part.slice(1))
    .join("")}.v1`;
}

function validate({ manifest, openapi, eventSchema, eventCatalog, stateCatalog, readSource }) {
  if (manifest.schemaVersion !== 1) fail("unsupported schemaVersion");
  const operations = sourceOperationMap(openapi);
  if (operations.size !== 109) fail(`expected 109 OpenAPI operations, received ${operations.size}`);
  const suiteIds = manifest.operationSuites.map((suite) => suite.id);
  if (duplicates(suiteIds).length > 0) fail("operation suite IDs are duplicated");
  const evidenceIds = manifest.operationSuites.flatMap((suite) => suite.operations);
  const evidenceDuplicates = duplicates(evidenceIds);
  if (evidenceDuplicates.length > 0)
    fail(`operation evidence is duplicated: ${evidenceDuplicates.join(", ")}`);
  const operationDiff = exactDiff(new Set(operations.keys()), new Set(evidenceIds));
  if (operationDiff.missing.length > 0 || operationDiff.orphan.length > 0)
    fail(
      `operation diff; missing=[${operationDiff.missing.join(", ")}], orphan=[${operationDiff.orphan.join(", ")}]`,
    );

  let operationCaseCount = 0;
  for (const suite of manifest.operationSuites) {
    if (!testExists(readSource, suite.file, suite.test))
      fail(`${suite.id} evidence test ${suite.test} is absent from ${suite.file}`);
    for (const operation of suite.operations) {
      const method = operations.get(operation);
      const profile =
        manifest.profileOverrides[operation] ?? (method === "get" ? "query" : "command");
      if (!allowedProfiles.has(profile)) fail(`${operation} has invalid profile ${profile}`);
      operationCaseCount += profileCases[profile].length;
      if (profile === "lease-command") operationCaseCount += profileCases.command.length;
      if (profile === "async-command") operationCaseCount += profileCases.command.length;
    }
  }
  for (const operation of Object.keys(manifest.profileOverrides))
    if (!operations.has(operation)) fail(`profile override is orphaned: ${operation}`);

  const schemaEvents = eventTypes(eventSchema);
  if (schemaEvents.size !== 23) fail(`expected 23 schema events, received ${schemaEvents.size}`);
  const catalogEvents = catalogEventTypes(eventCatalog);
  const eventRows = manifest.eventEvidence.map((row) => row.id);
  const eventDuplicates = duplicates(eventRows);
  if (eventDuplicates.length > 0)
    fail(`event evidence is duplicated: ${eventDuplicates.join(", ")}`);
  for (const [label, values] of [
    ["catalog", catalogEvents],
    ["evidence", new Set(eventRows)],
  ]) {
    const diff = exactDiff(schemaEvents, values);
    if (diff.missing.length > 0 || diff.orphan.length > 0)
      fail(
        `${label} event diff; missing=[${diff.missing.join(", ")}], orphan=[${diff.orphan.join(", ")}]`,
      );
  }
  for (const row of manifest.eventEvidence) {
    for (const [kind, evidence] of [
      ["producer", row.producer],
      ["consumer", row.consumer],
    ]) {
      if (!Array.isArray(evidence) || evidence.length !== 3)
        fail(`${row.id} ${kind} evidence must contain source, test file and test ID`);
      if (!testExists(readSource, evidence[1], evidence[2]))
        fail(`${row.id} ${kind} test ${evidence[2]} is missing`);
      const source = readSource(evidence[0]);
      const canonicalSource = source.toUpperCase().replace(/[^A-Z0-9]/g, "");
      const canonicalEvent = row.id.replaceAll("_", "");
      if (
        !source.includes(eventWireName(row.id)) &&
        !source.includes(`"${row.id}"`) &&
        !canonicalSource.includes(canonicalEvent)
      )
        fail(`${row.id} ${kind} source does not own the event ID`);
    }
  }

  const transitions = stateTransitions(stateCatalog);
  const transitionIds = transitions.map((transition) => transition.id);
  const transitionDuplicates = duplicates(transitionIds);
  if (transitionDuplicates.length > 0)
    fail(`state transition IDs are duplicated: ${transitionDuplicates.join(", ")}`);
  for (const transition of transitions) {
    const evidence = manifest.stateSuites[transition.aggregate];
    if (!evidence || !testExists(readSource, evidence[0], evidence[1]))
      fail(`${transition.id} has no exact state evidence`);
  }
  for (const aggregate of Object.keys(manifest.stateSuites))
    if (!transitions.some((transition) => transition.aggregate === aggregate))
      fail(`state evidence aggregate is orphaned: ${aggregate}`);

  return {
    operations: operations.size,
    operationCases: operationCaseCount,
    events: schemaEvents.size,
    transitions: transitions.length,
  };
}

const manifestPath = resolve(root, "docs/design/quality/exact-contract-coverage.json");
const manifest = JSON.parse(readFileSync(manifestPath, "utf8"));
const openapi = YAML.parse(readFileSync(resolve(root, "docs/design/api/openapi.yaml"), "utf8"));
const eventSchema = JSON.parse(
  readFileSync(resolve(root, "docs/design/contracts/event-payloads.schema.json"), "utf8"),
);
const eventCatalog = readFileSync(resolve(root, "docs/design/api/EVENT-CATALOG.md"), "utf8");
const stateCatalog = readFileSync(
  resolve(root, "docs/design/specs/STATE-TRANSITION-CATALOG.md"),
  "utf8",
);
const readSource = (file) => readFileSync(resolve(root, file), "utf8");
const result = validate({ manifest, openapi, eventSchema, eventCatalog, stateCatalog, readSource });

if (process.argv.includes("--self-test")) {
  const cases = [
    (value) => value.operationSuites[0].operations.pop(),
    (value) => value.operationSuites[1].operations.push(value.operationSuites[0].operations[0]),
    (value) => {
      value.profileOverrides.getSession = "wildcard";
    },
    (value) => value.eventEvidence.pop(),
    (value) => {
      value.eventEvidence[0].producer[0] = "crates/kernel/src/lib.rs";
    },
    (value) => {
      delete value.stateSuites.Workspace;
    },
  ];
  for (const mutate of cases) {
    const invalid = structuredClone(manifest);
    mutate(invalid);
    let rejected = false;
    try {
      validate({ manifest: invalid, openapi, eventSchema, eventCatalog, stateCatalog, readSource });
    } catch {
      rejected = true;
    }
    if (!rejected) fail("negative self-test accepted invalid coverage");
  }
}

console.log(
  `exact contract coverage passed: ${result.operations} operations/${result.operationCases} cases, ${result.events} events, ${result.transitions} state transitions`,
);
