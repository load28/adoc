import {
  CLAUDE_CLI_SPEC,
  CODEX_CLI_SPEC,
  CliAgentAdapter,
  Workspace,
  WritingEngine,
  type AgentAdapter,
  type ContextItem,
} from "@adoc/core";
import { DocRepository } from "@adoc/git";
import { LocalIndex } from "@adoc/indexer";
import { TauriFileSystem, TauriProcessRunner } from "./ports";

/**
 * App-level composition root: wires the pure domain packages to the Tauri
 * ports. One AppServices instance per opened workspace.
 */

export const fs = new TauriFileSystem();
export const runner = new TauriProcessRunner();

export class AppServices {
  private constructor(
    readonly workspace: Workspace,
    readonly repo: DocRepository,
    readonly index: LocalIndex,
  ) {}

  static async open(dir: string): Promise<AppServices> {
    const workspace = await Workspace.load(fs, dir);
    return new AppServices(workspace, new DocRepository(runner, dir), new LocalIndex(workspace));
  }

  static async init(dir: string, name: string): Promise<AppServices> {
    const workspace = await Workspace.init(fs, dir, { name });
    const repo = new DocRepository(runner, dir);
    if (!(await repo.isRepository())) await repo.init();
    return new AppServices(workspace, repo, new LocalIndex(workspace));
  }

  agent(id?: string): AgentAdapter {
    const agentId = id ?? this.workspace.config.agent ?? "claude";
    switch (agentId) {
      case "codex":
        return new CliAgentAdapter(CODEX_CLI_SPEC, runner);
      case "claude":
      default:
        return new CliAgentAdapter(CLAUDE_CLI_SPEC, runner);
    }
  }

  engine(onText?: (chunk: string) => void, agentId?: string): WritingEngine {
    return new WritingEngine({
      adapter: this.agent(agentId),
      registry: this.workspace.registry,
      rules: this.workspace.rules,
      agentContext: { workspaceDir: this.workspace.rootDir },
      onText,
    });
  }

  /** AI Context Retrieval (§21): 1-hop graph neighborhood of the document. */
  async contextFor(docId: string): Promise<ContextItem[]> {
    try {
      await this.index.rebuild();
      return await this.index.contextFor(docId);
    } catch {
      return [];
    }
  }
}
