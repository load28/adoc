import { createHash, verify } from "node:crypto";
import { readFileSync } from "node:fs";
import { canonical, signAndVerify, statement } from "./lib/provenance.mjs";

const sha256 = (value) => createHash("sha256").update(value).digest("hex");

if (process.argv.includes("--self-test")) {
  const fixture = statement({
    sourceSha: "a".repeat(40),
    sourceDigest: "b".repeat(64),
    subjects: [{ name: "adoc-api", digest: { sha256: "c".repeat(64) } }],
    materials: [{ uri: "Cargo.lock", digest: { sha256: "d".repeat(64) } }],
  });
  const proof = signAndVerify(fixture);
  const tampered = Buffer.from(canonical({ ...fixture, subject: [] }));
  if (
    verify(
      null,
      tampered,
      { key: Buffer.from(proof.publicKey, "base64"), format: "der", type: "spki" },
      Buffer.from(proof.signature, "base64"),
    )
  )
    throw new Error("tampered provenance was accepted");
  console.log(JSON.stringify({ schemaVersion: 1, selfTest: "passed" }));
} else {
  const sourceSha = process.env.ADOC_RELEASE_SHA;
  if (!/^[a-f0-9]{40}$/u.test(sourceSha ?? "")) throw new Error("ADOC_RELEASE_SHA is required");
  const inputs = ["Cargo.lock", "bun.lock", "Dockerfile"].map((path) => ({
    uri: path,
    digest: { sha256: sha256(readFileSync(path)) },
  }));
  const sourceDigest = sha256(canonical(inputs));
  const subjects = ["api", "worker", "web"].map((name) => ({
    name: `adoc-${name}`,
    digest: { sha256: process.env[`ADOC_${name.toUpperCase()}_DIGEST`] ?? "0".repeat(64) },
  }));
  const value = statement({ sourceSha, sourceDigest, subjects, materials: inputs });
  console.log(JSON.stringify({ schemaVersion: 1, statement: value, proof: signAndVerify(value) }));
}
