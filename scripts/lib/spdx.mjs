const supportedVersions = new Set(["SPDX-2.2", "SPDX-2.3"]);

export function validateSpdxDocument(value) {
  if (!value || typeof value !== "object") throw new Error("SBOM is not a JSON object");
  if (!supportedVersions.has(value.spdxVersion))
    throw new Error(`unsupported SPDX document version ${value.spdxVersion ?? "missing"}`);
  if (value.SPDXID !== "SPDXRef-DOCUMENT") throw new Error("SBOM document identity is invalid");
  if (
    typeof value.documentNamespace !== "string" ||
    !value.documentNamespace.startsWith("https://")
  )
    throw new Error("SBOM document namespace is invalid");
  if (!Array.isArray(value.packages) || value.packages.length === 0)
    throw new Error("SBOM package inventory is empty");
  return {
    version: value.spdxVersion,
    namespace: value.documentNamespace,
    packages: value.packages.length,
  };
}
