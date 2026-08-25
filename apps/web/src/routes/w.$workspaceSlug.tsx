import { createFileRoute, redirect } from "@tanstack/react-router";

import { loadShellBootstrap } from "../shell/server-bootstrap";
import { WorkspaceShell } from "../shell/workspace-shell";

export const Route = createFileRoute("/w/$workspaceSlug")({
  loader: async ({ params }) => {
    const bootstrap = await loadShellBootstrap();
    if (!bootstrap.authenticated || !bootstrap.session) throw redirect({ to: "/login" });
    const workspace = bootstrap.session.workspaces.find(
      (item) => item.slug === params.workspaceSlug,
    );
    if (!workspace) throw redirect({ to: "/workspaces" });
    return workspace;
  },
  component: WorkspaceRoute,
});

function WorkspaceRoute() {
  const workspace = Route.useLoaderData();
  return <WorkspaceShell slug={workspace.slug} name={workspace.name} />;
}
