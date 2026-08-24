import { stringify as stringifyYaml, parse as parseYaml } from "yaml";
import {
  BUILTIN_TYPES,
  DocumentTypeRegistry,
  createSkeleton,
  documentTypeFromData,
  parseDocument,
  serializeDocument,
  slugify,
} from "./document.js";
import { splitFrontmatter } from "./frontmatter.js";
import { mergeWritingRules, type WritingRulesConfig } from "./rules.js";
import { joinPath, type FileSystemPort } from "./ports.js";
import type {
  Document,
  DocumentRef,
  ProjectRef,
  WorkspaceConfig,
  WritingRule,
} from "./types.js";

/**
 * Workspace (DESIGN.md §4).
 *
 * The Git repository IS the workspace: documents, decisions, writing rules
 * and document types all live in the repo. This class only reads/writes
 * files through the FileSystemPort — it holds no state a fresh clone would
 * not reproduce.
 */

export const TEAMDOC_DIR = ".teamdoc";

export function scaffoldFiles(name: string): Record<string, string> {
  const files: Record<string, string> = {
    "workspace.yaml": stringifyYaml({ name }),
    [`${TEAMDOC_DIR}/config.yaml`]: stringifyYaml({ agent: "claude" }),
    [`${TEAMDOC_DIR}/writing-rules.yaml`]: [
      "# Team writing rules — merged on top of the built-in cognitive rules (C001–C010).",
      "# rules:",
      '#   - id: T001',
      '#     name: Korean First',
      '#     description: 본문은 한국어로 작성한다.',
      "# disable: []",
      "rules: []",
      "disable: []",
      "",
    ].join("\n"),
    [`${TEAMDOC_DIR}/cache/.gitignore`]: "*\n!.gitignore\n",
    "decisions/.gitkeep": "",
    "projects/.gitkeep": "",
  };
  for (const type of BUILTIN_TYPES) {
    files[`${TEAMDOC_DIR}/document-types/${type.id}.yaml`] = stringifyYaml(type);
  }
  return files;
}

export class Workspace {
  private constructor(
    readonly fs: FileSystemPort,
    readonly rootDir: string,
    readonly config: WorkspaceConfig,
    readonly registry: DocumentTypeRegistry,
    readonly rules: WritingRule[],
  ) {}

  // -- lifecycle -------------------------------------------------------------

  static async init(
    fs: FileSystemPort,
    rootDir: string,
    opts: { name: string },
  ): Promise<Workspace> {
    for (const [rel, content] of Object.entries(scaffoldFiles(opts.name))) {
      const abs = joinPath(rootDir, rel);
      const dir = abs.split("/").slice(0, -1).join("/");
      if (dir) await fs.mkdirp(dir);
      await fs.writeFile(abs, content);
    }
    return Workspace.load(fs, rootDir);
  }

  static async load(fs: FileSystemPort, rootDir: string): Promise<Workspace> {
    const wsYaml = joinPath(rootDir, "workspace.yaml");
    if (!(await fs.exists(wsYaml))) {
      throw new Error(`Not an adoc workspace (missing workspace.yaml): ${rootDir}`);
    }
    const wsData = (parseYaml(await fs.readFile(wsYaml)) ?? {}) as Record<string, unknown>;

    let agent: string | undefined;
    const configPath = joinPath(rootDir, TEAMDOC_DIR, "config.yaml");
    if (await fs.exists(configPath)) {
      const cfg = (parseYaml(await fs.readFile(configPath)) ?? {}) as Record<string, unknown>;
      if (cfg.agent) agent = String(cfg.agent);
    }

    // Document types: builtins + repo-defined (repo wins on id collision).
    const registry = new DocumentTypeRegistry(BUILTIN_TYPES);
    const typesDir = joinPath(rootDir, TEAMDOC_DIR, "document-types");
    if (await fs.exists(typesDir)) {
      for (const file of await fs.listFiles(typesDir)) {
        if (!file.endsWith(".yaml") && !file.endsWith(".yml")) continue;
        const data = parseYaml(await fs.readFile(joinPath(typesDir, file)));
        if (data && typeof data === "object") {
          registry.register(documentTypeFromData(data as Record<string, unknown>));
        }
      }
    }

    // Writing rules: cognitive defaults + team config.
    let rulesConfig: WritingRulesConfig | undefined;
    const rulesPath = joinPath(rootDir, TEAMDOC_DIR, "writing-rules.yaml");
    if (await fs.exists(rulesPath)) {
      rulesConfig = (parseYaml(await fs.readFile(rulesPath)) ?? undefined) as
        | WritingRulesConfig
        | undefined;
    }
    const rules = mergeWritingRules(rulesConfig);

    return new Workspace(
      fs,
      rootDir,
      { name: String(wsData.name ?? "workspace"), agent },
      registry,
      rules,
    );
  }

  // -- projects ----------------------------------------------------------------

  async listProjects(): Promise<ProjectRef[]> {
    const projectsDir = joinPath(this.rootDir, "projects");
    if (!(await this.fs.exists(projectsDir))) return [];
    const projects = new Map<string, ProjectRef>();
    for (const file of await this.fs.listFiles(projectsDir)) {
      const id = file.split("/")[0];
      if (!id || id.startsWith(".")) continue;
      if (!projects.has(id)) {
        projects.set(id, { id, name: id, path: `projects/${id}` });
      }
      if (file === `${id}/project.yaml`) {
        const data = (parseYaml(await this.fs.readFile(joinPath(projectsDir, file))) ?? {}) as Record<string, unknown>;
        if (data.name) projects.get(id)!.name = String(data.name);
      }
    }
    return [...projects.values()];
  }

  async createProject(id: string, name?: string): Promise<ProjectRef> {
    const rel = `projects/${id}`;
    await this.fs.mkdirp(joinPath(this.rootDir, rel, "documents"));
    await this.fs.writeFile(
      joinPath(this.rootDir, rel, "project.yaml"),
      stringifyYaml({ id, name: name ?? id }),
    );
    return { id, name: name ?? id, path: rel };
  }

  // -- documents ----------------------------------------------------------------

  /** All document markdown files, as workspace-relative paths. */
  async listDocumentPaths(): Promise<string[]> {
    const paths: string[] = [];
    const projectsDir = joinPath(this.rootDir, "projects");
    if (await this.fs.exists(projectsDir)) {
      for (const file of await this.fs.listFiles(projectsDir)) {
        if (/^[^/]+\/documents\/.+\.md$/.test(file)) paths.push(`projects/${file}`);
      }
    }
    const decisionsDir = joinPath(this.rootDir, "decisions");
    if (await this.fs.exists(decisionsDir)) {
      for (const file of await this.fs.listFiles(decisionsDir)) {
        if (file.endsWith(".md")) paths.push(`decisions/${file}`);
      }
    }
    return paths.sort();
  }

  async listDocuments(): Promise<DocumentRef[]> {
    const refs: DocumentRef[] = [];
    for (const rel of await this.listDocumentPaths()) {
      try {
        refs.push(await this.documentRef(rel));
      } catch {
        // unreadable/broken file: skip from listing, keep the rest working
      }
    }
    return refs;
  }

  private async documentRef(rel: string): Promise<DocumentRef> {
    const source = await this.fs.readFile(joinPath(this.rootDir, rel));
    const { data, body } = splitFrontmatter(source);
    const title = body.match(/^#\s+(.+)$/m)?.[1]?.trim() ?? rel;
    const projectMatch = rel.match(/^projects\/([^/]+)\//);
    return {
      id: String(data.id ?? rel),
      title,
      type: String(data.type ?? "unknown"),
      status: data.status !== undefined ? String(data.status) : undefined,
      path: rel,
      project: projectMatch?.[1],
    };
  }

  async readDocument(rel: string): Promise<Document> {
    const source = await this.fs.readFile(joinPath(this.rootDir, rel));
    return parseDocument(source, this.registry);
  }

  async readRaw(rel: string): Promise<string> {
    return this.fs.readFile(joinPath(this.rootDir, rel));
  }

  async writeRaw(rel: string, content: string): Promise<void> {
    const abs = joinPath(this.rootDir, rel);
    const dir = abs.split("/").slice(0, -1).join("/");
    if (dir) await this.fs.mkdirp(dir);
    await this.fs.writeFile(abs, content);
  }

  async writeDocument(rel: string, doc: Document): Promise<void> {
    await this.writeRaw(rel, serializeDocument(doc));
  }

  /**
   * Create a new skeleton document.
   * Decisions get a sequential `NNN-` prefix (DESIGN.md §4).
   */
  async createDocument(opts: {
    typeId: string;
    title: string;
    project?: string;
    authors?: string[];
  }): Promise<{ ref: DocumentRef; document: Document }> {
    const typeDef = this.registry.require(opts.typeId);
    const id = slugify(opts.title);
    const doc = createSkeleton(typeDef, { id, title: opts.title, authors: opts.authors });

    let rel: string;
    if (opts.typeId === "decision") {
      const existing = (await this.listDocumentPaths()).filter((p) => p.startsWith("decisions/"));
      const next = existing.length + 1;
      rel = `decisions/${String(next).padStart(3, "0")}-${id}.md`;
    } else {
      const project = opts.project ?? "general";
      if (!(await this.fs.exists(joinPath(this.rootDir, "projects", project)))) {
        await this.createProject(project);
      }
      rel = `projects/${project}/documents/${id}.md`;
    }

    if (await this.fs.exists(joinPath(this.rootDir, rel))) {
      throw new Error(`Document already exists: ${rel}`);
    }
    await this.writeDocument(rel, doc);
    return { ref: await this.documentRef(rel), document: doc };
  }
}
