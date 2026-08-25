import type {
  DocumentContent,
  DocumentContent_Block,
  DocumentContent_Inline,
} from "@adoc/contracts";
import type { ReactNode } from "react";

import "./document-content.css";

export function ContentRenderer({
  content,
  assetUrl,
}: Readonly<{
  content: DocumentContent;
  assetUrl?: (assetId: string) => string;
}>) {
  return (
    <div className="document-content">
      {content.root.children.map((block) => (
        <ContentBlock key={block.id} block={block} assetUrl={assetUrl} />
      ))}
    </div>
  );
}

function ContentBlock({
  block,
  assetUrl,
}: Readonly<{
  block: DocumentContent_Block;
  assetUrl?: (assetId: string) => string;
}>) {
  switch (block.type) {
    case "paragraph":
      return <p>{inline(block.children)}</p>;
    case "heading": {
      const Tag = `h${Math.min(6, Math.max(1, block.level))}` as "h1";
      return <Tag>{inline(block.children)}</Tag>;
    }
    case "quote":
      return (
        <blockquote>
          <BlockChildren blocks={block.children} assetUrl={assetUrl} />
        </blockquote>
      );
    case "callout":
      return (
        <aside data-tone={block.tone}>
          <BlockChildren blocks={block.children} assetUrl={assetUrl} />
        </aside>
      );
    case "codeBlock":
      return (
        <pre data-language={block.language ?? undefined}>
          <code>{block.text}</code>
        </pre>
      );
    case "divider":
      return <hr />;
    case "image":
      return (
        <figure>
          {assetUrl ? <img src={assetUrl(block.assetId)} alt={block.alt ?? ""} /> : null}
          {block.caption ? <figcaption>{block.caption}</figcaption> : null}
        </figure>
      );
    case "file":
      return assetUrl ? (
        <a href={assetUrl(block.assetId)}>{block.caption ?? "Attachment"}</a>
      ) : (
        <span>{block.caption ?? "Attachment"}</span>
      );
    case "toggle":
      return (
        <details>
          <summary>{inline(block.summary)}</summary>
          <BlockChildren blocks={block.children} assetUrl={assetUrl} />
        </details>
      );
    case "bulletList":
    case "orderedList": {
      const List = block.type === "orderedList" ? "ol" : "ul";
      return (
        <List start={block.type === "orderedList" ? (block.start ?? undefined) : undefined}>
          {block.items.map((item) => (
            <li key={item.id}>
              <BlockChildren blocks={item.children} assetUrl={assetUrl} />
            </li>
          ))}
        </List>
      );
    }
    case "taskList":
      return (
        <ul className="document-task-list">
          {block.items.map((item) => (
            <li key={item.id}>
              <input
                type="checkbox"
                checked={Boolean(item.checked)}
                readOnly
                aria-label="Task status"
              />
              <BlockChildren blocks={item.children} assetUrl={assetUrl} />
            </li>
          ))}
        </ul>
      );
    case "table":
      return (
        <div className="document-table-scroll">
          <table>
            <tbody>
              {block.rows.map((row) => (
                <tr key={row.id}>
                  {row.cells.map((cell) => {
                    const Cell = cell.type === "tableHeader" ? "th" : "td";
                    return (
                      <Cell key={cell.id} colSpan={cell.colspan} rowSpan={cell.rowspan}>
                        <BlockChildren blocks={cell.children} assetUrl={assetUrl} />
                      </Cell>
                    );
                  })}
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      );
  }
}

function BlockChildren({
  blocks,
  assetUrl,
}: Readonly<{ blocks: DocumentContent_Block[]; assetUrl?: (assetId: string) => string }>) {
  return blocks.map((block) => <ContentBlock key={block.id} block={block} assetUrl={assetUrl} />);
}

function inline(children: DocumentContent_Inline[]): ReactNode[] {
  const occurrences = new Map<string, number>();
  return children.map((child) => {
    const identity = JSON.stringify(child);
    const occurrence = (occurrences.get(identity) ?? 0) + 1;
    occurrences.set(identity, occurrence);
    const key = `${identity}:${occurrence}`;
    if (child.type === "hardBreak") return <br key={key} />;
    let value: ReactNode = child.text;
    for (const mark of child.marks ?? []) {
      if (mark.type === "bold") value = <strong>{value}</strong>;
      else if (mark.type === "italic") value = <em>{value}</em>;
      else if (mark.type === "underline") value = <u>{value}</u>;
      else if (mark.type === "strike") value = <s>{value}</s>;
      else if (mark.type === "code") value = <code>{value}</code>;
      else if (mark.type === "subscript") value = <sub>{value}</sub>;
      else if (mark.type === "superscript") value = <sup>{value}</sup>;
      else if (mark.type === "link") value = <a href={mark.href}>{value}</a>;
      else if (mark.type === "highlight") value = <mark data-token={mark.token}>{value}</mark>;
      else if (mark.type === "textColor")
        value = <span data-color-token={mark.token}>{value}</span>;
    }
    return <span key={key}>{value}</span>;
  });
}
