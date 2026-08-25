import type { DocumentContent, DocumentOperation } from "@adoc/contracts";
import {
  ApiClient,
  ApiProblemError,
  type PublishedVersion,
  type VersionPage,
} from "@adoc/ui-domain";
import Button from "@atlaskit/button/default/button";
import Checkbox from "@atlaskit/checkbox";
import InlineMessage from "@atlaskit/inline-message";
import Lozenge from "@atlaskit/lozenge";
import { Inline, Stack, Text } from "@atlaskit/primitives";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";

import { browserCommand } from "../shell/browser-command";
import { RoutePending, RouteProblem } from "../shell/common-states";
import { useTranslation } from "../shell/product-app-provider";
import { ContentRenderer } from "./content-renderer";
import "./document-content.css";

const api = new ApiClient();

export function VersionHistoryPanel({
  workspaceId,
  workspaceSlug,
  documentId,
  from,
  to,
}: Readonly<{
  workspaceId: string;
  workspaceSlug: string;
  documentId: string;
  from?: string;
  to?: string;
}>) {
  const t = useTranslation();
  const queryClient = useQueryClient();
  const [selected, setSelected] = useState<string[]>([from, to].filter(Boolean) as string[]);
  const versions = useQuery({
    queryKey: ["versions", workspaceId, documentId],
    queryFn: ({ signal }) => api.versions(workspaceId, documentId, undefined, signal),
  });
  const document = useQuery({
    queryKey: ["document", workspaceId, documentId],
    queryFn: ({ signal }) => api.document(workspaceId, documentId, signal),
  });
  const pair = selected.length === 2 ? selected : undefined;
  const fromVersionId = pair?.[0] ?? "";
  const toVersionId = pair?.[1] ?? "";
  const diff = useQuery({
    queryKey: ["version-diff", workspaceId, documentId, pair?.[0], pair?.[1]],
    queryFn: ({ signal }) =>
      api.versionDiff(workspaceId, documentId, fromVersionId, toVersionId, signal),
    enabled: Boolean(pair),
  });
  const left = useQuery({
    queryKey: ["version", workspaceId, documentId, pair?.[0]],
    queryFn: ({ signal }) => api.version(workspaceId, documentId, fromVersionId, signal),
    enabled: Boolean(pair),
  });
  const right = useQuery({
    queryKey: ["version", workspaceId, documentId, pair?.[1]],
    queryFn: ({ signal }) => api.version(workspaceId, documentId, toVersionId, signal),
    enabled: Boolean(pair),
  });
  const restore = useMutation({
    mutationFn: (versionId: string) => {
      if (!document.data) throw new Error("DOCUMENT_UNAVAILABLE");
      return api.restoreVersion(
        workspaceId,
        documentId,
        versionId,
        document.data.revision,
        browserCommand(),
      );
    },
    onSuccess: async () => {
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: ["document", workspaceId, documentId] }),
        queryClient.invalidateQueries({ queryKey: ["versions", workspaceId, documentId] }),
      ]);
      window.location.assign(documentUrl(workspaceSlug, documentId, "draft"));
    },
  });

  if (versions.isPending || document.isPending) return <RoutePending />;
  if (versions.error || document.error)
    return (
      <RouteProblem code="VERSION_HISTORY_UNAVAILABLE" onRetry={() => void versions.refetch()} />
    );
  return (
    <aside aria-label={t("editor.history")}>
      <Stack space="space.200">
        <h2>{t("editor.history")}</h2>
        <Text>{t("editor.compareHint")}</Text>
        <VersionList
          page={versions.data}
          selected={selected}
          onSelect={(id) =>
            setSelected((current) => {
              const next = selectVersionIds(current, id);
              replaceVersionSearch(next);
              return next;
            })
          }
          onRestore={(id) => restore.mutate(id)}
          restorePending={restore.isPending}
        />
        {restore.error ? <HistoryProblem error={restore.error} /> : null}
        {pair ? (
          diff.isPending || left.isPending || right.isPending ? (
            <RoutePending />
          ) : diff.error || left.error || right.error ? (
            <HistoryProblem error={diff.error ?? left.error ?? right.error} />
          ) : (
            <VersionComparison
              left={left.data}
              right={right.data}
              operations={diff.data.operations as DocumentOperation[]}
            />
          )
        ) : null}
      </Stack>
    </aside>
  );
}

function VersionList({
  page,
  selected,
  onSelect,
  onRestore,
  restorePending,
}: Readonly<{
  page: VersionPage;
  selected: string[];
  onSelect: (id: string) => void;
  onRestore: (id: string) => void;
  restorePending: boolean;
}>) {
  const t = useTranslation();
  return (
    <ol className="resource-list">
      {page.items.map((version) => (
        <li key={version.id}>
          <Stack space="space.075">
            <Inline space="space.100" alignBlock="center" shouldWrap>
              <Checkbox
                isChecked={selected.includes(version.id)}
                label={`${t("editor.compareVersion")} ${version.number}`}
                onChange={() => onSelect(version.id)}
              />
              <Lozenge>v{version.number}</Lozenge>
              <Text>{version.summary}</Text>
            </Inline>
            <Text>{new Date(version.publishedAt).toLocaleString()}</Text>
            <Button
              appearance="subtle"
              isDisabled={restorePending}
              onClick={() => onRestore(version.id)}
            >
              {t("editor.restoreVersion")}
            </Button>
          </Stack>
        </li>
      ))}
    </ol>
  );
}

function VersionComparison({
  left,
  right,
  operations,
}: Readonly<{
  left: PublishedVersion;
  right: PublishedVersion;
  operations: DocumentOperation[];
}>) {
  const t = useTranslation();
  return (
    <section aria-label={t("editor.versionDiff")}>
      <h3>{t("editor.versionDiff")}</h3>
      <div className="version-comparison">
        <section>
          <h4>v{left.number}</h4>
          <ContentRenderer content={left.content as DocumentContent} />
        </section>
        <section>
          <h4>v{right.number}</h4>
          <ContentRenderer content={right.content as DocumentContent} />
        </section>
      </div>
      <ol>
        {operations.map((operation) => (
          <li className="version-diff-operation" key={operation.opId}>
            <code>{operation.kind}</code> · {operation.scope.kind}
          </li>
        ))}
      </ol>
      {operations.length === 0 ? <Text>{t("editor.noVersionChanges")}</Text> : null}
    </section>
  );
}

function HistoryProblem({ error }: Readonly<{ error: unknown }>) {
  const code = error instanceof ApiProblemError ? error.problem.code : "VERSION_COMMAND_FAILED";
  return <InlineMessage appearance="error" title={code} />;
}

function documentUrl(workspaceSlug: string, documentId: string, mode: "published" | "draft") {
  return `/w/${encodeURIComponent(workspaceSlug)}/docs/${encodeURIComponent(documentId)}?mode=${mode}`;
}

export function selectVersionIds(current: string[], id: string): string[] {
  return current.includes(id)
    ? current.filter((value) => value !== id)
    : [...current.slice(-1), id];
}

function replaceVersionSearch(selected: string[]) {
  const url = new URL(window.location.href);
  url.searchParams.set("mode", "published");
  url.searchParams.set("panel", "history");
  if (selected.length === 2) {
    url.searchParams.set("from", selected[0] ?? "");
    url.searchParams.set("to", selected[1] ?? "");
  } else {
    url.searchParams.delete("from");
    url.searchParams.delete("to");
  }
  window.history.replaceState(window.history.state, "", url);
}
