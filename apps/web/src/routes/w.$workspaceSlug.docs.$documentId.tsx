import { parseDocumentSearch } from "@adoc/ui-domain";
import { createFileRoute } from "@tanstack/react-router";

import { getRouteApi } from "@tanstack/react-router";

import { DocumentEditorScreen } from "../editor/document-editor-screen";
import { DocumentCollaborationPanel } from "../collaboration/collaboration-knowledge-screen";
import { AIInspector } from "../ai/ai-inspector";
import { useTranslation } from "../shell/product-app-provider";

export const Route = createFileRoute("/w/$workspaceSlug/docs/$documentId")({
  validateSearch: parseDocumentSearch,
  component: DocumentScreen,
});

function DocumentScreen() {
  const t = useTranslation();
  const { mode, panel, discussion, review, job, proposal } = Route.useSearch();
  const { documentId, workspaceSlug } = Route.useParams();
  const workspace = getRouteApi("/w/$workspaceSlug").useLoaderData();
  const selectedPanel = panel ?? "discussion";
  return (
    <>
      {mode === "draft" ? (
        <DocumentEditorScreen workspaceId={workspace.id} documentId={documentId} />
      ) : (
        <main className="resource-screen">
          <h1>{t("route.document")}</h1>
        </main>
      )}
      {selectedPanel === "ai" ? (
        <AIInspector
          workspaceId={workspace.id}
          workspaceSlug={workspaceSlug}
          documentId={documentId}
          jobId={job}
          proposalId={proposal}
        />
      ) : (
        <DocumentCollaborationPanel
          workspaceId={workspace.id}
          workspaceSlug={workspaceSlug}
          documentId={documentId}
          panel={selectedPanel}
          discussionId={discussion}
          reviewId={review}
        />
      )}
    </>
  );
}
