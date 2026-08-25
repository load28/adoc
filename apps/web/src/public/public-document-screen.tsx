import type { DocumentContent } from "@adoc/contracts";
import { ApiClient } from "@adoc/ui-domain";
import { Stack, Text } from "@atlaskit/primitives";
import { useQuery } from "@tanstack/react-query";

import { RoutePending } from "../shell/common-states";
import "./public-document.css";

const api = new ApiClient();

export function PublicDocumentScreen({ token }: Readonly<{ token: string }>) {
  const query = useQuery({
    queryKey: ["public-document", token],
    queryFn: ({ signal }) => api.publicDocument(token, signal),
    retry: false,
  });
  if (query.isPending) return <RoutePending />;
  if (query.error) return <PublicNotFound />;
  return (
    <main className="public-document">
      <article>
        <Stack space="space.250">
          <header>
            <h1>{query.data.title}</h1>
            <Text>
              v{query.data.versionNumber} · {new Date(query.data.publishedAt).toLocaleString()}
            </Text>
          </header>
          <PublicContent content={query.data.content as DocumentContent} token={token} />
        </Stack>
      </article>
    </main>
  );
}

function PublicNotFound() {
  return (
    <main className="public-document">
      <Stack space="space.150">
        <h1>문서를 찾을 수 없습니다</h1>
        <Text>링크가 만료됐거나 더 이상 공개되지 않는 문서입니다.</Text>
      </Stack>
    </main>
  );
}

function PublicContent({ content, token }: Readonly<{ content: DocumentContent; token: string }>) {
  return (
    <div className="public-content">
      {content.root.children.map((node) => (
        <PublicNode key={node.id} node={node} token={token} />
      ))}
    </div>
  );
}

function PublicNode({
  node,
  token,
}: Readonly<{ node: DocumentContent["root"]["children"][number]; token: string }>) {
  const text = "children" in node && Array.isArray(node.children) ? inlineText(node.children) : "";
  switch (node.type) {
    case "paragraph":
      return <p>{text}</p>;
    case "heading": {
      const Tag = `h${Math.min(6, Math.max(2, node.level + 1))}` as "h2";
      return <Tag>{text}</Tag>;
    }
    case "quote":
      return (
        <blockquote>
          {node.children.map((child) => (
            <PublicNode key={child.id} node={child} token={token} />
          ))}
        </blockquote>
      );
    case "callout":
      return (
        <aside>
          {node.children.map((child) => (
            <PublicNode key={child.id} node={child} token={token} />
          ))}
        </aside>
      );
    case "codeBlock":
      return (
        <pre>
          <code>{node.text}</code>
        </pre>
      );
    case "divider":
      return <hr />;
    case "image":
      return (
        <figure>
          <img
            src={`/public/v1/documents/${encodeURIComponent(token)}/files/${encodeURIComponent(node.assetId)}`}
            alt={node.alt ?? ""}
          />
          {node.caption && <figcaption>{node.caption}</figcaption>}
        </figure>
      );
    case "file":
      return <Text>첨부파일{node.caption ? `: ${node.caption}` : ""}</Text>;
    case "bulletList":
    case "orderedList":
    case "taskList":
      return (
        <ul>
          {node.items.map((item) => (
            <li key={item.id}>
              {item.children.map((child) => (
                <PublicNode key={child.id} node={child} token={token} />
              ))}
            </li>
          ))}
        </ul>
      );
    case "toggle":
      return (
        <details>
          <summary>{inlineText(node.summary)}</summary>
          {node.children.map((child) => (
            <PublicNode key={child.id} node={child} token={token} />
          ))}
        </details>
      );
    case "table":
      return (
        <div className="public-table">
          <table>
            <tbody>
              {node.rows.map((row) => (
                <tr key={row.id}>
                  {row.cells.map((cell) =>
                    cell.type === "tableHeader" ? (
                      <th key={cell.id} colSpan={cell.colspan} rowSpan={cell.rowspan}>
                        {cell.children.map((child) => (
                          <PublicNode key={child.id} node={child} token={token} />
                        ))}
                      </th>
                    ) : (
                      <td key={cell.id} colSpan={cell.colspan} rowSpan={cell.rowspan}>
                        {cell.children.map((child) => (
                          <PublicNode key={child.id} node={child} token={token} />
                        ))}
                      </td>
                    ),
                  )}
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      );
  }
}

function inlineText(children: unknown[]): string {
  return children
    .map((child) => {
      if (!child || typeof child !== "object") return "";
      const value = child as Record<string, unknown>;
      if (value.type === "text" && typeof value.text === "string") return value.text;
      if (value.type === "hardBreak") return "\n";
      return "";
    })
    .join("");
}
