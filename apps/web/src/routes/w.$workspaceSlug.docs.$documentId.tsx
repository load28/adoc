import { parseDocumentSearch } from "@adoc/ui-domain";
import { createFileRoute } from "@tanstack/react-router";

import { getRouteApi } from "@tanstack/react-router";

import { DocumentEditorScreen } from "../editor/document-editor-screen";
import { ReservedScreen } from "../shell/reserved-screen";
import { useTranslation } from "../shell/product-app-provider";

export const Route = createFileRoute("/w/$workspaceSlug/docs/$documentId")({
  validateSearch: parseDocumentSearch,
  component: DocumentScreen,
});

function DocumentScreen() {
  const t = useTranslation();
  const { mode } = Route.useSearch();
  const { documentId } = Route.useParams();
  const workspace = getRouteApi("/w/$workspaceSlug").useLoaderData();
  if (mode !== "draft") return <ReservedScreen title={t("route.document")} />;
  return <DocumentEditorScreen workspaceId={workspace.id} documentId={documentId} />;
}
