import { createFileRoute, getRouteApi } from "@tanstack/react-router";

import { VocabularyScreen } from "../collaboration/collaboration-knowledge-screen";

export const Route = createFileRoute("/w/$workspaceSlug/vocabulary")({
  component: VocabularyRoute,
});

function VocabularyRoute() {
  const workspace = getRouteApi("/w/$workspaceSlug").useLoaderData();
  return <VocabularyScreen workspaceId={workspace.id} />;
}
