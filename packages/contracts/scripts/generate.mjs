import { createHash } from "node:crypto";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import { basename, resolve } from "node:path";
import { compile } from "json-schema-to-typescript";
import openapiTS, { astToString } from "openapi-typescript";
import { parse } from "yaml";

const root = resolve(import.meta.dirname, "../../..");
const selfTest = process.argv.includes("--self-test");
const outputArgument = process.argv.slice(2).find((argument) => !argument.startsWith("--"));
const output = resolve(outputArgument ?? "packages/contracts/src/generated");
const sources = [
  "docs/design/api/openapi.yaml",
  "docs/design/api/asyncapi.yaml",
  "docs/design/contracts/ai-contracts.schema.json",
  "docs/design/contracts/document-content.schema.json",
  "docs/design/contracts/document-operation.schema.json",
  "docs/design/contracts/event-payloads.schema.json",
];
const contractNames = new Map([
  ["ai-contracts.schema.json", "AiContracts"],
  ["document-content.schema.json", "DocumentContent"],
  ["document-operation.schema.json", "DocumentOperation"],
  ["event-payloads.schema.json", "EventPayloads"],
]);

const textBySource = new Map(
  await Promise.all(
    sources.map(async (path) => [path, await readFile(resolve(root, path), "utf8")]),
  ),
);
const openapi = parse(textBySource.get(sources[0]));
const asyncapi = parse(textBySource.get(sources[1]));

function pointer(document, reference) {
  if (!reference.startsWith("#/")) throw new Error(`unsupported local reference: ${reference}`);
  return reference
    .slice(2)
    .split("/")
    .reduce(
      (value, segment) => value?.[segment.replaceAll("~1", "/").replaceAll("~0", "~")],
      document,
    );
}

function resolveObject(document, value) {
  return value?.$ref?.startsWith("#/") ? pointer(document, value.$ref) : value;
}

function rewriteReference(reference, namespace, dialect) {
  if (/^https?:/u.test(reference)) throw new Error(`network reference is forbidden: ${reference}`);
  if (reference.startsWith("#/$defs/")) return `#/$defs/${namespace}__${reference.slice(8)}`;
  if (dialect === "openapi" && reference.startsWith("#/components/schemas/")) {
    return `#/$defs/OpenApi__${reference.slice(21)}`;
  }
  const [file, fragment = ""] = reference.split("#", 2);
  const target = contractNames.get(basename(file));
  if (target)
    return fragment.startsWith("/$defs/")
      ? `#/$defs/${target}__${fragment.slice(7)}`
      : `#/$defs/${target}`;
  if (reference.startsWith("#")) throw new Error(`unsupported fragment reference: ${reference}`);
  throw new Error(`reference leaves canonical source set: ${reference}`);
}

if (selfTest) {
  if (rewriteReference("#/$defs/block", "DocumentContent") !== "#/$defs/DocumentContent__block") {
    throw new Error("local fragment normalization failed");
  }
  try {
    rewriteReference("https://example.com/schema.json", "External");
    throw new Error("network reference was accepted");
  } catch (error) {
    if (!String(error.message).includes("network reference is forbidden")) throw error;
  }
  console.log("contract reference self-test passed");
  process.exit(0);
}

function transform(value, namespace, dialect = "contract") {
  if (Array.isArray(value)) return value.map((item) => transform(item, namespace, dialect));
  if (!value || typeof value !== "object") return value;
  const result = {};
  for (const [key, item] of Object.entries(value)) {
    if (key === "$defs" || key === "$id" || key === "$schema" || key.startsWith("x-")) continue;
    result[key] =
      key === "$ref"
        ? rewriteReference(item, namespace, dialect)
        : transform(item, namespace, dialect);
  }
  return result;
}

const defs = {};
for (const path of sources.slice(2)) {
  const schema = JSON.parse(textBySource.get(path));
  const namespace = contractNames.get(basename(path));
  defs[namespace] = transform(schema, namespace);
  for (const [name, definition] of Object.entries(schema.$defs ?? {})) {
    defs[`${namespace}__${name}`] = transform(definition, namespace);
  }
}
for (const [name, schema] of Object.entries(openapi.components?.schemas ?? {})) {
  defs[`OpenApi__${name}`] = transform(schema, "OpenApi", "openapi");
}

function parameterContainer(parameters) {
  const groups = new Map();
  for (const raw of parameters) {
    const parameter = resolveObject(openapi, raw);
    if (!parameter?.name || !parameter?.in || !parameter.schema) continue;
    const group = groups.get(parameter.in) ?? {
      type: "object",
      additionalProperties: false,
      properties: {},
      required: [],
    };
    group.properties[parameter.name] = transform(parameter.schema, "OpenApi", "openapi");
    if (parameter.required) group.required.push(parameter.name);
    groups.set(parameter.in, group);
  }
  return groups;
}

function mediaSchema(content) {
  const media = content?.["application/json"] ?? Object.values(content ?? {})[0];
  return media?.schema ? transform(media.schema, "OpenApi", "openapi") : undefined;
}

const operations = [];
for (const [path, pathItem] of Object.entries(openapi.paths ?? {})) {
  for (const method of ["get", "post", "put", "patch", "delete", "options", "head", "trace"]) {
    const operation = pathItem[method];
    if (!operation?.operationId) continue;
    const typeName = operation.operationId[0].toUpperCase() + operation.operationId.slice(1);
    const request = { type: "object", additionalProperties: false, properties: {}, required: [] };
    const groups = parameterContainer([
      ...(pathItem.parameters ?? []),
      ...(operation.parameters ?? []),
    ]);
    for (const [location, schema] of groups) {
      if (!schema.required.length) delete schema.required;
      request.properties[location] = schema;
      if (
        [...(pathItem.parameters ?? []), ...(operation.parameters ?? [])].some((raw) => {
          const parameter = resolveObject(openapi, raw);
          return parameter?.in === location && parameter.required;
        })
      )
        request.required.push(location);
    }
    const requestBody = resolveObject(openapi, operation.requestBody);
    const body = mediaSchema(requestBody?.content);
    if (body) {
      request.properties.body = body;
      if (requestBody.required) request.required.push("body");
    }
    if (!request.required.length) delete request.required;
    defs[`Operation__${typeName}Request`] = request;

    const responses = [];
    for (const [status, raw] of Object.entries(operation.responses ?? {})) {
      const response = resolveObject(openapi, raw);
      const bodySchema = mediaSchema(response?.content);
      const wrapper = {
        type: "object",
        additionalProperties: false,
        required: ["status"],
        properties: { status: { const: status } },
      };
      if (bodySchema) {
        wrapper.required.push("body");
        wrapper.properties.body = bodySchema;
      }
      responses.push(wrapper);
    }
    defs[`Operation__${typeName}Response`] = responses.length
      ? { oneOf: responses }
      : { type: "null" };
    operations.push({
      operationId: operation.operationId,
      method: method.toUpperCase(),
      path,
      request: `Operation__${typeName}Request`,
      response: `Operation__${typeName}Response`,
    });
  }
}

for (const [name, schema] of Object.entries(asyncapi.components?.schemas ?? {})) {
  defs[`AsyncApi__${name}`] = transform(schema, "AsyncApi");
}
const asyncMessages = [];
for (const [name, message] of Object.entries(asyncapi.components?.messages ?? {})) {
  const headers = message.headers?.$ref?.split("/").at(-1);
  defs[`AsyncApi__${name}`] = {
    type: "object",
    additionalProperties: false,
    required: ["headers", "payload"],
    properties: {
      headers: { $ref: `#/$defs/AsyncApi__${headers}` },
      payload: { $ref: "#/$defs/EventPayloads" },
    },
  };
  asyncMessages.push(name);
}

const bundle = {
  $schema: "https://json-schema.org/draft/2020-12/schema",
  $id: "https://adoc.local/generated/contract-bundle.schema.json",
  title: "AdocContractBundle",
  oneOf: Object.keys(defs).map((name) => ({ $ref: `#/$defs/${name}` })),
  $defs: defs,
};
const hashes = Object.fromEntries(
  sources.map((path) => [path, createHash("sha256").update(textBySource.get(path)).digest("hex")]),
);
const manifest = {
  generatorVersion: 1,
  tools: {
    "json-schema-to-typescript": "15.0.4",
    "openapi-typescript": "7.13.0",
    yaml: "2.9.0",
    typify: "0.7.0",
  },
  sources: hashes,
  counts: {
    openapiOperations: operations.length,
    asyncapiOperations: Object.keys(asyncapi.operations ?? {}).length,
    asyncapiMessages: asyncMessages.length,
    definitions: Object.keys(defs).length,
  },
};

await mkdir(output, { recursive: true });
await writeFile(
  resolve(output, "contract-bundle.schema.json"),
  `${JSON.stringify(bundle, null, 2)}\n`,
);
await writeFile(resolve(output, "manifest.json"), `${JSON.stringify(manifest, null, 2)}\n`);
await writeFile(
  resolve(output, "contracts.ts"),
  await compile(bundle, "AdocContractBundle", {
    bannerComment: "/* Generated from canonical schemas. Do not edit. */",
    style: { singleQuote: false },
    unreachableDefinitions: true,
  }),
);
const openapiAst = await openapiTS(new URL(`file://${resolve(root, sources[0])}`));
await writeFile(
  resolve(output, "openapi.ts"),
  `/* Generated from canonical OpenAPI. Do not edit. */\n${astToString(openapiAst)}`,
);
const operationSource = `/* Generated from canonical OpenAPI and AsyncAPI. Do not edit. */\nexport const operations = ${JSON.stringify(operations, null, 2)} as const;\nexport type OperationId = typeof operations[number]["operationId"];\nexport const asyncMessages = ${JSON.stringify(asyncMessages)} as const;\n`;
await writeFile(resolve(output, "operations.ts"), operationSource);
