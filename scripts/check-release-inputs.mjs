import { createHash } from "node:crypto";
import { readFile } from "node:fs/promises";

const inputs = [
  "Cargo.lock",
  "bun.lock",
  "Dockerfile",
  "infra/migrations/manifest.json",
  "packages/contracts/src/generated/manifest.json",
];
const hashes = {};
for (const path of inputs) {
  const bytes = await readFile(new URL(`../${path}`, import.meta.url));
  hashes[path] = createHash("sha256").update(bytes).digest("hex");
}
const migrations = JSON.parse(
  await readFile(new URL("../infra/migrations/manifest.json", import.meta.url), "utf8"),
);
const contracts = JSON.parse(
  await readFile(
    new URL("../packages/contracts/src/generated/manifest.json", import.meta.url),
    "utf8",
  ),
);
const dockerfile = await readFile(new URL("../Dockerfile", import.meta.url), "utf8");
if (!Array.isArray(migrations.migrations) || migrations.migrations.length === 0)
  throw new Error("migration manifest is empty");
if (!contracts.sources || typeof contracts.sources !== "object")
  throw new Error("contract manifest inputs are missing");
for (const title of ["adoc-api", "adoc-worker", "adoc-web"])
  if (!dockerfile.includes(`org.opencontainers.image.title="${title}"`))
    throw new Error(`OCI label missing for ${title}`);
const sourceDigest = createHash("sha256")
  .update(JSON.stringify(hashes, Object.keys(hashes).sort()))
  .digest("hex");
console.log(JSON.stringify({ schemaVersion: 1, sourceDigest, inputs: hashes }));
