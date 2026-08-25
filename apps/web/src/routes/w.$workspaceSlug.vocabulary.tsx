import { createFileRoute } from "@tanstack/react-router";

import { ReservedScreen } from "../shell/reserved-screen";

export const Route = createFileRoute("/w/$workspaceSlug/vocabulary")({
  component: () => <ReservedScreen title="Vocabulary" />,
});
