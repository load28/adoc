import "./document-editor.css";

import type { DocumentContent } from "@adoc/contracts";
import {
  applyOperations,
  createEditorOperationBatch,
  editorJsonToProductContent,
  productContentToEditorJson,
} from "@adoc/editor-schema";
import Button from "@atlaskit/button/default/button";
import InlineMessage from "@atlaskit/inline-message";
import { Box, Inline, Stack } from "@atlaskit/primitives";
import {
  ApiClient,
  ApiProblemError,
  type DraftView,
  type EditLeaseView,
  OperationBuffer,
  editorShortcut,
  type OperationBufferState,
} from "@adoc/ui-domain";
import { EditorContent, useEditor } from "@tiptap/react";
import { useCallback, useEffect, useRef, useState } from "react";

import { RoutePending, RouteProblem } from "../shell/common-states";
import { useTranslation } from "../shell/product-app-provider";
import { BrowserRecoveryStore } from "./browser-recovery-store";
import { editorExtensions } from "./extensions";

type ReadyEditor = {
  draft: DraftView;
  lease: EditLeaseView;
  leaseToken: string;
  clientInstanceId: string;
};

export function DocumentEditorScreen({
  workspaceId,
  documentId,
}: Readonly<{ workspaceId: string; documentId: string }>) {
  const [attempt, setAttempt] = useState(0);
  const [ready, setReady] = useState<ReadyEditor>();
  const [problem, setProblem] = useState<ApiProblemError>();

  useEffect(() => {
    void attempt;
    const controller = new AbortController();
    const client = new ApiClient();
    void (async () => {
      try {
        const document = await client.document(workspaceId, documentId, controller.signal);
        const csrfToken = readCookie("adoc_csrf");
        if (!csrfToken) throw new Error("CSRF token is unavailable");
        const draft =
          document.draft ??
          (await client.createDraft(workspaceId, documentId, commandHeaders(csrfToken)));
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
  }, [workspaceId, documentId, attempt]);

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
      documentId={documentId}
      ready={ready}
    />
  );
}

function DocumentEditor({
  workspaceId,
  documentId,
  ready,
}: Readonly<{ workspaceId: string; documentId: string; ready: ReadyEditor }>) {
  const t = useTranslation();
  const [bufferState, setBufferState] = useState<OperationBufferState>("IDLE");
  const [runtimeProblem, setRuntimeProblem] = useState<string>();
  const [uploading, setUploading] = useState(false);
  const contentRef = useRef(ready.draft.content as DocumentContent);
  const bufferRef = useRef<OperationBuffer>();
  const flushTimer = useRef<ReturnType<typeof setTimeout>>();
  const composition = useRef(false);
  const undoRef = useRef<() => void>(() => {});
  const uploadRef = useRef<(file: File) => void>(() => {});
  const leaseRef = useRef(ready.lease);
  const csrfToken = readCookie("adoc_csrf");

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
          const shortcut = editorShortcut(event);
          if (shortcut === "SAVE") {
            event.preventDefault();
            void bufferRef.current?.flush().catch(reportRuntimeError);
            return true;
          }
          if (shortcut === "UNDO") {
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
      const asset = await client.completeFileUpload(
        workspaceId,
        upload.assetId,
        checksum,
        file.size,
        commandHeaders(csrfToken),
      );
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
            src: `/api/v1/workspaces/${encodeURIComponent(workspaceId)}/files/${encodeURIComponent(asset.id)}/content`,
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
  return (
    <main id="main-content" className="adoc-editor-layout">
      <EditorToolbar
        editor={editor}
        uploading={uploading}
        onUndo={() => void undo()}
        onUpload={(file) => void uploadFile(file)}
        onSave={() => void bufferRef.current?.flush().catch(reportRuntimeError)}
      />
      {runtimeProblem ? (
        <InlineMessage
          appearance={bufferState === "CONFLICT" ? "warning" : "error"}
          title={bufferState === "CONFLICT" ? t("editor.conflict") : runtimeProblem}
        />
      ) : null}
      <Box as="div" role="status" aria-live="polite">
        {bufferState === "IDLE"
          ? t("editor.saved")
          : bufferState === "OFFLINE"
            ? t("editor.offline")
            : t("editor.saving")}
      </Box>
      <section className="adoc-editor-canvas" aria-label={t("editor.canvas")}>
        <EditorContent editor={editor} />
      </section>
    </main>
  );
}

function EditorToolbar({
  editor,
  uploading,
  onUpload,
  onUndo,
  onSave,
}: Readonly<{
  editor: NonNullable<ReturnType<typeof useEditor>>;
  uploading: boolean;
  onUpload: (file: File) => void;
  onUndo: () => void;
  onSave: () => void;
}>) {
  const t = useTranslation();
  const fileInput = useRef<HTMLInputElement>(null);
  const command = (action: () => void) => {
    action();
    editor.commands.focus();
  };
  return (
    <div className="adoc-editor-toolbar" role="toolbar" aria-label={t("editor.canvas")}>
      <Button
        isSelected={editor.isActive("bold")}
        onClick={() => command(() => editor.chain().focus().toggleBold().run())}
      >
        {t("editor.bold")}
      </Button>
      <Button
        isSelected={editor.isActive("italic")}
        onClick={() => command(() => editor.chain().focus().toggleItalic().run())}
      >
        {t("editor.italic")}
      </Button>
      <Button
        isSelected={editor.isActive("underline")}
        onClick={() => command(() => editor.chain().focus().toggleUnderline().run())}
      >
        {t("editor.underline")}
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
      <Button onClick={onUndo}>{t("editor.undo")}</Button>
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
      <Button appearance="primary" onClick={onSave}>
        {t("editor.saveNow")}
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
