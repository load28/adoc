import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";

const registry = JSON.parse(readFileSync("infra/security/advisory-exceptions.json", "utf8"));
const ids = registry.exceptions.map(({ id }) => id);
if (new Set(ids).size !== ids.length || ids.some((id) => !/^RUSTSEC-\d{4}-\d{4}$/u.test(id)))
  throw new Error("advisory exception registry contains an invalid or duplicate ID");

const args = ["audit", ...ids.flatMap((id) => ["--ignore", id])];
if (process.argv.includes("--self-test")) {
  console.log(JSON.stringify({ schemaVersion: 1, command: "cargo", args, exceptions: ids.length }));
  process.exit(0);
}

const result = spawnSync("cargo", args, { stdio: "inherit" });
if (result.error) throw result.error;
if (result.status !== 0) process.exit(result.status ?? 1);
