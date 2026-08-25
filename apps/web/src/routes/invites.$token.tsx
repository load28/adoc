import { ApiClient, ApiProblemError } from "@adoc/ui-domain";
import Button from "@atlaskit/button/default/button";
import InlineMessage from "@atlaskit/inline-message";
import { Box, Stack, Text } from "@atlaskit/primitives";
import { useMutation } from "@tanstack/react-query";
import { createFileRoute, redirect } from "@tanstack/react-router";

import { useTranslation } from "../shell/product-app-provider";
import { browserCommand } from "../shell/browser-command";
import { loadInvitationPreview, loadShellBootstrap } from "../shell/server-bootstrap";

export const Route = createFileRoute("/invites/$token")({
  loader: async ({ params }) => {
    const bootstrap = await loadShellBootstrap();
    if (!bootstrap.authenticated) {
      throw redirect({
        to: "/login",
        search: { returnTo: `/invites/${encodeURIComponent(params.token)}` },
      });
    }
    return loadInvitationPreview({ data: { token: params.token } });
  },
  component: InvitationScreen,
});

function InvitationScreen() {
  const t = useTranslation();
  const preview = Route.useLoaderData();
  const { token } = Route.useParams();
  const accept = useMutation({
    mutationFn: () => new ApiClient().acceptInvitation(token, browserCommand()),
    onSuccess: () => window.location.assign(`/w/${encodeURIComponent(preview.workspaceSlug)}/home`),
  });
  const code =
    accept.error instanceof ApiProblemError ? accept.error.problem.code : "INVITATION_INVALID";
  return (
    <Box as="main" id="main-content" padding="space.600">
      <Stack space="space.300" alignInline="center">
        <Box as="h1">{t("invitation.title")}</Box>
        <Text>{preview.workspaceName}</Text>
        <Text>
          {t(preview.role === "ADMIN" ? "invitation.roleAdmin" : "invitation.roleMember")}
        </Text>
        <Text>{new Date(preview.expiresAt).toLocaleString()}</Text>
        <Button
          appearance="primary"
          isLoading={accept.isPending}
          isDisabled={accept.isPending}
          onClick={() => accept.mutate()}
        >
          {t("invitation.accept")}
        </Button>
        {accept.error ? (
          <InlineMessage appearance="error" title={t("common.unavailable")}>
            <p>{code}</p>
          </InlineMessage>
        ) : null}
      </Stack>
    </Box>
  );
}
