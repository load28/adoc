import { createFileRoute } from "@tanstack/react-router";

import { PublicDocumentScreen as PublicViewer } from "../public/public-document-screen";

export const Route = createFileRoute("/p/$publicToken")({
  component: PublicDocumentScreen,
});

function PublicDocumentScreen() {
  return <PublicViewer token={Route.useParams().publicToken} />;
}
