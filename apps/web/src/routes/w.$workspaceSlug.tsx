import { createFileRoute, redirect } from "@tanstack/react-router";

import { loadShellBootstrap, loadWorkspaceBySlug } from "../shell/server-bootstrap";
import { WorkspaceShell } from "../shell/workspace-shell";

export const Route = createFileRoute("/w/$workspaceSlug")({
  loader: async ({ params }) => {
    const bootstrap = await loadShellBootstrap();
    if (!bootstrap.authenticated || !bootstrap.session)
      throw redirect({ to: "/login", search: { returnTo: `/w/${params.workspaceSlug}/home` } });
    return loadWorkspaceBySlug({ data: { workspaceSlug: params.workspaceSlug } });
  },
  component: WorkspaceRoute,
});

function WorkspaceRoute() {
  const workspace = Route.useLoaderData();
  return <WorkspaceShell id={workspace.id} slug={workspace.slug} name={workspace.name} />;
}
