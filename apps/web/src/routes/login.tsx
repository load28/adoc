import LinkButton from "@atlaskit/button/link";
import { beginGoogleLoginUrl, canonicalReturnTo } from "@adoc/ui-domain";
import { Box, Stack } from "@atlaskit/primitives";
import { createFileRoute, redirect } from "@tanstack/react-router";

import { useTranslation } from "../shell/product-app-provider";
import { loadShellBootstrap } from "../shell/server-bootstrap";

export const Route = createFileRoute("/login")({
  validateSearch: (search) =>
    typeof search.returnTo === "string"
      ? { returnTo: canonicalReturnTo(search.returnTo) }
      : { returnTo: undefined },
  loader: async () => {
    const bootstrap = await loadShellBootstrap();
    if (bootstrap.authenticated) throw redirect({ to: "/workspaces" });
  },
  component: LoginScreen,
});

function LoginScreen() {
  const t = useTranslation();
  const returnTo = canonicalReturnTo(Route.useSearch().returnTo);
  return (
    <Box as="main" id="main-content" padding="space.600">
      <Stack space="space.300" alignInline="center">
        <Box as="h1">{t("app.name")}</Box>
        <p>{t("auth.loginDescription")}</p>
        <LinkButton href={beginGoogleLoginUrl(returnTo)} appearance="primary">
          {t("auth.login")}
        </LinkButton>
      </Stack>
    </Box>
  );
}
