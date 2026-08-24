import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");

function read(relativePath) {
  return readFileSync(resolve(root, relativePath), "utf8");
}

const toolVersions = new Map(
  read(".tool-versions")
    .trim()
    .split("\n")
    .map((line) => line.trim().split(/\s+/u)),
);
const packageManifest = JSON.parse(read("package.json"));
const rustToolchain = read("rust-toolchain.toml");
const workflow = read(".github/workflows/ci.yml");

const expected = {
  rust: toolVersions.get("rust"),
  bun: toolVersions.get("bun"),
  node: toolVersions.get("nodejs"),
};

const contracts = [
  ["packageManager", packageManifest.packageManager, `bun@${expected.bun}`],
  ["Bun engine", packageManifest.engines?.bun, expected.bun],
  ["Node engine", packageManifest.engines?.node, expected.node],
  ["Rust toolchain", rustToolchain.match(/channel = "([^"]+)"/u)?.[1], expected.rust],
  ["CI Bun", workflow.match(/bun-version: ([^\s]+)/u)?.[1], expected.bun],
  ["CI Node", workflow.match(/node-version: ([^\s]+)/u)?.[1], expected.node],
  ["CI Rust", workflow.match(/toolchain: ([^\s]+)/u)?.[1], expected.rust],
];

for (const [label, actual, wanted] of contracts) {
  if (actual !== wanted) throw new Error(`${label} is ${actual ?? "missing"}; expected ${wanted}`);
}

const runtime = {
  rust: execFileSync("rustc", ["--version"], { encoding: "utf8" }).match(/rustc ([^\s]+)/u)?.[1],
  bun: execFileSync("bun", ["--version"], { encoding: "utf8" }).trim(),
  node: execFileSync("node", ["--version"], { encoding: "utf8" }).trim().replace(/^v/u, ""),
};

for (const [tool, wanted] of Object.entries(expected)) {
  if (runtime[tool] !== wanted) {
    throw new Error(`${tool} runtime is ${runtime[tool] ?? "missing"}; expected ${wanted}`);
  }
}

console.log(`toolchains passed: Rust ${expected.rust}, Bun ${expected.bun}, Node ${expected.node}`);
