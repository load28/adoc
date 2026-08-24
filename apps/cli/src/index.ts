#!/usr/bin/env node
import { readFileSync } from "node:fs";
import * as path from "node:path";
import { Command } from "commander";
import {
  CLAUDE_CLI_SPEC,
  CODEX_CLI_SPEC,
  CliAgentAdapter,
  MockAgentAdapter,
  Workspace,
  WritingEngine,
  type AgentAdapter,
  type ContextItem,
  type Proposal,
} from "@adoc/core";
import { DocRepository } from "@adoc/git";
import { LocalIndex } from "@adoc/indexer";
import { NodeFileSystem, NodeProcessRunner } from "@adoc/node-ports";

/**
 * adoc — CLI shell over the same domain packages the Tauri desktop app uses.
 * Useful for headless usage, scripting, and verifying the whole flow end-to-end.
 */

const fs = new NodeFileSystem();
const runner = new NodeProcessRunner();
const program = new Command();

program
  .name("adoc")
  .description("Git-based AI collaborative documents")
  .option("-C, --dir <dir>", "workspace directory", ".")
  .option("--agent <id>", "agent to use: mock | claude | codex");

function rootDir(): string {
  return path.resolve(program.opts<{ dir: string }>().dir);
}

async function openWorkspace(): Promise<Workspace> {
  return Workspace.load(fs, rootDir());
}

function repo(): DocRepository {
  return new DocRepository(runner, rootDir());
}

function pickAdapter(workspace: Workspace): AgentAdapter {
  const id = program.opts<{ agent?: string }>().agent ?? workspace.config.agent ?? "claude";
  switch (id) {
    case "mock":
      return new MockAgentAdapter(() => {
        throw new Error("mock agent has no scripted response in the CLI; use claude or codex");
      });
    case "claude":
      return new CliAgentAdapter(CLAUDE_CLI_SPEC, runner);
    case "codex":
      return new CliAgentAdapter(CODEX_CLI_SPEC, runner);
    default:
      throw new Error(`unknown agent: ${id}`);
  }
}

function makeEngine(workspace: Workspace): WritingEngine {
  return new WritingEngine({
    adapter: pickAdapter(workspace),
    registry: workspace.registry,
    rules: workspace.rules,
    agentContext: { workspaceDir: workspace.rootDir },
  });
}

async function contextFor(workspace: Workspace, docId: string): Promise<ContextItem[]> {
  try {
    const index = new LocalIndex(workspace);
    await index.rebuild();
    return await index.contextFor(docId);
  } catch {
    return [];
  }
}

function printProposal(proposal: Proposal): void {
  console.log(proposal.diff);
  if (proposal.summary) console.log(`\nSummary: ${proposal.summary}`);
}

async function applyOrExplain(
  workspace: Workspace,
  rel: string,
  proposal: Proposal,
  apply: boolean,
): Promise<void> {
  printProposal(proposal);
  if (apply) {
    await workspace.writeRaw(rel, proposal.after);
    console.log(`\nApplied to working tree: ${rel}`);
    console.log("Not committed — review with `adoc diff`, record with `adoc share`.");
  } else {
    console.log("\nProposal only (AI changes are never auto-applied). Re-run with --apply to accept.");
  }
}

function readNotes(file: string | undefined): string {
  if (file) return readFileSync(file, "utf8");
  return readFileSync(0, "utf8"); // stdin
}

// -- workspace ---------------------------------------------------------------

program
  .command("init")
  .description("initialize a new team workspace (git repo + .teamdoc scaffolding)")
  .option("--name <name>", "workspace name", "workspace")
  .option("--remote <url>", "git remote to share with")
  .action(async (opts: { name: string; remote?: string }) => {
    const dir = rootDir();
    await fs.mkdirp(dir);
    await Workspace.init(fs, dir, { name: opts.name });
    const r = repo();
    if (!(await r.isRepository())) await r.init();
    if (opts.remote) await r.setRemote(opts.remote);
    console.log(`Initialized workspace "${opts.name}" in ${dir}`);
  });

program
  .command("list")
  .description("list all documents")
  .action(async () => {
    const workspace = await openWorkspace();
    for (const ref of await workspace.listDocuments()) {
      console.log(
        `${ref.path}\n  id=${ref.id} type=${ref.type} status=${ref.status ?? "-"}${ref.project ? ` project=${ref.project}` : ""}\n  ${ref.title}`,
      );
    }
  });

program
  .command("new")
  .description("create an empty document skeleton")
  .argument("<type>", "document type (design | proposal | decision | custom)")
  .argument("<title...>", "document title")
  .option("-p, --project <project>", "project id")
  .option("--author <author>", "author name")
  .action(async (type: string, titleWords: string[], opts: { project?: string; author?: string }) => {
    const workspace = await openWorkspace();
    const { ref } = await workspace.createDocument({
      typeId: type,
      title: titleWords.join(" "),
      project: opts.project,
      authors: opts.author ? [opts.author] : undefined,
    });
    console.log(`Created ${ref.path}`);
  });

program
  .command("show")
  .description("print a document")
  .argument("<path>", "workspace-relative document path")
  .action(async (rel: string) => {
    const workspace = await openWorkspace();
    console.log(await workspace.readRaw(rel));
  });

// -- AI: compose / rewrite / critique -----------------------------------------

program
  .command("compose")
  .description("AI-compose a new document from raw notes (notes from --notes file or stdin)")
  .argument("<type>", "document type")
  .requiredOption("--id <id>", "document id (also used as filename)")
  .option("-p, --project <project>", "project id")
  .option("--notes <file>", "file containing raw notes (default: stdin)")
  .option("--author <author>", "author name")
  .option("--apply", "write the proposal to the working tree", false)
  .action(async (type: string, opts: { id: string; project?: string; notes?: string; author?: string; apply: boolean }) => {
    const workspace = await openWorkspace();
    const engine = makeEngine(workspace);
    const notes = readNotes(opts.notes);
    console.error("Analyzing intent and composing…");
    const result = await engine.compose({
      notes,
      typeId: type,
      init: { id: opts.id, authors: opts.author ? [opts.author] : undefined },
    });
    console.log("--- Document Intent ---");
    console.log(JSON.stringify(result.intent, null, 2));
    console.log("--- Proposal ---");
    const rel =
      type === "decision"
        ? `decisions/${opts.id}.md`
        : `projects/${opts.project ?? "general"}/documents/${opts.id}.md`;
    await applyOrExplain(workspace, rel, result.proposal, opts.apply);
  });

program
  .command("rewrite")
  .description("AI-rewrite a document toward a goal (proposal + diff)")
  .argument("<path>", "workspace-relative document path")
  .requiredOption("--goal <goal>", '"더 간결하게", "근거 강화", …')
  .option("--section <id...>", "rewrite only these section ids")
  .option("--apply", "write the proposal to the working tree", false)
  .action(async (rel: string, opts: { goal: string; section?: string[]; apply: boolean }) => {
    const workspace = await openWorkspace();
    const engine = makeEngine(workspace);
    const document = await workspace.readDocument(rel);
    const context = await contextFor(workspace, document.id);
    const proposal = await engine.rewrite({ document, goal: opts.goal, target: opts.section, context });
    await applyOrExplain(workspace, rel, proposal, opts.apply);
  });

program
  .command("critique")
  .description("AI-critique a document (analysis only, never modifies)")
  .argument("<path>", "workspace-relative document path")
  .action(async (rel: string) => {
    const workspace = await openWorkspace();
    const engine = makeEngine(workspace);
    const document = await workspace.readDocument(rel);
    const context = await contextFor(workspace, document.id);
    const suggestions = await engine.critique({ document, context });
    if (suggestions.length === 0) {
      console.log("No issues found.");
      return;
    }
    for (const s of suggestions) {
      console.log(`[${s.severity}]${s.ruleId ? ` (${s.ruleId})` : ""}${s.section ? ` §${s.section}` : ""} ${s.message}`);
      if (s.proposal) console.log(`  → ${s.proposal}`);
    }
  });

// -- git-backed collaboration ---------------------------------------------------

program
  .command("sync")
  .description("최신 문서 가져오기 (git pull)")
  .action(async () => {
    const result = await repo().pullLatest();
    console.log(result.output);
    if (result.conflicts.length > 0) {
      console.log("\nConflicted documents:");
      for (const c of result.conflicts) console.log(`  ${c}`);
      console.log("Resolve with `adoc merge <path> --ai` or edit manually, then `adoc resolve <path>`.");
    }
  });

program
  .command("diff")
  .description("변경사항 보기 (uncommitted changes)")
  .argument("[path]", "limit to one document")
  .action(async (rel?: string) => {
    console.log(await repo().workingDiff(rel));
  });

program
  .command("share")
  .description("변경 기록 + 팀에 공유 (git commit + push)")
  .requiredOption("-m, --message <message>", "change description")
  .option("--no-push", "commit only, don't push")
  .action(async (opts: { message: string; push: boolean }) => {
    const r = repo();
    const sha = await r.recordChanges(opts.message);
    console.log(`Recorded ${sha.slice(0, 8)}: ${opts.message}`);
    if (opts.push) {
      console.log(await r.shareWithTeam());
      console.log("Shared with team.");
    }
  });

program
  .command("history")
  .description("문서 History (git log for one document)")
  .argument("<path>", "workspace-relative document path")
  .action(async (rel: string) => {
    for (const entry of await repo().historyFor(rel)) {
      const date = new Date(entry.date).toISOString().slice(0, 10);
      console.log(`${entry.sha.slice(0, 8)}  ${date}  ${entry.author}\n  ${entry.message}`);
    }
  });

program
  .command("restore")
  .description("특정 버전 복원 (restore a document from an old commit into the working tree)")
  .argument("<path>", "workspace-relative document path")
  .argument("<sha>", "commit to restore from")
  .action(async (rel: string, sha: string) => {
    await repo().restoreVersion(rel, sha);
    console.log(`Restored ${rel} from ${sha.slice(0, 8)} (working tree only — review and share).`);
  });

// -- conflicts -------------------------------------------------------------------

program
  .command("conflicts")
  .description("list conflicted documents")
  .action(async () => {
    const conflicts = await repo().conflicts();
    if (conflicts.length === 0) console.log("No conflicts.");
    for (const c of conflicts) console.log(c);
  });

program
  .command("merge")
  .description("resolve a conflicted document (shows versions; --ai proposes a merge)")
  .argument("<path>", "conflicted document path")
  .option("--ai", "generate an AI merge proposal", false)
  .option("--apply", "accept the AI proposal into the working tree and mark resolved", false)
  .action(async (rel: string, opts: { ai: boolean; apply: boolean }) => {
    const workspace = await openWorkspace();
    const r = repo();
    const versions = await r.conflictVersions(rel);
    if (!opts.ai) {
      console.log("=== CURRENT (yours) ===\n" + versions.current);
      console.log("=== INCOMING (theirs) ===\n" + versions.incoming);
      console.log("Edit the file, then `adoc resolve <path>` — or re-run with --ai.");
      return;
    }
    const engine = makeEngine(workspace);
    const proposal = await engine.merge(versions);
    printProposal(proposal);
    if (proposal.unresolved.length > 0) {
      console.log("\nUnresolved contradictions (human decision required):");
      for (const u of proposal.unresolved) {
        console.log(`  ${u.section ? `§${u.section}: ` : ""}${u.reason}`);
      }
    }
    if (opts.apply) {
      if (proposal.unresolved.length > 0) {
        console.log("\nNot marking resolved: contradictions remain. Written to working tree for manual editing.");
        await workspace.writeRaw(rel, proposal.after);
      } else {
        await workspace.writeRaw(rel, proposal.after);
        await r.markResolved(rel);
        console.log(`\nResolved ${rel}. Complete with \`adoc share -m "merge"\`.`);
      }
    }
  });

program
  .command("resolve")
  .description("mark a manually-edited conflict as resolved")
  .argument("<path>", "document path")
  .action(async (rel: string) => {
    await repo().markResolved(rel);
    console.log(`Marked resolved: ${rel}`);
  });

// -- index / search / graph -------------------------------------------------------

const indexCmd = program.command("index").description("local index (disposable cache)");

indexCmd
  .command("rebuild")
  .description("delete index → scan repository → rebuild")
  .action(async () => {
    const workspace = await openWorkspace();
    const index = await new LocalIndex(workspace).rebuild();
    console.log(`Indexed ${index.documents.length} documents, ${index.edges.length} relations.`);
  });

program
  .command("search")
  .description("search documents")
  .argument("<query...>", "search terms")
  .action(async (queryWords: string[]) => {
    const workspace = await openWorkspace();
    const hits = await new LocalIndex(workspace).search(queryWords.join(" "));
    for (const hit of hits) {
      console.log(`${hit.document.path}  (score ${hit.score})\n  ${hit.document.title}\n  …${hit.snippet}`);
    }
    if (hits.length === 0) console.log("No results.");
  });

program
  .command("related")
  .description("show a document's relations (Document Graph)")
  .argument("<id>", "document id")
  .action(async (id: string) => {
    const workspace = await openWorkspace();
    const index = new LocalIndex(workspace);
    await index.rebuild();
    const { outgoing, incoming } = await index.neighbors(id);
    for (const e of outgoing) console.log(`${id} --${e.type}--> ${e.to ?? e.target}${e.external ? " (external)" : ""}`);
    for (const e of incoming) console.log(`${e.from} --${e.type}--> ${id}`);
    if (outgoing.length + incoming.length === 0) console.log("No relations.");
  });

program.parseAsync(process.argv).catch((err: unknown) => {
  console.error(err instanceof Error ? err.message : String(err));
  process.exitCode = 1;
});
