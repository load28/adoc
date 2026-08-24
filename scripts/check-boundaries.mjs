import { execFileSync } from "node:child_process";
import { readdirSync, readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");

const rustAllowed = new Map([
  ["configuration", new Set()],
  ["contracts", new Set()],
  ["kernel", new Set()],
  ["domain", new Set(["kernel"])],
  ["ports", new Set(["kernel"])],
  ["application", new Set(["kernel", "domain", "ports"])],
  ["adapters", new Set(["application", "ports"])],
  ["telemetry", new Set(["configuration"])],
  [
    "test-support",
    new Set([
      "configuration",
      "contracts",
      "kernel",
      "domain",
      "ports",
      "application",
      "adapters",
      "telemetry",
    ]),
  ],
  ["app", new Set(["configuration", "contracts", "application", "adapters", "telemetry"])],
  ["tool", new Set(["contracts"])],
]);

const jsAllowed = new Map([
  ["contracts", new Set()],
  ["editor-schema", new Set(["contracts"])],
  ["i18n", new Set()],
  ["ui-domain", new Set(["contracts", "editor-schema", "i18n"])],
  ["web", new Set(["contracts", "editor-schema", "i18n", "ui-domain"])],
]);

function assertAcyclic(nodes, label) {
  const visiting = new Set();
  const visited = new Set();

  function visit(name, path) {
    if (visiting.has(name)) {
      throw new Error(`${label} dependency cycle: ${[...path, name].join(" -> ")}`);
    }
    if (visited.has(name)) return;

    visiting.add(name);
    const node = nodes.get(name);
    for (const dependency of node?.dependencies ?? []) {
      visit(dependency, [...path, name]);
    }
    visiting.delete(name);
    visited.add(name);
  }

  for (const name of nodes.keys()) visit(name, []);
}

function validateLayers(nodes, allowed, label) {
  for (const node of nodes.values()) {
    const allowedTargets = allowed.get(node.layer);
    if (!allowedTargets)
      throw new Error(`${label} package ${node.name} has unknown layer ${node.layer}`);

    for (const dependencyName of node.dependencies) {
      const target = nodes.get(dependencyName);
      if (!target) continue;
      if (!allowedTargets.has(target.layer)) {
        throw new Error(
          `${label} forbidden dependency: ${node.name} (${node.layer}) -> ${target.name} (${target.layer})`,
        );
      }
    }
  }
  assertAcyclic(nodes, label);
}

function loadRustGraph() {
  const metadata = JSON.parse(
    execFileSync("cargo", ["metadata", "--format-version", "1", "--locked"], {
      cwd: root,
      encoding: "utf8",
    }),
  );
  const workspaceIds = new Set(metadata.workspace_members);
  const packages = new Map(
    metadata.packages.filter((pkg) => workspaceIds.has(pkg.id)).map((pkg) => [pkg.id, pkg]),
  );
  const nodes = new Map();

  for (const pkg of packages.values()) {
    const resolved = metadata.resolve.nodes.find((node) => node.id === pkg.id);
    const dependencies = (resolved?.deps ?? [])
      .map((dependency) => packages.get(dependency.pkg)?.name)
      .filter(Boolean);
    nodes.set(pkg.name, {
      name: pkg.name,
      layer: pkg.metadata?.adoc?.layer,
      dependencies,
    });
  }
  return nodes;
}

function expandWorkspaces(workspaces) {
  const paths = [];
  for (const pattern of workspaces) {
    if (!pattern.endsWith("/*")) {
      paths.push(pattern);
      continue;
    }
    const parent = pattern.slice(0, -2);
    for (const entry of readdirSync(join(root, parent), { withFileTypes: true })) {
      if (entry.isDirectory()) paths.push(join(parent, entry.name));
    }
  }
  return paths;
}

function loadJavaScriptGraph() {
  const rootManifest = JSON.parse(readFileSync(join(root, "package.json"), "utf8"));
  const manifests = expandWorkspaces(rootManifest.workspaces).map((path) =>
    JSON.parse(readFileSync(join(root, path, "package.json"), "utf8")),
  );
  const workspaceNames = new Set(manifests.map((manifest) => manifest.name));
  const nodes = new Map();

  for (const manifest of manifests) {
    const dependencyFields = [
      manifest.dependencies,
      manifest.devDependencies,
      manifest.optionalDependencies,
      manifest.peerDependencies,
    ];
    const dependencies = dependencyFields
      .flatMap((field) => Object.keys(field ?? {}))
      .filter((name) => workspaceNames.has(name));
    nodes.set(manifest.name, {
      name: manifest.name,
      layer: manifest.adoc?.layer,
      dependencies: [...new Set(dependencies)],
    });
  }
  return nodes;
}

function expectFailure(action, expectedText) {
  try {
    action();
  } catch (error) {
    if (String(error.message).includes(expectedText)) return;
    throw error;
  }
  throw new Error(`self-test did not reject ${expectedText}`);
}

function runSelfTest() {
  const forbiddenRust = new Map([
    ["domain", { name: "domain", layer: "domain", dependencies: ["adapter"] }],
    ["adapter", { name: "adapter", layer: "adapters", dependencies: [] }],
  ]);
  expectFailure(
    () => validateLayers(forbiddenRust, rustAllowed, "Rust self-test"),
    "forbidden dependency",
  );

  const forbiddenJavaScript = new Map([
    ["contracts", { name: "contracts", layer: "contracts", dependencies: ["web"] }],
    ["web", { name: "web", layer: "web", dependencies: [] }],
  ]);
  expectFailure(
    () => validateLayers(forbiddenJavaScript, jsAllowed, "JavaScript self-test"),
    "forbidden dependency",
  );

  const cyclic = new Map([
    ["a", { name: "a", dependencies: ["b"] }],
    ["b", { name: "b", dependencies: ["a"] }],
  ]);
  expectFailure(() => assertAcyclic(cyclic, "self-test"), "dependency cycle");
  console.log("boundary self-test passed");
}

if (process.argv.includes("--self-test")) {
  runSelfTest();
} else {
  validateLayers(loadRustGraph(), rustAllowed, "Rust");
  validateLayers(loadJavaScriptGraph(), jsAllowed, "JavaScript");
  console.log("workspace boundaries passed");
}
