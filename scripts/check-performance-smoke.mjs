import process from "node:process";
import { performance } from "node:perf_hooks";

export function nearestRankPercentile(values, percentile) {
  if (!Array.isArray(values) || values.length === 0) throw new Error("latency sample is empty");
  if (!(percentile > 0 && percentile <= 1)) throw new Error("percentile is out of range");
  const sorted = [...values].sort((left, right) => left - right);
  return sorted[Math.ceil(percentile * sorted.length) - 1];
}

if (process.argv.includes("--self-test")) {
  if (nearestRankPercentile([5, 1, 4, 2, 3], 0.95) !== 5) throw new Error("p95 self-test failed");
  if (
    nearestRankPercentile(
      Array.from({ length: 100 }, (_, index) => index + 1),
      0.95,
    ) !== 95
  )
    throw new Error("nearest rank boundary failed");
  console.log("performance smoke self-test passed");
} else {
  const base = new URL(process.env.ADOC_SMOKE_ORIGIN ?? "http://127.0.0.1:18080");
  const api = new URL(process.env.ADOC_SMOKE_API_ORIGIN ?? "http://127.0.0.1:18081");
  const workloads = [
    { name: "api_ready", url: new URL("/health/ready", api), status: 200, thresholdMs: 500 },
    { name: "web_live", url: new URL("/health/live", base), status: 200, thresholdMs: 500 },
    { name: "ssr_login", url: new URL("/login", base), status: 200, thresholdMs: 1_000 },
  ];
  const evidence = [];
  for (const workload of workloads) {
    await sample(workload);
    const durations = [];
    for (let index = 0; index < 30; index += 1) durations.push(await sample(workload));
    const p95Ms = nearestRankPercentile(durations, 0.95);
    const maximumMs = Math.max(...durations);
    evidence.push({ name: workload.name, samples: durations.length, p95Ms, maximumMs, errors: 0 });
    if (p95Ms > workload.thresholdMs)
      throw new Error(
        `${workload.name} p95 ${p95Ms.toFixed(2)}ms exceeds ${workload.thresholdMs}ms`,
      );
  }
  console.log(JSON.stringify({ schemaVersion: 1, workloads: evidence }));
}

async function sample(workload) {
  const started = performance.now();
  const response = await fetch(workload.url, {
    redirect: "manual",
    signal: AbortSignal.timeout(5_000),
  });
  await response.arrayBuffer();
  const duration = performance.now() - started;
  if (response.status !== workload.status)
    throw new Error(`${workload.name} returned ${response.status}, expected ${workload.status}`);
  return Number(duration.toFixed(3));
}
