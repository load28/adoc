import type { Node as ProseMirrorNode } from "@tiptap/pm/model";
import type { Editor } from "@tiptap/react";

export function moveCurrentBlock(editor: Editor, direction: -1 | 1) {
  const current = currentTopLevelBlocks(editor);
  if (!current) return;
  if (direction < 0 && current.startIndex === 0) return;
  if (direction > 0 && current.endIndex === editor.state.doc.childCount - 1) return;
  editor.commands.command(({ tr, dispatch }) => {
    const fragment = current.nodes.map((node) => node.copy(node.content));
    if (direction < 0) {
      const previous = tr.doc.child(current.startIndex - 1);
      tr.delete(current.position, current.position + current.size);
      tr.insert(current.position - previous.nodeSize, fragment);
    } else {
      const next = tr.doc.child(current.endIndex + 1);
      tr.delete(current.position, current.position + current.size);
      tr.insert(current.position + next.nodeSize, fragment);
    }
    dispatch?.(tr.scrollIntoView());
    return true;
  });
}

export function duplicateCurrentBlock(editor: Editor) {
  const current = currentTopLevelBlocks(editor);
  if (!current) return;
  const copies = current.nodes.map((node) => renewBlockIds(node.toJSON()));
  editor.commands.insertContentAt(current.position + current.size, copies);
}

export function deleteCurrentBlock(editor: Editor) {
  const current = currentTopLevelBlocks(editor);
  if (!current) return;
  editor.commands.command(({ tr, dispatch }) => {
    tr.delete(current.position, current.position + current.size);
    dispatch?.(tr.scrollIntoView());
    return true;
  });
}

export function emptyEditorParagraph() {
  return {
    type: "paragraph",
    attrs: { blockId: crypto.randomUUID() },
    content: [],
  };
}

export function safeLink(value: string): string | undefined {
  const normalized = value.trim();
  if (!normalized || normalized.length > 2048 || normalized.includes("\\")) return undefined;
  if (normalized.startsWith("/")) return normalized.startsWith("//") ? undefined : normalized;
  try {
    const url = new URL(normalized);
    return ["https:", "http:", "mailto:"].includes(url.protocol) ? normalized : undefined;
  } catch {
    return undefined;
  }
}

export function moveCurrentTableRow(editor: Editor, direction: -1 | 1) {
  const table = tableSelection(editor);
  if (!table) return;
  const target = table.rowIndex + direction;
  if (target < 0 || target >= table.node.childCount) return;
  const rows = tableRows(table.node);
  const sourceRow = rows[table.rowIndex];
  const targetRow = rows[target];
  if (!sourceRow || !targetRow) return;
  rows[table.rowIndex] = targetRow;
  rows[target] = sourceRow;
  replaceTable(editor, table.position, table.node, rows);
}

export function moveCurrentTableColumn(editor: Editor, direction: -1 | 1) {
  const table = tableSelection(editor);
  if (!table) return;
  const target = table.columnIndex + direction;
  if (target < 0 || target >= table.node.child(0).childCount) return;
  const rows = tableRows(table.node).map((row) => {
    const cells = Array.from({ length: row.childCount }, (_, cellIndex) => row.child(cellIndex));
    const sourceCell = cells[table.columnIndex];
    const targetCell = cells[target];
    if (!sourceCell || !targetCell) return row;
    cells[table.columnIndex] = targetCell;
    cells[target] = sourceCell;
    return row.type.create(row.attrs, cells, row.marks);
  });
  replaceTable(editor, table.position, table.node, rows);
}

export function sortCurrentTable(editor: Editor) {
  const table = tableSelection(editor);
  if (!table) return;
  const rows = tableRows(table.node);
  const header = rows[0]?.child(0)?.type.name === "tableHeader" ? rows.shift() : undefined;
  rows.sort((left, right) => left.child(0).textContent.localeCompare(right.child(0).textContent));
  replaceTable(editor, table.position, table.node, header ? [header, ...rows] : rows);
}

function currentTopLevelBlocks(editor: Editor) {
  const { $from, $to } = editor.state.selection;
  if ($from.depth < 1) return undefined;
  const startIndex = $from.index(0);
  const endIndex = Math.min($to.index(0), editor.state.doc.childCount - 1);
  const nodes = Array.from({ length: endIndex - startIndex + 1 }, (_, offset) =>
    editor.state.doc.child(startIndex + offset),
  );
  let position = 0;
  for (let cursor = 0; cursor < startIndex; cursor += 1)
    position += editor.state.doc.child(cursor).nodeSize;
  return {
    startIndex,
    endIndex,
    nodes,
    position,
    size: nodes.reduce((sum, node) => sum + node.nodeSize, 0),
  };
}

function renewBlockIds(node: Record<string, unknown>): Record<string, unknown> {
  const attrs = node.attrs as Record<string, unknown> | undefined;
  const content = node.content as Array<Record<string, unknown>> | undefined;
  return {
    ...node,
    ...(attrs
      ? { attrs: { ...attrs, ...(attrs.blockId ? { blockId: crypto.randomUUID() } : {}) } }
      : {}),
    ...(content ? { content: content.map(renewBlockIds) } : {}),
  };
}

function tableSelection(editor: Editor) {
  const { $from } = editor.state.selection;
  let tableDepth = -1;
  let rowDepth = -1;
  for (let depth = $from.depth; depth > 0; depth -= 1) {
    const name = $from.node(depth).type.name;
    if (rowDepth < 0 && name === "tableRow") rowDepth = depth;
    if (name === "table") {
      tableDepth = depth;
      break;
    }
  }
  if (tableDepth < 0 || rowDepth < 0) return undefined;
  return {
    node: $from.node(tableDepth),
    position: $from.before(tableDepth),
    rowIndex: $from.index(tableDepth),
    columnIndex: $from.index(rowDepth),
  };
}

function tableRows(table: ProseMirrorNode): ProseMirrorNode[] {
  return Array.from({ length: table.childCount }, (_, index) => table.child(index));
}

function replaceTable(
  editor: Editor,
  position: number,
  table: ProseMirrorNode,
  rows: ProseMirrorNode[],
) {
  editor.commands.command(({ tr, dispatch }) => {
    tr.replaceWith(
      position,
      position + table.nodeSize,
      table.type.create(table.attrs, rows, table.marks),
    );
    dispatch?.(tr.scrollIntoView());
    return true;
  });
}
