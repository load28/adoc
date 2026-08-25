import { readFileSync } from "node:fs";
import { parse } from "yaml";

const exceptions = JSON.parse(readFileSync("infra/security/advisory-exceptions.json", "utf8"));
const environment = JSON.parse(readFileSync("infra/security/environment-evidence.json", "utf8"));
const workflow = readFileSync(".github/workflows/ci.yml", "utf8");
const packageDocument = JSON.parse(readFileSync("package.json", "utf8"));
const auditRunner = readFileSync("scripts/audit-rust.mjs", "utf8");
const summary = validateSupplyChain(
  exceptions,
  environment,
  workflow,
  packageDocument,
  auditRunner,
  new Date(),
);
if (process.argv.includes("--self-test")) {
  expectFailure(() =>
    validateSupplyChain(
      {
        schemaVersion: 1,
        exceptions: [{ id: "RUSTSEC-0", owner: "x", reason: "x", expiresAt: "2020-01-01" }],
      },
      environment,
      workflow,
      packageDocument,
      auditRunner,
      new Date("2026-08-25T00:00:00Z"),
    ),
  );
  expectFailure(() =>
    validateSupplyChain(
      exceptions,
      { schemaVersion: 1, environmentSkips: [{}] },
      workflow,
      packageDocument,
      auditRunner,
      new Date(),
    ),
  );
  expectFailure(() =>
    validateSupplyChain(
      exceptions,
      environment,
      workflow.replace("  workflow_dispatch:\n", "  workflow_dispatch:\n  push:\n"),
      packageDocument,
      auditRunner,
      new Date(),
    ),
  );
  expectFailure(() =>
    validateSupplyChain(
      exceptions,
      environment,
      workflow,
      {
        ...packageDocument,
        scripts: { ...packageDocument.scripts, "ci:local": "bun run check" },
      },
      auditRunner,
      new Date(),
    ),
  );
}
console.log(JSON.stringify(summary));

export function validateSupplyChain(
  exceptionDocument,
  environmentDocument,
  workflowSource,
  packageDocument,
  auditRunnerSource,
  now,
) {
  if (exceptionDocument.schemaVersion !== 1 || environmentDocument.schemaVersion !== 1)
    throw new Error("unsupported supply-chain evidence schema");
  const ids = new Set();
  for (const exception of exceptionDocument.exceptions) {
    for (const field of ["id", "owner", "reason", "expiresAt"])
      if (!exception[field]) throw new Error(`advisory exception is missing ${field}`);
    if (ids.has(exception.id)) throw new Error(`duplicate advisory exception ${exception.id}`);
    ids.add(exception.id);
    if (Date.parse(exception.expiresAt) <= now.getTime())
      throw new Error(`advisory exception ${exception.id} has expired`);
  }
  for (const skip of environmentDocument.environmentSkips) {
    for (const field of ["id", "reasonCode", "dependency", "verificationCommand"])
      if (!skip[field]) throw new Error(`environment skip is missing ${field}`);
    if (!/_(UNAVAILABLE)$/u.test(skip.reasonCode))
      throw new Error(`${skip.id} does not describe an unavailable external dependency`);
  }
  const trigger = parse(workflowSource)?.on;
  if (
    !trigger ||
    typeof trigger !== "object" ||
    Array.isArray(trigger) ||
    Object.keys(trigger).length !== 1 ||
    !("workflow_dispatch" in trigger)
  )
    throw new Error("GitHub CI must be triggered only by workflow_dispatch");
  for (const marker of [
    "actions-rust-lang/audit@v1",
    "bun audit",
    "check-provenance.mjs",
    "check:production-readiness",
  ])
    if (!workflowSource.includes(marker)) throw new Error(`CI is missing ${marker}`);
  if (packageDocument.scripts?.["audit:rust"] !== "node scripts/audit-rust.mjs")
    throw new Error("Rust audit command does not use the advisory registry runner");
  const localCi = packageDocument.scripts?.["ci:local"];
  if (typeof localCi !== "string") throw new Error("local CI entrypoint is missing");
  let previousIndex = -1;
  for (const marker of [
    "bun run check",
    "bun run audit:rust",
    "bun audit --audit-level high",
    "bun run check:production-readiness",
    "bun run compose:integration",
    "bun run browser:check",
  ]) {
    const index = localCi.indexOf(marker);
    if (index <= previousIndex) throw new Error(`local CI is missing or misorders ${marker}`);
    previousIndex = index;
  }
  for (const id of ids)
    if (!workflowSource.includes(`ignore: ${id}`) || !auditRunnerSource.includes("--ignore"))
      throw new Error("advisory exception registry, audit runner, and CI ignore list differ");
  return {
    schemaVersion: 1,
    advisoryExceptions: exceptionDocument.exceptions.length,
    environmentSkips: environmentDocument.environmentSkips.length,
  };
}

function expectFailure(callback) {
  try {
    callback();
  } catch {
    return;
  }
  throw new Error("supply-chain negative self-test unexpectedly passed");
}
