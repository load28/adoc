import { existsSync, readFileSync, readdirSync } from "node:fs";
import { basename, join } from "node:path";

const catalog = readJson("infra/observability/catalog.json");
const dashboards = readdirSync("infra/observability/dashboards")
  .filter((file) => file.endsWith(".json"))
  .map((file) => readJson(join("infra/observability/dashboards", file)));
const alertDocument = readJson("infra/observability/alerts/production-alerts.json");
const registry = parseRustRegistry(readFileSync("crates/telemetry/src/lib.rs", "utf8"));

const summary = validateObservability(catalog, dashboards, alertDocument.alerts, registry);
if (process.argv.includes("--self-test")) {
  expectFailure(() =>
    validateObservability(catalog, dashboards.slice(1), alertDocument.alerts, registry),
  );
  expectFailure(() =>
    validateObservability(
      catalog,
      dashboards,
      alertDocument.alerts.map((alert, index) =>
        index === 0 ? { ...alert, metric: "unknown_metric", sli: undefined } : alert,
      ),
      registry,
    ),
  );
  expectFailure(() =>
    validateObservability(catalog, dashboards, alertDocument.alerts, registry.slice(1)),
  );
}
console.log(JSON.stringify(summary));

export function validateObservability(source, dashboardDocuments, alerts, rustRegistry) {
  if (source.schemaVersion !== 1) throw new Error("unsupported observability schema");
  exactIds(source.metrics, "name", "metric");
  exactIds(source.slis, "id", "SLI");
  exactIds(dashboardDocuments, "id", "dashboard");
  exactIds(alerts, "id", "alert");
  assertExactSet(
    dashboardDocuments.map(({ id }) => id),
    source.requiredDashboards,
    "dashboard IDs",
  );
  assertExactSet(
    alerts.map(({ id }) => id),
    source.requiredAlerts,
    "alert IDs",
  );
  const metrics = new Map(source.metrics.map((metric) => [metric.name, metric]));
  const slis = new Set(source.slis.map(({ id }) => id));
  for (const sli of source.slis) {
    for (const metric of [sli.metric, sli.goodMetric, sli.totalMetric].filter(Boolean))
      if (!metrics.has(metric)) throw new Error(`${sli.id} references unknown metric ${metric}`);
  }
  for (const dashboard of dashboardDocuments) {
    if (basename(`${dashboard.id}.json`, ".json") !== dashboard.id)
      throw new Error("dashboard ID is not canonical");
    for (const panel of dashboard.panels ?? []) {
      if (panel.sli && !slis.has(panel.sli))
        throw new Error(`${panel.id} references unknown SLI ${panel.sli}`);
      for (const metric of panel.metrics ?? [])
        if (!metrics.has(metric))
          throw new Error(`${panel.id} references unknown metric ${metric}`);
    }
  }
  for (const alert of alerts) {
    if ((!alert.metric && !alert.sli) || (alert.metric && alert.sli))
      throw new Error(`${alert.id} must reference exactly one metric or SLI`);
    if (alert.metric && !metrics.has(alert.metric))
      throw new Error(`${alert.id} references unknown metric ${alert.metric}`);
    if (alert.sli && !slis.has(alert.sli))
      throw new Error(`${alert.id} references unknown SLI ${alert.sli}`);
    if (!existsSync(alert.runbook)) throw new Error(`${alert.id} runbook does not exist`);
    for (const field of ["severity", "condition", "for", "owner", "impact"])
      if (!alert[field]) throw new Error(`${alert.id} is missing ${field}`);
  }
  assertExactSet(
    rustRegistry.map(({ name }) => name),
    source.metrics.map(({ name }) => name),
    "Rust metric registry",
  );
  for (const runtimeMetric of rustRegistry) {
    assertExactSet(
      runtimeMetric.labels,
      metrics.get(runtimeMetric.name).labels,
      `${runtimeMetric.name} labels`,
    );
  }
  return {
    schemaVersion: 1,
    slis: source.slis.length,
    metrics: source.metrics.length,
    dashboards: dashboardDocuments.length,
    alerts: alerts.length,
  };
}

function parseRustRegistry(source) {
  return [...source.matchAll(/MetricDescriptor::new\(\s*"([^"]+)",\s*&\[([^\]]*)\]/gu)].map(
    ([, name, labels]) => ({
      name,
      labels: [...labels.matchAll(/"([^"]+)"/gu)].map((match) => match[1]),
    }),
  );
}

function exactIds(values, key, label) {
  const ids = values.map((value) => value[key]);
  if (ids.some((id) => typeof id !== "string" || !id)) throw new Error(`${label} ID is invalid`);
  if (new Set(ids).size !== ids.length) throw new Error(`${label} IDs contain duplicates`);
}

function assertExactSet(actual, expected, label) {
  const missing = expected.filter((value) => !actual.includes(value));
  const extra = actual.filter((value) => !expected.includes(value));
  const duplicates = actual.filter((value, index) => actual.indexOf(value) !== index);
  if (missing.length || extra.length || duplicates.length)
    throw new Error(`${label} differ: ${JSON.stringify({ missing, extra, duplicates })}`);
}

function expectFailure(callback) {
  try {
    callback();
  } catch {
    return;
  }
  throw new Error("observability negative self-test unexpectedly passed");
}

function readJson(path) {
  return JSON.parse(readFileSync(path, "utf8"));
}
