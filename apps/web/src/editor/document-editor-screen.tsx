import "./document-editor.css";

import type { DocumentContent } from "@adoc/contracts";
import {
  applyOperations,
  createEditorOperationBatch,
  editorJsonToProductContent,
  exportDocumentText,
  importDocumentText,
  productContentToEditorJson,
} from "@adoc/editor-schema";
import Button from "@atlaskit/button/default/button";
import InlineMessage from "@atlaskit/inline-message";
import { Box, Inline, Stack, Text } from "@atlaskit/primitives";
import TextArea from "@atlaskit/textarea";
import Textfield from "@atlaskit/textfield";
import {
  ApiClient,
  ApiProblemError,
  type DraftView,
  type DocumentDetail,
  type EditLeaseView,
  OperationBuffer,
  editorCommandHint,
  fileContentUrl,
  resolveEditorShortcut,
  type OperationBufferState,
} from "@adoc/ui-domain";
import { EditorContent, useEditor } from "@tiptap/react";
import { DragHandle } from "@tiptap/extension-drag-handle-react";
import { useCallback, useEffect, useRef, useState } from "react";
import { useQuery } from "@tanstack/react-query";

import { RoutePending, RouteProblem } from "../shell/common-states";
import { useTranslation } from "../shell/product-app-provider";
import { BrowserRecoveryStore } from "./browser-recovery-store";
import { editorExtensions } from "./extensions";
import {
  deleteCurrentBlock,
  duplicateCurrentBlock,
  emptyEditorParagraph,
  moveCurrentBlock,
  moveCurrentTableColumn,
  moveCurrentTableRow,
  safeLink,
  sortCurrentTable,
} from "./editor-structural-commands";

type ReadyEditor = {
  draft: DraftView;
  lease: EditLeaseView;
  leaseToken: string;
  clientInstanceId: string;
};

export function DocumentEditorScreen({
  workspaceId,
  workspaceSlug,
  documentId,
  initialDocument,
}: Readonly<{
  workspaceId: string;
  workspaceSlug: string;
  documentId: string;
  initialDocument: DocumentDetail;
}>) {
  const [attempt, setAttempt] = useState(0);
  const [ready, setReady] = useState<ReadyEditor>();
  const [problem, setProblem] = useState<ApiProblemError>();

  useEffect(() => {
    void attempt;
    const controller = new AbortController();
    const client = new ApiClient();
    void (async () => {
      try {
        const document =
          attempt === 0
            ? initialDocument
            : await client.document(workspaceId, documentId, controller.signal);
        const csrfToken = readCookie("adoc_csrf");
        if (!csrfToken) throw new Error("CSRF token is unavailable");
        const draft = await client
          .draft(workspaceId, documentId, controller.signal)
          .catch((error) => {
            if (error instanceof ApiProblemError && error.problem.code === "DRAFT_NOT_FOUND")
              return client.createDraft(workspaceId, documentId, commandHeaders(csrfToken));
            throw error;
          });
        const clientInstanceId = clientInstance(documentId);
        const lease = await client.acquireLease(
          workspaceId,
          documentId,
          document.revision,
          clientInstanceId,
          commandHeaders(csrfToken),
        );
        if (!lease.token) throw new Error("lease acquisition did not return a token");
        if (!controller.signal.aborted) {
          setReady({ draft, lease, leaseToken: lease.token, clientInstanceId });
          setProblem(undefined);
        }
      } catch (error) {
        if (controller.signal.aborted) return;
        if (error instanceof ApiProblemError) setProblem(error);
        else
          setProblem(
            new ApiProblemError({ code: "EDITOR_BOOTSTRAP_FAILED", message: String(error) }),
          );
      }
    })();
    return () => controller.abort();
  }, [workspaceId, documentId, initialDocument, attempt]);

  if (problem) {
    if (problem.problem.code === "EDIT_LEASE_HELD") {
      return <ReadOnlyNotice />;
    }
    return (
      <RouteProblem
        code={problem.problem.code}
        correlationId={problem.problem.correlationId}
        onRetry={() => setAttempt((value) => value + 1)}
      />
    );
  }
  if (!ready) return <RoutePending />;
  return (
    <DocumentEditor
      key={`${ready.draft.id}:${ready.lease.revision}`}
      workspaceId={workspaceId}
      workspaceSlug={workspaceSlug}
      documentId={documentId}
      ready={ready}
    />
  );
}

function DocumentEditor({
  workspaceId,
  workspaceSlug,
  documentId,
  ready,
}: Readonly<{
  workspaceId: string;
  workspaceSlug: string;
  documentId: string;
  ready: ReadyEditor;
}>) {
  const t = useTranslation();
  const [bufferState, setBufferState] = useState<OperationBufferState>("IDLE");
  const [runtimeProblem, setRuntimeProblem] = useState<string>();
  const [uploading, setUploading] = useState(false);
  const [showPublish, setShowPublish] = useState(false);
  const [summary, setSummary] = useState("");
  const [showFind, setShowFind] = useState(false);
  const [findText, setFindText] = useState("");
  const [replaceText, setReplaceText] = useState("");
  const [publishing, setPublishing] = useState(false);
  const contentRef = useRef(ready.draft.content as DocumentContent);
  const bufferRef = useRef<OperationBuffer>();
  const flushTimer = useRef<ReturnType<typeof setTimeout>>();
  const composition = useRef(false);
  const undoRef = useRef<() => void>(() => {});
  const uploadRef = useRef<(file: File) => void>(() => {});
  const recoveryStoreRef = useRef<BrowserRecoveryStore>();
  const leaseRef = useRef(ready.lease);
  const csrfToken = readCookie("adoc_csrf");
  const publishPolicy = useQuery({
    queryKey: ["publish-policy", workspaceId, documentId],
    queryFn: ({ signal }) => new ApiClient().publishPolicy(workspaceId, documentId, signal),
    enabled: showPublish,
  });

  const editor = useEditor({
    immediatelyRender: false,
    extensions: editorExtensions,
    content: editorDocument(contentRef.current, workspaceId),
    editorProps: {
      attributes: { "aria-label": t("editor.canvas"), spellcheck: "true" },
      handleDOMEvents: {
        compositionstart: () => {
          composition.current = true;
          return false;
        },
        compositionend: () => {
          composition.current = false;
          queueMicrotask(() => captureChange());
          return false;
        },
        drop: (_view, event) => {
          const file = event.dataTransfer?.files[0];
          if (!file) return false;
          event.preventDefault();
          uploadRef.current(file);
          return true;
        },
        keydown: (_view, event) => {
          const shortcut = resolveEditorShortcut(event);
          if (shortcut === "editor.save") {
            event.preventDefault();
            void bufferRef.current?.flush().catch(reportRuntimeError);
            return true;
          }
          if (shortcut === "editor.undo") {
            event.preventDefault();
            undoRef.current();
            return true;
          }
          return false;
        },
      },
    },
    onUpdate: () => {
      if (!composition.current) void captureChange();
    },
  });

  const reportRuntimeError = useCallback((error: unknown) => {
    setRuntimeProblem(
      error instanceof ApiProblemError ? error.problem.code : "EDITOR_RUNTIME_FAILED",
    );
  }, []);

  const captureChange = useCallback(async () => {
    if (!editor || composition.current || !bufferRef.current) return;
    try {
      const next = editorJsonToProductContent(editor.getJSON());
      const operations = createEditorOperationBatch(
        contentRef.current,
        next,
        bufferRef.current.revision,
      );
      if (operations.length === 0) return;
      contentRef.current = next;
      await bufferRef.current.enqueue(operations);
      if (flushTimer.current) clearTimeout(flushTimer.current);
      if (
        operations.length >= 20 ||
        operations.some((operation) => operation.kind !== "REPLACE_TEXT")
      ) {
        await bufferRef.current.flush();
      } else {
        flushTimer.current = setTimeout(() => {
          void bufferRef.current?.flush().catch(reportRuntimeError);
        }, 250);
      }
    } catch (error) {
      reportRuntimeError(error);
    }
  }, [editor, reportRuntimeError]);

  useEffect(() => {
    if (!csrfToken) {
      setRuntimeProblem("CSRF_UNAVAILABLE");
      return;
    }
    let disposed = false;
    void BrowserRecoveryStore.create(workspaceId, documentId, ready.draft.id)
      .then(async (store) => {
        if (disposed) return;
        const buffer = new OperationBuffer(
          ready.draft.revision,
          store,
          async (group, idempotencyKey) => {
            const result = await new ApiClient().applyDraftOperations(
              workspaceId,
              documentId,
              group.baseRevision,
              ready.leaseToken,
              ready.clientInstanceId,
              group.operations,
              { csrfToken, idempotencyKey },
            );
            return result;
          },
          setBufferState,
        );
        recoveryStoreRef.current = store;
        const recovered = await store.load();
        let recoveredContent = contentRef.current;
        for (const group of recovered) {
          const result = await applyOperations({
            content: recoveredContent,
            baseRevision: group.baseRevision,
            operations: group.operations,
            references: [],
          });
          recoveredContent = result.content;
        }
        if (recovered.length > 0) {
          contentRef.current = recoveredContent;
          editor?.commands.setContent(editorDocument(recoveredContent, workspaceId), {
            emitUpdate: false,
          });
          buffer.restore(recovered);
        }
        bufferRef.current = buffer;
        if (recovered.length > 0) await buffer.flush();
      })
      .catch(reportRuntimeError);
    return () => {
      disposed = true;
    };
  }, [csrfToken, documentId, editor, ready, reportRuntimeError, workspaceId]);

  useEffect(() => {
    if (!csrfToken) return;
    const client = new ApiClient();
    const renew = async () => {
      try {
        const lease = await client.renewLease(
          workspaceId,
          documentId,
          leaseRef.current.revision,
          ready.leaseToken,
          ready.clientInstanceId,
          commandHeaders(csrfToken),
        );
        leaseRef.current = lease;
      } catch (error) {
        reportRuntimeError(error);
      }
    };
    const interval = setInterval(() => void renew(), 30_000);
    const onVisibility = () => {
      if (document.visibilityState === "visible") void renew();
    };
    document.addEventListener("visibilitychange", onVisibility);
    return () => {
      clearInterval(interval);
      document.removeEventListener("visibilitychange", onVisibility);
      void client
        .releaseLease(
          workspaceId,
          documentId,
          leaseRef.current.revision,
          ready.leaseToken,
          ready.clientInstanceId,
          commandHeaders(csrfToken),
        )
        .catch(() => undefined);
    };
  }, [csrfToken, documentId, ready, reportRuntimeError, workspaceId]);

  useEffect(() => {
    const onOnline = () => {
      bufferRef.current?.setOnline(true);
      void bufferRef.current?.flush().catch(reportRuntimeError);
    };
    const onOffline = () => bufferRef.current?.setOnline(false);
    const onBeforeUnload = (event: BeforeUnloadEvent) => {
      if (!bufferRef.current?.hasUnsynced) return;
      event.preventDefault();
    };
    window.addEventListener("online", onOnline);
    window.addEventListener("offline", onOffline);
    window.addEventListener("beforeunload", onBeforeUnload);
    return () => {
      window.removeEventListener("online", onOnline);
      window.removeEventListener("offline", onOffline);
      window.removeEventListener("beforeunload", onBeforeUnload);
      if (flushTimer.current) clearTimeout(flushTimer.current);
    };
  }, [reportRuntimeError]);

  if (!editor) return <RoutePending />;
  const undo = async () => {
    const buffer = bufferRef.current;
    const inverse = buffer?.takeUndo();
    if (!buffer || !inverse || inverse.length === 0) return;
    try {
      const result = await applyOperations({
        content: contentRef.current,
        baseRevision: buffer.revision,
        operations: inverse,
        references: [],
      });
      contentRef.current = result.content;
      editor.commands.setContent(editorDocument(result.content, workspaceId), {
        emitUpdate: false,
      });
      await buffer.enqueue(inverse);
      await buffer.flush();
    } catch (error) {
      reportRuntimeError(error);
    }
  };
  undoRef.current = () => void undo();
  const uploadFile = async (file: File) => {
    if (!csrfToken) return;
    setUploading(true);
    try {
      const checksum = await sha256(file);
      const client = new ApiClient();
      const upload = await client.createFileUpload(
        workspaceId,
        {
          name: file.name,
          mimeType: file.type || "application/octet-stream",
          size: file.size,
          checksum,
        },
        commandHeaders(csrfToken),
      );
      await client.uploadFileBytes(upload.uploadUrl, upload.uploadToken, csrfToken, file);
      const completed = await client.completeFileUpload(
        workspaceId,
        upload.assetId,
        checksum,
        file.size,
        commandHeaders(csrfToken),
      );
      const asset = await client.file(workspaceId, completed.id);
      if (asset.status === "FAILED") {
        await client.deleteFile(workspaceId, asset.id, asset.revision, commandHeaders(csrfToken));
        throw new Error(`file validation failed: ${asset.failureCode ?? "unknown"}`);
      }
      if (asset.status !== "READY") throw new Error(`file did not become READY: ${asset.status}`);
      const blockId = crypto.randomUUID();
      if (asset.mimeType.startsWith("image/")) {
        editor.commands.insertContent({
          type: "image",
          attrs: {
            blockId,
            assetId: asset.id,
            alt: file.name,
            caption: file.name,
            src: fileContentUrl(workspaceId, asset.id),
          },
        });
      } else {
        editor.commands.insertContent({
          type: "fileBlock",
          attrs: { blockId, assetId: asset.id, caption: file.name },
        });
      }
    } catch (error) {
      reportRuntimeError(error);
    } finally {
      setUploading(false);
    }
  };
  uploadRef.current = (file) => void uploadFile(file);
  const importFile = async (file: File) => {
    try {
      const format = /\.md|\.markdown$/i.test(file.name) ? "markdown" : "plain";
      const imported = importDocumentText(await file.text(), format);
      if (!window.confirm(t("editor.importConfirm"))) return;
      editor.commands.setContent(editorDocument(imported, workspaceId));
      await captureChange();
    } catch (error) {
      reportRuntimeError(error);
    }
  };
  const exportCurrent = (format: "markdown" | "plain") => {
    try {
      const current = editorJsonToProductContent(editor.getJSON());
      downloadEditorText(exportDocumentText(current, format), format);
    } catch (error) {
      reportRuntimeError(error);
    }
  };
  const findNext = () => {
    if (!findText) return;
    const current = editor.state.selection.to;
    const matches: Array<{ from: number; to: number }> = [];
    editor.state.doc.descendants((node, position) => {
      if (!node.isText || !node.text) return;
      let offset = node.text.indexOf(findText);
      while (offset >= 0) {
        matches.push({ from: position + offset, to: position + offset + findText.length });
        offset = node.text.indexOf(findText, offset + findText.length);
      }
    });
    const match = matches.find((value) => value.from >= current) ?? matches[0];
    if (match) editor.commands.setTextSelection(match);
  };
  const replaceSelection = () => {
    if (editor.state.selection.empty) findNext();
    else editor.chain().focus().insertContent(replaceText).run();
  };
  const publish = async () => {
    const buffer = bufferRef.current;
    if (!csrfToken || !buffer || !summary.trim() || uploading) return;
    setPublishing(true);
    try {
      await buffer.flush();
      if (buffer.hasUnsynced || buffer.state !== "IDLE") throw new Error("DRAFT_NOT_SAVED");
      await new ApiClient().publishDocument(
        workspaceId,
        documentId,
        buffer.revision,
        {
          summary: summary.trim(),
          clientInstanceId: ready.clientInstanceId,
          leaseToken: ready.leaseToken,
        },
        commandHeaders(csrfToken),
      );
      window.location.assign(
        `/w/${encodeURIComponent(workspaceSlug)}/docs/${encodeURIComponent(documentId)}?mode=published`,
      );
    } catch (error) {
      reportRuntimeError(error);
    } finally {
      setPublishing(false);
    }
  };
  return (
    <main id="main-content" className="adoc-editor-layout">
      <EditorToolbar
        editor={editor}
        uploading={uploading}
        onUndo={() => void undo()}
        onUpload={(file) => void uploadFile(file)}
        onImport={(file) => void importFile(file)}
        onExport={exportCurrent}
        onSave={() => void bufferRef.current?.flush().catch(reportRuntimeError)}
        onFind={() => setShowFind((value) => !value)}
        onPublish={() => setShowPublish((value) => !value)}
      />
      {showFind ? (
        <section aria-label={t("editor.findReplace")}>
          <Stack space="space.100">
            <Textfield
              aria-label={t("editor.find")}
              value={findText}
              onChange={(event) => setFindText(event.currentTarget.value)}
            />
            <Textfield
              aria-label={t("editor.replace")}
              value={replaceText}
              onChange={(event) => setReplaceText(event.currentTarget.value)}
            />
            <Inline space="space.100">
              <Button onClick={findNext}>{t("editor.findNext")}</Button>
              <Button onClick={replaceSelection}>{t("editor.replace")}</Button>
            </Inline>
          </Stack>
        </section>
      ) : null}
      {showPublish ? (
        <section aria-label={t("editor.publish")}>
          <Stack space="space.100">
            <label htmlFor="publish-summary">{t("editor.publishSummary")}</label>
            {publishPolicy.data ? (
              <Text>
                {publishPolicy.data.mode === "DIRECT"
                  ? t("editor.directPublish")
                  : `${t("editor.reviewPublish")} · ${publishPolicy.data.requiredApprovals}`}
              </Text>
            ) : null}
            {publishPolicy.error ? (
              <InlineMessage appearance="error" title="PUBLISH_POLICY_UNAVAILABLE" />
            ) : null}
            <TextArea
              id="publish-summary"
              value={summary}
              maxLength={1000}
              onChange={(event) => setSummary(event.currentTarget.value)}
            />
            <Inline space="space.100">
              <Button
                appearance="primary"
                isLoading={publishing}
                isDisabled={
                  !summary.trim() ||
                  uploading ||
                  bufferState !== "IDLE" ||
                  publishPolicy.isPending ||
                  Boolean(publishPolicy.error)
                }
                onClick={() => void publish()}
              >
                {t("editor.publish")}
              </Button>
              <Button appearance="subtle" onClick={() => setShowPublish(false)}>
                {t("common.cancel")}
              </Button>
            </Inline>
          </Stack>
        </section>
      ) : null}
      {runtimeProblem ? (
        <InlineMessage
          appearance={bufferState === "CONFLICT" ? "warning" : "error"}
          title={bufferState === "CONFLICT" ? t("editor.conflict") : runtimeProblem}
        />
      ) : null}
      {bufferState === "CONFLICT" || runtimeProblem === "PUBLISH_BASE_STALE" ? (
        <Stack space="space.100">
          <Text>{t("editor.conflictRecovery")}</Text>
          <Inline space="space.100" shouldWrap>
            <Button
              onClick={() =>
                downloadEditorText(JSON.stringify(contentRef.current, null, 2), "plain")
              }
            >
              {t("editor.exportRecovery")}
            </Button>
            <Button
              onClick={() =>
                window.location.assign(
                  `/w/${encodeURIComponent(workspaceSlug)}/docs/${encodeURIComponent(documentId)}?mode=published&panel=history`,
                )
              }
            >
              {t("editor.openVersionDiff")}
            </Button>
            {bufferState === "CONFLICT" ? (
              <Button
                appearance="warning"
                onClick={() => {
                  if (!window.confirm(t("editor.discardLocalConfirm"))) return;
                  void recoveryStoreRef.current?.clear().then(() => window.location.reload());
                }}
              >
                {t("editor.useServerDraft")}
              </Button>
            ) : null}
          </Inline>
        </Stack>
      ) : null}
      <Box as="div" role="status" aria-live="polite">
        {bufferState === "IDLE"
          ? t("editor.saved")
          : bufferState === "OFFLINE"
            ? t("editor.offline")
            : t("editor.saving")}
      </Box>
      <section className="adoc-editor-canvas" aria-label={t("editor.canvas")}>
        <DragHandle editor={editor} nested>
          <Button appearance="subtle" spacing="compact" aria-label={t("editor.dragBlock")}>
            ⋮⋮
          </Button>
        </DragHandle>
        <EditorContent editor={editor} />
      </section>
    </main>
  );
}

function EditorToolbar({
  editor,
  uploading,
  onUpload,
  onImport,
  onExport,
  onUndo,
  onSave,
  onFind,
  onPublish,
}: Readonly<{
  editor: NonNullable<ReturnType<typeof useEditor>>;
  uploading: boolean;
  onUpload: (file: File) => void;
  onImport: (file: File) => void;
  onExport: (format: "markdown" | "plain") => void;
  onUndo: () => void;
  onSave: () => void;
  onFind: () => void;
  onPublish: () => void;
}>) {
  const t = useTranslation();
  const fileInput = useRef<HTMLInputElement>(null);
  const importInput = useRef<HTMLInputElement>(null);
  const [showBlocks, setShowBlocks] = useState(false);
  const [linkHref, setLinkHref] = useState("");
  const [codeLanguage, setCodeLanguage] = useState("");
  const [mediaCaption, setMediaCaption] = useState("");
  const command = (action: () => void) => {
    action();
    editor.commands.focus();
  };
  return (
    <div className="adoc-editor-toolbar" role="toolbar" aria-label={t("editor.canvas")}>
      <Button
        isSelected={editor.isActive("bold")}
        title={editorCommandHint("text.bold")}
        onClick={() => command(() => editor.chain().focus().toggleBold().run())}
      >
        {t("editor.bold")}
      </Button>
      <Button
        isSelected={editor.isActive("italic")}
        title={editorCommandHint("text.italic")}
        onClick={() => command(() => editor.chain().focus().toggleItalic().run())}
      >
        {t("editor.italic")}
      </Button>
      <Button
        isSelected={editor.isActive("underline")}
        title={editorCommandHint("text.underline")}
        onClick={() => command(() => editor.chain().focus().toggleUnderline().run())}
      >
        {t("editor.underline")}
      </Button>
      <Button
        isSelected={editor.isActive("strike")}
        onClick={() => command(() => editor.chain().focus().toggleStrike().run())}
      >
        {t("editor.strike")}
      </Button>
      <Button
        isSelected={editor.isActive("code")}
        onClick={() => command(() => editor.chain().focus().toggleCode().run())}
      >
        {t("editor.inlineCode")}
      </Button>
      <Button onClick={() => command(() => editor.chain().focus().toggleSubscript().run())}>
        {t("editor.subscript")}
      </Button>
      <Button onClick={() => command(() => editor.chain().focus().toggleSuperscript().run())}>
        {t("editor.superscript")}
      </Button>
      <Button
        onClick={() =>
          command(() => editor.chain().focus().setMark("highlight", { token: "discovery" }).run())
        }
      >
        {t("editor.highlight")}
      </Button>
      <Textfield
        aria-label={t("editor.linkAddress")}
        value={linkHref}
        placeholder="https://"
        onChange={(event) => setLinkHref(event.currentTarget.value)}
      />
      <Button
        isDisabled={!safeLink(linkHref)}
        onClick={() =>
          command(() => {
            const href = safeLink(linkHref);
            if (href) editor.chain().focus().extendMarkRange("link").setLink({ href }).run();
          })
        }
      >
        {t("editor.link")}
      </Button>
      <Button
        isSelected={editor.isActive("heading", { level: 2 })}
        onClick={() => command(() => editor.chain().focus().toggleHeading({ level: 2 }).run())}
      >
        {t("editor.heading")}
      </Button>
      <Button
        isSelected={editor.isActive("bulletList")}
        onClick={() => command(() => editor.chain().focus().toggleBulletList().run())}
      >
        {t("editor.bulletList")}
      </Button>
      <Button
        isSelected={editor.isActive("orderedList")}
        onClick={() => command(() => editor.chain().focus().toggleOrderedList().run())}
      >
        {t("editor.orderedList")}
      </Button>
      <Button
        isSelected={editor.isActive("taskList")}
        onClick={() => command(() => editor.chain().focus().toggleTaskList().run())}
      >
        {t("editor.taskList")}
      </Button>
      <Button
        onClick={() =>
          command(() =>
            editor.chain().focus().insertTable({ rows: 3, cols: 3, withHeaderRow: true }).run(),
          )
        }
      >
        {t("editor.table")}
      </Button>
      <Button onClick={() => setShowBlocks((value) => !value)}>{t("editor.commandPalette")}</Button>
      {showBlocks ? (
        <Inline space="space.050" shouldWrap>
          <Button onClick={() => command(() => editor.chain().focus().setParagraph().run())}>
            {t("editor.paragraph")}
          </Button>
          <Button onClick={() => command(() => editor.chain().focus().toggleBlockquote().run())}>
            {t("editor.quote")}
          </Button>
          <Button onClick={() => command(() => editor.chain().focus().toggleCodeBlock().run())}>
            {t("editor.codeBlock")}
          </Button>
          <Button onClick={() => command(() => editor.chain().focus().setHorizontalRule().run())}>
            {t("editor.divider")}
          </Button>
          <Button
            onClick={() =>
              command(() =>
                editor.commands.insertContent({
                  type: "callout",
                  attrs: { blockId: crypto.randomUUID(), tone: "info", icon: null },
                  content: [emptyEditorParagraph()],
                }),
              )
            }
          >
            {t("editor.callout")}
          </Button>
          <Button
            onClick={() =>
              command(() =>
                editor.commands.insertContent({
                  type: "toggle",
                  attrs: {
                    blockId: crypto.randomUUID(),
                    summary: [{ type: "text", text: t("editor.toggleSummary") }],
                  },
                  content: [emptyEditorParagraph()],
                }),
              )
            }
          >
            {t("editor.toggle")}
          </Button>
          <Button onClick={() => command(() => moveCurrentBlock(editor, -1))}>
            {t("editor.moveBlockUp")}
          </Button>
          <Button onClick={() => command(() => moveCurrentBlock(editor, 1))}>
            {t("editor.moveBlockDown")}
          </Button>
          <Button onClick={() => command(() => duplicateCurrentBlock(editor))}>
            {t("editor.duplicateBlock")}
          </Button>
          <Button onClick={() => command(() => deleteCurrentBlock(editor))}>
            {t("editor.deleteBlock")}
          </Button>
        </Inline>
      ) : null}
      {editor.isActive("table") ? (
        <Inline space="space.050" shouldWrap>
          <Button onClick={() => command(() => editor.chain().focus().addRowAfter().run())}>
            {t("editor.addRow")}
          </Button>
          <Button onClick={() => command(() => editor.chain().focus().deleteRow().run())}>
            {t("editor.deleteRow")}
          </Button>
          <Button onClick={() => command(() => editor.chain().focus().addColumnAfter().run())}>
            {t("editor.addColumn")}
          </Button>
          <Button onClick={() => command(() => editor.chain().focus().deleteColumn().run())}>
            {t("editor.deleteColumn")}
          </Button>
          <Button onClick={() => command(() => editor.chain().focus().toggleHeaderRow().run())}>
            {t("editor.toggleHeader")}
          </Button>
          <Button onClick={() => command(() => moveCurrentTableRow(editor, -1))}>
            {t("editor.moveRowUp")}
          </Button>
          <Button onClick={() => command(() => moveCurrentTableRow(editor, 1))}>
            {t("editor.moveRowDown")}
          </Button>
          <Button onClick={() => command(() => moveCurrentTableColumn(editor, -1))}>
            {t("editor.moveColumnLeft")}
          </Button>
          <Button onClick={() => command(() => moveCurrentTableColumn(editor, 1))}>
            {t("editor.moveColumnRight")}
          </Button>
          <Button onClick={() => command(() => sortCurrentTable(editor))}>
            {t("editor.sortTable")}
          </Button>
          <Button onClick={() => command(() => editor.chain().focus().deleteTable().run())}>
            {t("editor.deleteTable")}
          </Button>
        </Inline>
      ) : null}
      {editor.isActive("codeBlock") ? (
        <Inline space="space.050">
          <Textfield
            aria-label={t("editor.codeLanguage")}
            value={codeLanguage}
            onChange={(event) => setCodeLanguage(event.currentTarget.value)}
          />
          <Button
            onClick={() =>
              command(() =>
                editor
                  .chain()
                  .focus()
                  .updateAttributes("codeBlock", { language: codeLanguage || null })
                  .run(),
              )
            }
          >
            {t("common.confirm")}
          </Button>
        </Inline>
      ) : null}
      {editor.isActive("image") || editor.isActive("fileBlock") ? (
        <Inline space="space.050">
          <Textfield
            aria-label={t("editor.mediaCaption")}
            value={mediaCaption}
            onChange={(event) => setMediaCaption(event.currentTarget.value)}
          />
          <Button
            onClick={() =>
              command(() =>
                editor
                  .chain()
                  .focus()
                  .updateAttributes(editor.isActive("image") ? "image" : "fileBlock", {
                    caption: mediaCaption || null,
                    ...(editor.isActive("image") ? { alt: mediaCaption } : {}),
                  })
                  .run(),
              )
            }
          >
            {t("common.confirm")}
          </Button>
        </Inline>
      ) : null}
      <Button title={editorCommandHint("editor.undo")} onClick={onUndo}>
        {t("editor.undo")}
      </Button>
      <Button onClick={onFind}>{t("editor.findReplace")}</Button>
      <input
        ref={fileInput}
        type="file"
        hidden
        tabIndex={-1}
        onChange={(event) => {
          const file = event.currentTarget.files?.[0];
          if (file) onUpload(file);
          event.currentTarget.value = "";
        }}
      />
      <Button
        isDisabled={uploading}
        isLoading={uploading}
        onClick={() => fileInput.current?.click()}
      >
        {uploading ? t("editor.uploading") : t("editor.upload")}
      </Button>
      <input
        ref={importInput}
        type="file"
        accept=".md,.markdown,.txt,text/markdown,text/plain"
        hidden
        tabIndex={-1}
        onChange={(event) => {
          const file = event.currentTarget.files?.[0];
          if (file) onImport(file);
          event.currentTarget.value = "";
        }}
      />
      <Button onClick={() => importInput.current?.click()}>{t("editor.import")}</Button>
      <Button onClick={() => onExport("markdown")}>{t("editor.exportMarkdown")}</Button>
      <Button onClick={() => onExport("plain")}>{t("editor.exportPlain")}</Button>
      <Button onClick={() => window.print()}>{t("editor.exportPdf")}</Button>
      <Button appearance="primary" title={editorCommandHint("editor.save")} onClick={onSave}>
        {t("editor.saveNow")}
      </Button>
      <Button appearance="primary" onClick={onPublish} isDisabled={uploading}>
        {t("editor.publish")}
      </Button>
    </div>
  );
}

function ReadOnlyNotice() {
  const t = useTranslation();
  return (
    <main id="main-content">
      <Stack space="space.200">
        <Inline alignBlock="center">
          <InlineMessage appearance="info" title={t("editor.readOnly")} />
        </Inline>
      </Stack>
    </main>
  );
}

function readCookie(name: string): string | undefined {
  const prefix = `${encodeURIComponent(name)}=`;
  return document.cookie
    .split(";")
    .map((value) => value.trim())
    .find((value) => value.startsWith(prefix))
    ?.slice(prefix.length);
}

function clientInstance(documentId: string): string {
  const key = `adoc.editor.client.${documentId}`;
  const existing = sessionStorage.getItem(key);
  if (existing) return existing;
  const value = crypto.randomUUID();
  sessionStorage.setItem(key, value);
  return value;
}

function commandHeaders(csrfToken: string) {
  return { csrfToken, idempotencyKey: crypto.randomUUID() };
}

async function sha256(file: Blob): Promise<string> {
  const digest = new Uint8Array(await crypto.subtle.digest("SHA-256", await file.arrayBuffer()));
  return [...digest].map((byte) => byte.toString(16).padStart(2, "0")).join("");
}

function downloadEditorText(body: string, format: "markdown" | "plain") {
  const blob = new Blob([body], {
    type: format === "markdown" ? "text/markdown;charset=utf-8" : "text/plain;charset=utf-8",
  });
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement("a");
  anchor.href = url;
  anchor.download = `draft.${format === "markdown" ? "md" : "txt"}`;
  anchor.click();
  URL.revokeObjectURL(url);
}

function editorDocument(content: DocumentContent, workspaceId: string) {
  const json = productContentToEditorJson(content);
  const visit = (node: typeof json) => {
    if (node.type === "image" && typeof node.attrs?.assetId === "string") {
      node.attrs.src = `/api/v1/workspaces/${encodeURIComponent(workspaceId)}/files/${encodeURIComponent(node.attrs.assetId)}/content`;
    }
    for (const child of node.content ?? []) visit(child);
  };
  visit(json);
  return json;
}
