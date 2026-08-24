import {
  Workspace,
  joinPath,
  splitFrontmatter,
  metadataFromFrontmatter,
  TEAMDOC_DIR,
  type ContextItem,
  type Relation,
} from "@adoc/core";

/**
 * Local Index (DESIGN.md §20).
 *
 * The Git repository is the Source of Truth; this index is a disposable
 * cache persisted under `.teamdoc/cache/` (gitignored). Deleting it and
 * rebuilding from the repository must always reproduce the same state:
 *
 *   delete index → scan repository → rebuild
 *
 * It feeds Search, the Document Graph (§18) and AI Context Retrieval (§21).
 */

export interface IndexedDocument {
  id: string;
  title: string;
  type: string;
  status?: string;
  path: string;
  project?: string;
  authors: string[];
  headings: string[];
  relations: Relation[];
  /** Plain lowercased text used for naive scoring. */
  text: string;
  /** First ~400 chars of body — used as document summary in AI context. */
  excerpt: string;
}

export interface GraphEdge {
  /** Document id the relation starts from. */
  from: string;
  /** Raw relation target (`document:x`, `github:owner/repo#1`, bare id). */
  target: string;
  /** Resolved internal document id, when the target is a document we know. */
  to?: string;
  type: string;
  external: boolean;
}

export interface WorkspaceIndex {
  version: 1;
  builtAt: string;
  documents: IndexedDocument[];
  edges: GraphEdge[];
}

export interface SearchHit {
  document: IndexedDocument;
  score: number;
  snippet: string;
}

const CACHE_PATH = `${TEAMDOC_DIR}/cache/index.json`;

// ---------------------------------------------------------------------------
// Building
// ---------------------------------------------------------------------------

export async function buildIndex(workspace: Workspace): Promise<WorkspaceIndex> {
  const documents: IndexedDocument[] = [];

  for (const rel of await workspace.listDocumentPaths()) {
    let source: string;
    try {
      source = await workspace.readRaw(rel);
    } catch {
      continue; // unreadable file: skip, keep indexing the rest
    }
    const { data, body } = splitFrontmatter(source);
    const meta = metadataFromFrontmatter(data);
    const title = body.match(/^#\s+(.+)$/m)?.[1]?.trim() ?? rel;
    const headings = [...body.matchAll(/^##\s+(.+)$/gm)].map((m) => m[1].trim());
    const plain = body
      .replace(/```[\s\S]*?```/g, " ")
      .replace(/[#>*_`|\[\]()-]/g, " ")
      .replace(/\s+/g, " ")
      .trim();
    documents.push({
      id: meta.id || rel,
      title,
      type: meta.type || "unknown",
      status: meta.status,
      path: rel,
      project: rel.match(/^projects\/([^/]+)\//)?.[1],
      authors: meta.authors ?? [],
      headings,
      relations: meta.relations ?? [],
      text: plain.toLowerCase(),
      excerpt: plain.slice(0, 400),
    });
  }

  const byId = new Map(documents.map((d) => [d.id, d]));
  const edges: GraphEdge[] = [];
  for (const doc of documents) {
    for (const rel of doc.relations) {
      const target = rel.target;
      let to: string | undefined;
      let external = false;
      if (target.startsWith("document:")) {
        to = target.slice("document:".length);
      } else if (target.includes(":")) {
        external = true; // github:..., url:..., etc.
      } else {
        to = target; // bare internal id
      }
      if (to && !byId.has(to)) {
        // dangling internal reference — keep it, but not resolved
        to = undefined;
      }
      edges.push({ from: doc.id, target, to, type: rel.type, external });
    }
  }

  return { version: 1, builtAt: new Date().toISOString(), documents, edges };
}

// ---------------------------------------------------------------------------
// LocalIndex — cache lifecycle + queries
// ---------------------------------------------------------------------------

const INVERSE: Record<string, string> = {
  supersedes: "superseded_by",
  implements: "implemented_by",
  implemented_by: "implements",
  motivated_by: "motivates",
  reverted_by: "reverts",
  related_to: "related_to",
  discussed_in: "discussed_in",
};

export class LocalIndex {
  private index: WorkspaceIndex | null = null;

  constructor(private workspace: Workspace) {}

  private cacheFile(): string {
    return joinPath(this.workspace.rootDir, CACHE_PATH);
  }

  /** Load from cache if present, otherwise rebuild from the repository. */
  async ensure(): Promise<WorkspaceIndex> {
    if (this.index) return this.index;
    if (await this.workspace.fs.exists(this.cacheFile())) {
      try {
        const parsed = JSON.parse(await this.workspace.fs.readFile(this.cacheFile())) as WorkspaceIndex;
        if (parsed.version === 1) {
          this.index = parsed;
          return parsed;
        }
      } catch {
        // corrupt cache → rebuild (the cache is disposable by design)
      }
    }
    return this.rebuild();
  }

  /** delete index → scan repository → rebuild (§20). */
  async rebuild(): Promise<WorkspaceIndex> {
    const index = await buildIndex(this.workspace);
    await this.workspace.fs.mkdirp(joinPath(this.workspace.rootDir, TEAMDOC_DIR, "cache"));
    await this.workspace.fs.writeFile(this.cacheFile(), JSON.stringify(index, null, 2));
    this.index = index;
    return index;
  }

  // -- Search ----------------------------------------------------------------

  async search(query: string, limit = 10): Promise<SearchHit[]> {
    const index = await this.ensure();
    const terms = query
      .toLowerCase()
      .split(/\s+/)
      .filter((t) => t.length > 0);
    if (terms.length === 0) return [];

    const hits: SearchHit[] = [];
    for (const doc of index.documents) {
      let score = 0;
      const title = doc.title.toLowerCase();
      const headings = doc.headings.join(" ").toLowerCase();
      for (const term of terms) {
        if (title.includes(term)) score += 5;
        if (headings.includes(term)) score += 3;
        score += countOccurrences(doc.text, term);
        if (doc.id.toLowerCase().includes(term)) score += 4;
      }
      if (score > 0) {
        hits.push({ document: doc, score, snippet: makeSnippet(doc.text, terms) });
      }
    }
    return hits.sort((a, b) => b.score - a.score).slice(0, limit);
  }

  // -- Document Graph (§18) ----------------------------------------------------

  async neighbors(docId: string): Promise<{ outgoing: GraphEdge[]; incoming: GraphEdge[] }> {
    const index = await this.ensure();
    return {
      outgoing: index.edges.filter((e) => e.from === docId),
      incoming: index.edges.filter((e) => e.to === docId),
    };
  }

  async get(docId: string): Promise<IndexedDocument | undefined> {
    const index = await this.ensure();
    return index.documents.find((d) => d.id === docId || d.path === docId);
  }

  // -- AI Context Retrieval (§21) ------------------------------------------------

  /**
   * Collect only the information related to the current task: the document's
   * 1-hop neighborhood in the graph, as compact excerpts.
   */
  async contextFor(docId: string, opts?: { maxItems?: number }): Promise<ContextItem[]> {
    const index = await this.ensure();
    const { outgoing, incoming } = await this.neighbors(docId);
    const items: ContextItem[] = [];
    const seen = new Set<string>([docId]);
    const maxItems = opts?.maxItems ?? 8;

    const pushDoc = (id: string | undefined, relation: string) => {
      if (!id || seen.has(id)) return;
      const doc = index.documents.find((d) => d.id === id);
      if (!doc) return;
      seen.add(id);
      items.push({
        kind: doc.type === "decision" ? "decision" : "document",
        label: `${doc.title} (${relation})`,
        text: doc.excerpt,
      });
    };

    for (const edge of outgoing) {
      if (items.length >= maxItems) break;
      if (edge.external) {
        items.push({
          kind: "external",
          label: `${edge.type}: ${edge.target}`,
          text: `External reference (${edge.target}). Do not invent its contents.`,
        });
      } else {
        pushDoc(edge.to, edge.type);
      }
    }
    for (const edge of incoming) {
      if (items.length >= maxItems) break;
      pushDoc(edge.from, INVERSE[edge.type] ?? `target of ${edge.type}`);
    }
    return items;
  }
}

function countOccurrences(text: string, term: string): number {
  let count = 0;
  let pos = text.indexOf(term);
  while (pos !== -1 && count < 50) {
    count++;
    pos = text.indexOf(term, pos + term.length);
  }
  return count;
}

function makeSnippet(text: string, terms: string[]): string {
  for (const term of terms) {
    const pos = text.indexOf(term);
    if (pos !== -1) {
      const start = Math.max(0, pos - 60);
      return (start > 0 ? "…" : "") + text.slice(start, pos + 90) + "…";
    }
  }
  return text.slice(0, 120);
}
