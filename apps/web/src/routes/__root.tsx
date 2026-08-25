import "@atlaskit/css-reset";

import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { HeadContent, Outlet, Scripts, createRootRoute } from "@tanstack/react-router";
import type { ReactNode } from "react";
import { useState } from "react";

import { PRODUCT_NAME } from "../product";
import { ProductAppProvider } from "../shell/product-app-provider";
import { loadShellBootstrap } from "../shell/server-bootstrap";
import themeBootstrapUrl from "../shell/theme-bootstrap.js?url";

export const Route = createRootRoute({
  loader: () => loadShellBootstrap(),
  head: () => ({
    meta: [
      { charSet: "utf-8" },
      { name: "viewport", content: "width=device-width, initial-scale=1" },
      { title: PRODUCT_NAME },
    ],
  }),
  component: RootComponent,
});

function RootComponent() {
  const bootstrap = Route.useLoaderData();
  const [queryClient] = useState(() => new QueryClient());
  return (
    <RootDocument locale={bootstrap.locale} theme={bootstrap.theme}>
      <QueryClientProvider client={queryClient}>
        <ProductAppProvider locale={bootstrap.locale} theme={bootstrap.theme}>
          <Outlet />
        </ProductAppProvider>
      </QueryClientProvider>
    </RootDocument>
  );
}

function RootDocument({
  children,
  locale,
  theme,
}: Readonly<{ children: ReactNode; locale: "ko" | "en"; theme: "LIGHT" | "DARK" | "SYSTEM" }>) {
  return (
    <html lang={locale} data-theme-preference={theme}>
      <head>
        <HeadContent />
        <script src={themeBootstrapUrl} />
      </head>
      <body>
        {children}
        <Scripts />
      </body>
    </html>
  );
}
