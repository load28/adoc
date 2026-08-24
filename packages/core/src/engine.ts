import { runAgent } from "./agents.js";
import { unifiedDiff } from "./diff.js";
import {
  DocumentTypeRegistry,
  parseDocument,
  serializeDocument,
  frontmatterFromMetadata,
} from "./document.js";
import { joinFrontmatter, splitFrontmatter } from "./frontmatter.js";
import { extractJson, parseIntent } from "./intent.js";
import { compileIntentPrompt, compilePrompt } from "./prompt.js";
import { rulesForRoles } from "./rules.js";
import type {
  AgentAdapter,
  AgentContext,
  ContextItem,
  Document,
  DocumentIntent,
  DocumentMetadata,
  MergeConflictInput,
  MergeProposal,
  Proposal,
  Suggestion,
  WritingRequest,
  WritingRule,
} from "./types.js";

/**
 * Writing Engine (DESIGN.md §6, §10).
 *
 * The engine decides WHAT is sent in WHICH structure; the LLM only renders
 * the actual wording. AI roles are separated (Composer / Rewriter / Critic /
 * Merger) and every result is a Proposal — the engine never writes to the
 * working tree, never commits, never pushes (§11).
 */

export interface EngineOptions {
  adapter: AgentAdapter;
  registry: DocumentTypeRegistry;
  rules: WritingRule[];
  agentContext?: AgentContext;
  onText?: (chunk: string) => void;
}

export interface ComposeInput {
  notes: string;
  typeId: string;
  init: { id: string; title?: string; authors?: string[] };
  context?: ContextItem[];
}

export interface ComposeResult {
  intent: DocumentIntent;
  document: Document;
  proposal: Proposal;
}

export interface RewriteInput {
  document: Document;
  goal: string;
  /** Section ids to rewrite; others must be preserved. */
  target?: string[];
  context?: ContextItem[];
}

export interface CritiqueInput {
  document: Document;
  context?: ContextItem[];
}

export class WritingEngine {
  constructor(private opts: EngineOptions) {}

  private ctx(): AgentContext {
    return this.opts.agentContext ?? {};
  }

  // -- Intent analysis (§7) -------------------------------------------------

  async extractIntent(notes: string): Promise<DocumentIntent> {
    const prompt = compileIntentPrompt(notes);
    const text = await runAgent(this.opts.adapter, { prompt }, this.ctx(), this.opts.onText);
    return parseIntent(extractJson(text));
  }

  // -- Composer (§10) -------------------------------------------------------

  async compose(input: ComposeInput): Promise<ComposeResult> {
    const typeDef = this.opts.registry.require(input.typeId);
    const intent = await this.extractIntent(input.notes);

    const request: WritingRequest = {
      task: "compose",
      documentType: typeDef,
      intent,
      source: [{ kind: "notes", label: "Raw notes", text: input.notes }],
      context: input.context ?? [],
      rules: this.opts.rules,
    };
    const body = await this.run(request);

    const metadata: DocumentMetadata = {
      id: input.init.id,
      type: typeDef.id,
      status: "draft",
      authors: input.init.authors,
      relations: [],
    };
    const source = joinFrontmatter(frontmatterFromMetadata(metadata), sanitizeBody(body));
    const document = parseDocument(source, this.opts.registry);
    if (input.init.title && !document.title) document.title = input.init.title;

    const after = serializeDocument(document);
    return {
      intent,
      document,
      proposal: {
        task: "compose",
        before: "",
        after,
        diff: unifiedDiff(`${metadata.id}.md`, "", after),
        summary: `Compose ${typeDef.name} "${document.title || input.init.title || metadata.id}"`,
      },
    };
  }

  // -- Rewriter (§10) -------------------------------------------------------

  async rewrite(input: RewriteInput): Promise<Proposal> {
    const typeDef = this.opts.registry.require(input.document.type);
    const before = serializeDocument(input.document);
    const targetRoles = input.target
      ? input.document.sections.filter((s) => input.target!.includes(s.id)).map((s) => s.role)
      : undefined;

    const request: WritingRequest = {
      task: "rewrite",
      documentType: typeDef,
      source: [{ kind: "document", label: "Current document", text: bodyOf(before) }],
      context: input.context ?? [],
      rules: rulesForRoles(this.opts.rules, targetRoles),
      target: input.target,
      goal: input.goal,
    };
    const body = await this.run(request);

    const source = joinFrontmatter(
      frontmatterFromMetadata(input.document.metadata),
      sanitizeBody(body),
    );
    const rewritten = parseDocument(source, this.opts.registry);
    const after = serializeDocument(rewritten);
    return {
      task: "rewrite",
      before,
      after,
      diff: unifiedDiff(`${input.document.id}.md`, before, after),
      summary: input.goal,
    };
  }

  // -- Critic (§10) — analyzes, never edits ---------------------------------

  async critique(input: CritiqueInput): Promise<Suggestion[]> {
    const typeDef = this.opts.registry.require(input.document.type);
    const request: WritingRequest = {
      task: "critique",
      documentType: typeDef,
      source: [
        { kind: "document", label: "Document under review", text: bodyOf(serializeDocument(input.document)) },
      ],
      context: input.context ?? [],
      rules: this.opts.rules,
    };
    const text = await this.run(request);
    const raw = extractJson(text);
    if (!Array.isArray(raw)) throw new Error("critique response must be a JSON array");
    return raw.map(toSuggestion);
  }

  // -- AI Merge (§16) — proposes, never resolves ----------------------------

  async merge(conflict: MergeConflictInput, typeId?: string): Promise<MergeProposal> {
    const { data: currentFm } = splitFrontmatter(conflict.current);
    const resolvedTypeId = typeId ?? String(currentFm.type ?? "design");
    const typeDef =
      this.opts.registry.get(resolvedTypeId) ?? this.opts.registry.require("design");

    const request: WritingRequest = {
      task: "merge",
      documentType: typeDef,
      source: [
        { kind: "conflict", label: "BASE (common ancestor)", text: bodyOf(conflict.base) },
        { kind: "conflict", label: "CURRENT (yours)", text: bodyOf(conflict.current) },
        { kind: "conflict", label: "INCOMING (theirs)", text: bodyOf(conflict.incoming) },
      ],
      context: [],
      rules: this.opts.rules,
    };
    const text = await this.run(request);
    const raw = extractJson(text) as Record<string, unknown>;
    const mergedBody = String(raw.markdown ?? "");
    if (!mergedBody.trim()) throw new Error("merge response did not contain markdown");

    const unresolved = Array.isArray(raw.unresolved)
      ? raw.unresolved.map((u) => {
          const rec = u as Record<string, unknown>;
          return {
            section: rec.section ? String(rec.section) : undefined,
            reason: String(rec.reason ?? ""),
          };
        })
      : [];

    // Frontmatter is not merged by the AI: keep CURRENT's metadata.
    const after = joinFrontmatter(currentFm, sanitizeBody(mergedBody));
    return {
      task: "merge",
      before: conflict.current,
      after,
      diff: unifiedDiff(conflict.path, conflict.current, after),
      summary: raw.summary ? String(raw.summary) : undefined,
      unresolved,
    };
  }

  // --------------------------------------------------------------------------

  private async run(request: WritingRequest): Promise<string> {
    const prompt = compilePrompt(request);
    return runAgent(this.opts.adapter, { prompt }, this.ctx(), this.opts.onText);
  }
}

/** Strip frontmatter so prompts always carry the body only. */
function bodyOf(source: string): string {
  return splitFrontmatter(source).body;
}

/** Agents sometimes wrap output in a fence or echo frontmatter — strip both. */
function sanitizeBody(body: string): string {
  let text = body.trim();
  const fenced = text.match(/^```(?:markdown|md)?\r?\n([\s\S]*?)\r?\n```$/);
  if (fenced) text = fenced[1].trim();
  const { body: withoutFm } = splitFrontmatter(text);
  return withoutFm.trim() + "\n";
}

function toSuggestion(raw: unknown): Suggestion {
  const rec = (raw ?? {}) as Record<string, unknown>;
  const severity = ["info", "warn", "error"].includes(String(rec.severity))
    ? (String(rec.severity) as Suggestion["severity"])
    : "info";
  return {
    ruleId: rec.ruleId ? String(rec.ruleId) : undefined,
    severity,
    section: rec.section ? String(rec.section) : undefined,
    message: String(rec.message ?? ""),
    proposal: rec.proposal ? String(rec.proposal) : undefined,
  };
}
