import { createFileRoute } from "@tanstack/react-router";

import { PRODUCT_NAME } from "../product";

export const Route = createFileRoute("/")({
  component: Home,
});

function Home() {
  return <main>{PRODUCT_NAME} web bootstrap</main>;
}
