import { spawn } from "node:child_process";
import { promises as fs } from "node:fs";
import * as path from "node:path";
import type { FileSystemPort, ProcessRunnerPort, ProcessResult } from "@adoc/core";

/**
 * Node implementations of the core ports — used by the CLI and by tests.
 * The Tauri desktop app implements the same ports over Rust commands.
 */

export class NodeFileSystem implements FileSystemPort {
  async readFile(p: string): Promise<string> {
    return fs.readFile(p, "utf8");
  }

  async writeFile(p: string, content: string): Promise<void> {
    await fs.mkdir(path.dirname(p), { recursive: true });
    await fs.writeFile(p, content, "utf8");
  }

  async exists(p: string): Promise<boolean> {
    try {
      await fs.access(p);
      return true;
    } catch {
      return false;
    }
  }

  async mkdirp(p: string): Promise<void> {
    await fs.mkdir(p, { recursive: true });
  }

  async listFiles(dir: string): Promise<string[]> {
    const out: string[] = [];
    const walk = async (current: string, prefix: string): Promise<void> => {
      const entries = await fs.readdir(current, { withFileTypes: true });
      for (const entry of entries) {
        if (entry.name === ".git" || entry.name === "node_modules") continue;
        const rel = prefix ? `${prefix}/${entry.name}` : entry.name;
        if (entry.isDirectory()) await walk(path.join(current, entry.name), rel);
        else out.push(rel);
      }
    };
    await walk(dir, "");
    return out;
  }

  async remove(p: string): Promise<void> {
    await fs.rm(p, { recursive: true, force: true });
  }
}

/** Commands the domain is allowed to run (DESIGN.md §12, §14). */
const ALLOWED_COMMANDS = new Set(["git", "claude", "codex"]);

export class NodeProcessRunner implements ProcessRunnerPort {
  constructor(private allowlist: Set<string> = ALLOWED_COMMANDS) {}

  private assertAllowed(command: string): void {
    if (!this.allowlist.has(command)) {
      throw new Error(`command not allowed: ${command}`);
    }
  }

  async run(
    command: string,
    args: string[],
    opts: { cwd: string; stdin?: string },
  ): Promise<ProcessResult> {
    return this.stream(command, args, opts, () => {});
  }

  stream(
    command: string,
    args: string[],
    opts: { cwd: string; stdin?: string; signal?: AbortSignal },
    onChunk: (chunk: string) => void,
  ): Promise<ProcessResult> {
    this.assertAllowed(command);
    return new Promise((resolve, reject) => {
      const child = spawn(command, args, {
        cwd: opts.cwd,
        signal: opts.signal,
        stdio: ["pipe", "pipe", "pipe"],
        env: process.env,
      });
      let stdout = "";
      let stderr = "";
      child.stdout.setEncoding("utf8");
      child.stderr.setEncoding("utf8");
      child.stdout.on("data", (chunk: string) => {
        stdout += chunk;
        onChunk(chunk);
      });
      child.stderr.on("data", (chunk: string) => {
        stderr += chunk;
      });
      child.on("error", reject);
      child.on("close", (code) => {
        resolve({ code: code ?? -1, stdout, stderr });
      });
      if (opts.stdin !== undefined) child.stdin.write(opts.stdin);
      child.stdin.end();
    });
  }
}
