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
  "Zlib",
]);

export function isAllowedLicense(expression, allowedLicenses = allowed) {
  const tokens = expression.replaceAll("/", " OR ").match(/\(|\)|[^\s()]+/gu) ?? [];
  let index = 0;

  function parsePrimary() {
    if (tokens[index] === "(") {
      index += 1;
      const value = parseOr();
      if (tokens[index] !== ")") throw new Error("unclosed SPDX expression group");
      index += 1;
      return value;
    }
    const license = tokens[index];
    if (!license || ["AND", "OR", "WITH", ")"].includes(license)) {
      throw new Error("invalid SPDX expression");
    }
    index += 1;
    if (tokens[index] === "WITH") {
      const exception = tokens[index + 1];
      if (!exception) throw new Error("missing SPDX exception");
      index += 2;
      return allowedLicenses.has(`${license} WITH ${exception}`);
    }
    return allowedLicenses.has(license);
  }

  function parseAnd() {
    let value = parsePrimary();
    while (tokens[index] === "AND") {
      index += 1;
      const right = parsePrimary();
      value = value && right;
    }
    return value;
  }

  function parseOr() {
    let value = parseAnd();
    while (tokens[index] === "OR") {
      index += 1;
      const right = parseAnd();
      value = value || right;
    }
    return value;
  }

  try {
    const accepted = parseOr();
    return index === tokens.length && accepted;
  } catch {
    return false;
  }
}

function main() {
  const metadata = JSON.parse(
    execFileSync("cargo", ["metadata", "--format-version", "1", "--locked"], {
      cwd: root,
      encoding: "utf8",
      maxBuffer: 16 * 1024 * 1024,
    }),
  );
  const rejected = metadata.packages.filter(
    (pkg) => !pkg.license || !isAllowedLicense(pkg.license),
  );

  if (rejected.length > 0) {
    const detail = rejected.map((pkg) => `${pkg.name}: ${pkg.license ?? "missing"}`).join("\n");
    throw new Error(`Cargo license policy rejected:\n${detail}`);
  }

  console.log(`Cargo license policy passed for ${metadata.packages.length} packages`);
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main();
}
