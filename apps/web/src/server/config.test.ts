import { describe, expect, test } from "bun:test";

import { parseWebRuntimeConfig } from "./config";
import { safeServerEvent, WebMetricRegistry } from "./telemetry";

const valid = { ADOC_ENV: "test", ADOC_RELEASE_SHA: "release-abc123" };

describe("web runtime configuration", () => {
  test("parses immutable defaults", () => {
    const config = parseWebRuntimeConfig(valid);
    expect(config.httpBind).toBe("0.0.0.0:8080");
    expect(config.shutdownGraceMs).toBe(30_000);
    expect(config.apiUpstream).toBeUndefined();
    expect(Object.isFrozen(config)).toBe(true);
  });

  test("rejects negative corpus", () => {
    const cases = [
      { ...valid, ADOC_UNKNOWN: "x" },
      { ...valid, ADOC_ENV: "preview" },
      { ...valid, ADOC_HTTP_BIND: "localhost:8080" },
      { ...valid, ADOC_SHUTDOWN_GRACE: "4s" },
      { ...valid, ADOC_PUBLIC_ORIGIN: "relative" },
      { ...valid, ADOC_API_UPSTREAM: "relative" },
      { ...valid, ADOC_ENV: "production", ADOC_PUBLIC_ORIGIN: "http://example.com" },
    ];
    for (const source of cases) expect(() => parseWebRuntimeConfig(source)).toThrow();
  });
});

describe("web server telemetry", () => {
  test("redacts nested forbidden fields", () => {
    const event = safeServerEvent(parseWebRuntimeConfig(valid), "REQUEST", {
      document_title: "private",
      nested: { prompt: "secret", result: "ok" },
    });
    const rendered = JSON.stringify(event);
    expect(rendered).not.toContain("private");
    expect(rendered).not.toContain("secret");
    expect(rendered).toContain("[REDACTED]");
    expect(rendered).toContain("ok");
  });

  test("rejects unknown and high-cardinality metrics", () => {
    const registry = new WebMetricRegistry();
    expect(() => registry.increment("missing", {})).toThrow();
    expect(() => registry.increment("http_requests_total", { user_id: "u1" })).toThrow();
    expect(() =>
      registry.increment("http_requests_total", {
        service: "00000000-0000-7000-8000-000000000001",
      }),
    ).toThrow();
    registry.increment("http_requests_total", { service: "web", status: "200" });
    expect([...registry.snapshot().values()]).toEqual([1]);
  });
});
