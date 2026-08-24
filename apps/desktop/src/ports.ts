import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { FileSystemPort, ProcessRunnerPort, ProcessResult } from "@adoc/core";

/**
 * Tauri implementations of the core ports. The Rust side only exposes a
 * filesystem and an allowlisted process runner — every piece of domain
 * logic runs here in the webview, identical to the CLI.
 */

export class TauriFileSystem implements FileSystemPort {
  readFile(path: string): Promise<string> {
    return invoke<string>("fs_read_file", { path });
  }

  writeFile(path: string, content: string): Promise<void> {
    return invoke("fs_write_file", { path, content });
  }

  exists(path: string): Promise<boolean> {
    return invoke<boolean>("fs_exists", { path });
  }

  mkdirp(path: string): Promise<void> {
    return invoke("fs_mkdirp", { path });
  }

  listFiles(dir: string): Promise<string[]> {
    return invoke<string[]>("fs_list_files", { dir });
  }

  remove(path: string): Promise<void> {
    return invoke("fs_remove", { path });
  }
}

let streamCounter = 0;

export class TauriProcessRunner implements ProcessRunnerPort {
  run(
    command: string,
    args: string[],
    opts: { cwd: string; stdin?: string },
  ): Promise<ProcessResult> {
    return invoke<ProcessResult>("proc_run", {
      command,
      args,
      cwd: opts.cwd,
      stdin: opts.stdin ?? null,
    });
  }

  async stream(
    command: string,
    args: string[],
    opts: { cwd: string; stdin?: string; signal?: AbortSignal },
    onChunk: (chunk: string) => void,
  ): Promise<ProcessResult> {
    const streamId = `s${Date.now()}-${streamCounter++}`;
    const unlisten = await listen<string>(`proc-chunk:${streamId}`, (event) => {
      onChunk(event.payload);
    });
    try {
      return await invoke<ProcessResult>("proc_stream", {
        streamId,
        command,
        args,
        cwd: opts.cwd,
        stdin: opts.stdin ?? null,
      });
    } finally {
      unlisten();
    }
  }
}
