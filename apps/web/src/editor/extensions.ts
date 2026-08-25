import { Extension, Mark, Node, mergeAttributes } from "@tiptap/core";
import Image from "@tiptap/extension-image";
import Subscript from "@tiptap/extension-subscript";
import Superscript from "@tiptap/extension-superscript";
import { TableKit } from "@tiptap/extension-table";
import TaskItem from "@tiptap/extension-task-item";
import TaskList from "@tiptap/extension-task-list";
import Underline from "@tiptap/extension-underline";
import { Plugin } from "@tiptap/pm/state";
import StarterKit from "@tiptap/starter-kit";

const nodesWithStableIds = [
  "paragraph",
  "heading",
  "blockquote",
  "bulletList",
  "orderedList",
  "listItem",
  "taskList",
  "taskItem",
  "codeBlock",
  "table",
  "tableRow",
  "tableCell",
  "tableHeader",
  "horizontalRule",
  "image",
  "callout",
  "toggle",
  "fileBlock",
];

const StableBlockId = Extension.create({
  name: "stableBlockId",
  addGlobalAttributes() {
    return [
      {
        types: nodesWithStableIds,
        attributes: {
          blockId: {
            default: null,
            parseHTML: (element) => element.getAttribute("data-block-id"),
            renderHTML: (attributes) =>
              typeof attributes.blockId === "string" ? { "data-block-id": attributes.blockId } : {},
          },
        },
      },
    ];
  },
  addProseMirrorPlugins() {
    return [
      new Plugin({
        appendTransaction: (_transactions, _oldState, newState) => {
          const transaction = newState.tr;
          let changed = false;
          newState.doc.descendants((node, position) => {
            if (nodesWithStableIds.includes(node.type.name) && !node.attrs.blockId) {
              transaction.setNodeMarkup(position, undefined, {
                ...node.attrs,
                blockId: crypto.randomUUID(),
              });
              changed = true;
            }
          });
          return changed ? transaction : null;
        },
      }),
    ];
  },
});

const Callout = Node.create({
  name: "callout",
  group: "block",
  content: "block+",
  defining: true,
  addAttributes() {
    return {
      tone: { default: "info" },
      icon: { default: null },
    };
  },
  parseHTML: () => [{ tag: "aside[data-callout]" }],
  renderHTML({ HTMLAttributes }) {
    return ["aside", mergeAttributes(HTMLAttributes, { "data-callout": HTMLAttributes.tone }), 0];
  },
});

const Toggle = Node.create({
  name: "toggle",
  group: "block",
  content: "block*",
  defining: true,
  addAttributes() {
    return { summary: { default: [] } };
  },
  parseHTML: () => [{ tag: "details[data-toggle]" }],
  renderHTML({ HTMLAttributes }) {
    const summary = Array.isArray(HTMLAttributes.summary)
      ? HTMLAttributes.summary.map((item: { text?: unknown }) => item.text ?? "").join("")
      : "";
    return [
      "details",
      mergeAttributes(HTMLAttributes, { "data-toggle": "" }),
      ["summary", {}, summary],
      ["div", {}, 0],
    ];
  },
});

const FileBlock = Node.create({
  name: "fileBlock",
  group: "block",
  atom: true,
  draggable: true,
  addAttributes() {
    return { assetId: { default: null }, caption: { default: null } };
  },
  parseHTML: () => [{ tag: "a[data-file-asset]" }],
  renderHTML({ HTMLAttributes }) {
    return [
      "a",
      mergeAttributes(HTMLAttributes, {
        "data-file-asset": HTMLAttributes.assetId,
        href: `/api/v1/files/${String(HTMLAttributes.assetId)}/content`,
      }),
      HTMLAttributes.caption || "Attachment",
    ];
  },
});

function tokenMark(name: "highlight" | "textColor", attribute: string) {
  return Mark.create({
    name,
    addAttributes() {
      return { token: { default: null } };
    },
    parseHTML: () => [{ tag: `span[${attribute}]` }],
    renderHTML({ HTMLAttributes }) {
      return ["span", mergeAttributes(HTMLAttributes, { [attribute]: HTMLAttributes.token }), 0];
    },
  });
}

const ProductImage = Image.extend({
  addAttributes() {
    return {
      ...this.parent?.(),
      assetId: { default: null },
      caption: { default: null },
    };
  },
});

export const editorExtensions = [
  StarterKit.configure({ undoRedo: false }),
  Underline,
  Subscript,
  Superscript,
  tokenMark("highlight", "data-highlight-token"),
  tokenMark("textColor", "data-text-color-token"),
  TaskList,
  TaskItem.configure({ nested: true }),
  TableKit.configure({ table: { resizable: true } }),
  ProductImage.configure({ allowBase64: false, resize: { enabled: true, minWidth: 64 } }),
  StableBlockId,
  Callout,
  Toggle,
  FileBlock,
];
