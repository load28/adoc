import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";

const runJson = (script, args = []) =>
  JSON.parse(execFileSync("node", [script, ...args], { encoding: "utf8" }).trim());
const sourceSha = execFileSync("git", ["rev-parse", "HEAD"], { encoding: "utf8" }).trim();
const releaseInputs = runJson("scripts/check-release-inputs.mjs");
const completion = runJson("scripts/check-completion-audit.mjs");
const evidence = {
  observability: runJson("scripts/check-observability.mjs", ["--self-test"]),
  performance: runJson("scripts/check-performance-profiles.mjs", ["--self-test"]),
  supplyChain: runJson("scripts/check-supply-chain.mjs", ["--self-test"]),
  provenance: runJson("scripts/check-provenance.mjs", ["--self-test"]),
  disasterRecovery: runJson("scripts/check-dr-proof.mjs", ["--self-test"]),
  acceptance: runJson("scripts/check-acceptance.mjs", ["--self-test"]),
  browserAcceptance: parseBrowser(
    execFileSync("node", ["scripts/check-browser-acceptance.mjs", "--self-test"], {
      encoding: "utf8",
    }),
  ),
  completion,
};
const environment = JSON.parse(readFileSync("infra/security/environment-evidence.json", "utf8"));
const result = validateProof({ sourceSha, releaseInputs, evidence, environment });
if (process.argv.includes("--self-test")) {
  const tampered = validateProof({
    sourceSha,
    releaseInputs: { ...releaseInputs, sourceDigest: "0".repeat(64) },
    evidence,
    environment,
  });
  if (tampered.proofIdentity === result.proofIdentity)
    throw new Error("production proof identity ignores source changes");
  expectFailure(() =>
    validateProof({
      sourceSha,
      releaseInputs,
      evidence: { ...evidence, completion: { ...completion, partial: 1 } },
      environment,
    }),
  );
}
console.log(JSON.stringify(result));

export function validateProof(proof) {
  if (!/^[a-f0-9]{40}$/u.test(proof.sourceSha)) throw new Error("source SHA is invalid");
  if (!/^[a-f0-9]{64}$/u.test(proof.releaseInputs.sourceDigest))
    throw new Error("source digest is invalid");
  if (proof.evidence.completion.partial !== 0)
    throw new Error(
      `completion audit still has ${proof.evidence.completion.partial} partial entries`,
    );
  if (proof.environment.schemaVersion !== 1 || !proof.environment.environmentSkips.length)
    throw new Error("environment evidence is missing");
  const identityInput = {
    sourceSha: proof.sourceSha,
    sourceDigest: proof.releaseInputs.sourceDigest,
    evidence: proof.evidence,
    environmentSkips: proof.environment.environmentSkips.map(
      ({ id, reasonCode, dependency, verificationCommand }) => ({
        id,
        reasonCode,
        dependency,
        verificationCommand,
      }),
    ),
  };
  return {
    schemaVersion: 1,
    sourceSha: proof.sourceSha,
    sourceDigest: proof.releaseInputs.sourceDigest,
    evidence: proof.evidence,
    environmentSkips: proof.environment.environmentSkips.length,
    proofIdentity: createHash("sha256").update(canonical(identityInput)).digest("hex"),
  };
}

function parseBrowser(output) {
  const match = output.match(/browser acceptance contract passed: (\d+) exact scenarios/u);
  if (!match) throw new Error("browser acceptance output is invalid");
  return { schemaVersion: 1, scenarios: Number(match[1]) };
}

function canonical(value) {
  if (Array.isArray(value)) return `[${value.map(canonical).join(",")}]`;
  if (value && typeof value === "object")
    return `{${Object.keys(value)
      .sort()
      .map((key) => `${JSON.stringify(key)}:${canonical(value[key])}`)
      .join(",")}}`;
  return JSON.stringify(value);
}

function expectFailure(callback) {
  try {
    callback();
  } catch {
    return;
  }
  throw new Error("production readiness negative self-test unexpectedly passed");
}
