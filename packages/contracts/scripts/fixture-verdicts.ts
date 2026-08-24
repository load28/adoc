import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { validateContract, type ContractName } from "../src/validator";

const root = resolve(import.meta.dir, "../../..");
const names: ContractName[] = ["aiTask", "content", "event", "operation"];
const verdicts: Record<string, boolean> = {};

for (const name of names) {
  for (const kind of ["valid", "invalid"] as const) {
    const file = name === "aiTask" ? "ai-task" : name;
    const value = JSON.parse(
      readFileSync(resolve(root, `docs/design/quality/fixtures/${file}.${kind}.json`), "utf8"),
    );
    verdicts[`${file}.${kind}`] = validateContract(name, value).valid;
  }
}
process.stdout.write(`${JSON.stringify(verdicts)}\n`);
