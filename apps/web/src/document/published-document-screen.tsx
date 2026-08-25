import type { DocumentContent } from "@adoc/contracts";
import { exportDocumentText } from "@adoc/editor-schema";
import { ApiClient, type DocumentDetail } from "@adoc/ui-domain";
import Button from "@atlaskit/button/default/button";
import LinkButton from "@atlaskit/button/link";
import { Inline, Stack, Text } from "@atlaskit/primitives";
import { useQuery } from "@tanstack/react-query";

import { RouteEmpty, RoutePending, RouteProblem } from "../shell/common-states";
import { useTranslation } from "../shell/product-app-provider";
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
          assetUrl={(assetId) =>
            `/api/v1/workspaces/${encodeURIComponent(workspaceId)}/files/${encodeURIComponent(assetId)}/content`
          }
        />
      </Stack>
    </main>
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
