import { readFileSync } from "node:fs";

const profiles = JSON.parse(readFileSync("docs/design/quality/performance-profiles.json", "utf8"));
const summary = validateProfiles(profiles);
if (process.argv.includes("--self-test")) {
  expectFailure(() => validateProfiles({ ...profiles, profiles: profiles.profiles.slice(1) }));
  expectFailure(() =>
    validateProfiles({
      ...profiles,
      profiles: profiles.profiles.map((profile) =>
        profile.id === "soak" ? { ...profile, durationSeconds: 60 } : profile,
      ),
    }),
  );
  expectFailure(() =>
    validateProfiles({
      ...profiles,
      workloads: profiles.workloads.map((workload) =>
        workload.id === "document-read" ? { ...workload, targetP95Ms: 301 } : workload,
      ),
    }),
  );
}
console.log(JSON.stringify(summary));

export function validateProfiles(document) {
  if (document.schemaVersion !== 1) throw new Error("unsupported performance profile schema");
  const expectedWorkloads = [
    "document-read",
    "command-ack",
    "public-viewer",
    "search-first-page",
    "ai-first-progress",
  ];
  const expectedProfiles = ["repository", "load", "stress", "soak", "spike", "degradation"];
  exactSequence(
    document.workloads.map(({ id }) => id),
    expectedWorkloads,
    "workloads",
  );
  exactSequence(
    document.profiles.map(({ id }) => id),
    expectedProfiles,
    "profiles",
  );
  const targets = new Map([
    ["document-read", 300],
    ["command-ack", 500],
    ["public-viewer", 500],
    ["search-first-page", 1500],
    ["ai-first-progress", 2000],
  ]);
  for (const workload of document.workloads) {
    if (workload.targetP95Ms !== targets.get(workload.id))
      throw new Error(`${workload.id} p95 target differs from PROD-06`);
    if (workload.targetP99Ms < workload.targetP95Ms || workload.maxErrorRate < 0)
      throw new Error(`${workload.id} threshold is invalid`);
  }
  const minimumDurations = new Map([
    ["repository", 30],
    ["load", 1800],
    ["stress", 1],
    ["soak", 28800],
    ["spike", 300],
    ["degradation", 1800],
  ]);
  for (const profile of document.profiles) {
    if (profile.durationSeconds < minimumDurations.get(profile.id))
      throw new Error(`${profile.id} duration is below TEST-06`);
    if (profile.tier === "repository" && profile.id !== "repository")
      throw new Error("only the bounded repository profile may run on every commit");
    if (!profile.workloads?.length || !profile.resourceBudget || !profile.stop)
      throw new Error(`${profile.id} is incomplete`);
    for (const workload of profile.workloads)
      if (!expectedWorkloads.includes(workload))
        throw new Error(`${profile.id} references unknown workload ${workload}`);
  }
  exactSequence(
    document.profiles.find(({ id }) => id === "degradation").faults,
    [
      "redis-loss",
      "opensearch-timeout",
      "object-storage-partial-write",
      "ai-timeout",
      "sse-disconnect",
    ],
    "degradation faults",
  );
  return {
    schemaVersion: 1,
    workloads: document.workloads.length,
    profiles: document.profiles.length,
    environmentProfiles: document.profiles.filter(({ tier }) => tier === "environment").length,
  };
}

function exactSequence(actual, expected, label) {
  if (JSON.stringify(actual) !== JSON.stringify(expected))
    throw new Error(`${label} differ: ${JSON.stringify({ actual, expected })}`);
  if (new Set(actual).size !== actual.length) throw new Error(`${label} contain duplicates`);
}

function expectFailure(callback) {
  try {
    callback();
  } catch {
    return;
  }
  throw new Error("performance profile negative self-test unexpectedly passed");
}
