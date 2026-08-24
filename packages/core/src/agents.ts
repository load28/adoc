import { renderPromptText } from "./prompt.js";
import type {
  AgentAdapter,
  AgentCapabilities,
  AgentContext,
  AgentEvent,
  AgentRequest,
} from "./types.js";
import type { ProcessRunnerPort } from "./ports.js";

/**
 * Agent adapters (DESIGN.md §12).
 *
 * The Writing Engine never knows which agent runs. Each adapter handles
 * invocation, streaming and output shape differences. CLI adapters are
 * built on the ProcessRunnerPort so the same code runs under Node (CLI)
 * and Tauri (Rust process runner).
 */

// ---------------------------------------------------------------------------
// Mock adapter — deterministic, for tests and offline development
// ---------------------------------------------------------------------------

export type MockHandler = (request: AgentRequest) => string;

export class MockAgentAdapter implements AgentAdapter {
  constructor(private handler: MockHandler) {}

  capabilities(): AgentCapabilities {
    return { id: "mock", displayName: "Mock Agent", streaming: false };
  }

  async *execute(request: AgentRequest, _context: AgentContext): AsyncIterable<AgentEvent> {
    yield { type: "started" };
    try {
      const text = this.handler(request);
      yield { type: "text", text };
      yield { type: "completed", text };
    } catch (err) {
      yield { type: "error", message: err instanceof Error ? err.message : String(err) };
    }
  }
}

// ---------------------------------------------------------------------------
// Local CLI agents (claude / codex)
// ---------------------------------------------------------------------------

export interface CliAgentSpec {
  id: string;
  displayName: string;
  command: string;
  /** Build argv; the prompt itself is passed via stdin. */
  args: string[];
}

export const CLAUDE_CLI_SPEC: CliAgentSpec = {
  id: "claude",
  displayName: "Claude Code CLI",
  command: "claude",
  // -p reads the prompt from stdin in non-interactive print mode.
  args: ["-p", "--output-format", "text"],
};

export const CODEX_CLI_SPEC: CliAgentSpec = {
  id: "codex",
  displayName: "Codex CLI",
  command: "codex",
  args: ["exec", "-"],
};

export class CliAgentAdapter implements AgentAdapter {
  constructor(
    private spec: CliAgentSpec,
    private runner: ProcessRunnerPort,
  ) {}

  capabilities(): AgentCapabilities {
    return { id: this.spec.id, displayName: this.spec.displayName, streaming: true };
  }

  async *execute(request: AgentRequest, context: AgentContext): AsyncIterable<AgentEvent> {
    yield { type: "started" };
    const chunks: string[] = [];
    const queue: AgentEvent[] = [];
    let notify: (() => void) | null = null;

    const push = (event: AgentEvent) => {
      queue.push(event);
      notify?.();
    };

    const done = this.runner
      .stream(
        this.spec.command,
        this.spec.args,
        {
          cwd: context.workspaceDir ?? ".",
          stdin: renderPromptText(request.prompt),
          signal: context.signal,
        },
        (chunk) => {
          chunks.push(chunk);
          push({ type: "text", text: chunk });
        },
      )
      .then((result) => {
        if (result.code !== 0) {
          push({ type: "error", message: `${this.spec.command} exited ${result.code}: ${result.stderr.slice(0, 2000)}` });
        } else {
          push({ type: "completed", text: chunks.join("") });
        }
      })
      .catch((err: unknown) => {
        push({ type: "error", message: err instanceof Error ? err.message : String(err) });
      });

    let finished = false;
    while (!finished) {
      if (queue.length === 0) {
        await new Promise<void>((resolve) => {
          notify = resolve;
        });
        notify = null;
      }
      while (queue.length > 0) {
        const event = queue.shift()!;
        if (event.type === "completed" || event.type === "error") finished = true;
        yield event;
      }
    }
    await done;
  }
}

// ---------------------------------------------------------------------------
// Helper: run an adapter to completion and return the final text
// ---------------------------------------------------------------------------

export async function runAgent(
  adapter: AgentAdapter,
  request: AgentRequest,
  context: AgentContext,
  onText?: (chunk: string) => void,
): Promise<string> {
  for await (const event of adapter.execute(request, context)) {
    if (event.type === "text") onText?.(event.text);
    if (event.type === "completed") return event.text;
    if (event.type === "error") throw new Error(event.message);
  }
  throw new Error(`agent ${adapter.capabilities().id} ended without completing`);
}
