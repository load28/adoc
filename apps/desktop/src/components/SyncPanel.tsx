import { useState } from "react";
import { splitFrontmatter, type MergeConflictInput } from "@adoc/core";
import type { RepoStatus } from "@adoc/git";

/**
 * Git-backed collaboration, translated to document UX (§14–§15):
 *   최신 문서 가져오기 / 변경 기록 / 팀에 공유 / 문서 충돌 해결.
 * Conflict markers never reach the user — they see CURRENT vs INCOMING
 * with the five resolution choices from the design.
 */
export function SyncPanel({
  status,
  conflict,
  busy,
  onPull,
  onShare,
  onOpenConflict,
  onResolveConflict,
  onAiMerge,
}: {
  status: RepoStatus | null;
  conflict: MergeConflictInput | null;
  busy: string | null;
  onPull: () => void;
  onShare: (message: string) => void;
  onOpenConflict: (path: string) => void;
  onResolveConflict: (path: string, content: string) => void;
  onAiMerge: (conflict: MergeConflictInput) => void;
}) {
  const [message, setMessage] = useState("");

  return (
    <div className="panel-body">
      <h4>동기화</h4>
      {status && (
        <div className="hint">
          브랜치 {status.branch} · 내 변경 {status.changed.length}건
          {status.behind > 0 ? ` · 팀 변경 ${status.behind}건 대기` : ""}
          {status.ahead > 0 ? ` · 공유 대기 ${status.ahead}건` : ""}
        </div>
      )}
      <button disabled={busy !== null} onClick={onPull}>
        {busy === "pull" ? "가져오는 중…" : "최신 문서 가져오기"}
      </button>

      <h4>팀에 공유</h4>
      {status && status.changed.length > 0 && (
        <div className="hint">{status.changed.join(", ")}</div>
      )}
      <input
        placeholder="변경 설명 (예: Architecture rationale 개선)"
        value={message}
        onChange={(e) => setMessage(e.target.value)}
      />
      <button
        className="primary"
        disabled={busy !== null || message.trim() === "" || !status || status.changed.length === 0}
        onClick={() => {
          onShare(message.trim());
          setMessage("");
        }}
      >
        {busy === "share" ? "공유 중…" : "변경 기록 + 팀에 공유"}
      </button>

      {status && status.conflicted.length > 0 && (
        <>
          <h4 style={{ color: "var(--red)" }}>문서 충돌</h4>
          {status.conflicted.map((path) => (
            <button key={path} onClick={() => onOpenConflict(path)}>
              {path}
            </button>
          ))}
        </>
      )}

      {conflict && (
        <>
          <h4>충돌 해결 — {conflict.path}</h4>
          <div className="conflict-columns">
            <div>
              <div className="hint">Current (내 버전)</div>
              <pre>{conflict.current}</pre>
            </div>
            <div>
              <div className="hint">Incoming (팀 버전)</div>
              <pre>{conflict.incoming}</pre>
            </div>
          </div>
          <div className="chip-row">
            <button disabled={busy !== null} onClick={() => onResolveConflict(conflict.path, conflict.current)}>
              Current 사용
            </button>
            <button disabled={busy !== null} onClick={() => onResolveConflict(conflict.path, conflict.incoming)}>
              Incoming 사용
            </button>
            <button
              disabled={busy !== null}
              onClick={() =>
                // keep CURRENT as-is (frontmatter included), append INCOMING's body
                onResolveConflict(
                  conflict.path,
                  conflict.current.trimEnd() +
                    "\n\n" +
                    splitFrontmatter(conflict.incoming).body.trimEnd() +
                    "\n",
                )
              }
            >
              둘 다 유지
            </button>
            <button className="primary" disabled={busy !== null} onClick={() => onAiMerge(conflict)}>
              {busy === "merge" ? "병합 중…" : "AI로 병합"}
            </button>
          </div>
          <div className="hint">
            직접 수정: 에디터에서 파일을 열어 편집한 뒤 "Current 사용" 대신 저장 후 해결 표시하세요. AI 병합은
            제안일 뿐이며, 모순은 임의로 결정하지 않습니다 (§16).
          </div>
        </>
      )}
    </div>
  );
}
