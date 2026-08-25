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
  return (
    <Link ref={ref} to={String(href)} {...rest}>
      {children}
    </Link>
  );
});

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

export const themeBootstrapScript = `(function(){var p=document.documentElement.dataset.themePreference;var m=p==='DARK'?'dark':p==='LIGHT'?'light':window.matchMedia('(prefers-color-scheme: dark)').matches?'dark':'light';document.documentElement.dataset.colorMode=m;})();`;
