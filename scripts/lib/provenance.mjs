import { createHash, generateKeyPairSync, sign, verify } from "node:crypto";

const sha256 = (value) => createHash("sha256").update(value).digest("hex");
export function statement({ sourceSha, sourceDigest, subjects, materials }) {
  return {
    _type: "https://in-toto.io/Statement/v1",
    subject: subjects,
    predicateType: "https://slsa.dev/provenance/v1",
    predicate: {
      buildDefinition: {
        buildType: "https://github.com/load28/adoc/build/v1",
        externalParameters: { sourceSha, sourceDigest },
        internalParameters: {},
        resolvedDependencies: materials,
      },
      runDetails: { builder: { id: "https://github.com/load28/adoc/.github/workflows/ci.yml" } },
    },
  };
}

export function signAndVerify(value) {
  const bytes = Buffer.from(canonical(value));
  const { privateKey, publicKey } = generateKeyPairSync("ed25519");
  const signature = sign(null, bytes, privateKey);
  if (!verify(null, bytes, publicKey, signature)) throw new Error("provenance signature failed");
  return {
    algorithm: "Ed25519",
    statementDigest: sha256(bytes),
    signature: signature.toString("base64"),
    publicKey: publicKey.export({ format: "der", type: "spki" }).toString("base64"),
  };
}

export function canonical(value) {
  return JSON.stringify(sort(value));
}

function sort(value) {
  if (Array.isArray(value)) return value.map(sort);
  if (value && typeof value === "object")
    return Object.fromEntries(
      Object.keys(value)
        .sort()
        .map((key) => [key, sort(value[key])]),
    );
  return value;
}
