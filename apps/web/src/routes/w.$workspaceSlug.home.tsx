import { createFileRoute } from "@tanstack/react-router";
import { Box, Stack } from "@atlaskit/primitives";

import { useTranslation } from "../shell/product-app-provider";

export const Route = createFileRoute("/w/$workspaceSlug/home")({
  component: WorkspaceHome,
});

function WorkspaceHome() {
  const t = useTranslation();
  return (
    <Box as="main" id="main-content" padding="space.400">
      <Stack space="space.200">
        <Box as="h1">{t("navigation.home")}</Box>
        <p>{t("workspace.documents")}</p>
      </Stack>
    </Box>
  );
}
