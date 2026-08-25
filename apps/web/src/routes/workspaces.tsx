import { ApiClient, ApiProblemError } from "@adoc/ui-domain";
import Button from "@atlaskit/button/default/button";
import EmptyState from "@atlaskit/empty-state";
import InlineMessage from "@atlaskit/inline-message";
import { Box, Stack } from "@atlaskit/primitives";
import Textfield from "@atlaskit/textfield";
import { useMutation } from "@tanstack/react-query";
import { createFileRoute, redirect } from "@tanstack/react-router";
import { useState } from "react";

import { useTranslation } from "../shell/product-app-provider";
import { loadShellBootstrap } from "../shell/server-bootstrap";
import LinkButton from "@atlaskit/button/link";
import { browserCommand } from "../shell/browser-command";

export const Route = createFileRoute("/workspaces")({
  loader: async () => {
    const bootstrap = await loadShellBootstrap();
    if (!bootstrap.authenticated || !bootstrap.session)
      throw redirect({ to: "/login", search: { returnTo: "/workspaces" } });
    return bootstrap.session.workspaces;
  },
  component: WorkspaceList,
});

function WorkspaceList() {
  const workspaces = Route.useLoaderData();
  const t = useTranslation();
  const [name, setName] = useState("");
  const create = useMutation({
    mutationFn: () => new ApiClient().createWorkspace(name.trim(), browserCommand()),
    onSuccess: (workspace) =>
      window.location.assign(`/w/${encodeURIComponent(workspace.slug)}/home`),
  });
  const error = create.error instanceof ApiProblemError ? create.error.problem.code : undefined;
  return (
    <Box as="main" id="main-content" padding="space.400">
      <Stack space="space.300">
        <Box as="h1">{t("workspace.list")}</Box>
        {workspaces.length === 0 ? (
          <EmptyState header={t("workspace.empty")} headingLevel={2} />
        ) : (
          <Stack space="space.100">
            {workspaces.map((workspace) => (
              <LinkButton key={workspace.id} href={`/w/${workspace.slug}/home`} shouldFitContainer>
                {workspace.name}
              </LinkButton>
            ))}
          </Stack>
        )}
        <Box as="h2">{t("workspace.create")}</Box>
        <form
          onSubmit={(event) => {
            event.preventDefault();
            if (name.trim().length > 0 && name.trim().length <= 200) create.mutate();
          }}
        >
          <Stack space="space.150">
            <label htmlFor="workspace-name">{t("workspace.name")}</label>
            <Textfield
              id="workspace-name"
              value={name}
              maxLength={200}
              onChange={(event) => setName(event.currentTarget.value)}
            />
            <Button
              type="submit"
              appearance="primary"
              isLoading={create.isPending}
              isDisabled={create.isPending || name.trim().length === 0}
            >
              {t("workspace.create")}
            </Button>
            {error ? (
              <InlineMessage appearance="error" title={t("common.unavailable")}>
                <p>{error}</p>
              </InlineMessage>
            ) : null}
          </Stack>
        </form>
      </Stack>
    </Box>
  );
}
