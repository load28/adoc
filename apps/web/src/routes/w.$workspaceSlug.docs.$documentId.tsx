import { parseDocumentSearch } from "@adoc/ui-domain";
import { createFileRoute } from "@tanstack/react-router";

import { getRouteApi } from "@tanstack/react-router";

import { DocumentEditorScreen } from "../editor/document-editor-screen";
import { DocumentCollaborationPanel } from "../collaboration/collaboration-knowledge-screen";
import { AIInspector } from "../ai/ai-inspector";
import { PublishedDocumentScreen } from "../document/published-document-screen";
import { VersionHistoryPanel } from "../document/version-history-panel";
import { loadDocumentRoute } from "../shell/server-bootstrap";

export const Route = createFileRoute("/w/$workspaceSlug/docs/$documentId")({
  validateSearch: parseDocumentSearch,
  loader: ({ params }) =>
    loadDocumentRoute({
      data: { workspaceSlug: params.workspaceSlug, documentId: params.documentId },
    }),
  component: DocumentScreen,
});

function DocumentScreen() {
  const { mode, panel, discussion, review, job, proposal, from, to } = Route.useSearch();
  const { documentId, workspaceSlug } = Route.useParams();
  const workspace = getRouteApi("/w/$workspaceSlug").useLoaderData();
  const routeDocument = Route.useLoaderData();
  const selectedPanel = panel ?? "discussion";
  return (
    <>
      {mode === "draft" ? (
        <DocumentEditorScreen
          workspaceId={workspace.id}
          workspaceSlug={workspaceSlug}
          documentId={documentId}
          initialDocument={routeDocument.document}
        />
      ) : (
        <PublishedDocumentScreen
          workspaceId={workspace.id}
          workspaceSlug={workspaceSlug}
          documentId={documentId}
          initialDocument={routeDocument.document}
        />
      )}
      {selectedPanel === "history" ? (
        <VersionHistoryPanel
          workspaceId={workspace.id}
          workspaceSlug={workspaceSlug}
          documentId={documentId}
          from={from}
          to={to}
        />
      ) : selectedPanel === "ai" ? (
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
