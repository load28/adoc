import { useCallback, useEffect, useState } from "react";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import type { DocumentRef, MergeConflictInput, Suggestion } from "@adoc/core";
import type { HistoryEntry, RepoStatus } from "@adoc/git";
import { AppServices } from "./services";
import { Sidebar } from "./components/Sidebar";
import { AiPanel } from "./components/AiPanel";
import { HistoryPanel } from "./components/HistoryPanel";
import { SyncPanel } from "./components/SyncPanel";
import { ProposalModal, type PendingProposal } from "./components/ProposalModal";

type Tab = "ai" | "history" | "sync";

export default function App() {
  const [services, setServices] = useState<AppServices | null>(null);
  const [docs, setDocs] = useState<DocumentRef[]>([]);
  const [selected, setSelected] = useState<string | null>(null);
  const [content, setContent] = useState("");
  const [saved, setSaved] = useState("");
  const [tab, setTab] = useState<Tab>("ai");
  const [busy, setBusy] = useState<string | null>(null);
  const [stream, setStream] = useState("");
  const [suggestions, setSuggestions] = useState<Suggestion[] | null>(null);
  const [pending, setPending] = useState<PendingProposal | null>(null);
  const [status, setStatus] = useState<RepoStatus | null>(null);
  const [history, setHistory] = useState<HistoryEntry[] | null>(null);
  const [historySha, setHistorySha] = useState<string | null>(null);
  const [historyDiff, setHistoryDiff] = useState("");
  const [conflict, setConflict] = useState<MergeConflictInput | null>(null);
  const [message, setMessage] = useState<string | null>(null);

  const dirty = content !== saved;

  const notify = (text: string) => {
    setMessage(text);
    window.setTimeout(() => setMessage(null), 5000);
  };

  const fail = (err: unknown) => {
    const text = err instanceof Error ? err.message : String(err);
    setMessage(text);
    window.setTimeout(() => setMessage(null), 8000);
  };

  const refresh = useCallback(async (svc: AppServices) => {
    setDocs(await svc.workspace.listDocuments());
    setStatus(await svc.repo.status());
  }, []);

  // Reopen the last workspace on launch.
  useEffect(() => {
    const last = localStorage.getItem("adoc.workspace");
    if (last) {
      AppServices.open(last)
        .then(async (svc) => {
          setServices(svc);
          await refresh(svc);
        })
        .catch(() => localStorage.removeItem("adoc.workspace"));
    }
  }, [refresh]);

  const openWorkspace = async (mode: "open" | "init") => {
    try {
      const dir = await openDialog({ directory: true, title: "워크스페이스 폴더 선택" });
      if (typeof dir !== "string") return;
      const svc =
        mode === "open"
          ? await AppServices.open(dir)
          : await AppServices.init(dir, dir.split("/").pop() ?? "workspace");
      localStorage.setItem("adoc.workspace", dir);
      setServices(svc);
      setSelected(null);
      setContent("");
      setSaved("");
      await refresh(svc);
    } catch (err) {
      fail(err);
    }
  };

  const selectDoc = async (rel: string) => {
    if (!services) return;
    try {
      const raw = await services.workspace.readRaw(rel);
      setSelected(rel);
      setContent(raw);
      setSaved(raw);
      setSuggestions(null);
      setHistory(await services.repo.historyFor(rel));
      setHistorySha(null);
      setHistoryDiff("");
    } catch (err) {
      fail(err);
    }
  };

  const saveDoc = async () => {
    if (!services || !selected) return;
    try {
      await services.workspace.writeRaw(selected, content);
      setSaved(content);
      await refresh(services);
      notify("저장됨 (Working Tree). Sync 탭에서 팀에 공유할 수 있습니다.");
    } catch (err) {
      fail(err);
    }
  };

  const withBusy = async (label: string, fn: () => Promise<void>) => {
    setBusy(label);
    setStream("");
    try {
      await fn();
    } catch (err) {
      fail(err);
    } finally {
      setBusy(null);
    }
  };

  const onStream = (chunk: string) => setStream((prev) => (prev + chunk).slice(-4000));

  // -- AI actions: every result is a Proposal (§11) ---------------------------

  const compose = (notes: string, typeId: string, id: string, project: string) =>
    withBusy("compose", async () => {
      if (!services) return;
      const engine = services.engine(onStream);
      const result = await engine.compose({ notes, typeId, init: { id } });
      const rel =
        typeId === "decision"
          ? `decisions/${id}.md`
          : `projects/${project}/documents/${id}.md`;
      setPending({ rel, proposal: result.proposal });
    });

  const rewrite = (goal: string) =>
    withBusy("rewrite", async () => {
      if (!services || !selected) return;
      const document = await services.workspace.readDocument(selected);
      const context = await services.contextFor(document.id);
      const proposal = await services.engine(onStream).rewrite({ document, goal, context });
      setPending({ rel: selected, proposal });
    });

  const critique = () =>
    withBusy("critique", async () => {
      if (!services || !selected) return;
      const document = await services.workspace.readDocument(selected);
      const context = await services.contextFor(document.id);
      setSuggestions(await services.engine(onStream).critique({ document, context }));
    });

  const acceptProposal = async () => {
    if (!services || !pending) return;
    try {
      await services.workspace.writeRaw(pending.rel, pending.proposal.after);
      const isMerge =
        pending.isConflictResolution &&
        !("unresolved" in pending.proposal && pending.proposal.unresolved.length > 0);
      if (isMerge) {
        await services.repo.markResolved(pending.rel);
        setConflict(null);
        notify("충돌이 해결되었습니다. Sync 탭에서 공유하세요.");
      } else {
        notify("Working Tree에 적용되었습니다. 커밋은 Sync 탭에서 직접 수행합니다.");
      }
      setPending(null);
      await refresh(services);
      await selectDoc(pending.rel);
    } catch (err) {
      fail(err);
    }
  };

  // -- Git actions (§14–§16) ---------------------------------------------------

  const pull = () =>
    withBusy("pull", async () => {
      if (!services) return;
      const result = await services.repo.pullLatest();
      await refresh(services);
      if (result.conflicts.length > 0) {
        notify(`문서 충돌 ${result.conflicts.length}건 — Sync 탭에서 해결하세요.`);
      } else {
        notify(result.ok ? "최신 문서를 가져왔습니다." : `가져오기 실패: ${result.output}`);
      }
      if (selected) await selectDoc(selected);
    });

  const share = (msg: string) =>
    withBusy("share", async () => {
      if (!services) return;
      await services.repo.recordChanges(msg);
      try {
        await services.repo.shareWithTeam();
        notify("팀에 공유되었습니다.");
      } catch (err) {
        notify(`기록됨 — 공유(push) 실패: ${err instanceof Error ? err.message : err}`);
      }
      await refresh(services);
    });

  const openConflict = async (path: string) => {
    if (!services) return;
    try {
      setConflict(await services.repo.conflictVersions(path));
    } catch (err) {
      fail(err);
    }
  };

  const resolveConflict = (path: string, resolvedContent: string) =>
    withBusy("resolve", async () => {
      if (!services) return;
      await services.workspace.writeRaw(path, resolvedContent);
      await services.repo.markResolved(path);
      setConflict(null);
      await refresh(services);
      notify("충돌이 해결되었습니다. Sync 탭에서 공유하세요.");
    });

  const aiMerge = (input: MergeConflictInput) =>
    withBusy("merge", async () => {
      if (!services) return;
      const proposal = await services.engine(onStream).merge(input);
      setPending({ rel: input.path, proposal, isConflictResolution: true });
    });

  // -- History (§17) -------------------------------------------------------------

  const selectHistory = async (entry: HistoryEntry) => {
    if (!services || !selected || !history) return;
    try {
      setHistorySha(entry.sha);
      const idx = history.findIndex((h) => h.sha === entry.sha);
      const parent = history[idx + 1];
      setHistoryDiff(
        parent ? await services.repo.diffBetween(selected, parent.sha, entry.sha) : "",
      );
    } catch (err) {
      fail(err);
    }
  };

  const restoreVersion = (entry: HistoryEntry) =>
    withBusy("restore", async () => {
      if (!services || !selected) return;
      await services.repo.restoreVersion(selected, entry.sha);
      await selectDoc(selected);
      await refresh(services);
      notify("복원되었습니다 (Working Tree). 공유 전까지 팀에는 영향이 없습니다.");
    });

  // ----------------------------------------------------------------------------

  if (!services) {
    return (
      <div className="welcome">
        <h1>adoc</h1>
        <div className="hint" style={{ textAlign: "center" }}>
          Git 기반 AI 협업 문서.
          <br />
          Git Repository가 Source of Truth입니다 — 워크스페이스 폴더를 선택하세요.
        </div>
        <div style={{ display: "flex", gap: 8 }}>
          <button className="primary" onClick={() => openWorkspace("open")}>
            워크스페이스 열기
          </button>
          <button onClick={() => openWorkspace("init")}>새 워크스페이스 만들기</button>
        </div>
        {message && <div className="hint">{message}</div>}
      </div>
    );
  }

  return (
    <div className="app">
      <div className="topbar">
        <span className="title">adoc</span>
        <span className="badge">{services.workspace.config.name}</span>
        <span className="hint">{services.workspace.rootDir}</span>
        <span className="spacer" />
        <button onClick={() => openWorkspace("open")}>워크스페이스 변경</button>
      </div>

      <Sidebar docs={docs} selected={selected} onSelect={selectDoc} />

      <div className="editor-area">
        <div className="editor-toolbar">
          <span className="path">{selected ?? "문서를 선택하거나 AI 탭에서 새로 작성하세요"}</span>
          {dirty && <span className="badge">수정됨</span>}
          <button className="primary" disabled={!selected || !dirty} onClick={saveDoc}>
            저장
          </button>
        </div>
        <textarea
          className="editor-textarea"
          value={content}
          spellCheck={false}
          placeholder="문서는 Markdown + Frontmatter 개방 포맷으로 저장됩니다 (§2.2)."
          disabled={!selected}
          onChange={(e) => setContent(e.target.value)}
        />
      </div>

      <div className="panel">
        <div className="panel-tabs">
          {(["ai", "history", "sync"] as Tab[]).map((t) => (
            <button key={t} className={tab === t ? "active" : ""} onClick={() => setTab(t)}>
              {t === "ai" ? "AI" : t === "history" ? "History" : "Sync"}
            </button>
          ))}
        </div>
        {tab === "ai" && (
          <AiPanel
            hasDocument={selected !== null}
            busy={busy}
            stream={stream}
            suggestions={suggestions}
            onCompose={compose}
            onRewrite={rewrite}
            onCritique={critique}
          />
        )}
        {tab === "history" && (
          <HistoryPanel
            entries={history}
            selectedSha={historySha}
            diff={historyDiff}
            busy={busy !== null}
            onSelect={selectHistory}
            onRestore={restoreVersion}
          />
        )}
        {tab === "sync" && (
          <SyncPanel
            status={status}
            conflict={conflict}
            busy={busy}
            onPull={pull}
            onShare={share}
            onOpenConflict={openConflict}
            onResolveConflict={resolveConflict}
            onAiMerge={aiMerge}
          />
        )}
      </div>

      <div className="statusbar">
        <span>{status ? `브랜치 ${status.branch}` : ""}</span>
        <span>{status && status.changed.length > 0 ? `변경 ${status.changed.length}건` : ""}</span>
        <span style={{ flex: 1 }} />
        <span>{busy ? `${busy}…` : message ?? ""}</span>
      </div>

      {pending && (
        <ProposalModal
          pending={pending}
          busy={busy !== null}
          onAccept={acceptProposal}
          onReject={() => setPending(null)}
        />
      )}
    </div>
  );
}
