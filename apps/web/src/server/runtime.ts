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
