import { readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { parse } from "yaml";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const compose = parse(readFileSync(join(root, "compose.yaml"), "utf8"));
const requiredServices = new Set([
  "volume-init",
  "postgres",
  "redis",
  "migrate",
  "api",
  "worker",
  "web",
  "opensearch",
  "ai-runner",
  "otel-collector",
  "backup",
  "test-runner",
]);

for (const service of requiredServices) {
  if (!compose.services?.[service]) throw new Error(`missing Compose service: ${service}`);
}
for (const [name, service] of Object.entries(compose.services)) {
  if (service.privileged) throw new Error(`${name} must not be privileged`);
  if (service.image && /:(latest|\d+)$/u.test(service.image))
    throw new Error(`${name} image must use an exact patch tag`);
  for (const port of service.ports ?? []) {
    if (typeof port === "string" && !port.startsWith("127.0.0.1:"))
      throw new Error(`${name} host port must bind loopback`);
  }
  for (const [key, value] of Object.entries(service.environment ?? {})) {
    if (/(PASSWORD|SECRET|TOKEN|DATABASE_URL|REDIS_URL)$/u.test(key) && !key.endsWith("_FILE"))
      throw new Error(`${name} secret environment must use a file reference: ${key}`);
    if (typeof value === "string" && /postgres:\/\/|redis:\/\//u.test(value))
      throw new Error(`${name} embeds a credential URL`);
  }
}

for (const service of ["api", "worker", "web", "postgres", "redis", "opensearch"]) {
  if (!compose.services[service].healthcheck)
    throw new Error(`${service} must declare a healthcheck`);
}
for (const service of ["api", "worker"]) {
  if (
    compose.services[service].depends_on?.["volume-init"]?.condition !==
    "service_completed_successfully"
  )
    throw new Error(`${service} must wait for object volume ownership initialization`);
}
for (const profile of ["search", "ai-local", "observability", "backup", "test"]) {
  if (!Object.values(compose.services).some((service) => service.profiles?.includes(profile)))
    throw new Error(`missing Compose profile: ${profile}`);
}
for (const volume of [
  "postgres_data",
  "object_data",
  "backup_data",
  "redis_data",
  "opensearch_data",
]) {
  if (!compose.volumes?.[volume]) throw new Error(`missing named volume: ${volume}`);
}
for (const secret of Object.values(compose.secrets ?? {})) {
  if (!secret.file?.includes("/.local/secrets/"))
    throw new Error("Compose secrets must resolve from the ignored local secret directory");
}

console.log("Compose static contract passed");
