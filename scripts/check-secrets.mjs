import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const patterns = [
  ["private key", /-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----/u],
  ["AWS access key", /\b(?:AKIA|ASIA)[A-Z0-9]{16}\b/u],
  ["Google API key", /\bAIza[0-9A-Za-z_-]{35}\b/u],
  ["GitHub token", /\bgh[pousr]_[0-9A-Za-z]{30,}\b/u],
  ["OpenAI key", /\bsk-[A-Za-z0-9_-]{20,}\b/u],
];

const output = execFileSync(
  "git",
  ["ls-files", "--cached", "--others", "--exclude-standard", "-z"],
  { cwd: root },
);
const files = output.toString("utf8").split("\0").filter(Boolean);
const findings = [];

for (const file of files) {
  const content = readFileSync(resolve(root, file));
  if (content.includes(0)) continue;
  const text = content.toString("utf8");
  for (const [label, pattern] of patterns) {
    if (pattern.test(text)) findings.push(`${file}: ${label}`);
  }
}

if (findings.length > 0) throw new Error(`Potential secrets found:\n${findings.join("\n")}`);
console.log(`secret pattern scan passed for ${files.length} files`);
