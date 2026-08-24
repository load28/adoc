import { useState } from "react";
import type { Suggestion } from "@adoc/core";

const REWRITE_GOALS = [
  "더 간결하게",
  "쉽게 설명",
  "논리 구조 개선",
  "기술적으로 구체화",
  "중복 제거",
  "근거 강화",
  "반론 추가",
];

/**
 * AI roles are separated (§10): Composer / Rewriter / Critic.
 * Every action produces a Proposal reviewed in the ProposalModal — the panel
 * itself never mutates the document.
 */
export function AiPanel({
  hasDocument,
  busy,
  stream,
  suggestions,
  onCompose,
  onRewrite,
  onCritique,
}: {
  hasDocument: boolean;
  busy: string | null;
  stream: string;
  suggestions: Suggestion[] | null;
  onCompose: (notes: string, typeId: string, id: string, project: string) => void;
  onRewrite: (goal: string) => void;
  onCritique: () => void;
}) {
  const [notes, setNotes] = useState("");
  const [typeId, setTypeId] = useState("design");
  const [docId, setDocId] = useState("");
  const [project, setProject] = useState("general");
  const [customGoal, setCustomGoal] = useState("");

  return (
    <div className="panel-body">
      <h4>Composer — 생각을 문서로</h4>
      <div className="hint">
        자유롭게 생각을 적으면 Intent 분석 → 인지 규칙 적용을 거쳐 구조화된 문서 초안을 제안합니다 (§6–§7).
      </div>
      <textarea
        rows={5}
        placeholder={"예)\n지금 인증에서 토큰을 앱에서 관리하고 있는데\n서버 세션으로 바꾸려고 한다.\nPR 124에서 한번 실험했다."}
        value={notes}
        onChange={(e) => setNotes(e.target.value)}
      />
      <div className="form-grid">
        <label>종류</label>
        <select value={typeId} onChange={(e) => setTypeId(e.target.value)}>
          <option value="design">Design</option>
          <option value="proposal">Proposal</option>
          <option value="decision">Decision</option>
        </select>
        <label>문서 ID</label>
        <input placeholder="auth-v2" value={docId} onChange={(e) => setDocId(e.target.value)} />
        <label>프로젝트</label>
        <input value={project} onChange={(e) => setProject(e.target.value)} />
      </div>
      <button
        className="primary"
        disabled={busy !== null || notes.trim() === "" || docId.trim() === ""}
        onClick={() => onCompose(notes, typeId, docId.trim(), project.trim() || "general")}
      >
        {busy === "compose" ? "작성 중…" : "AI로 문서 작성"}
      </button>

      <h4>Rewriter — 목적에 맞게 개선</h4>
      <div className="chip-row">
        {REWRITE_GOALS.map((goal) => (
          <button key={goal} disabled={!hasDocument || busy !== null} onClick={() => onRewrite(goal)}>
            {goal}
          </button>
        ))}
      </div>
      <div style={{ display: "flex", gap: 6 }}>
        <input
          style={{ flex: 1 }}
          placeholder="직접 목표 입력…"
          value={customGoal}
          onChange={(e) => setCustomGoal(e.target.value)}
        />
        <button
          disabled={!hasDocument || busy !== null || customGoal.trim() === ""}
          onClick={() => {
            onRewrite(customGoal.trim());
            setCustomGoal("");
          }}
        >
          실행
        </button>
      </div>

      <h4>Critic — 문제 분석 (문서를 바꾸지 않음)</h4>
      <button disabled={!hasDocument || busy !== null} onClick={onCritique}>
        {busy === "critique" ? "분석 중…" : "인지 규칙으로 검토"}
      </button>
      {suggestions !== null &&
        (suggestions.length === 0 ? (
          <div className="hint">발견된 문제가 없습니다.</div>
        ) : (
          suggestions.map((s, i) => (
            <div key={i} className={`suggestion ${s.severity}`}>
              <div className="rule">
                [{s.severity}]{s.ruleId ? ` ${s.ruleId}` : ""}
                {s.section ? ` · §${s.section}` : ""}
              </div>
              {s.message}
              {s.proposal && <div className="hint">→ {s.proposal}</div>}
            </div>
          ))
        ))}

      {busy !== null && stream && <div className="stream-box">{stream}</div>}
    </div>
  );
}
