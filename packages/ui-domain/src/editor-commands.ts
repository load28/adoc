export type EditorCommandId =
  | "editor.save"
  | "editor.undo"
  | "editor.redo"
  | "text.bold"
  | "text.italic"
  | "text.underline"
  | "text.strike"
  | "text.code"
  | "text.link"
  | "block.palette"
  | "block.paragraph"
  | "block.heading"
  | "block.quote"
  | "block.code"
  | "block.divider"
  | "block.bulletList"
  | "block.orderedList"
  | "block.taskList"
  | "block.table"
  | "block.delete"
  | "block.duplicate"
  | "block.moveUp"
  | "block.moveDown"
  | "file.upload"
  | "document.import"
  | "document.exportMarkdown"
  | "document.exportPlain"
  | "document.exportPdf"
  | "document.publish";

export type EditorCommandContext = {
  editable: boolean;
  composing: boolean;
  uploading: boolean;
  selection: "caret" | "text" | "block" | "multiBlock" | "tableCell";
};

export const EDITOR_COMMANDS: ReadonlyArray<{
  id: EditorCommandId;
  shortcut?: string;
}> = [
  { id: "editor.save", shortcut: "Mod+S" },
  { id: "editor.undo", shortcut: "Mod+Z" },
  { id: "editor.redo", shortcut: "Mod+Shift+Z" },
  { id: "text.bold", shortcut: "Mod+B" },
  { id: "text.italic", shortcut: "Mod+I" },
  { id: "text.underline", shortcut: "Mod+U" },
  { id: "text.strike" },
  { id: "text.code" },
  { id: "text.link", shortcut: "Mod+K" },
  { id: "block.palette", shortcut: "/" },
  { id: "block.paragraph" },
  { id: "block.heading" },
  { id: "block.quote" },
  { id: "block.code" },
  { id: "block.divider" },
  { id: "block.bulletList" },
  { id: "block.orderedList" },
  { id: "block.taskList" },
  { id: "block.table" },
  { id: "block.delete" },
  { id: "block.duplicate" },
  { id: "block.moveUp", shortcut: "Alt+Shift+ArrowUp" },
  { id: "block.moveDown", shortcut: "Alt+Shift+ArrowDown" },
  { id: "file.upload" },
  { id: "document.import" },
  { id: "document.exportMarkdown" },
  { id: "document.exportPlain" },
  { id: "document.exportPdf" },
  { id: "document.publish" },
] as const;

export function resolveEditorShortcut(
  event: Pick<
    KeyboardEvent,
    "isComposing" | "keyCode" | "key" | "metaKey" | "ctrlKey" | "shiftKey" | "altKey"
  >,
): EditorCommandId | undefined {
  if (event.isComposing || event.keyCode === 229) return undefined;
  const mod = event.metaKey || event.ctrlKey;
  const key = event.key.toLowerCase();
  if (mod && key === "s") return "editor.save";
  if (mod && key === "z" && event.shiftKey) return "editor.redo";
  if (mod && key === "z") return "editor.undo";
  if (mod && key === "b") return "text.bold";
  if (mod && key === "i") return "text.italic";
  if (mod && key === "u") return "text.underline";
  if (mod && key === "k") return "text.link";
  if (event.altKey && event.shiftKey && event.key === "ArrowUp") return "block.moveUp";
  if (event.altKey && event.shiftKey && event.key === "ArrowDown") return "block.moveDown";
  return undefined;
}

export function editorCommandAvailable(
  id: EditorCommandId,
  context: EditorCommandContext,
): boolean {
  if (context.composing) return false;
  if (id.startsWith("document.export")) return true;
  if (!context.editable) return false;
  if (id === "document.publish") return !context.uploading;
  if (id.startsWith("text.")) return context.selection === "text" || context.selection === "caret";
  if (id.startsWith("block.move") || id === "block.delete" || id === "block.duplicate")
    return ["block", "multiBlock", "caret"].includes(context.selection);
  return true;
}

export function editorCommandHint(id: EditorCommandId): string | undefined {
  return EDITOR_COMMANDS.find((command) => command.id === id)?.shortcut;
}
