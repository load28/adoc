import { describe, expect, test } from "bun:test";

import { editorCommandAvailable, resolveEditorShortcut } from "../src";

describe("editor command registry", () => {
  test("uses one command id for keymap and permission availability", () => {
    const event = {
      isComposing: false,
      keyCode: 66,
      key: "b",
      metaKey: true,
      ctrlKey: false,
      shiftKey: false,
      altKey: false,
    };
    const id = resolveEditorShortcut(event);
    expect(id).toBe("text.bold");
    if (!id) throw new Error("shortcut did not resolve");
    expect(
      editorCommandAvailable(id, {
        editable: true,
        composing: false,
        uploading: false,
        selection: "text",
      }),
    ).toBe(true);
  });

  test("blocks every mutation during composition and publish during upload", () => {
    expect(
      editorCommandAvailable("text.bold", {
        editable: true,
        composing: true,
        uploading: false,
        selection: "text",
      }),
    ).toBe(false);
    expect(
      editorCommandAvailable("document.publish", {
        editable: true,
        composing: false,
        uploading: true,
        selection: "caret",
      }),
    ).toBe(false);
  });
});
