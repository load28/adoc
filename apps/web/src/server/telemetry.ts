import type { WebRuntimeConfig } from "./config";

const forbidden = [
  "authorization",
  "content",
  "cookie",
  "credential",
  "file_name",
  "password",
  "prompt",
  "query",
  "secret",
  "signed_url",
  "title",
  "token",
];
const metrics = new Map<string, ReadonlySet<string>>([
  ["http_requests_total", new Set(["method", "status", "code", "service"])],
  ["http_request_duration_ms", new Set(["method", "status", "service"])],
]);

export function safeServerEvent(
  config: WebRuntimeConfig,
  code: string,
  fields: Readonly<Record<string, unknown>> = {},
) {
  return Object.freeze({
    service: "web",
    version: config.releaseSha,
    code,
    fields: sanitize(fields),
  });
}

export class WebMetricRegistry {
  readonly #values = new Map<string, number>();

  increment(name: string, labels: Readonly<Record<string, string>>, value = 1): void {
    const allowed = metrics.get(name);
    if (!allowed) throw new Error(`unknown metric: ${name}`);
    const normalized = Object.entries(labels).sort(([left], [right]) => left.localeCompare(right));
    for (const [key, labelValue] of normalized) {
      if (!allowed.has(key)) throw new Error(`metric ${name} does not permit label ${key}`);
      if (!safeLabel(key, labelValue))
        throw new Error("metric label value is unsafe for cardinality");
    }
    const key = `${name}{${normalized.map(([label, labelValue]) => `${label}=${labelValue}`).join(",")}}`;
    this.#values.set(key, (this.#values.get(key) ?? 0) + value);
  }

  snapshot(): ReadonlyMap<string, number> {
    return new Map(this.#values);
  }
}

function sanitize(value: unknown): unknown {
  if (Array.isArray(value)) return value.map(sanitize);
  if (value && typeof value === "object")
    return Object.fromEntries(
      Object.entries(value).map(([key, nested]) => [
        key,
        forbidden.some((part) => key.toLowerCase().replaceAll("-", "_").includes(part))
          ? "[REDACTED]"
          : sanitize(nested),
      ]),
    );
  return value;
}

function safeLabel(key: string, value: string): boolean {
  if (!value || value.length > 64 || key.endsWith("_id")) return false;
  if (/^[0-9a-f]{8}-[0-9a-f-]{27}$/iu.test(value)) return false;
  return /^[A-Za-z0-9_./-]+$/u.test(value);
}
