import { mkdtemp, rm, writeFile, mkdir } from "node:fs/promises";
import { tmpdir } from "node:os";
import * as path from "node:path";
import { afterAll, beforeAll, describe, expect, it } from "vitest";
import { DocRepository } from "@adoc/git";
import { NodeProcessRunner } from "@adoc/node-ports";

const runner = new NodeProcessRunner();

async function git(cwd: string, ...args: string[]): Promise<string> {
  const r = await runner.run("git", args, { cwd });
  if (r.code !== 0) throw new Error(`git ${args.join(" ")}: ${r.stderr}`);
  return r.stdout;
}

async function configureIdentity(cwd: string, name: string): Promise<void> {
  await git(cwd, "config", "user.name", name);
  await git(cwd, "config", "user.email", `${name}@example.com`);
}

describe("DocRepository", () => {
  let root: string;
  let remote: string;
  let dirA: string;
  let dirB: string;
  let repoA: DocRepository;
  let repoB: DocRepository;
  const DOC = "projects/app/documents/auth.md";

  beforeAll(async () => {
    root = await mkdtemp(path.join(tmpdir(), "adoc-git-"));
    remote = path.join(root, "remote.git");
    dirA = path.join(root, "a");
    dirB = path.join(root, "b");
    await mkdir(remote, { recursive: true });
    await git(remote, "init", "--bare", "-b", "main");
    await git(root, "clone", remote, dirA);
    await git(root, "clone", remote, dirB);
    await configureIdentity(dirA, "minmin");
    await configureIdentity(dirB, "chulsoo");
    repoA = new DocRepository(runner, dirA);
    repoB = new DocRepository(runner, dirB);
  });

  afterAll(async () => {
    await rm(root, { recursive: true, force: true });
  });

  it("records changes and shares with the team", async () => {
    await mkdir(path.dirname(path.join(dirA, DOC)), { recursive: true });
    await writeFile(path.join(dirA, DOC), "# Auth\n\n## Summary\n\n초안.\n");
    const sha = await repoA.recordChanges("Initial design");
    expect(sha).toMatch(/^[0-9a-f]{40}$/);
    await repoA.shareWithTeam();

    const pulled = await repoB.pullLatest();
    expect(pulled.ok).toBe(true);
    expect(pulled.conflicts).toEqual([]);
  });

  it("exposes document history with author and message (§17)", async () => {
    await writeFile(path.join(dirA, DOC), "# Auth\n\n## Summary\n\n서버 세션으로 관리한다.\n");
    await repoA.recordChanges("Architecture rationale 개선");
    await repoA.shareWithTeam();

    const history = await repoA.historyFor(DOC);
    expect(history.length).toBe(2);
    expect(history[0].message).toBe("Architecture rationale 개선");
    expect(history[0].author).toBe("minmin");
    expect(history[1].message).toBe("Initial design");

    const old = await repoA.showVersion(DOC, history[1].sha);
    expect(old).toContain("초안.");

    const diff = await repoA.diffBetween(DOC, history[1].sha, history[0].sha);
    expect(diff).toContain("-초안.");
    expect(diff).toContain("+서버 세션으로 관리한다.");
  });

  it("restores an old version into the working tree", async () => {
    const history = await repoA.historyFor(DOC);
    await repoA.restoreVersion(DOC, history[1].sha);
    const status = await repoA.status();
    expect(status.changed).toContain(DOC);
    // put it back
    await repoA.restoreVersion(DOC, history[0].sha);
  });

  it("extracts BASE/CURRENT/INCOMING for conflicts (§15, §16)", async () => {
    // B is behind: create competing edits on the same line.
    await writeFile(path.join(dirB, DOC), "# Auth\n\n## Summary\n\n서버 세션과 refresh token을 함께 사용한다.\n");
    await repoB.recordChanges("Alternative B 추가");

    await writeFile(path.join(dirA, DOC), "# Auth\n\n## Summary\n\n서버 세션만 사용한다.\n");
    await repoA.recordChanges("세션 단일화");
    await repoA.shareWithTeam();

    const pulled = await repoB.pullLatest();
    expect(pulled.ok).toBe(false);
    expect(pulled.conflicts).toEqual([DOC]);
    expect(await repoB.conflicts()).toEqual([DOC]);

    const versions = await repoB.conflictVersions(DOC);
    expect(versions.base).toContain("초안.");
    expect(versions.current).toContain("refresh token");
    expect(versions.incoming).toContain("서버 세션만 사용한다");

    // resolve after (simulated) human review and complete the merge
    await writeFile(path.join(dirB, DOC), "# Auth\n\n## Summary\n\n서버 세션을 기본으로, refresh token을 보조로 사용한다.\n");
    await repoB.markResolved(DOC);
    await repoB.completeMerge();
    expect(await repoB.conflicts()).toEqual([]);
    await repoB.shareWithTeam();
  });
});
