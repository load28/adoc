import {
  ApiClient,
  ApiProblemError,
  type DocumentTreeNode,
  type DocumentView,
  type ImpactPreview,
} from "@adoc/ui-domain";
import Button from "@atlaskit/button/default/button";
import LinkButton from "@atlaskit/button/link";
import InlineMessage from "@atlaskit/inline-message";
import { Inline, Stack, Text } from "@atlaskit/primitives";
import Textfield from "@atlaskit/textfield";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";

import { browserCommand } from "../shell/browser-command";
import { useTranslation } from "../shell/product-app-provider";
import "./document-tree-navigation.css";

const api = new ApiClient();

type FlatNode = { node: DocumentTreeNode; parentId: string | null };
type MoveTarget = { parentId: string | null; afterId: string | null; label: string };
type Action = "create" | "rename" | "move" | "trash" | undefined;

export function DocumentTreeNavigation({
  workspaceId,
  workspaceSlug,
}: Readonly<{ workspaceId: string; workspaceSlug: string }>) {
  const t = useTranslation();
  const client = useQueryClient();
  const tree = useQuery({
    queryKey: [workspaceId, "document-tree"],
    queryFn: ({ signal }) => api.documentTree(workspaceId, signal),
  });
  const [action, setAction] = useState<Action>();
  const [selected, setSelected] = useState<DocumentView>();
  const [parentId, setParentId] = useState<string | null>(null);
  const [title, setTitle] = useState("");
  const [reason, setReason] = useState("");
  const [moveTarget, setMoveTarget] = useState<MoveTarget>();
  const [impact, setImpact] = useState<ImpactPreview>();

  const refresh = async () =>
    client.invalidateQueries({ queryKey: [workspaceId, "document-tree"] });
  const close = () => {
    setAction(undefined);
    setSelected(undefined);
    setTitle("");
    setReason("");
    setMoveTarget(undefined);
    setImpact(undefined);
  };
  const create = useMutation({
    mutationFn: () =>
      api.createDocument(workspaceId, title.trim(), parentId, null, browserCommand()),
    onSuccess: async (document) => {
      await refresh();
      close();
      window.location.assign(
        `/w/${encodeURIComponent(workspaceSlug)}/docs/${encodeURIComponent(document.id)}?mode=published`,
      );
    },
  });
  const rename = useMutation({
    mutationFn: () =>
      api.renameDocument(
        workspaceId,
        requiredDocument(selected).id,
        requiredDocument(selected).revision,
        title.trim(),
        browserCommand(),
      ),
    onSuccess: async () => {
      await refresh();
      close();
    },
  });
  const preview = useMutation({
    mutationFn: (target: MoveTarget) => {
      const command = browserCommand();
      return api.previewDocumentMove(
        workspaceId,
        requiredDocument(selected).id,
        requiredDocument(selected).revision,
        target.parentId,
        target.afterId,
        command.csrfToken,
      );
    },
    onSuccess: (value, target) => {
      setMoveTarget(target);
      setImpact(value);
    },
  });
  const move = useMutation({
    mutationFn: () => {
      const target = requiredMoveTarget(moveTarget);
      const previewValue = requiredImpact(impact);
      const document = requiredDocument(selected);
      return api.moveDocument(
        workspaceId,
        document.id,
        document.revision,
        target.parentId,
        target.afterId,
        previewValue.previewToken,
        browserCommand(),
      );
    },
    onSuccess: async () => {
      await refresh();
      close();
    },
  });
  const trash = useMutation({
    mutationFn: () => {
      const document = requiredDocument(selected);
      return api.trashDocument(
        workspaceId,
        document.id,
        document.revision,
        reason.trim(),
        browserCommand(),
      );
    },
    onSuccess: async () => {
      await refresh();
      close();
    },
  });

  if (tree.isPending) return <Text>{t("common.loading")}</Text>;
  if (tree.error) return <TreeProblem error={tree.error} retry={() => void tree.refetch()} />;
  const flat = flatten(tree.data.nodes);
  const descendants = selected ? descendantIds(tree.data.nodes, selected.id) : new Set<string>();
  const mutationError = create.error ?? rename.error ?? preview.error ?? move.error ?? trash.error;
  return (
    <nav className="document-tree" aria-label={t("workspace.documents")}>
      <Stack space="space.100">
        <Inline alignBlock="center" spread="space-between">
          <Text weight="bold">{t("workspace.documents")}</Text>
          <Button
            appearance="subtle"
            spacing="compact"
            onClick={() => {
              close();
              setParentId(null);
              setAction("create");
            }}
          >
            {t("workspace.createDocument")}
          </Button>
        </Inline>
        {tree.data.nodes.length === 0 ? <Text>{t("workspace.emptyDocuments")}</Text> : null}
        <TreeNodes
          nodes={tree.data.nodes}
          workspaceSlug={workspaceSlug}
          onAction={(nextAction, node) => {
            close();
            setAction(nextAction);
            setSelected(node.document);
            setParentId(node.document.id);
            setTitle(nextAction === "rename" ? node.document.title : "");
          }}
        />
        {action === "create" ? (
          <TreeForm
            label={t("workspace.documentTitle")}
            value={title}
            onChange={setTitle}
            submitLabel={t("workspace.createDocument")}
            pending={create.isPending}
            disabled={!title.trim()}
            onSubmit={() => create.mutate()}
            onCancel={close}
          />
        ) : null}
        {action === "rename" ? (
          <TreeForm
            label={t("workspace.documentTitle")}
            value={title}
            onChange={setTitle}
            submitLabel={t("workspace.renameDocument")}
            pending={rename.isPending}
            disabled={!title.trim()}
            onSubmit={() => rename.mutate()}
            onCancel={close}
          />
        ) : null}
        {action === "trash" ? (
          <TreeForm
            label={t("workspace.trashReason")}
            value={reason}
            onChange={setReason}
            submitLabel={t("workspace.trashDocument")}
            pending={trash.isPending}
            disabled={!reason.trim()}
            onSubmit={() => trash.mutate()}
            onCancel={close}
          />
        ) : null}
        {action === "move" && selected ? (
          <MoveChooser
            selected={selected}
            flat={flat}
            descendants={descendants}
            impact={impact}
            target={moveTarget}
            previewPending={preview.isPending}
            movePending={move.isPending}
            onPreview={(target) => preview.mutate(target)}
            onCommit={() => move.mutate()}
            onCancel={close}
          />
        ) : null}
        {mutationError ? <MutationProblem error={mutationError} /> : null}
      </Stack>
    </nav>
  );
}

export function TreeNodes({
  nodes,
  workspaceSlug,
  onAction,
}: Readonly<{
  nodes: DocumentTreeNode[];
  workspaceSlug: string;
  onAction: (action: Exclude<Action, undefined>, node: DocumentTreeNode) => void;
}>) {
  const t = useTranslation();
  return (
    <ul className="document-tree-list">
      {nodes.map((node) => (
        <li key={node.document.id}>
          <LinkButton
            appearance="subtle"
            spacing="compact"
            href={`/w/${encodeURIComponent(workspaceSlug)}/docs/${encodeURIComponent(node.document.id)}?mode=published`}
          >
            {node.document.title}
          </LinkButton>
          <Inline space="space.025" shouldWrap>
            {allowedTreeActions(node.effectiveAccess).includes("create") ? (
              <Button
                spacing="compact"
                appearance="subtle"
                onClick={() => onAction("create", node)}
              >
                +
              </Button>
            ) : null}
            {allowedTreeActions(node.effectiveAccess).includes("rename") ? (
              <>
                <Button
                  spacing="compact"
                  appearance="subtle"
                  onClick={() => onAction("rename", node)}
                >
                  {t("workspace.renameDocument")}
                </Button>
                <Button
                  spacing="compact"
                  appearance="subtle"
                  onClick={() => onAction("move", node)}
                >
                  {t("workspace.moveDocument")}
                </Button>
                <Button
                  spacing="compact"
                  appearance="subtle"
                  onClick={() => onAction("trash", node)}
                >
                  {t("workspace.trashDocument")}
                </Button>
              </>
            ) : null}
          </Inline>
          {node.children.length > 0 ? (
            <TreeNodes nodes={node.children} workspaceSlug={workspaceSlug} onAction={onAction} />
          ) : null}
        </li>
      ))}
    </ul>
  );
}

function TreeForm({
  label,
  value,
  onChange,
  submitLabel,
  pending,
  disabled,
  onSubmit,
  onCancel,
}: Readonly<{
  label: string;
  value: string;
  onChange: (value: string) => void;
  submitLabel: string;
  pending: boolean;
  disabled: boolean;
  onSubmit: () => void;
  onCancel: () => void;
}>) {
  const t = useTranslation();
  return (
    <form
      className="document-tree-form"
      onSubmit={(event) => {
        event.preventDefault();
        if (!disabled) onSubmit();
      }}
    >
      <label htmlFor="tree-action-input">{label}</label>
      <Textfield
        id="tree-action-input"
        value={value}
        maxLength={500}
        onChange={(event) => onChange(event.currentTarget.value)}
      />
      <Inline space="space.050">
        <Button
          type="submit"
          appearance="primary"
          isLoading={pending}
          isDisabled={disabled || pending}
        >
          {submitLabel}
        </Button>
        <Button appearance="subtle" onClick={onCancel} isDisabled={pending}>
          {t("common.cancel")}
        </Button>
      </Inline>
    </form>
  );
}

function MoveChooser({
  selected,
  flat,
  descendants,
  impact,
  target,
  previewPending,
  movePending,
  onPreview,
  onCommit,
  onCancel,
}: Readonly<{
  selected: DocumentView;
  flat: FlatNode[];
  descendants: Set<string>;
  impact?: ImpactPreview;
  target?: MoveTarget;
  previewPending: boolean;
  movePending: boolean;
  onPreview: (target: MoveTarget) => void;
  onCommit: () => void;
  onCancel: () => void;
}>) {
  const t = useTranslation();
  const eligible = flat.filter(
    ({ node }) => node.document.id !== selected.id && !descendants.has(node.document.id),
  );
  const accessById = new Map(flat.map(({ node }) => [node.document.id, node.effectiveAccess]));
  return (
    <Stack space="space.100">
      <Text weight="bold">{t("workspace.moveDocument")}</Text>
      <Button
        appearance="subtle"
        isDisabled={previewPending}
        onClick={() =>
          onPreview({ parentId: null, afterId: null, label: t("workspace.moveToRoot") })
        }
      >
        {t("workspace.moveToRoot")}
      </Button>
      {eligible.map(({ node, parentId }) => (
        <Stack key={node.document.id} space="space.025">
          {node.effectiveAccess === "CONTRIBUTOR" || node.effectiveAccess === "EDITOR" ? (
            <Button
              appearance="subtle"
              isDisabled={previewPending}
              onClick={() =>
                onPreview({
                  parentId: node.document.id,
                  afterId: null,
                  label: `${t("workspace.moveInto")}: ${node.document.title}`,
                })
              }
            >
              {t("workspace.moveInto")}: {node.document.title}
            </Button>
          ) : null}
          {parentId === null ||
          allowedTreeActions(accessById.get(parentId) ?? "NO_ACCESS").includes("create") ? (
            <Button
              appearance="subtle"
              isDisabled={previewPending}
              onClick={() =>
                onPreview({
                  parentId,
                  afterId: node.document.id,
                  label: `${t("workspace.moveAfter")}: ${node.document.title}`,
                })
              }
            >
              {t("workspace.moveAfter")}: {node.document.title}
            </Button>
          ) : null}
        </Stack>
      ))}
      {impact && target ? (
        <InlineMessage appearance="info" title={target.label}>
          <p>
            {t("workspace.permissionChanges")}: {impact.permissionChanges}
          </p>
          <p>
            {t("workspace.policyChanges")}: {impact.policyChanges}
          </p>
          <Button appearance="primary" isLoading={movePending} onClick={onCommit}>
            {t("workspace.moveCommit")}
          </Button>
        </InlineMessage>
      ) : null}
      <Button appearance="subtle" onClick={onCancel} isDisabled={previewPending || movePending}>
        {t("common.cancel")}
      </Button>
    </Stack>
  );
}

function TreeProblem({ error, retry }: Readonly<{ error: unknown; retry: () => void }>) {
  const t = useTranslation();
  const code = error instanceof ApiProblemError ? error.problem.code : "TREE_UNAVAILABLE";
  return (
    <InlineMessage appearance="error" title={t("common.unavailable")}>
      <p>{code}</p>
      <Button onClick={retry}>{t("common.retry")}</Button>
    </InlineMessage>
  );
}

function MutationProblem({ error }: Readonly<{ error: unknown }>) {
  const t = useTranslation();
  const code = error instanceof ApiProblemError ? error.problem.code : "COMMAND_FAILED";
  return (
    <InlineMessage appearance="error" title={t("common.unavailable")}>
      <p>{code}</p>
    </InlineMessage>
  );
}

export function allowedTreeActions(
  access: DocumentTreeNode["effectiveAccess"],
): Array<"create" | "rename" | "move" | "trash"> {
  if (access === "EDITOR") return ["create", "rename", "move", "trash"];
  if (access === "CONTRIBUTOR") return ["create"];
  return [];
}

function flatten(nodes: DocumentTreeNode[], parentId: string | null = null): FlatNode[] {
  return nodes.flatMap((node) => [{ node, parentId }, ...flatten(node.children, node.document.id)]);
}

function descendantIds(nodes: DocumentTreeNode[], id: string): Set<string> {
  const selected = flatten(nodes).find(({ node }) => node.document.id === id)?.node;
  return new Set(selected ? flatten(selected.children).map(({ node }) => node.document.id) : []);
}

function requiredDocument(value?: DocumentView): DocumentView {
  if (!value) throw new Error("document action target is unavailable");
  return value;
}

function requiredMoveTarget(value?: MoveTarget): MoveTarget {
  if (!value) throw new Error("move target is unavailable");
  return value;
}

function requiredImpact(value?: ImpactPreview): ImpactPreview {
  if (!value) throw new Error("move preview is unavailable");
  return value;
}
