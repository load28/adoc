import Ajv2020, { type ErrorObject } from "ajv/dist/2020.js";
import addFormats from "ajv-formats";
import bundle from "./generated/contract-bundle.schema.json";

export type ContractName = "aiTask" | "content" | "event" | "operation";

const definitions: Record<ContractName, string> = {
  aiTask: "AiContracts__task",
  content: "DocumentContent",
  event: "EventPayloads",
  operation: "DocumentOperation",
};

const ajv = new Ajv2020({ allErrors: true, strict: true, strictTypes: false });
addFormats(ajv);
ajv.addSchema(bundle);

export function validateContract(
  name: ContractName,
  value: unknown,
): { valid: boolean; errors: ErrorObject[] | null | undefined } {
  const validator = ajv.getSchema(`${bundle.$id}#/$defs/${definitions[name]}`);
  if (!validator) throw new Error(`missing contract definition: ${definitions[name]}`);
  const valid = validator(value);
  if (typeof valid !== "boolean") throw new Error("asynchronous schemas are not supported");
  return { valid, errors: validator.errors };
}
