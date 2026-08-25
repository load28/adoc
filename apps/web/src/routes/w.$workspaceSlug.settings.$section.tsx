import { parseSettingsSearch, parseSettingsSection } from "@adoc/ui-domain";
import { createFileRoute, getRouteApi, notFound } from "@tanstack/react-router";

import { SettingsAuditScreen } from "../operations/settings-audit-screen";

export const Route = createFileRoute("/w/$workspaceSlug/settings/$section")({
  validateSearch: parseSettingsSearch,
  loader: ({ params }) => {
    const section = parseSettingsSection(params.section);
    if (!section) throw notFound();
    return section;
  },
  component: SettingsScreen,
});

function SettingsScreen() {
  const workspace = getRouteApi("/w/$workspaceSlug").useLoaderData();
  const search = Route.useSearch();
  return (
    <SettingsAuditScreen
      workspaceId={workspace.id}
      workspaceSlug={Route.useParams().workspaceSlug}
      section={Route.useLoaderData()}
      documentId={search.document}
      search={search}
    />
  );
}
