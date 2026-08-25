import { EmptyState } from "../components/product/legacy";
import { Box } from "../components/product/legacy";

import { useTranslation } from "./product-app-provider";

export function ReservedScreen({ title }: Readonly<{ title: string }>) {
  const t = useTranslation();
  return (
    <Box padding="space.400">
      <EmptyState header={title} description={t("route.preparing")} headingLevel={1} />
    </Box>
  );
}
