import type {
  DocumentContent,
  DocumentContent_Block,
  DocumentContent_Inline,
} from "@adoc/contracts";
import { validateContract } from "@adoc/contracts";

import { createUuid } from "./id-factory";

export type ImportFormat = "markdown" | "plain";
export type ExportFormat = ImportFormat;

export function importDocumentText(
  source: string,
  format: ImportFormat,
  idFactory: () => string = createUuid,
): DocumentContent {
  const normalized = source.replace(/\r\n?/g, "\n");
  const blocks =
    format === "plain" ? plainBlocks(normalized, idFactory) : markdownBlocks(normalized, idFactory);
  const content: DocumentContent = {
    schemaVersion: 1,
    root: {
      type: "doc",
      children: blocks.length > 0 ? blocks : [paragraph("", idFactory)],
    },
  };
  if (!validateContract("content", content).valid) throw new Error("IMPORT_CONTENT_INVALID");
  return content;
}

export function exportDocumentText(content: DocumentContent, format: ExportFormat): string {
  if (!validateContract("content", content).valid) throw new Error("EXPORT_CONTENT_INVALID");
  return content.root.children
    .map((block) => serializeBlock(block, format))
    .join("\n\n")
    .trimEnd();
}

function plainBlocks(source: string, idFactory: () => string): DocumentContent_Block[] {
  return source
    .split(/\n{2,}/)
    .map((value) => value.trimEnd())
    .filter(Boolean)
    .map((value) => paragraph(value, idFactory));
}

function markdownBlocks(source: string, idFactory: () => string): DocumentContent_Block[] {
  const lines = source.split("\n");
  const result: DocumentContent_Block[] = [];
  for (let index = 0; index < lines.length; ) {
    const line = lines[index] ?? "";
    if (!line.trim()) {
      index += 1;
      continue;
    }
    const fence = line.match(/^```([\w+-]*)\s*$/);
    if (fence) {
      const body: string[] = [];
      index += 1;
      while (index < lines.length && !/^```\s*$/.test(lines[index] ?? "")) {
        body.push(lines[index] ?? "");
        index += 1;
      }
      if (index < lines.length) index += 1;
      result.push({
        id: idFactory(),
        type: "codeBlock",
        language: fence[1] || null,
        text: body.join("\n"),
      });
      continue;
    }
    const heading = line.match(/^(#{1,6})\s+(.+)$/);
    if (heading) {
      result.push({
        id: idFactory(),
        type: "heading",
        level: heading[1]?.length ?? 1,
        children: inline(heading[2] ?? ""),
      });
      index += 1;
      continue;
    }
    if (/^(---|\*\*\*|___)\s*$/.test(line)) {
      result.push({ id: idFactory(), type: "divider" });
      index += 1;
      continue;
    }
    const quote = line.match(/^>\s?(.*)$/);
    if (quote) {
      const quoted: string[] = [];
      while (index < lines.length) {
        const match = (lines[index] ?? "").match(/^>\s?(.*)$/);
        if (!match) break;
        quoted.push(match[1] ?? "");
        index += 1;
      }
      result.push({
        id: idFactory(),
        type: "quote",
        children: [paragraph(quoted.join("\n"), idFactory)],
      });
      continue;
    }
    const list = line.match(/^(\s*)([-*+] |\d+\. )(\[[ xX]\] )?(.*)$/);
    if (list) {
      const ordered = /\d+\. /.test(list[2] ?? "");
      const task = Boolean(list[3]);
      const items = [];
      while (index < lines.length) {
        const item = (lines[index] ?? "").match(/^(\s*)([-*+] |\d+\. )(\[[ xX]\] )?(.*)$/);
        if (!item || Boolean(/\d+\. /.test(item[2] ?? "")) !== ordered || Boolean(item[3]) !== task)
          break;
        items.push({
          id: idFactory(),
          type: "listItem" as const,
          ...(task ? { checked: /\[[xX]\]/.test(item[3] ?? "") } : {}),
          children: [paragraph(item[4] ?? "", idFactory)],
        });
        index += 1;
      }
      result.push({
        id: idFactory(),
        type: task ? "taskList" : ordered ? "orderedList" : "bulletList",
        ...(ordered ? { start: Number.parseInt(list[2] ?? "1", 10) || 1 } : {}),
        items,
      } as DocumentContent_Block);
      continue;
    }
    const paragraphLines = [line];
    index += 1;
    while (index < lines.length && lines[index]?.trim()) {
      if (
        /^(#{1,6})\s|^```|^>\s?|^(---|\*\*\*|___)\s*$|^(\s*)([-*+] |\d+\. )/.test(
          lines[index] ?? "",
        )
      )
        break;
      paragraphLines.push(lines[index] ?? "");
      index += 1;
    }
    result.push(paragraph(paragraphLines.join("\n"), idFactory));
  }
  return result;
}

function paragraph(
  value: string,
  idFactory: () => string,
): Extract<DocumentContent_Block, { type: "paragraph" }> {
  return { id: idFactory(), type: "paragraph", children: inline(value) };
}

function inline(value: string): DocumentContent_Inline[] {
  return value
    .split("\n")
    .flatMap((text, index) => [
      ...(index > 0 ? ([{ type: "hardBreak" }] as DocumentContent_Inline[]) : []),
      ...(text ? ([{ type: "text", text }] as DocumentContent_Inline[]) : []),
    ]);
}

function inlineText(children: DocumentContent_Inline[], markdown: boolean): string {
  return children
    .map((child) => {
      if (child.type === "hardBreak") return markdown ? "  \n" : "\n";
      if (!markdown || !child.marks?.length) return child.text;
      return child.marks.reduce((value, mark) => {
        if (mark.type === "bold") return `**${value}**`;
        if (mark.type === "italic") return `*${value}*`;
        if (mark.type === "strike") return `~~${value}~~`;
        if (mark.type === "code") return `\`${value}\``;
        if (mark.type === "link") return `[${value}](${mark.href})`;
        return value;
      }, child.text);
    })
    .join("");
}

function serializeBlock(block: DocumentContent_Block, format: ExportFormat, depth = 0): string {
  const markdown = format === "markdown";
  switch (block.type) {
    case "paragraph":
      return inlineText(block.children, markdown);
    case "heading":
      return markdown
        ? `${"#".repeat(block.level)} ${inlineText(block.children, true)}`
        : inlineText(block.children, false);
    case "quote":
      return block.children
        .map((child) => serializeBlock(child, format, depth + 1))
        .join("\n")
        .split("\n")
        .map((line) => (markdown ? `> ${line}` : line))
        .join("\n");
    case "callout":
      return block.children.map((child) => serializeBlock(child, format, depth + 1)).join("\n");
    case "codeBlock":
      return markdown ? `\`\`\`${block.language ?? ""}\n${block.text}\n\`\`\`` : block.text;
    case "divider":
      return markdown ? "---" : "";
    case "image":
      return markdown
        ? `![${block.alt ?? ""}](asset:${block.assetId})`
        : (block.alt ?? block.caption ?? "");
    case "file":
      return markdown
        ? `[${block.caption ?? "Attachment"}](asset:${block.assetId})`
        : (block.caption ?? "Attachment");
    case "toggle":
      return [
        inlineText(block.summary, markdown),
        ...block.children.map((child) => serializeBlock(child, format, depth + 1)),
      ].join("\n");
    case "bulletList":
    case "orderedList":
    case "taskList":
      return block.items
        .map((item, index) => {
          const prefix =
            block.type === "orderedList"
              ? `${(block.start ?? 1) + index}.`
              : block.type === "taskList"
                ? `- [${item.checked ? "x" : " "}]`
                : "-";
          const body = item.children
            .map((child) => serializeBlock(child, format, depth + 1))
            .join("\n");
          return markdown ? `${"  ".repeat(depth)}${prefix} ${body}` : body;
        })
        .join("\n");
    case "table":
      return block.rows
        .map((row) =>
          row.cells
            .map((cell) => cell.children.map((child) => serializeBlock(child, format)).join(" "))
            .join(markdown ? " | " : "\t"),
        )
        .join("\n");
  }
}
