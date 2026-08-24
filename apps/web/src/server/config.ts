export type WebEnvironment = "development" | "test" | "staging" | "production";
export type WebLogLevel = "trace" | "debug" | "info" | "warn" | "error";

export interface WebRuntimeConfig {
  readonly environment: WebEnvironment;
  readonly releaseSha: string;
  readonly httpBind: string;
  readonly publicOrigin?: URL;
  readonly shutdownGraceMs: number;
  readonly logLevel: WebLogLevel;
  readonly otelEndpoint?: URL;
}

const keys = new Set([
  "ADOC_ENV",
  "ADOC_RELEASE_SHA",
  "ADOC_HTTP_BIND",
  "ADOC_PUBLIC_ORIGIN",
  "ADOC_SHUTDOWN_GRACE",
  "ADOC_LOG_LEVEL",
  "ADOC_OTEL_ENDPOINT",
]);

export function parseWebRuntimeConfig(
  source: Readonly<Record<string, string | undefined>>,
): WebRuntimeConfig {
  for (const key of Object.keys(source)) {
    if (key.startsWith("ADOC_") && !keys.has(key))
      throw new Error(`unknown configuration key: ${key}`);
  }
  const environment = oneOf(required(source, "ADOC_ENV"), "ADOC_ENV", [
    "development",
    "test",
    "staging",
    "production",
  ] as const);
  const releaseSha = required(source, "ADOC_RELEASE_SHA");
  if (releaseSha.trim().length === 0) throw new Error("invalid configuration for ADOC_RELEASE_SHA");
  const publicOrigin = optionalUrl(source.ADOC_PUBLIC_ORIGIN, "ADOC_PUBLIC_ORIGIN");
  if (environment === "production" && publicOrigin?.protocol !== "https:")
    throw new Error("production requires ADOC_PUBLIC_ORIGIN with HTTPS");
  const otelEndpoint = optionalUrl(source.ADOC_OTEL_ENDPOINT, "ADOC_OTEL_ENDPOINT");
  if (environment === "production" && otelEndpoint && otelEndpoint.protocol !== "https:")
    throw new Error("production requires ADOC_OTEL_ENDPOINT with HTTPS");
  return Object.freeze({
    environment,
    releaseSha,
    httpBind: socket(source.ADOC_HTTP_BIND ?? "0.0.0.0:8080"),
    publicOrigin,
    shutdownGraceMs: duration(source.ADOC_SHUTDOWN_GRACE ?? "30s", 5_000, 120_000),
    logLevel: oneOf(source.ADOC_LOG_LEVEL ?? "info", "ADOC_LOG_LEVEL", [
      "trace",
      "debug",
      "info",
      "warn",
      "error",
    ] as const),
    otelEndpoint,
  });
}

function required(source: Readonly<Record<string, string | undefined>>, key: string): string {
  const value = source[key];
  if (value === undefined) throw new Error(`missing required configuration key: ${key}`);
  return value;
}

function oneOf<const T extends readonly string[]>(
  value: string,
  key: string,
  choices: T,
): T[number] {
  if (choices.includes(value)) return value as T[number];
  throw new Error(`invalid configuration for ${key}`);
}

function optionalUrl(value: string | undefined, key: string): URL | undefined {
  if (value === undefined) return undefined;
  try {
    const parsed = new URL(value);
    if (!parsed.hostname) throw new Error("missing host");
    return parsed;
  } catch {
    throw new Error(`invalid configuration for ${key}`);
  }
}

function socket(value: string): string {
  const match = /^(?:\d{1,3}(?:\.\d{1,3}){3}|\[[0-9a-fA-F:]+\]):(\d{1,5})$/u.exec(value);
  const port = Number(match?.[1]);
  if (!match || port < 1 || port > 65_535)
    throw new Error("invalid configuration for ADOC_HTTP_BIND");
  return value;
}

function duration(value: string, minimum: number, maximum: number): number {
  const match = /^(\d+)(ms|s|m)$/u.exec(value);
  if (!match) throw new Error("invalid configuration for ADOC_SHUTDOWN_GRACE");
  const multipliers = { ms: 1, s: 1_000, m: 60_000 } as const;
  const milliseconds = Number(match[1]) * multipliers[match[2] as keyof typeof multipliers];
  if (milliseconds < minimum || milliseconds > maximum)
    throw new Error("invalid configuration for ADOC_SHUTDOWN_GRACE");
  return milliseconds;
}
