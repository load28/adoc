// @ts-expect-error Vite emits the TanStack server entry before this runtime bundle is built.
import startHandler from "../../dist/server/server.js";

import { parseWebRuntimeConfig } from "./config";
import { resolveClientAsset } from "./static-assets";
import { safeServerEvent } from "./telemetry";

const source = Object.fromEntries(
  Object.entries(Bun.env).filter(([key]) => key.startsWith("ADOC_")),
);
const config = parseWebRuntimeConfig(source);
const [hostname, port] = config.httpBind.split(":");
const distributionRoot = new URL("../", import.meta.url);
const server = Bun.serve({
  hostname,
  port: Number(port),
  async fetch(request) {
    const pathname = new URL(request.url).pathname;
    if (pathname === "/health/live")
      return Response.json({ status: "ok", service: "web", releaseSha: config.releaseSha });
    const asset = resolveClientAsset(pathname);
    if (asset) {
      if (request.method !== "GET" && request.method !== "HEAD")
        return new Response(null, { status: 405, headers: { Allow: "GET, HEAD" } });
      const file = Bun.file(new URL(asset.relativePath, distributionRoot));
      if (!(await file.exists())) return new Response(null, { status: 404 });
      const headers = {
        "Cache-Control": "public, max-age=31536000, immutable",
        "Content-Type": asset.contentType,
        "X-Content-Type-Options": "nosniff",
      };
      return request.method === "HEAD"
        ? new Response(null, { headers })
        : new Response(file, { headers });
    }
    if (pathname.startsWith("/api/v1/")) {
      if (!config.apiUpstream) return Response.json({ status: "unavailable" }, { status: 503 });
      const target = new URL(`${pathname}${new URL(request.url).search}`, config.apiUpstream);
      const headers = new Headers(request.headers);
      headers.delete("connection");
      headers.delete("content-length");
      headers.delete("host");
      return fetch(target, {
        method: request.method,
        headers,
        body: request.method === "GET" || request.method === "HEAD" ? undefined : request.body,
        redirect: "manual",
      });
    }
    return startHandler.fetch(request);
  },
});

console.log(JSON.stringify(safeServerEvent(config, "SERVICE_STARTED", { port: server.port })));

async function shutdown() {
  await server.stop(true);
  process.exit(0);
}

process.on("SIGTERM", shutdown);
process.on("SIGINT", shutdown);
