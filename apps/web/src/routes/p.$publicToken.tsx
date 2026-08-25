import { createFileRoute } from "@tanstack/react-router";

import { ReservedScreen } from "../shell/reserved-screen";
import { useTranslation } from "../shell/product-app-provider";

export const Route = createFileRoute("/p/$publicToken")({
  component: PublicDocumentScreen,
});

function PublicDocumentScreen() {
  const t = useTranslation();
  return <ReservedScreen title={t("route.publicDocument")} />;
}
