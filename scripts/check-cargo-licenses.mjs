import { execFileSync } from "node:child_process";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const allowed = new Set([
  "0BSD",
  "Apache-2.0",
  "Apache-2.0 WITH LLVM-exception",
  "BSD-2-Clause",
  "BSD-3-Clause",
  "CC0-1.0",
  "ISC",
  "MIT",
  "MIT-0",
  "MPL-2.0",
  "Unicode-3.0",
  "Unlicense",
]);

const metadata = JSON.parse(
  execFileSync("cargo", ["metadata", "--format-version", "1", "--locked"], {
    cwd: root,
    encoding: "utf8",
  }),
);

const rejected = metadata.packages.filter((pkg) => {
  if (!pkg.license) return true;
  const choices = pkg.license.split(/\s+OR\s+/u).map((value) => value.replace(/[()]/gu, "").trim());
  return !choices.some((choice) => allowed.has(choice));
});

if (rejected.length > 0) {
  const detail = rejected.map((pkg) => `${pkg.name}: ${pkg.license ?? "missing"}`).join("\n");
  throw new Error(`Cargo license policy rejected:\n${detail}`);
}

console.log(`Cargo license policy passed for ${metadata.packages.length} packages`);
