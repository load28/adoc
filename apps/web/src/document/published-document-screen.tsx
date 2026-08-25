import type { DocumentContent } from "@adoc/contracts";
import { exportDocumentText } from "@adoc/editor-schema";
import {
  ApiClient,
  type DocumentDetail,
  type PublicLinkCreated,
  fileContentUrl,
} from "@adoc/ui-domain";
import Button from "@atlaskit/button/default/button";
import LinkButton from "@atlaskit/button/link";
import { Inline, Stack, Text } from "@atlaskit/primitives";
import Textfield from "@atlaskit/textfield";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";

import { RouteEmpty, RoutePending, RouteProblem } from "../shell/common-states";
import { useTranslation } from "../shell/product-app-provider";
import { browserCommand } from "../shell/browser-command";
import { ContentRenderer } from "./content-renderer";

const api = new ApiClient();

export function PublishedDocumentScreen({
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
  const t = useTranslation();
  const query = useQuery({
    queryKey: ["document", workspaceId, documentId],
    queryFn: ({ signal }) => api.document(workspaceId, documentId, signal),
    initialData: initialDocument,
  });
  if (query.isPending) return <RoutePending />;
  if (query.error)
    return <RouteProblem code="DOCUMENT_UNAVAILABLE" onRetry={() => void query.refetch()} />;
  const version = query.data.publishedVersion;
  if (!version)
    return (
      <RouteEmpty
        action={
          <LinkButton href={documentUrl(workspaceSlug, documentId, "draft")} appearance="primary">
            {t("editor.edit")}
          </LinkButton>
        }
      />
    );
  const content = version.content as DocumentContent;
  return (
    <main id="main-content" className="document-view">
      <Stack space="space.250">
        <header className="document-view-header">
          <Stack space="space.100">
            <h1>{query.data.title}</h1>
            <Text>
              v{version.number} · {version.summary}
            </Text>
            <div className="document-view-actions">
              <Inline space="space.100" shouldWrap>
                <LinkButton
                  href={documentUrl(workspaceSlug, documentId, "draft")}
                  appearance="primary"
                >
                  {t("editor.edit")}
                </LinkButton>
                <LinkButton
                  href={`${documentUrl(workspaceSlug, documentId, "published")}&panel=history`}
                >
                  {t("editor.history")}
                </LinkButton>
                <Button onClick={() => downloadText(query.data, content, "markdown")}>
                  Markdown
                </Button>
                <Button onClick={() => downloadText(query.data, content, "plain")}>
                  {t("editor.plainText")}
                </Button>
                <Button onClick={() => window.print()}>PDF</Button>
              </Inline>
            </div>
          </Stack>
        </header>
        <ContentRenderer
          content={content}
          assetUrl={(assetId) => fileContentUrl(workspaceId, assetId)}
        />
        <PublicLinkManager workspaceId={workspaceId} documentId={documentId} />
      </Stack>
    </main>
  );
}

function PublicLinkManager({
  workspaceId,
  documentId,
}: Readonly<{ workspaceId: string; documentId: string }>) {
  const client = useQueryClient();
  const [expiresAt, setExpiresAt] = useState("");
  const [created, setCreated] = useState<PublicLinkCreated>();
  const links = useQuery({
    queryKey: ["public-links", workspaceId, documentId],
    queryFn: ({ signal }) => api.publicLinks(workspaceId, documentId, signal),
    retry: false,
  });
  const create = useMutation({
    mutationFn: () =>
      api.createPublicLink(
        workspaceId,
        documentId,
        expiresAt ? new Date(expiresAt).toISOString() : null,
        browserCommand(),
      ),
    onSuccess: async (value) => {
      setCreated(value);
      await client.invalidateQueries({ queryKey: ["public-links", workspaceId, documentId] });
    },
  });
  const revoke = useMutation({
    mutationFn: (link: NonNullable<typeof links.data>[number]) =>
      api.revokePublicLink(workspaceId, documentId, link, browserCommand()),
    onSuccess: async () =>
      client.invalidateQueries({ queryKey: ["public-links", workspaceId, documentId] }),
  });
  if (links.error) return null;
  return (
    <section aria-labelledby="public-links-heading">
      <Stack space="space.100">
        <h2 id="public-links-heading">공개 링크</h2>
        <label htmlFor="public-link-expiry">만료 시각 (선택)</label>
        <Textfield
          id="public-link-expiry"
          type="datetime-local"
          value={expiresAt}
          onChange={(event) => setExpiresAt(event.currentTarget.value)}
        />
        <Button onClick={() => create.mutate()} isLoading={create.isPending}>
          읽기 전용 공개 링크 만들기
        </Button>
        {created ? (
          <LinkButton href={created.url} target="_blank">
            방금 만든 링크 열기
          </LinkButton>
        ) : null}
        <ul>
          {(links.data ?? []).map((link) => (
            <li key={link.id}>
              <Inline space="space.100">
                <Text>{link.revokedAt ? "해지됨" : (link.expiresAt ?? "만료 없음")}</Text>
                {!link.revokedAt ? (
                  <Button onClick={() => revoke.mutate(link)} isLoading={revoke.isPending}>
                    해지
                  </Button>
                ) : null}
              </Inline>
            </li>
          ))}
        </ul>
      </Stack>
    </section>
  );
}

function documentUrl(workspaceSlug: string, documentId: string, mode: "published" | "draft") {
  return `/w/${encodeURIComponent(workspaceSlug)}/docs/${encodeURIComponent(documentId)}?mode=${mode}`;
}

function downloadText(
  document: DocumentDetail,
  content: DocumentContent,
  format: "markdown" | "plain",
) {
  const body = exportDocumentText(content, format);
  const blob = new Blob([body], { type: format === "markdown" ? "text/markdown" : "text/plain" });
  const url = URL.createObjectURL(blob);
  const anchor = window.document.createElement("a");
  anchor.href = url;
  anchor.download = `${safeFilename(document.title)}.${format === "markdown" ? "md" : "txt"}`;
  anchor.click();
  URL.revokeObjectURL(url);
}

function safeFilename(value: string) {
  const normalized = [...value]
    .map((character) =>
      character.charCodeAt(0) < 32 || '\\/:*?"<>|'.includes(character) ? "-" : character,
    )
    .join("");
  return normalized.slice(0, 120) || "document";
}
