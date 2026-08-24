import { createHash } from "node:crypto";
import { readFile, readdir, writeFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import path from "node:path";

const repositoryRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const canonicalPath = path.join(repositoryRoot, "docs/design/data/schema.sql");
const migrationPath = path.join(repositoryRoot, "infra/migrations/0001_canonical_baseline.sql");
const migrationDirectory = path.join(repositoryRoot, "infra/migrations");
const manifestPath = path.join(migrationDirectory, "manifest.json");
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

function sha256(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}

async function migrationFiles() {
  const files = (await readdir(migrationDirectory))
    .filter((file) => /^\d{4}_[a-z0-9_]+\.sql$/u.test(file))
    .sort();
  if (files.length === 0) throw new Error("at least one migration is required");
  for (const [index, file] of files.entries()) {
    const expected = String(index + 1).padStart(4, "0");
    if (!file.startsWith(`${expected}_`))
      throw new Error(`migration versions must be contiguous: expected ${expected}, found ${file}`);
  }
  return files;
}

async function migrationChecksums() {
  return Promise.all(
    (await migrationFiles()).map(async (file) => ({
      file,
      sha256: sha256(await readFile(path.join(migrationDirectory, file))),
    })),
  );
}

async function readManifest() {
  const manifest = JSON.parse(await readFile(manifestPath, "utf8"));
  if (manifest.version !== 1 || !Array.isArray(manifest.migrations))
    throw new Error("migration manifest must use version 1");
  return manifest;
}

async function checkManifest() {
  const manifest = await readManifest();
  const actual = await migrationChecksums();
  if (JSON.stringify(manifest.migrations) !== JSON.stringify(actual))
    throw new Error("migration manifest drift detected; never modify sealed migrations");
}

async function sealMigrations() {
  let existing = { version: 1, migrations: [] };
  try {
    existing = await readManifest();
  } catch (error) {
    if (error?.code !== "ENOENT") throw error;
  }
  const actual = await migrationChecksums();
  for (const [index, sealed] of existing.migrations.entries()) {
    if (JSON.stringify(sealed) !== JSON.stringify(actual[index]))
      throw new Error(`sealed migration changed or disappeared: ${sealed.file}`);
  }
  await writeFile(
    manifestPath,
    `${JSON.stringify({ version: 1, migrations: actual }, null, 2)}\n`,
    "utf8",
  );
}

async function main() {
  if (process.argv.includes("--check")) {
    await checkManifest();
    return;
  }
  if (process.argv.includes("--baseline")) {
    const canonical = await readFile(canonicalPath, "utf8");
    await writeFile(migrationPath, generateBaseline(canonical), "utf8");
    return;
  }
  await sealMigrations();
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  await main();
}
