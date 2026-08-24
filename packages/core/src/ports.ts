/**
 * Ports — the only seams between the pure domain and the outside world.
 *
 * Node (CLI) and Tauri (desktop) each provide their own implementation,
 * so every package in `packages/` stays runtime-agnostic.
 */

export interface FileSystemPort {
  readFile(path: string): Promise<string>;
  writeFile(path: string, content: string): Promise<void>;
  exists(path: string): Promise<boolean>;
  mkdirp(path: string): Promise<void>;
  /** Recursive file listing returning paths relative to `dir`. */
  listFiles(dir: string): Promise<string[]>;
  remove(path: string): Promise<void>;
}

export interface ProcessResult {
  code: number;
  stdout: string;
  stderr: string;
}

export interface ProcessRunnerPort {
  /**
   * Run a command to completion. Implementations MUST restrict `command`
   * to an allowlist (git, claude, codex) — the domain never runs
   * arbitrary binaries.
   */
  run(
    command: string,
    args: string[],
    opts: { cwd: string; stdin?: string },
  ): Promise<ProcessResult>;

  /** Run a command streaming stdout chunks (used for agent CLIs). */
  stream(
    command: string,
    args: string[],
    opts: { cwd: string; stdin?: string; signal?: AbortSignal },
    onChunk: (chunk: string) => void,
  ): Promise<ProcessResult>;
}

/** Join workspace-relative segments with `/` (paths are POSIX-style keys). */
export function joinPath(...segments: string[]): string {
  return segments
    .filter((s) => s.length > 0)
    .join("/")
    .replace(/\/+/g, "/");
}
