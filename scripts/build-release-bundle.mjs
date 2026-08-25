import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import {
  cpSync,
  mkdirSync,
  readdirSync,
  readFileSync,
  rmSync,
  statSync,
  writeFileSync,
} from "node:fs";
import { basename, join, resolve } from "node:path";
import { signAndVerify, statement } from "./lib/provenance.mjs";
import { validateSpdxDocument } from "./lib/spdx.mjs";

const root = resolve(new URL("..", import.meta.url).pathname);
const run = (command, args, options = {}) => {
  const result = spawnSync(command, args, {
    cwd: root,
    encoding: options.binary ? undefined : "utf8",
    stdio: options.inherit ? "inherit" : "pipe",
    maxBuffer: 32 * 1024 * 1024,
    env: options.env ? { ...process.env, ...options.env } : process.env,
  });
  if (result.status !== 0) {
    const detail = options.inherit ? "" : `\n${result.stderr || result.stdout || ""}`;
    throw new Error(`${command} ${args.join(" ")} failed${detail}`);
  }
  return options.inherit ? "" : result.stdout.trim();
};
const sha256 = (value) => createHash("sha256").update(value).digest("hex");
const json = (value) => `${JSON.stringify(value, null, 2)}\n`;

function files(directory, prefix = "") {
  return readdirSync(directory, { withFileTypes: true })
    .flatMap((entry) => {
      const relative = join(prefix, entry.name);
      return entry.isDirectory() ? files(join(directory, entry.name), relative) : [relative];
    })
    .sort();
}

function validateVersions() {
  const packagePaths = [
    "package.json",
    "apps/web/package.json",
    "packages/contracts/package.json",
    "packages/editor-schema/package.json",
    "packages/i18n/package.json",
    "packages/ui-domain/package.json",
  ];
  const versions = packagePaths.map((path) => JSON.parse(readFileSync(join(root, path))).version);
  const cargo = readFileSync(join(root, "Cargo.toml"), "utf8").match(
    /\[workspace\.package\][\s\S]*?\nversion = "([^"]+)"/u,
  )?.[1];
  const version = versions[0];
  if (
    !/^\d+\.\d+\.\d+$/u.test(version) ||
    versions.some((value) => value !== version) ||
    cargo !== version
  )
    throw new Error("Cargo and JavaScript release versions are not identical SemVer values");
  return version;
}

function identity(manifest) {
  return sha256(
    JSON.stringify({
      version: manifest.version,
      sourceSha: manifest.sourceSha,
      sourceDigest: manifest.sourceDigest,
      migrations: manifest.migrations,
      contracts: manifest.contracts,
      images: manifest.images.map(({ title, id, revision, version }) => ({
        title,
        id,
        revision,
        version,
      })),
    }),
  );
}

function migrationVersion(manifest) {
  if (!Array.isArray(manifest.migrations) || manifest.migrations.length === 0)
    throw new Error("migration manifest is empty");
  const versions = manifest.migrations.map(({ file }, index) => {
    const parsed = file?.match(/^(\d{4})_[a-z0-9_]+\.sql$/u)?.[1];
    if (!parsed) throw new Error(`migration ${index + 1} has an invalid filename`);
    return Number(parsed);
  });
  if (versions.some((version, index) => version !== index + 1))
    throw new Error("migration versions are not contiguous and ordered");
  return String(versions.at(-1));
}

if (process.argv.includes("--self-test")) {
  const fixture = {
    version: "1.0.0",
    sourceSha: "a".repeat(40),
    sourceDigest: "b".repeat(64),
    migrations: "21",
    contracts: "c".repeat(64),
    images: [{ title: "adoc-api", id: "sha256:x", revision: "a".repeat(40), version: "1.0.0" }],
  };
  if (identity(fixture) !== identity({ ...fixture, generatedAt: new Date().toISOString() }))
    throw new Error("release identity includes volatile fields");
  if (identity(fixture) === identity({ ...fixture, sourceDigest: "d".repeat(64) }))
    throw new Error("release identity ignores source changes");
  if (
    migrationVersion({
      migrations: [{ file: "0001_initial.sql" }, { file: "0002_next.sql" }],
    }) !== "2"
  )
    throw new Error("migration version extraction is not canonical");
  console.log(JSON.stringify({ schemaVersion: 1, selfTest: "passed" }));
  process.exit(0);
}

if (!process.argv.includes("--verify"))
  throw new Error("release bundle requires --verify so gates cannot be bypassed");
if (run("git", ["branch", "--show-current"]) !== "main")
  throw new Error("release must run on main");
if (run("git", ["status", "--porcelain"]) !== "") throw new Error("release requires a clean tree");
run("bun", ["run", "check"], { inherit: true });
run("bun", ["run", "compose:integration"], { inherit: true });
run("bun", ["run", "browser:check"], { inherit: true });

const version = validateVersions();
const sourceSha = run("git", ["rev-parse", "HEAD"]);
const shortSha = sourceSha.slice(0, 12);
const releaseInputs = JSON.parse(run("node", ["scripts/check-release-inputs.mjs"]));
const acceptance = JSON.parse(run("node", ["scripts/check-acceptance.mjs"]));
const productionProof = JSON.parse(
  run("node", ["scripts/check-production-readiness.mjs", "--self-test"]),
);
const environmentEvidence = JSON.parse(
  readFileSync(join(root, "infra/security/environment-evidence.json")),
);
const disasterRecovery = JSON.parse(readFileSync(join(root, "dist/evidence/compose-dr.json")));
const releaseRoot = join(root, "dist", "release");
const directoryName = `adoc-${version}-${shortSha}`;
const directory = join(releaseRoot, directoryName);
rmSync(directory, { recursive: true, force: true });
mkdirSync(join(directory, "evidence"), { recursive: true });
mkdirSync(join(directory, "sbom"), { recursive: true });
cpSync(join(root, "docs", "design", "contracts"), join(directory, "contracts"), {
  recursive: true,
});
cpSync(join(root, "infra", "migrations"), join(directory, "migrations"), { recursive: true });

const images = [];
const imageTags = [];
for (const target of ["api", "worker", "web"]) {
  const tag = `adoc-release-${target}:${version}-${shortSha}`;
  run(
    "docker",
    [
      "build",
      "--target",
      target,
      "--build-arg",
      `ADOC_RELEASE_SHA=${sourceSha}`,
      "--build-arg",
      `ADOC_RELEASE_VERSION=${version}`,
      "--tag",
      tag,
      ".",
    ],
    { inherit: true },
  );
  const inspected = JSON.parse(run("docker", ["image", "inspect", tag]))[0];
  const labels = inspected.Config?.Labels ?? {};
  const expectedTitle = `adoc-${target}`;
  if (
    labels["org.opencontainers.image.title"] !== expectedTitle ||
    labels["org.opencontainers.image.revision"] !== sourceSha ||
    labels["org.opencontainers.image.version"] !== version
  )
    throw new Error(`${target} image identity labels do not match the release`);
  const sbomPath = join(directory, "sbom", `${target}.spdx.json`);
  const sbom = run("docker", ["sbom", "--format", "spdx-json", tag]);
  const parsedSbom = JSON.parse(sbom);
  const sbomIdentity = validateSpdxDocument(parsedSbom);
  writeFileSync(sbomPath, `${sbom}\n`);
  images.push({
    title: expectedTitle,
    tag,
    id: inspected.Id,
    revision: sourceSha,
    version,
    sbom: `sbom/${target}.spdx.json`,
    sbomVersion: sbomIdentity.version,
    sbomNamespace: sbomIdentity.namespace,
    sbomPackages: sbomIdentity.packages,
  });
  imageTags.push(tag);
}
run("docker", ["save", "--output", join(directory, "images.tar"), ...imageTags], {
  inherit: true,
});

const materials = Object.entries(releaseInputs.inputs).map(([uri, digest]) => ({
  uri,
  digest: { sha256: digest },
}));
const provenanceStatement = statement({
  sourceSha,
  sourceDigest: releaseInputs.sourceDigest,
  subjects: images.map((image) => ({
    name: image.title,
    digest: { sha256: image.id.replace(/^sha256:/u, "") },
  })),
  materials,
});
const provenance = {
  schemaVersion: 1,
  localCandidate: true,
  productionIdentity: false,
  statement: provenanceStatement,
  proof: signAndVerify(provenanceStatement),
};
writeFileSync(join(directory, "evidence", "provenance.json"), json(provenance));
writeFileSync(join(directory, "evidence", "environment-skips.json"), json(environmentEvidence));
writeFileSync(join(directory, "evidence", "production-readiness.json"), json(productionProof));
writeFileSync(join(directory, "evidence", "disaster-recovery.json"), json(disasterRecovery));

const migrationManifest = JSON.parse(readFileSync(join(root, "infra/migrations/manifest.json")));
const contractManifest = readFileSync(join(root, "packages/contracts/src/generated/manifest.json"));
const manifest = {
  schemaVersion: 1,
  version,
  localCandidate: true,
  generatedAt: new Date().toISOString(),
  sourceSha,
  sourceDigest: releaseInputs.sourceDigest,
  migrations: migrationVersion(migrationManifest),
  contracts: sha256(contractManifest),
  acceptance: { scenarios: acceptance.scenarios, workstreams: acceptance.workstreams },
  gates: ["bun run check", "bun run compose:integration", "bun run browser:check"],
  productionProof: {
    identity: productionProof.proofIdentity,
    path: "evidence/production-readiness.json",
  },
  disasterRecovery: {
    identity: disasterRecovery.proofIdentity,
    path: "evidence/disaster-recovery.json",
  },
  provenance: {
    digest: provenance.proof.statementDigest,
    path: "evidence/provenance.json",
    productionIdentity: false,
  },
  environmentSkips: environmentEvidence.environmentSkips.map(({ id, reasonCode }) => ({
    id,
    reasonCode,
  })),
  images,
};
manifest.releaseIdentity = identity(manifest);
writeFileSync(join(directory, "manifest.json"), json(manifest));
writeFileSync(
  join(directory, "evidence", "acceptance.json"),
  json({
    ...acceptance,
    sourceSha,
    sourceDigest: releaseInputs.sourceDigest,
    gates: manifest.gates,
  }),
);
const checksums = files(directory)
  .filter((path) => path !== "checksums.sha256")
  .map((path) => `${sha256(readFileSync(join(directory, path)))}  ${path}`)
  .join("\n");
writeFileSync(join(directory, "checksums.sha256"), `${checksums}\n`);
for (const line of checksums.split("\n")) {
  const [expected, path] = line.split("  ");
  if (sha256(readFileSync(join(directory, path))) !== expected)
    throw new Error(`checksum verification failed for ${path}`);
}
mkdirSync(releaseRoot, { recursive: true });
const archive = join(releaseRoot, `${directoryName}.tar.gz`);
rmSync(archive, { force: true });
run("tar", ["-C", releaseRoot, "-czf", archive, directoryName]);
console.log(
  JSON.stringify({
    schemaVersion: 1,
    artifact: archive,
    bytes: statSync(archive).size,
    sha256: sha256(readFileSync(archive)),
    releaseIdentity: manifest.releaseIdentity,
    file: basename(archive),
  }),
);
