import EmptyState from "@atlaskit/empty-state";
import { Box, Stack } from "@atlaskit/primitives";
import { createFileRoute, redirect } from "@tanstack/react-router";

import { useTranslation } from "../shell/product-app-provider";
import { loadShellBootstrap } from "../shell/server-bootstrap";
import LinkButton from "@atlaskit/button/link";

export const Route = createFileRoute("/workspaces")({
  loader: async () => {
    const bootstrap = await loadShellBootstrap();
    if (!bootstrap.authenticated || !bootstrap.session) throw redirect({ to: "/login" });
    return bootstrap.session.workspaces;
  },
  component: WorkspaceList,
});

function WorkspaceList() {
  const workspaces = Route.useLoaderData();
  const t = useTranslation();
  if (workspaces.length === 0) {
    return (
      <main id="main-content">
        <EmptyState header={t("workspace.empty")} headingLevel={1} />
      </main>
    );
  }
  return (
    <Box as="main" id="main-content" padding="space.400">
      <Stack space="space.300">
        <Box as="h1">{t("workspace.list")}</Box>
        <Stack space="space.100">
          {workspaces.map((workspace) => (
            <LinkButton key={workspace.id} href={`/w/${workspace.slug}/home`} shouldFitContainer>
              {workspace.name}
            </LinkButton>
          ))}
        </Stack>
      </Stack>
    </Box>
  );
}
