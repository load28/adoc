import { createFileRoute, getRouteApi } from "@tanstack/react-router";

import { TrashScreen } from "../operations/trash-screen";

export const Route = createFileRoute("/w/$workspaceSlug/trash")({
  component: TrashRoute,
});

function TrashRoute() {
  const workspace = getRouteApi("/w/$workspaceSlug").useLoaderData();
  return <TrashScreen workspaceId={workspace.id} />;
}
