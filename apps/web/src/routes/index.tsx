import { createFileRoute, redirect } from "@tanstack/react-router";

import { loadShellBootstrap } from "../shell/server-bootstrap";

export const Route = createFileRoute("/")({
  loader: async () => {
    const bootstrap = await loadShellBootstrap();
    throw redirect({ to: bootstrap.authenticated ? "/workspaces" : "/login" });
  },
});
