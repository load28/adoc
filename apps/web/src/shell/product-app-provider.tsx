import AppProvider, { type RouterLinkComponentProps } from "@atlaskit/app-provider";
import { setBooleanFeatureFlagResolver } from "@atlaskit/platform-feature-flags";
import type { Locale, MessageKey } from "@adoc/i18n";
import { translate } from "@adoc/i18n";
import { Link } from "@tanstack/react-router";
import { type ReactNode, createContext, forwardRef, useContext } from "react";

export type ThemePreference = "LIGHT" | "DARK" | "SYSTEM";

type Translator = (key: MessageKey) => string;

const TranslationContext = createContext<Translator>(() => {
  throw new Error("translation context is unavailable");
});

setBooleanFeatureFlagResolver(() => false);

const RouterLink = forwardRef<HTMLAnchorElement, RouterLinkComponentProps>(function RouterLink(
  { href, children, ...rest },
  ref,
) {
  const target = routerTarget(String(href));
  if (!target)
    return (
      <a ref={ref} href={String(href)} {...rest}>
        {children}
      </a>
    );
  return (
    <Link ref={ref} to={target.pathname} search={target.search} hash={target.hash} {...rest}>
      {children}
    </Link>
  );
});

export function routerTarget(
  href: string,
): { pathname: string; search: Record<string, string>; hash: string } | undefined {
  if (!href.startsWith("/") || href.startsWith("//")) return undefined;
  const url = new URL(href, "https://adoc.invalid");
  if (url.pathname.startsWith("/api/") || url.pathname.startsWith("/public/")) return undefined;
  return {
    pathname: url.pathname,
    search: Object.fromEntries(url.searchParams),
    hash: url.hash.slice(1),
  };
}

export function ProductAppProvider({
  children,
  locale,
  theme,
}: Readonly<{ children: ReactNode; locale: Locale; theme: ThemePreference }>) {
  const translator: Translator = (key) => translate(locale, key);
  return (
    <TranslationContext.Provider value={translator}>
      <AppProvider defaultColorMode={toColorMode(theme)} routerLinkComponent={RouterLink}>
        {children}
      </AppProvider>
    </TranslationContext.Provider>
  );
}

export function useTranslation(): Translator {
  return useContext(TranslationContext);
}

export function toColorMode(theme: ThemePreference): "light" | "dark" | "auto" {
  if (theme === "LIGHT") return "light";
  if (theme === "DARK") return "dark";
  return "auto";
}
