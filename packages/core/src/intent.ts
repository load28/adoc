import type { DocumentIntent } from "./types.js";

/**
 * Document Intent utilities (DESIGN.md §7).
 *
 * Raw user thoughts are never handed to the LLM directly: the engine first
 * asks the agent to extract a semantic structure, parsed here defensively —
 * agent output may include prose around the JSON payload.
 */

export function extractJson(text: string): unknown {
  // Prefer fenced blocks, then fall back to the first balanced object/array.
  const fence = text.match(/```(?:json)?\s*([\s\S]*?)```/);
  const candidates: string[] = [];
  if (fence) candidates.push(fence[1]);
  const start = text.search(/[[{]/);
  if (start !== -1) candidates.push(balancedFrom(text, start));
  for (const candidate of candidates) {
    try {
      return JSON.parse(candidate);
    } catch {
      // try next candidate
    }
  }
  throw new Error("agent response did not contain valid JSON");
}

function balancedFrom(text: string, start: number): string {
  const open = text[start];
  const close = open === "{" ? "}" : "]";
  let depth = 0;
  let inString = false;
  for (let i = start; i < text.length; i++) {
    const ch = text[i];
    if (inString) {
      if (ch === "\\") i++;
      else if (ch === '"') inString = false;
      continue;
    }
    if (ch === '"') inString = true;
    else if (ch === open) depth++;
    else if (ch === close) {
      depth--;
      if (depth === 0) return text.slice(start, i + 1);
    }
  }
  return text.slice(start);
}

function toStringArray(value: unknown): string[] | undefined {
  if (value === undefined || value === null) return undefined;
  if (Array.isArray(value)) return value.map(String).filter((s) => s.trim() !== "");
  const s = String(value).trim();
  return s === "" ? undefined : [s];
}

export function parseIntent(raw: unknown): DocumentIntent {
  if (!raw || typeof raw !== "object" || Array.isArray(raw)) {
    throw new Error("intent payload must be a JSON object");
  }
  const rec = raw as Record<string, unknown>;
  return {
    intent: String(rec.intent ?? "KNOWLEDGE_NOTE"),
    problem: rec.problem !== undefined && rec.problem !== null ? String(rec.problem) : undefined,
    motivation: toStringArray(rec.motivation),
    proposal: rec.proposal !== undefined && rec.proposal !== null ? String(rec.proposal) : undefined,
    tradeoffs: toStringArray(rec.tradeoffs ?? rec.tradeoff),
    evidence: toStringArray(rec.evidence),
    unknowns: toStringArray(rec.unknowns ?? rec.unknown),
  };
}

export function describeIntent(intent: DocumentIntent): string {
  const lines: string[] = [`Intent: ${intent.intent}`];
  if (intent.problem) lines.push(`Problem: ${intent.problem}`);
  if (intent.motivation?.length) lines.push(`Motivation:\n${intent.motivation.map((m) => `  - ${m}`).join("\n")}`);
  if (intent.proposal) lines.push(`Proposal: ${intent.proposal}`);
  if (intent.tradeoffs?.length) lines.push(`Tradeoffs:\n${intent.tradeoffs.map((t) => `  - ${t}`).join("\n")}`);
  if (intent.evidence?.length) lines.push(`Evidence:\n${intent.evidence.map((e) => `  - ${e}`).join("\n")}`);
  if (intent.unknowns?.length) lines.push(`Unknowns:\n${intent.unknowns.map((u) => `  - ${u}`).join("\n")}`);
  return lines.join("\n");
}
