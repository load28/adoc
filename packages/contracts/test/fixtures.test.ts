import { describe, expect, test } from "bun:test";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { validateContract, type ContractName } from "../src/validator";

const root = resolve(import.meta.dir, "../../..");
const cases: Array<[ContractName, boolean]> = [
  ["aiTask", true],
  ["aiTask", false],
  ["content", true],
  ["content", false],
  ["event", true],
  ["event", false],
  ["operation", true],
  ["operation", false],
];

describe("canonical contract fixtures", () => {
  for (const [name, expected] of cases) {
    test(`${name}.${expected ? "valid" : "invalid"}`, () => {
      const path = resolve(
        root,
        `docs/design/quality/fixtures/${name === "aiTask" ? "ai-task" : name}.${expected ? "valid" : "invalid"}.json`,
      );
      const value = JSON.parse(readFileSync(path, "utf8"));
      expect(validateContract(name, value).valid).toBe(expected);
    });
  }
});
