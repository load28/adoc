import { splitFrontmatter, joinFrontmatter } from "./frontmatter.js";
import type {
  Document,
  DocumentMetadata,
  DocumentSection,
  DocumentTypeDef,
  Relation,
  SectionSpec,
} from "./types.js";

// ---------------------------------------------------------------------------
// Built-in document types (DESIGN.md §5, §9)
// ---------------------------------------------------------------------------

export const DESIGN_TYPE: DocumentTypeDef = {
  id: "design",
  name: "Design",
  sections: [
    { role: "summary", title: "Summary", required: true, guidance: "핵심 결론과 문서의 목적을 먼저 제시한다." },
    { role: "problem", title: "Problem", required: true, guidance: "해결하려는 문제를 구현 세부사항보다 먼저 설명한다." },
    { role: "context", title: "Context", guidance: "문제를 이해하는 데 필요한 배경." },
    { role: "constraints", title: "Constraints", guidance: "설계가 지켜야 하는 제약." },
    { role: "proposal", title: "Proposal", required: true, guidance: "제안하는 해결 방향." },
    { role: "architecture", title: "Architecture", guidance: "구조와 구성요소." },
    { role: "alternatives", title: "Alternatives", guidance: "검토한 대안과 각 대안을 선택하지 않은 이유." },
    { role: "tradeoffs", title: "Trade-offs", guidance: "제안이 감수하는 비용." },
    { role: "decision", title: "Decision", guidance: "결정과 그 이유." },
    { role: "consequences", title: "Consequences", guidance: "결정이 가져오는 결과." },
  ],
};

export const PROPOSAL_TYPE: DocumentTypeDef = {
  id: "proposal",
  name: "Proposal",
  sections: [
    { role: "proposal", title: "Proposal", required: true, guidance: "무엇을 하자는 것인지 결론부터." },
    { role: "motivation", title: "Motivation", required: true, guidance: "왜 지금 필요한가." },
    { role: "benefit", title: "Expected Benefit", guidance: "기대 효과." },
    { role: "cost", title: "Cost / Risk", guidance: "비용과 위험." },
    { role: "alternatives", title: "Alternatives", guidance: "대안과 기각 이유." },
    { role: "next-step", title: "Next Step", guidance: "다음 행동." },
  ],
};

export const DECISION_TYPE: DocumentTypeDef = {
  id: "decision",
  name: "Decision",
  sections: [
    { role: "decision", title: "Decision", required: true, guidance: "무엇을 결정했는가." },
    { role: "rationale", title: "Rationale", required: true, guidance: "결정의 이유 — Decision에는 반드시 근거가 있어야 한다 (C006)." },
    { role: "alternatives", title: "Alternatives Considered", guidance: "검토한 대안과 기각 이유 (C007)." },
    { role: "tradeoffs", title: "Accepted Trade-offs", guidance: "감수하기로 한 비용." },
    { role: "consequences", title: "Consequences", guidance: "결정의 결과." },
  ],
};

export const BUILTIN_TYPES: DocumentTypeDef[] = [DESIGN_TYPE, PROPOSAL_TYPE, DECISION_TYPE];

// ---------------------------------------------------------------------------
// Type registry — extensible without touching the Writing Engine (§9)
// ---------------------------------------------------------------------------

export class DocumentTypeRegistry {
  private types = new Map<string, DocumentTypeDef>();

  constructor(defs: DocumentTypeDef[] = BUILTIN_TYPES) {
    for (const def of defs) this.register(def);
  }

  register(def: DocumentTypeDef): void {
    this.types.set(def.id, def);
  }

  get(id: string): DocumentTypeDef | undefined {
    return this.types.get(id);
  }

  require(id: string): DocumentTypeDef {
    const def = this.get(id);
    if (!def) throw new Error(`Unknown document type: ${id}`);
    return def;
  }

  all(): DocumentTypeDef[] {
    return [...this.types.values()];
  }
}

/** Parse a document-type YAML file's already-parsed data into a def. */
export function documentTypeFromData(data: Record<string, unknown>): DocumentTypeDef {
  const id = String(data.id ?? "");
  if (!id) throw new Error("document type yaml requires `id`");
  const rawSections = Array.isArray(data.sections) ? data.sections : [];
  const sections: SectionSpec[] = rawSections.map((s) => {
    const sec = s as Record<string, unknown>;
    return {
      role: String(sec.role ?? slugify(String(sec.title ?? ""))),
      title: String(sec.title ?? sec.role ?? ""),
      required: Boolean(sec.required ?? false),
      guidance: sec.guidance ? String(sec.guidance) : undefined,
    };
  });
  return { id, name: String(data.name ?? id), sections };
}

// ---------------------------------------------------------------------------
// Markdown ⇄ Document (open format: Markdown + Frontmatter, §2.2)
// ---------------------------------------------------------------------------

export function slugify(text: string): string {
  return (
    text
      .trim()
      .toLowerCase()
      // keep unicode letters/digits so Korean headings survive
      .replace(/[^\p{L}\p{N}]+/gu, "-")
      .replace(/^-+|-+$/g, "") || "section"
  );
}

function parseRelations(value: unknown): Relation[] | undefined {
  if (!Array.isArray(value)) return undefined;
  const rels: Relation[] = [];
  for (const item of value) {
    if (item && typeof item === "object") {
      const rec = item as Record<string, unknown>;
      if (rec.type && rec.target) rels.push({ type: String(rec.type), target: String(rec.target) });
    }
  }
  return rels;
}

const KNOWN_KEYS = new Set(["id", "type", "status", "authors", "relations"]);

export function metadataFromFrontmatter(data: Record<string, unknown>): DocumentMetadata {
  const extra: Record<string, unknown> = {};
  for (const [k, v] of Object.entries(data)) {
    if (!KNOWN_KEYS.has(k)) extra[k] = v;
  }
  return {
    id: String(data.id ?? ""),
    type: String(data.type ?? ""),
    status: data.status !== undefined ? String(data.status) : undefined,
    authors: Array.isArray(data.authors) ? data.authors.map(String) : undefined,
    relations: parseRelations(data.relations),
    extra: Object.keys(extra).length > 0 ? extra : undefined,
  };
}

export function frontmatterFromMetadata(meta: DocumentMetadata): Record<string, unknown> {
  const data: Record<string, unknown> = { id: meta.id, type: meta.type };
  if (meta.status !== undefined) data.status = meta.status;
  if (meta.authors) data.authors = meta.authors;
  if (meta.relations) data.relations = meta.relations.map((r) => ({ type: r.type, target: r.target }));
  if (meta.extra) Object.assign(data, meta.extra);
  return data;
}

/**
 * Parse Markdown into the semantic Document model.
 *
 * Layout convention:
 *   `# Title`          → document title
 *   `## Heading` ...   → one section each; heading is matched (case-insensitive)
 *                        against the type's section titles to recover its role.
 * Unknown headings become `custom` sections — free-form content is never lost.
 */
export function parseDocument(source: string, registry: DocumentTypeRegistry): Document {
  const { data, body } = splitFrontmatter(source);
  const metadata = metadataFromFrontmatter(data);
  const typeDef = registry.get(metadata.type);

  const roleByTitle = new Map<string, string>();
  for (const spec of typeDef?.sections ?? []) {
    roleByTitle.set(spec.title.trim().toLowerCase(), spec.role);
  }

  const lines = body.split(/\r?\n/);
  let title = "";
  const sections: DocumentSection[] = [];
  let currentHeading: string | null = null;
  let buffer: string[] = [];
  let inFence = false;

  const flush = () => {
    if (currentHeading === null && buffer.join("").trim() === "") {
      buffer = [];
      return;
    }
    const heading = currentHeading ?? "Preamble";
    const role =
      currentHeading === null
        ? "preamble"
        : roleByTitle.get(heading.trim().toLowerCase()) ?? "custom";
    sections.push({
      id: slugify(heading),
      role,
      heading,
      content: buffer.join("\n").replace(/^\n+/, "").replace(/\s+$/, ""),
    });
    buffer = [];
  };

  for (const line of lines) {
    if (/^(```|~~~)/.test(line.trim())) inFence = !inFence;
    if (!inFence && /^#\s+/.test(line) && title === "") {
      title = line.replace(/^#\s+/, "").trim();
      continue;
    }
    if (!inFence && /^##\s+/.test(line)) {
      flush();
      currentHeading = line.replace(/^##\s+/, "").trim();
      continue;
    }
    buffer.push(line);
  }
  flush();

  return {
    id: metadata.id,
    type: metadata.type,
    title,
    metadata,
    sections,
  };
}

/** Serialize the Document model back to Markdown + Frontmatter. */
export function serializeDocument(doc: Document): string {
  const parts: string[] = [`# ${doc.title}`.trimEnd()];
  for (const section of doc.sections) {
    if (section.role === "preamble") {
      parts.push(section.content);
      continue;
    }
    parts.push(`## ${section.heading}`, section.content);
  }
  const body = parts.filter((p) => p !== "").join("\n\n") + "\n";
  return joinFrontmatter(frontmatterFromMetadata(doc.metadata), body);
}

/** Create an empty skeleton document for a type. */
export function createSkeleton(
  typeDef: DocumentTypeDef,
  init: { id: string; title: string; authors?: string[]; status?: string },
): Document {
  return {
    id: init.id,
    type: typeDef.id,
    title: init.title,
    metadata: {
      id: init.id,
      type: typeDef.id,
      status: init.status ?? "draft",
      authors: init.authors,
      relations: [],
    },
    sections: typeDef.sections.map((spec) => ({
      id: slugify(spec.title),
      role: spec.role,
      heading: spec.title,
      content: "",
    })),
  };
}
