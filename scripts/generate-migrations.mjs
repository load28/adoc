import { readFile, writeFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import path from "node:path";

const repositoryRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const canonicalPath = path.join(repositoryRoot, "docs/design/data/schema.sql");
const migrationPath = path.join(repositoryRoot, "infra/migrations/0001_canonical_baseline.sql");
const header = [
  "-- GENERATED FILE. DO NOT EDIT.",
  "-- Source: docs/design/data/schema.sql",
  "-- SQLx owns the transaction around this migration.",
  "",
].join("\n");

export function generateBaseline(canonical) {
  if (canonical.includes("\r")) {
    throw new Error("canonical schema must use LF line endings");
  }
  if (!canonical.endsWith("\n")) {
    throw new Error("canonical schema must end with a newline");
  }
  const beginMarker = "\nBEGIN;\n";
  const beginIndex = canonical.indexOf(beginMarker);
  if (beginIndex < 0 || canonical.indexOf(beginMarker, beginIndex + 1) >= 0) {
    throw new Error("canonical schema must contain one outer BEGIN;");
  }
  if (!canonical.endsWith("\nCOMMIT;\n")) {
    throw new Error("canonical schema must end with COMMIT;");
  }

  const body = `${canonical.slice(0, beginIndex + 1)}${canonical.slice(
    beginIndex + beginMarker.length,
    -"COMMIT;\n".length,
  )}`;
  return `${header}${body}`;
}

async function main() {
  const canonical = await readFile(canonicalPath, "utf8");
  const generated = generateBaseline(canonical);

  if (process.argv.includes("--check")) {
    let committed;
    try {
      committed = await readFile(migrationPath, "utf8");
    } catch (error) {
      if (error?.code === "ENOENT") {
        throw new Error(`generated migration is missing: ${migrationPath}`);
      }
      throw error;
    }
    if (committed !== generated) {
      throw new Error("generated migration drift detected; run bun run migrations:generate");
    }
    return;
  }

  await writeFile(migrationPath, generated, "utf8");
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  await main();
}
