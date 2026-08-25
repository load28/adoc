import LinkButton from "@atlaskit/button/link";
import { Box, Stack } from "@atlaskit/primitives";
import { createFileRoute, redirect } from "@tanstack/react-router";

import { useTranslation } from "../shell/product-app-provider";
import { loadShellBootstrap } from "../shell/server-bootstrap";

export const Route = createFileRoute("/login")({
  loader: async () => {
    const bootstrap = await loadShellBootstrap();
    if (bootstrap.authenticated) throw redirect({ to: "/workspaces" });
  },
  component: LoginScreen,
});

function LoginScreen() {
  const t = useTranslation();
  return (
    <Box as="main" id="main-content" padding="space.600">
      <Stack space="space.300" alignInline="center">
        <Box as="h1">{t("app.name")}</Box>
        <p>{t("auth.loginDescription")}</p>
        <LinkButton href="/api/v1/auth/google/start?returnTo=%2Fworkspaces" appearance="primary">
          {t("auth.login")}
        </LinkButton>
      </Stack>
    </Box>
  );
}
