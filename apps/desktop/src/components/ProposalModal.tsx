import type { MergeProposal, Proposal } from "@adoc/core";
import { DiffView } from "./DiffView";

export interface PendingProposal {
  /** Workspace-relative path the proposal applies to. */
  rel: string;
  proposal: Proposal | MergeProposal;
  /** Set when this proposal resolves a git conflict. */
  isConflictResolution?: boolean;
}

/**
 * AI 변경은 항상 제안이다 (§11):
 * Current Document → AI → Proposed Change → Diff → Accept / Reject.
 */
export function ProposalModal({
  pending,
  busy,
  onAccept,
  onReject,
}: {
  pending: PendingProposal;
  busy: boolean;
  onAccept: () => void;
  onReject: () => void;
}) {
  const unresolved = "unresolved" in pending.proposal ? pending.proposal.unresolved : [];
  return (
    <div className="modal-backdrop">
      <div className="modal">
        <div>
          <h3 style={{ margin: 0 }}>AI 제안 검토</h3>
          <div className="hint">
            {pending.rel}
            {pending.proposal.summary ? ` — ${pending.proposal.summary}` : ""}
          </div>
        </div>
        {unresolved.length > 0 && (
          <div className="unresolved">
            <b>해결되지 않은 모순 — 사람의 결정이 필요합니다:</b>
            <ul style={{ margin: "6px 0 0", paddingLeft: 18 }}>
              {unresolved.map((u, i) => (
                <li key={i}>
                  {u.section ? `§${u.section}: ` : ""}
                  {u.reason}
                </li>
              ))}
            </ul>
          </div>
        )}
        <DiffView diff={pending.proposal.diff} />
        <div className="modal-actions">
          <button onClick={onReject} disabled={busy}>
            거절
          </button>
          <button className="primary" onClick={onAccept} disabled={busy}>
            수락 — Working Tree에 적용
          </button>
        </div>
        <div className="hint">수락해도 커밋되지 않습니다. 변경 기록/공유는 Sync 탭에서 직접 수행합니다.</div>
      </div>
    </div>
  );
}
