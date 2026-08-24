import { mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import * as path from "node:path";
import { afterAll, beforeAll, describe, expect, it } from "vitest";
import { Workspace, joinPath, serializeDocument, parseDocument } from "@adoc/core";
import { LocalIndex } from "@adoc/indexer";
import { NodeFileSystem } from "@adoc/node-ports";

const fs = new NodeFileSystem();

describe("LocalIndex", () => {
  let root: string;
  let workspace: Workspace;
  let index: LocalIndex;

  beforeAll(async () => {
    root = await mkdtemp(path.join(tmpdir(), "adoc-index-"));
    workspace = await Workspace.init(fs, root, { name: "team" });

    // auth-v1 (superseded) and auth-v2 (implements a PR, supersedes v1)
    const { document: v1 } = await workspace.createDocument({
      typeId: "design",
      title: "Auth Design v1",
      project: "app",
    });
    v1.sections[0].content = "토큰 기반 인증 설계.";
    v1.metadata.id = "auth-v1";
    v1.id = "auth-v1";
    await workspace.writeDocument("projects/app/documents/auth-design-v1.md", v1);

    const { document: v2 } = await workspace.createDocument({
      typeId: "design",
      title: "Auth Design v2",
      project: "app",
    });
    v2.metadata.id = "auth-v2";
    v2.id = "auth-v2";
    v2.sections[0].content = "인증 상태를 서버 세션으로 일원화한다.";
    v2.metadata.relations = [
      { type: "supersedes", target: "document:auth-v1" },
      { type: "implemented_by", target: "github:load28/app#142" },
    ];
    await workspace.writeDocument("projects/app/documents/auth-design-v2.md", v2);

    const { document: decision } = await workspace.createDocument({
      typeId: "decision",
      title: "Use server sessions",
    });
    decision.metadata.id = "use-server-sessions";
    decision.id = "use-server-sessions";
    decision.sections[0].content = "서버 세션을 채택한다.";
    decision.metadata.relations = [{ type: "motivated_by", target: "document:auth-v2" }];
    await workspace.writeDocument("decisions/001-use-server-sessions.md", decision);

    index = new LocalIndex(workspace);
  });

  afterAll(async () => {
    await rm(root, { recursive: true, force: true });
  });

  it("scans the repository into a rebuildable index (§20)", async () => {
    const built = await index.rebuild();
    expect(built.documents.map((d) => d.id).sort()).toEqual([
      "auth-v1",
      "auth-v2",
      "use-server-sessions",
    ]);
    expect(await fs.exists(joinPath(root, ".teamdoc/cache/index.json"))).toBe(true);
  });

  it("searches by title, heading and content", async () => {
    const hits = await index.search("서버 세션");
    expect(hits.length).toBeGreaterThanOrEqual(2);
    expect(hits.map((h) => h.document.id)).toContain("auth-v2");
    expect(hits[0].snippet.length).toBeGreaterThan(0);
  });

  it("exposes the document graph with resolved and external edges (§18)", async () => {
    const { outgoing, incoming } = await index.neighbors("auth-v2");
    expect(outgoing).toHaveLength(2);
    const supersedes = outgoing.find((e) => e.type === "supersedes");
    expect(supersedes?.to).toBe("auth-v1");
    expect(supersedes?.external).toBe(false);
    const implementedBy = outgoing.find((e) => e.type === "implemented_by");
    expect(implementedBy?.external).toBe(true);
    expect(implementedBy?.target).toBe("github:load28/app#142");
    // decision → motivated_by → auth-v2 arrives as an incoming edge
    expect(incoming.map((e) => e.from)).toContain("use-server-sessions");
  });

  it("retrieves only task-relevant context for AI (§21)", async () => {
    const context = await index.contextFor("auth-v2");
    const labels = context.map((c) => c.label).join(" | ");
    expect(labels).toContain("Auth Design v1 (supersedes)");
    expect(labels).toContain("implemented_by: github:load28/app#142");
    expect(labels).toContain("Use server sessions");
    const decisionItem = context.find((c) => c.kind === "decision");
    expect(decisionItem?.text).toContain("서버 세션을 채택한다");
  });

  it("survives cache deletion — Git repo is the source of truth (§2.1)", async () => {
    await fs.remove(joinPath(root, ".teamdoc/cache/index.json"));
    const fresh = new LocalIndex(workspace);
    const rebuilt = await fresh.ensure();
    expect(rebuilt.documents).toHaveLength(3);
  });

  it("keeps documents readable by plain markdown tools (§2.2)", async () => {
    const raw = await workspace.readRaw("projects/app/documents/auth-design-v2.md");
    expect(raw.startsWith("---\n")).toBe(true);
    const doc = parseDocument(raw, workspace.registry);
    expect(serializeDocument(doc)).toBe(raw);
  });
});
