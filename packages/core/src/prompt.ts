import { describeIntent } from "./intent.js";
import type { PromptIR, WritingRequest, WritingRule, DocumentTypeDef } from "./types.js";

/**
 * Prompt Compiler (DESIGN.md §13).
 *
 * Prompt strings are never scattered through the app: a WritingRequest is
 * compiled into a PromptIR here, and Agent Adapters turn the IR into their
 * own invocation. Swapping the agent never touches the Writing Engine.
 */

function renderRules(rules: WritingRule[]): string {
  return rules
    .map((r) => `- [${r.id}] ${r.name}: ${r.description}${r.appliesTo ? ` (applies to: ${r.appliesTo.join(", ")})` : ""}`)
    .join("\n");
}

function renderStructure(typeDef: DocumentTypeDef): string {
  return typeDef.sections
    .map(
      (s) =>
        `- "## ${s.title}" (role: ${s.role}${s.required ? ", required" : ""})${s.guidance ? ` — ${s.guidance}` : ""}`,
    )
    .join("\n");
}

const BASE_SYSTEM = `You are the writing engine of a collaborative team documentation tool.
You help humans write documents that minimize the reader's cognitive load.
You never invent facts that are not present in the provided material.
When information is missing, leave an explicit "TODO:" marker instead of guessing.
Write in the same language the source material is written in.`;

const MARKDOWN_CONTRACT = `Return ONLY the complete Markdown document body (no YAML frontmatter, no commentary before or after).
Start with a single "# <title>" line, then "## <section>" headings exactly matching the required structure.
Omit optional sections you have no material for.`;

const SUGGESTIONS_CONTRACT = `Return ONLY a JSON array of suggestions. Each item:
{"ruleId": "C001" | null, "severity": "info"|"warn"|"error", "section": "<heading or null>", "message": "<what is wrong and why it hurts the reader>", "proposal": "<optional concrete fix>"}
Do not rewrite the document. Analysis only.`;

const MERGE_CONTRACT = `Return ONLY a JSON object:
{"markdown": "<merged document body>", "summary": "<one-line merge summary>", "unresolved": [{"section": "<heading or null>", "reason": "<contradiction you must not decide>"}]}
Merge rules:
- Preserve the meaning and intent of both authors as much as possible.
- Remove duplicated explanations.
- Never add new facts.
- Never pick a side between contradictory decisions: list them under "unresolved" and keep both versions in the markdown, clearly labeled.`;

const INTENT_CONTRACT = `Return ONLY a JSON object:
{"intent": "DESIGN_PROPOSAL"|"DECISION_RECORD"|"PROBLEM_REPORT"|"KNOWLEDGE_NOTE", "problem": string|null, "motivation": string[], "proposal": string|null, "tradeoffs": string[], "evidence": string[], "unknowns": string[]}
Extract only what the notes actually say. Put open questions in "unknowns".`;

export function compilePrompt(request: WritingRequest): PromptIR {
  const sys: string[] = [BASE_SYSTEM];

  sys.push(`\n# Document type: ${request.documentType.name}\nSection structure (information delivery order):\n${renderStructure(request.documentType)}`);

  if (request.rules.length > 0) {
    sys.push(`\n# Writing rules (mandatory)\n${renderRules(request.rules)}`);
  }

  const user: string[] = [];

  switch (request.task) {
    case "compose":
      user.push("Task: compose a new document from the material below, following the section structure and writing rules.");
      break;
    case "rewrite":
      user.push(
        request.target?.length
          ? `Task: rewrite ONLY these sections: ${request.target.join(", ")}. Reproduce every other part of the document byte-for-byte.`
          : "Task: rewrite the document below.",
      );
      if (request.goal) user.push(`Rewrite goal: ${request.goal}`);
      break;
    case "critique":
      user.push("Task: critique the document below against the writing rules. Do not modify it.");
      break;
    case "merge":
      user.push("Task: merge the conflicting versions below into one document.");
      break;
  }

  if (request.intent) {
    user.push(`# Document intent (extracted)\n${describeIntent(request.intent)}`);
  }

  for (const block of request.source) {
    user.push(`# ${block.label ?? block.kind}\n${block.text}`);
  }

  if (request.context.length > 0) {
    user.push(
      "# Related context (read-only reference — do not copy verbatim)\n" +
        request.context.map((c) => `## ${c.label} (${c.kind})\n${c.text}`).join("\n\n"),
    );
  }

  let output: PromptIR["output"];
  switch (request.task) {
    case "critique":
      user.push(SUGGESTIONS_CONTRACT);
      output = "json-suggestions";
      break;
    case "merge":
      user.push(MERGE_CONTRACT);
      output = "json-merge";
      break;
    default:
      user.push(MARKDOWN_CONTRACT);
      output = "markdown-document";
  }

  return { system: sys.join("\n"), user: user.join("\n\n"), output };
}

/** Intent extraction is a pre-step, not a WritingTask (DESIGN.md §7). */
export function compileIntentPrompt(notes: string): PromptIR {
  return {
    system: BASE_SYSTEM,
    user: `Task: analyze the raw notes below and extract their semantic structure.\n\n# Raw notes\n${notes}\n\n${INTENT_CONTRACT}`,
    output: "json-intent",
  };
}

/** Render a PromptIR to a single plain-text prompt (default for CLI agents). */
export function renderPromptText(prompt: PromptIR): string {
  return `${prompt.system}\n\n---\n\n${prompt.user}`;
}
