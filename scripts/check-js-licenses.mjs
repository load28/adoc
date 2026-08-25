import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("../", import.meta.url));
const exceptions = JSON.parse(
  readFileSync(new URL("../infra/licenses/js-exceptions.json", import.meta.url), "utf8"),
);
const webManifest = JSON.parse(
  readFileSync(new URL("../apps/web/package.json", import.meta.url), "utf8"),
);

for (const exception of exceptions) {
  if (exception.license !== "Apache-2.0")
    fail(`${exception.package}: unsupported exception license`);
  if (!exception.licenseUrl.startsWith(`${exception.repository}/raw/`)) {
    fail(`${exception.package}: license URL is outside the declared official repository`);
  }
  if (webManifest.dependencies?.[exception.package] !== exception.version) {
    fail(`${exception.package}: dependency version is not exactly ${exception.version}`);
  }
  const installed = JSON.parse(
    readFileSync(
      new URL(`../apps/web/node_modules/${exception.package}/package.json`, import.meta.url),
      "utf8",
    ),
  );
  if (installed.name !== exception.package || installed.version !== exception.version) {
    fail(`${exception.package}: installed package identity differs from the approved exception`);
  }
  const repository =
    typeof installed.repository === "string" ? installed.repository : installed.repository?.url;
  if (repository !== exception.repository) {
    fail(`${exception.package}: installed repository differs from the approved exception`);
  }
}

const allowed = [
  "0BSD",
  "Apache-2.0",
  "BlueOak-1.0.0",
  "BSD-2-Clause",
  "BSD-3-Clause",
  "CC-BY-3.0",
  "CC-BY-4.0",
  "CC0-1.0",
  "ISC",
  "MIT",
  "MIT-0",
  "MPL-2.0",
  "Python-2.0",
  "Unicode-3.0",
  "Unlicense",
].join(";");
const excluded = exceptions.map((item) => `${item.package}@${item.version}`).join(";");
const checker = new URL("../node_modules/.bin/license-checker-rseidelsohn", import.meta.url);
const result = spawnSync(
  fileURLToPath(checker),
  [
    "--start",
    root,
    "--summary",
    "--excludePrivatePackages",
    "--onlyAllow",
    allowed,
    "--excludePackages",
    excluded,
  ],
  { stdio: "inherit" },
);
if (result.status !== 0) process.exit(result.status ?? 1);
console.log(`JavaScript license exceptions passed: ${exceptions.length}`);

function fail(message) {
  console.error(message);
  process.exit(1);
}
