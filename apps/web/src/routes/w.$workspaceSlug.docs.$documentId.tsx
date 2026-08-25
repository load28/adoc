import { parseDocumentSearch } from "@adoc/ui-domain";
import { createFileRoute } from "@tanstack/react-router";

import { ReservedScreen } from "../shell/reserved-screen";
import { useTranslation } from "../shell/product-app-provider";

export const Route = createFileRoute("/w/$workspaceSlug/docs/$documentId")({
  validateSearch: parseDocumentSearch,
  component: DocumentScreen,
});

function DocumentScreen() {
  const t = useTranslation();
  return <ReservedScreen title={t("route.document")} />;
}
