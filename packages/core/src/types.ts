/**
 * Core domain types for the Git-based AI collaborative document app.
 *
 * These mirror the interfaces in DESIGN.md. Everything here is
 * environment-agnostic: no Node, no browser, no Tauri imports.
 */

// ---------------------------------------------------------------------------
// Identifiers
// ---------------------------------------------------------------------------

export type DocumentId = string;
export type SectionId = string;
export type ProjectId = string;

// ---------------------------------------------------------------------------
// Document type definitions (loaded from .teamdoc/document-types/*.yaml)
// ---------------------------------------------------------------------------

/** The semantic role a section plays inside a document type. */
export type SectionRole = string;

export interface SectionSpec {
  role: SectionRole;
  /** Human heading used in Markdown (e.g. "Trade-offs"). */
  title: string;
  required?: boolean;
  /** Guidance given to the Writing Engine about what belongs here. */
  guidance?: string;
}

export interface DocumentTypeDef {
  id: string;
  name: string;
  /** Ordered section structure — the information delivery order (DESIGN.md §9). */
  sections: SectionSpec[];
}

export type DocumentType = DocumentTypeDef["id"];

// ---------------------------------------------------------------------------
// Document model (DESIGN.md §5)
// ---------------------------------------------------------------------------

export type RelationType =
  | "related_to"
  | "supersedes"
  | "implements"
  | "implemented_by"
  | "motivated_by"
  | "discussed_in"
  | "reverted_by"
  | (string & {});

export interface Relation {
  type: RelationType;
  /**
   * Relation target, e.g.
   *   `document:auth-v1`
   *   `github:load28/app#124`
   */
  target: string;
}

export interface DocumentMetadata {
  id: DocumentId;
  type: DocumentType;
  status?: string;
  authors?: string[];
  relations?: Relation[];
  /** Unknown frontmatter keys are preserved verbatim on round-trip. */
  extra?: Record<string, unknown>;
}

export type EditorContent = string;

export interface DocumentSection {
  id: SectionId;
  role: SectionRole;
  /** Markdown heading as written (may differ in case from the spec title). */
  heading: string;
  content: EditorContent;
}

export interface Document {
  id: DocumentId;
  type: DocumentType;
  title: string;
  metadata: DocumentMetadata;
  sections: DocumentSection[];
}

// ---------------------------------------------------------------------------
// Cognitive / team writing rules (DESIGN.md §8)
// ---------------------------------------------------------------------------

export interface WritingRule {
  id: string;
  name: string;
  description: string;
  /** Restrict a rule to specific section roles; empty = document-wide. */
  appliesTo?: SectionRole[];
  /** Whether the rule ships with the app (cognitive) or comes from the team. */
  origin: "cognitive" | "team";
}

// ---------------------------------------------------------------------------
// Document intent (DESIGN.md §7)
// ---------------------------------------------------------------------------

export type IntentKind =
  | "DESIGN_PROPOSAL"
  | "DECISION_RECORD"
  | "PROBLEM_REPORT"
  | "KNOWLEDGE_NOTE"
  | (string & {});

export interface DocumentIntent {
  intent: IntentKind;
  problem?: string;
  motivation?: string[];
  proposal?: string;
  tradeoffs?: string[];
  evidence?: string[];
  unknowns?: string[];
}

// ---------------------------------------------------------------------------
// Writing requests → Prompt IR (DESIGN.md §13)
// ---------------------------------------------------------------------------

export type WritingTask = "compose" | "rewrite" | "critique" | "merge";

export interface ContentBlock {
  kind: "notes" | "document" | "section" | "conflict" | "instruction";
  label?: string;
  text: string;
}

export interface ContextItem {
  kind: "document" | "decision" | "external" | "rule-note";
  label: string;
  text: string;
}

export interface WritingRequest {
  task: WritingTask;
  documentType: DocumentTypeDef;
  intent?: DocumentIntent;
  /** The primary material the task operates on. */
  source: ContentBlock[];
  /** Related information retrieved for the task (DESIGN.md §21). */
  context: ContextItem[];
  rules: WritingRule[];
  /** Restrict the task to specific sections (rewrite only those). */
  target?: SectionId[];
  /** Free-form user goal, e.g. "더 간결하게". */
  goal?: string;
}

/** Intermediate representation the Prompt Compiler produces. */
export interface PromptIR {
  system: string;
  user: string;
  /** Contract describing what the agent must return. */
  output: "markdown-document" | "json-intent" | "json-suggestions" | "json-merge";
}

// ---------------------------------------------------------------------------
// Agent adapters (DESIGN.md §12)
// ---------------------------------------------------------------------------

export interface AgentCapabilities {
  id: string;
  displayName: string;
  streaming: boolean;
}

export interface AgentRequest {
  prompt: PromptIR;
}

export interface AgentContext {
  /** Workspace root, used by CLI agents for cwd. */
  workspaceDir?: string;
  signal?: AbortSignal;
}

export type AgentEvent =
  | { type: "started" }
  | { type: "text"; text: string }
  | { type: "completed"; text: string }
  | { type: "error"; message: string };

export interface AgentAdapter {
  capabilities(): AgentCapabilities;
  execute(request: AgentRequest, context: AgentContext): AsyncIterable<AgentEvent>;
}

// ---------------------------------------------------------------------------
// Proposals — AI changes are always suggestions (DESIGN.md §11)
// ---------------------------------------------------------------------------

export interface Proposal {
  task: WritingTask;
  /** Serialized markdown before the change ("" for compose). */
  before: string;
  /** Serialized markdown the AI proposes. */
  after: string;
  /** Unified diff between before and after. */
  diff: string;
  summary?: string;
}

export interface Suggestion {
  ruleId?: string;
  severity: "info" | "warn" | "error";
  section?: string;
  message: string;
  proposal?: string;
}

export interface MergeConflictInput {
  path: string;
  base: string;
  current: string;
  incoming: string;
}

export interface MergeProposal extends Proposal {
  /** Contradictions the AI could not resolve — surfaced, never decided (§16). */
  unresolved: { section?: string; reason: string }[];
}

// ---------------------------------------------------------------------------
// Workspace (DESIGN.md §4)
// ---------------------------------------------------------------------------

export interface WorkspaceConfig {
  name: string;
  /** Default agent id (mock | claude | codex). */
  agent?: string;
}

export interface ProjectRef {
  id: ProjectId;
  name: string;
  /** Workspace-relative path, e.g. `projects/compiler`. */
  path: string;
}

export interface DocumentRef {
  id: DocumentId;
  title: string;
  type: DocumentType;
  status?: string;
  /** Workspace-relative path, e.g. `projects/compiler/documents/parser.md`. */
  path: string;
  project?: ProjectId;
}
