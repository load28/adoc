import { parseSettingsSection } from "@adoc/ui-domain";
import { createFileRoute, notFound } from "@tanstack/react-router";

import { ReservedScreen } from "../shell/reserved-screen";

export const Route = createFileRoute("/w/$workspaceSlug/settings/$section")({
  loader: ({ params }) => {
    const section = parseSettingsSection(params.section);
    if (!section) throw notFound();
    return section;
  },
  component: SettingsScreen,
});

function SettingsScreen() {
  return <ReservedScreen title={`Settings · ${Route.useLoaderData()}`} />;
}
