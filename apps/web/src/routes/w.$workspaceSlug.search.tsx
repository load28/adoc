import { createFileRoute, getRouteApi } from "@tanstack/react-router";

import { SearchScreen } from "../collaboration/collaboration-knowledge-screen";

export const Route = createFileRoute("/w/$workspaceSlug/search")({
  validateSearch: (value: Record<string, unknown>) => ({
    q: typeof value.q === "string" && value.q.length <= 500 ? value.q : "",
  }),
  component: SearchRoute,
});

function SearchRoute() {
  const workspace = getRouteApi("/w/$workspaceSlug").useLoaderData();
  return <SearchScreen workspaceId={workspace.id} initialQuery={Route.useSearch().q} />;
}
