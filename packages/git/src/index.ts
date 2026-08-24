import type { MergeConflictInput, ProcessRunnerPort, ProcessResult } from "@adoc/core";

/**
 * Document-centric Git layer (DESIGN.md §14–§17).
 *
 * Git is the collaboration model, but users never see raw git. This class
 * translates document-level operations to git plumbing over the
 * ProcessRunnerPort, so it runs identically under Node and Tauri.
 *
 *   git pull   → pullLatest()        최신 문서 가져오기
 *   git diff   → workingDiff()       변경사항 보기
 *   git commit → recordChanges()     변경 기록
 *   git push   → shareWithTeam()     팀에 공유
 *   git log    → historyFor()        문서 History
 *   conflict   → conflicts()/conflictVersions()  문서 충돌 해결
 */

export interface HistoryEntry {
  sha: string;
  author: string;
  date: string; // ISO 8601
  message: string;
}

export interface RepoStatus {
  branch: string;
  /** Workspace-relative paths with uncommitted changes (incl. untracked). */
  changed: string[];
  /** Paths currently in merge conflict. */
  conflicted: string[];
  ahead: number;
  behind: number;
}

export interface PullResult {
  ok: boolean;
  /** Files that ended up in conflict after the pull. */
  conflicts: string[];
  output: string;
}

const US = "\x1f"; // unit separator for safe log parsing

export class DocRepository {
  constructor(
    private runner: ProcessRunnerPort,
    readonly rootDir: string,
  ) {}

  private async git(args: string[], opts?: { allowFail?: boolean; stdin?: string }): Promise<ProcessResult> {
    const result = await this.runner.run("git", args, { cwd: this.rootDir, stdin: opts?.stdin });
    if (result.code !== 0 && !opts?.allowFail) {
      throw new Error(`git ${args.join(" ")} failed (${result.code}): ${result.stderr.trim() || result.stdout.trim()}`);
    }
    return result;
  }

  // -- lifecycle --------------------------------------------------------------

  async isRepository(): Promise<boolean> {
    const r = await this.git(["rev-parse", "--is-inside-work-tree"], { allowFail: true });
    return r.code === 0 && r.stdout.trim() === "true";
  }

  async init(): Promise<void> {
    await this.git(["init", "-b", "main"]);
  }

  async clone(remoteUrl: string): Promise<void> {
    await this.git(["clone", remoteUrl, "."]);
  }

  async setRemote(url: string): Promise<void> {
    const existing = await this.git(["remote", "get-url", "origin"], { allowFail: true });
    if (existing.code === 0) await this.git(["remote", "set-url", "origin", url]);
    else await this.git(["remote", "add", "origin", url]);
  }

  async currentBranch(): Promise<string> {
    const r = await this.git(["rev-parse", "--abbrev-ref", "HEAD"], { allowFail: true });
    return r.code === 0 ? r.stdout.trim() : "main";
  }

  // -- status / diff ------------------------------------------------------------

  async status(): Promise<RepoStatus> {
    const branch = await this.currentBranch();
    const porcelain = await this.git(["status", "--porcelain=v1"], { allowFail: true });
    const changed: string[] = [];
    const conflicted: string[] = [];
    for (const line of porcelain.stdout.split("\n")) {
      if (!line.trim()) continue;
      const xy = line.slice(0, 2);
      const path = line.slice(3).trim();
      if (xy.includes("U") || xy === "AA" || xy === "DD") conflicted.push(path);
      else changed.push(path);
    }

    let ahead = 0;
    let behind = 0;
    const counts = await this.git(
      ["rev-list", "--left-right", "--count", `${branch}...origin/${branch}`],
      { allowFail: true },
    );
    if (counts.code === 0) {
      const [a, b] = counts.stdout.trim().split(/\s+/).map(Number);
      ahead = a || 0;
      behind = b || 0;
    }
    return { branch, changed, conflicted, ahead, behind };
  }

  /** 변경사항 보기 — diff of uncommitted work (optionally one document). */
  async workingDiff(path?: string): Promise<string> {
    const args = ["diff", "HEAD", "--"];
    if (path) args.push(path);
    const r = await this.git(args, { allowFail: true });
    return r.stdout;
  }

  // -- sync --------------------------------------------------------------------

  /** 최신 문서 가져오기 — pull with merge (never rebase: history is shared). */
  async pullLatest(): Promise<PullResult> {
    await this.git(["fetch", "origin"], { allowFail: true });
    const branch = await this.currentBranch();
    const r = await this.git(["pull", "--no-rebase", "origin", branch], { allowFail: true });
    const status = await this.status();
    return {
      ok: r.code === 0,
      conflicts: status.conflicted,
      output: (r.stdout + r.stderr).trim(),
    };
  }

  /**
   * 변경 기록 — commit. Only ever called from an explicit human action
   * (DESIGN.md §11: AI never commits by default).
   */
  async recordChanges(message: string, paths?: string[]): Promise<string> {
    if (paths && paths.length > 0) await this.git(["add", "--", ...paths]);
    else await this.git(["add", "-A"]);
    const r = await this.git(["commit", "-m", message], { allowFail: true });
    if (r.code !== 0) {
      throw new Error(`commit failed: ${(r.stdout + r.stderr).trim()}`);
    }
    const sha = await this.git(["rev-parse", "HEAD"]);
    return sha.stdout.trim();
  }

  /** 팀에 공유 — push the current branch. */
  async shareWithTeam(): Promise<string> {
    const branch = await this.currentBranch();
    const r = await this.git(["push", "-u", "origin", branch]);
    return (r.stdout + r.stderr).trim();
  }

  // -- history (§17) -------------------------------------------------------------

  async historyFor(path: string, limit = 50): Promise<HistoryEntry[]> {
    const r = await this.git(
      ["log", `-n`, String(limit), `--format=%H${US}%an${US}%aI${US}%s`, "--follow", "--", path],
      { allowFail: true },
    );
    if (r.code !== 0) return [];
    return r.stdout
      .split("\n")
      .filter((line) => line.includes(US))
      .map((line) => {
        const [sha, author, date, message] = line.split(US);
        return { sha, author, date, message };
      });
  }

  async showVersion(path: string, sha: string): Promise<string> {
    const r = await this.git(["show", `${sha}:${path}`]);
    return r.stdout;
  }

  async diffBetween(path: string, fromSha: string, toSha: string): Promise<string> {
    const r = await this.git(["diff", fromSha, toSha, "--", path], { allowFail: true });
    return r.stdout;
  }

  /** 특정 버전 복원 — restore a document's content from an old commit. */
  async restoreVersion(path: string, sha: string): Promise<void> {
    await this.git(["restore", "--source", sha, "--worktree", "--", path]);
  }

  // -- conflicts (§15) -------------------------------------------------------------

  async conflicts(): Promise<string[]> {
    const r = await this.git(["diff", "--name-only", "--diff-filter=U"], { allowFail: true });
    return r.stdout.split("\n").filter((l) => l.trim() !== "");
  }

  /**
   * Extract the three-way versions of a conflicted file for the semantic
   * comparison UI and AI Merge (§15–§16). Conflict markers never reach the user.
   */
  async conflictVersions(path: string): Promise<MergeConflictInput> {
    const stage = async (n: 1 | 2 | 3): Promise<string> => {
      const r = await this.git(["show", `:${n}:${path}`], { allowFail: true });
      return r.code === 0 ? r.stdout : "";
    };
    return {
      path,
      base: await stage(1),
      current: await stage(2),
      incoming: await stage(3),
    };
  }

  /** Mark a conflicted document as resolved (after human review, §16). */
  async markResolved(path: string): Promise<void> {
    await this.git(["add", "--", path]);
  }

  /** Complete a merge once every conflict is resolved. */
  async completeMerge(message?: string): Promise<void> {
    await this.git(["commit", "--no-edit", ...(message ? ["-m", message] : [])]);
  }
}
