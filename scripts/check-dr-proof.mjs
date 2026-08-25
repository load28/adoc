import { createHash } from "node:crypto";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname } from "node:path";

const contract = JSON.parse(readFileSync("infra/release/dr-contract.json", "utf8"));
const migrationManifest = JSON.parse(readFileSync("infra/migrations/manifest.json", "utf8"));
validateContract(contract, migrationManifest);

if (process.argv.includes("--self-test")) {
  expectFailure(() => validateContract({ ...contract, currentMigration: 22 }, migrationManifest));
  expectFailure(() =>
    validateEvidence(contract, {
      sourceMigration: 21,
      restoredMigration: 20,
      durationSeconds: 1,
      steps: contract.requiredSteps,
    }),
  );
  console.log(
    JSON.stringify({
      schemaVersion: 1,
      currentMigration: contract.currentMigration,
      requiredSteps: contract.requiredSteps.length,
      selfTest: "passed",
    }),
  );
  process.exit(0);
}

const recordIndex = process.argv.indexOf("--record");
if (recordIndex === -1) throw new Error("DR proof requires --self-test or --record");
const [output, started, ended, sourceMigration, restoredMigration] = process.argv.slice(
  recordIndex + 1,
  recordIndex + 6,
);
const evidence = {
  schemaVersion: 1,
  generatedAt: new Date(Number(ended) * 1000).toISOString(),
  sourceMigration: Number(sourceMigration),
  restoredMigration: Number(restoredMigration),
  durationSeconds: Number(ended) - Number(started),
  rpoTargetSeconds: contract.rpoTargetSeconds,
  rtoTargetSeconds: contract.rtoTargetSeconds,
  productionClaim: false,
  steps: contract.requiredSteps,
};
validateEvidence(contract, evidence);
evidence.proofIdentity = createHash("sha256")
  .update(JSON.stringify({ ...evidence, generatedAt: undefined }))
  .digest("hex");
mkdirSync(dirname(output), { recursive: true });
writeFileSync(output, `${JSON.stringify(evidence, null, 2)}\n`);
console.log(JSON.stringify(evidence));

export function validateContract(value, migrations) {
  const latest = migrations.migrations.length;
  if (value.schemaVersion !== 1 || value.currentMigration !== latest)
    throw new Error("DR current migration differs from the sealed migration manifest");
  if (
    value.rollbackApplication.minimumMigration > latest ||
    value.rollbackApplication.maximumMigration < latest
  )
    throw new Error("previous application schema range cannot read the current migration");
  if (value.retentionDays.primaryPurge !== 30 || value.retentionDays.backup !== 35)
    throw new Error("DR retention differs from the canonical 30/35 day policy");
  if (new Set(value.requiredSteps).size !== 10)
    throw new Error("DR proof does not define the exact ten-step drill");
}

export function validateEvidence(value, evidence) {
  if (
    evidence.sourceMigration !== value.currentMigration ||
    evidence.restoredMigration !== value.currentMigration
  )
    throw new Error("restored migration does not match the source");
  if (evidence.durationSeconds < 0 || evidence.durationSeconds > value.rtoTargetSeconds)
    throw new Error("local restore duration is invalid or exceeds the RTO target");
  if (
    evidence.steps.length !== value.requiredSteps.length ||
    value.requiredSteps.some((step) => !evidence.steps.includes(step))
  )
    throw new Error("DR evidence omits a required drill step");
}

function expectFailure(callback) {
  try {
    callback();
  } catch {
    return;
  }
  throw new Error("DR negative self-test unexpectedly passed");
}
