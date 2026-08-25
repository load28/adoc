import EmptyState from "@atlaskit/empty-state";
import { Box } from "@atlaskit/primitives";

import { useTranslation } from "./product-app-provider";

export function ReservedScreen({ title }: Readonly<{ title: string }>) {
  const t = useTranslation();
  return (
    <Box padding="space.400">
      <EmptyState header={title} description={t("route.preparing")} headingLevel={1} />
    </Box>
  );
}
