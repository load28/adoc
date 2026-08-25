import type { ErrorObject, ValidateFunction } from "ajv";
import {
  validateAiTask,
  validateContent,
  validateEvent,
  validateOperation,
} from "./generated/validators.js";

export type ContractName = "aiTask" | "content" | "event" | "operation";

const validators: Record<ContractName, ValidateFunction> = {
  aiTask: validateAiTask,
  content: validateContent,
  event: validateEvent,
  operation: validateOperation,
};

export function validateContract(
  name: ContractName,
  value: unknown,
): { valid: boolean; errors: ErrorObject[] | null | undefined } {
  const validator = validators[name];
  const valid = validator(value);
  if (typeof valid !== "boolean") throw new Error("asynchronous schemas are not supported");
  return { valid, errors: validator.errors };
}
