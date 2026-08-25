import { readFile } from "node:fs/promises";
import { resolve } from "node:path";

const root = resolve(new URL("..", import.meta.url).pathname);
const read = (path) => readFile(resolve(root, path), "utf8");

export function validateAudit(audit, index) {
  if (audit.schemaVersion !== 1) throw new Error("unsupported completion audit schema");
  const expected = (prefix, count) =>
    Array.from({ length: count }, (_, value) => `${prefix}-${String(value + 1).padStart(2, "0")}`);
  for (const [name, entries, ids] of [
    ["requirements", audit.requirements, expected("RQ", 20)],
    ["screens", audit.screens, expected("SCR", 22)],
  ]) {
    if (JSON.stringify(entries.map(({ id }) => id)) !== JSON.stringify(ids))
      throw new Error(`${name} are not an ordered complete set`);
  }
  for (const entry of [...audit.requirements, ...audit.screens, ...audit.qualityGates]) {
    if (!new Set(["partial", "complete", "environment_skip"]).has(entry.status))
      throw new Error(`${entry.id} has invalid status`);
    if (entry.status === "partial" && (!Array.isArray(entry.tasks) || entry.tasks.length === 0))
      throw new Error(`${entry.id} partial status has no remediation task`);
    if (!Array.isArray(entry.evidence) || entry.evidence.length === 0)
      throw new Error(`${entry.id} has no direct evidence`);
    if (entry.status === "environment_skip") {
      for (const field of ["reasonCode", "dependency", "verificationCommand"])
        if (!entry[field]) throw new Error(`${entry.id} environment skip is missing ${field}`);
      if (!entry.reasonCode.endsWith("_UNAVAILABLE"))
        throw new Error(`${entry.id} environment skip is not an unavailable external dependency`);
    }
    for (const task of entry.tasks ?? []) {
      if (!index.includes(`| ${task} |`)) throw new Error(`${entry.id} references missing ${task}`);
    }
  }
  if (new Set(audit.qualityGates.map(({ id }) => id)).size !== audit.qualityGates.length)
    throw new Error("duplicate quality gate id");
  return {
    requirements: audit.requirements.length,
    screens: audit.screens.length,
    qualityGates: audit.qualityGates.length,
    partial: [...audit.requirements, ...audit.screens, ...audit.qualityGates].filter(
      ({ status }) => status === "partial",
    ).length,
  };
}

const audit = JSON.parse(await read("docs/design/quality/implementation-completion-audit.json"));
const index = await read("docs/tasks/INDEX.md");
console.log(JSON.stringify(validateAudit(audit, index)));
