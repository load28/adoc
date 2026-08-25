import {
  type DocumentContent,
  type DocumentContent_Block,
  type DocumentContent_Inline,
  type DocumentContent_Mark,
  type DocumentOperation,
  validateContract,
} from "@adoc/contracts";

export type EditorJson = {
  type: string;
  attrs?: Record<string, unknown>;
  content?: EditorJson[];
  marks?: Array<{ type: string; attrs?: Record<string, unknown> }>;
  text?: string;
};

export class UnsupportedEditorContentError extends Error {
  constructor(readonly nodeType: string) {
    super(`unsupported editor content: ${nodeType}`);
    this.name = "UnsupportedEditorContentError";
  }
}

export function productContentToEditorJson(content: DocumentContent): EditorJson {
  assertContent(content);
  return { type: "doc", content: content.root.children.map(blockToEditor) };
}

export function editorJsonToProductContent(json: EditorJson): DocumentContent {
  if (json.type !== "doc") throw new UnsupportedEditorContentError(json.type);
  const content: DocumentContent = {
    schemaVersion: 1,
    root: { type: "doc", children: (json.content ?? []).map(editorToBlock) },
  };
  assertContent(content);
  return content;
}

export function createEditorOperationBatch(
  before: DocumentContent,
  after: DocumentContent,
  draftRevision: number,
  idFactory: () => string = crypto.randomUUID,
): DocumentOperation[] {
  assertContent(before);
  assertContent(after);
  if (JSON.stringify(before) === JSON.stringify(after)) return [];
  const previous = before.root.children;
  const next = after.root.children;
  let prefix = 0;
  while (prefix < previous.length && prefix < next.length && same(previous[prefix], next[prefix])) {
    prefix += 1;
  }
  let suffix = 0;
  while (
    suffix < previous.length - prefix &&
    suffix < next.length - prefix &&
    same(previous[previous.length - 1 - suffix], next[next.length - 1 - suffix])
  ) {
    suffix += 1;
  }
  const removed = previous.slice(prefix, previous.length - suffix);
  const inserted = next.slice(prefix, next.length - suffix);
  const operationId = idFactory();
  const precondition = { draftRevision };
  if (removed.length === 0) {
    let previousOperationId: string | undefined;
    return inserted.map((block, offset) => {
      const opId = offset === 0 ? operationId : idFactory();
      const operation = {
        opId,
        kind: "INSERT_BLOCK",
        scope: { kind: "DOCUMENT" },
        precondition,
        ...(previousOperationId ? { dependsOn: [previousOperationId] } : {}),
        parentId: null,
        index: prefix + offset,
        block,
      } as DocumentOperation;
      previousOperationId = opId;
      return operation;
    });
  }
  const region =
    removed.length === 1
      ? ({ kind: "BLOCK", blockId: removed[0]?.id } as const)
      : ({
          kind: "BLOCK_RANGE",
          startBlockId: removed[0]?.id,
          endBlockId: removed.at(-1)?.id,
        } as const);
  return [
    {
      opId: operationId,
      kind: "REPLACE_REGION",
      scope: region,
      precondition,
      region,
      blocks: inserted,
    } as DocumentOperation,
  ];
}

function blockToEditor(block: DocumentContent_Block): EditorJson {
  const attrs = { blockId: block.id };
  switch (block.type) {
    case "paragraph":
      return { type: "paragraph", attrs, content: inlineToEditor(block.children) };
    case "heading":
      return {
        type: "heading",
        attrs: { ...attrs, level: block.level },
        content: inlineToEditor(block.children),
      };
    case "quote":
      return { type: "blockquote", attrs, content: block.children.map(blockToEditor) };
    case "callout":
      return {
        type: "callout",
        attrs: { ...attrs, tone: block.tone, icon: block.icon ?? null },
        content: block.children.map(blockToEditor),
      };
    case "bulletList":
    case "orderedList":
    case "taskList":
      return {
        type: block.type,
        attrs: { ...attrs, ...(block.start ? { start: block.start } : {}) },
        content: block.items.map((item) => ({
          type: block.type === "taskList" ? "taskItem" : "listItem",
          attrs: { blockId: item.id, checked: item.checked ?? null },
          content: item.children.map(blockToEditor),
        })),
      };
    case "codeBlock":
      return {
        type: "codeBlock",
        attrs: { ...attrs, language: block.language ?? null },
        content: block.text ? [{ type: "text", text: block.text }] : [],
      };
    case "table":
      return {
        type: "table",
        attrs,
        content: block.rows.map((row) => ({
          type: "tableRow",
          attrs: { blockId: row.id },
          content: row.cells.map((cell) => ({
            type: cell.type,
            attrs: {
              blockId: cell.id,
              colspan: cell.colspan ?? 1,
              rowspan: cell.rowspan ?? 1,
            },
            content: cell.children.map(blockToEditor),
          })),
        })),
      };
    case "toggle":
      return {
        type: "toggle",
        attrs: { ...attrs, summary: block.summary },
        content: block.children.map(blockToEditor),
      };
    case "divider":
      return { type: "horizontalRule", attrs };
    case "image":
      return {
        type: "image",
        attrs: {
          ...attrs,
          assetId: block.assetId,
          alt: block.alt,
          caption: block.caption ?? null,
          width: block.width ?? null,
          src: "",
        },
      };
    case "file":
      return {
        type: "fileBlock",
        attrs: { ...attrs, assetId: block.assetId, caption: block.caption ?? null },
      };
  }
}

function editorToBlock(node: EditorJson): DocumentContent_Block {
  const id = requiredString(node.attrs?.blockId, `${node.type}.blockId`);
  switch (node.type) {
    case "paragraph":
      return { id, type: "paragraph", children: editorToInline(node.content) };
    case "heading":
      return {
        id,
        type: "heading",
        level: requiredNumber(node.attrs?.level, "heading.level"),
        children: editorToInline(node.content),
      };
    case "blockquote":
      return {
        id,
        type: "quote",
        children: nonEmpty(node.content?.map(editorToBlock), "quote") as never,
      };
    case "callout":
      return {
        id,
        type: "callout",
        tone: requiredTone(node.attrs?.tone),
        icon: nullableString(node.attrs?.icon),
        children: nonEmpty(node.content?.map(editorToBlock), "callout") as never,
      };
    case "bulletList":
    case "orderedList":
    case "taskList":
      return {
        id,
        type: node.type,
        ...(node.type === "orderedList" && typeof node.attrs?.start === "number"
          ? { start: node.attrs.start }
          : {}),
        items: nonEmpty(
          node.content?.map((item) => ({
            id: requiredString(item.attrs?.blockId, "listItem.blockId"),
            type: "listItem" as const,
            ...(node.type === "taskList" ? { checked: Boolean(item.attrs?.checked) } : {}),
            children: nonEmpty(item.content?.map(editorToBlock), "listItem") as never,
          })),
          node.type,
        ) as never,
      };
    case "codeBlock":
      return {
        id,
        type: "codeBlock",
        language: nullableString(node.attrs?.language),
        text: (node.content ?? []).map((child) => child.text ?? "").join(""),
      };
    case "table":
      return {
        id,
        type: "table",
        rows: nonEmpty(
          node.content?.map((row) => ({
            id: requiredString(row.attrs?.blockId, "tableRow.blockId"),
            type: "tableRow" as const,
            cells: nonEmpty(
              row.content?.map((cell) => ({
                id: requiredString(cell.attrs?.blockId, "tableCell.blockId"),
                type:
                  cell.type === "tableHeader" ? ("tableHeader" as const) : ("tableCell" as const),
                colspan: optionalNumber(cell.attrs?.colspan),
                rowspan: optionalNumber(cell.attrs?.rowspan),
                children: nonEmpty(cell.content?.map(editorToBlock), "tableCell") as never,
              })),
              "tableRow",
            ) as never,
          })),
          "table",
        ) as never,
      };
    case "toggle":
      return {
        id,
        type: "toggle",
        summary: Array.isArray(node.attrs?.summary)
          ? (node.attrs.summary as DocumentContent_Inline[])
          : [],
        children: (node.content ?? []).map(editorToBlock),
      };
    case "horizontalRule":
      return { id, type: "divider" };
    case "image":
      return {
        id,
        type: "image",
        assetId: requiredString(node.attrs?.assetId, "image.assetId"),
        alt: typeof node.attrs?.alt === "string" ? node.attrs.alt : "",
        caption: nullableString(node.attrs?.caption),
        width: optionalNumber(node.attrs?.width) ?? null,
      };
    case "fileBlock":
      return {
        id,
        type: "file",
        assetId: requiredString(node.attrs?.assetId, "file.assetId"),
        caption: nullableString(node.attrs?.caption),
      };
    default:
      throw new UnsupportedEditorContentError(node.type);
  }
}

function inlineToEditor(children: DocumentContent_Inline[]): EditorJson[] {
  return children.map((child) =>
    child.type === "hardBreak"
      ? { type: "hardBreak" }
      : {
          type: "text",
          text: child.text,
          ...(child.marks ? { marks: child.marks.map(markToEditor) } : {}),
        },
  );
}

function editorToInline(children: EditorJson[] | undefined): DocumentContent_Inline[] {
  return (children ?? []).map((child) => {
    if (child.type === "hardBreak") return { type: "hardBreak" };
    if (child.type !== "text" || typeof child.text !== "string") {
      throw new UnsupportedEditorContentError(child.type);
    }
    return {
      type: "text",
      text: child.text,
      ...(child.marks?.length ? { marks: child.marks.map(editorToMark) } : {}),
    };
  });
}

function markToEditor(mark: DocumentContent_Mark): {
  type: string;
  attrs?: Record<string, unknown>;
} {
  if (mark.type === "link")
    return { type: "link", attrs: { href: mark.href, title: mark.title ?? null } };
  if (mark.type === "highlight" || mark.type === "textColor")
    return { type: mark.type, attrs: { token: mark.token } };
  return { type: mark.type };
}

function editorToMark(mark: {
  type: string;
  attrs?: Record<string, unknown>;
}): DocumentContent_Mark {
  if (mark.type === "link")
    return {
      type: "link",
      href: requiredString(mark.attrs?.href, "link.href"),
      title: nullableString(mark.attrs?.title),
    };
  if (mark.type === "highlight" || mark.type === "textColor")
    return { type: mark.type, token: requiredString(mark.attrs?.token, `${mark.type}.token`) };
  if (
    ["bold", "italic", "underline", "strike", "code", "subscript", "superscript"].includes(
      mark.type,
    )
  ) {
    return { type: mark.type } as DocumentContent_Mark;
  }
  throw new UnsupportedEditorContentError(mark.type);
}

function same(left: unknown, right: unknown): boolean {
  return JSON.stringify(left) === JSON.stringify(right);
}

function assertContent(content: DocumentContent): void {
  if (!validateContract("content", content).valid)
    throw new UnsupportedEditorContentError("invalid-product-content");
}

function nonEmpty<T>(value: T[] | undefined, label: string): [T, ...T[]] {
  if (!value || value.length === 0) throw new UnsupportedEditorContentError(label);
  return value as [T, ...T[]];
}

function requiredString(value: unknown, label: string): string {
  if (typeof value !== "string" || value.length === 0)
    throw new UnsupportedEditorContentError(label);
  return value;
}

function requiredNumber(value: unknown, label: string): number {
  if (typeof value !== "number") throw new UnsupportedEditorContentError(label);
  return value;
}

function optionalNumber(value: unknown): number | undefined {
  return typeof value === "number" ? value : undefined;
}

function nullableString(value: unknown): string | null {
  return typeof value === "string" ? value : null;
}

function requiredTone(value: unknown): "info" | "success" | "warning" | "danger" | "note" {
  if (["info", "success", "warning", "danger", "note"].includes(String(value)))
    return value as never;
  throw new UnsupportedEditorContentError("callout.tone");
}
