import type { DocumentContent } from "@adoc/contracts";
import { exportDocumentText } from "@adoc/editor-schema";
import {
  ApiClient,
  type DocumentDetail,
  type PublicLinkCreated,
  fileContentUrl,
} from "@adoc/ui-domain";
import { Button } from "../components/product/legacy";
import { LinkButton } from "../components/product/legacy";
import { Inline, Stack, Text } from "../components/product/legacy";
import { Textfield } from "../components/product/legacy";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { type ReactNode, useState } from "react";

import { PageHeader } from "../components/product/page";
import { EmptyState } from "../components/product/legacy";
import { RoutePending, RouteProblem } from "../shell/common-states";
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
      <UnpublishedDocumentState
        title={query.data.title}
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
        <PageHeader
          eyebrow="PUBLISHED DOCUMENT"
          title={query.data.title}
          status={
            <span className="inline-flex h-6 items-center rounded-full border border-success/25 bg-success/10 px-2 text-xs font-medium text-success-foreground">
              발행됨 · v{version.number}
            </span>
          }
          description={version.summary}
          actions={
            <>
              <LinkButton
                href={documentUrl(workspaceSlug, documentId, "draft")}
                appearance="primary"
              >
                {t("editor.edit")}
              </LinkButton>
              <LinkButton
                href={`${documentUrl(workspaceSlug, documentId, "published")}&panel=discussion`}
              >
                토론
              </LinkButton>
              <LinkButton
                href={`${documentUrl(workspaceSlug, documentId, "published")}&panel=history`}
              >
                {t("editor.history")}
              </LinkButton>
            </>
          }
        />
        <article className="document-reading-surface">
          <ContentRenderer
            content={content}
            assetUrl={(assetId) => fileContentUrl(workspaceId, assetId)}
          />
        </article>
        <section className="document-utilities" aria-labelledby="document-export-heading">
          <div>
            <h2 id="document-export-heading">내보내기</h2>
            <p>현재 발행 버전을 원하는 형식으로 저장합니다.</p>
          </div>
          <Inline space="space.100" shouldWrap>
            <Button onClick={() => downloadText(query.data, content, "markdown")}>Markdown</Button>
            <Button onClick={() => downloadText(query.data, content, "plain")}>
              {t("editor.plainText")}
            </Button>
            <Button onClick={() => window.print()}>PDF</Button>
          </Inline>
        </section>
        <PublicLinkManager workspaceId={workspaceId} documentId={documentId} />
      </Stack>
    </main>
  );
}

export function UnpublishedDocumentState({
  title,
  action,
}: Readonly<{ title: string; action?: ReactNode }>) {
  const t = useTranslation();
  return (
    <main id="main-content">
      <EmptyState
        header={title}
        description={t("document.unpublished")}
        headingLevel={1}
        primaryAction={action}
      />
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
    <section className="document-utilities" aria-labelledby="public-links-heading">
      <Stack space="space.100">
        <div>
          <h2 id="public-links-heading">공개 링크</h2>
          <p className="mt-1 text-sm text-muted-foreground">
            최신 발행 버전만 읽을 수 있는 제한된 링크를 만듭니다.
          </p>
        </div>
        <div className="grid max-w-xl gap-2 sm:grid-cols-[1fr_auto] sm:items-end">
          <div className="grid gap-2">
            <label htmlFor="public-link-expiry">만료 시각 (선택)</label>
            <Textfield
              id="public-link-expiry"
              type="datetime-local"
              value={expiresAt}
              onChange={(event) => setExpiresAt(event.currentTarget.value)}
            />
          </div>
          <Button onClick={() => create.mutate()} isLoading={create.isPending}>
            공개 링크 만들기
          </Button>
        </div>
        {created ? (
          <LinkButton href={created.url} target="_blank">
            방금 만든 링크 열기
          </LinkButton>
        ) : null}
        <ul className="divide-y rounded-lg border">
          {(links.data ?? []).map((link) => (
            <li key={link.id} className="flex items-center justify-between gap-3 px-4 py-3">
              <Inline space="space.100" alignBlock="center">
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
