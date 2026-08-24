// @ts-expect-error Vite emits the TanStack server entry before this runtime bundle is built.
import startHandler from "../../dist/server/server.js";

import { parseWebRuntimeConfig } from "./config";
import { safeServerEvent } from "./telemetry";

const source = Object.fromEntries(
  Object.entries(Bun.env).filter(([key]) => key.startsWith("ADOC_")),
);
const config = parseWebRuntimeConfig(source);
const [hostname, port] = config.httpBind.split(":");
const server = Bun.serve({
  hostname,
  port: Number(port),
  async fetch(request) {
    const pathname = new URL(request.url).pathname;
    if (pathname === "/health/live")
      return Response.json({ status: "ok", service: "web", releaseSha: config.releaseSha });
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
