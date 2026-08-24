import type { HistoryEntry } from "@adoc/git";
import { DiffView } from "./DiffView";

/**
 * Document History (§17): git log rendered as a document timeline —
 * no SHAs up front, author + message + date. Selecting an entry shows the
 * diff against its parent; any version can be restored to the working tree.
 */
export function HistoryPanel({
  entries,
  selectedSha,
  diff,
  busy,
  onSelect,
  onRestore,
}: {
  entries: HistoryEntry[] | null;
  selectedSha: string | null;
  diff: string;
  busy: boolean;
  onSelect: (entry: HistoryEntry) => void;
  onRestore: (entry: HistoryEntry) => void;
}) {
  if (entries === null) return <div className="panel-body hint">문서를 선택하면 History가 표시됩니다.</div>;
  if (entries.length === 0) return <div className="panel-body hint">아직 기록된 변경이 없습니다.</div>;

  return (
    <div className="panel-body">
      {entries.map((entry) => (
        <div key={entry.sha}>
          <div
            className="history-item"
            style={selectedSha === entry.sha ? { borderColor: "var(--accent)" } : undefined}
            onClick={() => onSelect(entry)}
          >
            <div className="when">
              {new Date(entry.date).toLocaleDateString("ko-KR", { month: "short", day: "numeric" })} ·{" "}
              <span className="who">{entry.author}</span>
            </div>
            <div>{entry.message}</div>
          </div>
          {selectedSha === entry.sha && (
            <div style={{ marginTop: 6, display: "flex", flexDirection: "column", gap: 6 }}>
              {diff ? <DiffView diff={diff} /> : <div className="hint">이 버전에서 변경 없음</div>}
              <button disabled={busy} onClick={() => onRestore(entry)}>
                이 버전으로 복원 (Working Tree)
              </button>
            </div>
          )}
        </div>
      ))}
    </div>
  );
}
