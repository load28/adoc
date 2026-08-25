import { createFileRoute, getRouteApi } from "@tanstack/react-router";

import { InboxScreen } from "../collaboration/collaboration-knowledge-screen";

export const Route = createFileRoute("/w/$workspaceSlug/inbox")({
  component: InboxRoute,
});

function InboxRoute() {
  const workspace = getRouteApi("/w/$workspaceSlug").useLoaderData();
  return <InboxScreen workspaceId={workspace.id} />;
}
