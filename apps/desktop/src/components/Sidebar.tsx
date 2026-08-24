import { useMemo } from "react";
import type { DocumentRef } from "@adoc/core";

export function Sidebar({
  docs,
  selected,
  onSelect,
}: {
  docs: DocumentRef[];
  selected: string | null;
  onSelect: (rel: string) => void;
}) {
  const groups = useMemo(() => {
    const byProject = new Map<string, DocumentRef[]>();
    const decisions: DocumentRef[] = [];
    for (const doc of docs) {
      if (doc.path.startsWith("decisions/")) {
        decisions.push(doc);
      } else {
        const key = doc.project ?? "general";
        if (!byProject.has(key)) byProject.set(key, []);
        byProject.get(key)!.push(doc);
      }
    }
    return { byProject, decisions };
  }, [docs]);

  return (
    <div className="sidebar">
      {[...groups.byProject.entries()].map(([project, refs]) => (
        <div key={project}>
          <h3>{project}</h3>
          {refs.map((doc) => (
            <div
              key={doc.path}
              className={`doc-item ${selected === doc.path ? "selected" : ""}`}
              onClick={() => onSelect(doc.path)}
              title={doc.path}
            >
              {doc.title} <span className="meta">{doc.type}</span>
              {doc.status && <span className="badge">{doc.status}</span>}
            </div>
          ))}
        </div>
      ))}
      {groups.decisions.length > 0 && (
        <div>
          <h3>Decisions</h3>
          {groups.decisions.map((doc) => (
            <div
              key={doc.path}
              className={`doc-item ${selected === doc.path ? "selected" : ""}`}
              onClick={() => onSelect(doc.path)}
              title={doc.path}
            >
              {doc.title}
              {doc.status && <span className="badge">{doc.status}</span>}
            </div>
          ))}
        </div>
      )}
      {docs.length === 0 && <div className="hint">아직 문서가 없습니다. AI 탭에서 첫 문서를 작성해보세요.</div>}
    </div>
  );
}
