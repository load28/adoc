import { createFileRoute } from "@tanstack/react-router";

import { ReservedScreen } from "../shell/reserved-screen";
import { useTranslation } from "../shell/product-app-provider";

export const Route = createFileRoute("/invites/$token")({
  component: InvitationScreen,
});

function InvitationScreen() {
  const t = useTranslation();
  return <ReservedScreen title={t("route.invitation")} />;
}
