/** Renders a unified diff with add/del coloring — the Diff Review of §11. */
export function DiffView({ diff }: { diff: string }) {
  const lines = diff.split("\n");
  return (
    <div className="diff">
      {lines.map((line, i) => {
        let cls = "ctx";
        if (line.startsWith("+") && !line.startsWith("+++")) cls = "add";
        else if (line.startsWith("-") && !line.startsWith("---")) cls = "del";
        else if (line.startsWith("@@")) cls = "hunk";
        return (
          <span key={i} className={cls}>
            {line || " "}
          </span>
        );
      })}
    </div>
  );
}
