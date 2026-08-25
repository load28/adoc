import type { Locale, MessageKey } from "@adoc/i18n";
import { translate } from "@adoc/i18n";
import { type ReactNode, createContext, useContext, useEffect } from "react";

export type ThemePreference = "LIGHT" | "DARK" | "SYSTEM";

type Translator = (key: MessageKey) => string;

const TranslationContext = createContext<Translator>(() => {
  throw new Error("translation context is unavailable");
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
  useEffect(() => {
    const root = document.documentElement;
    const media = window.matchMedia("(prefers-color-scheme: dark)");
    const apply = () => {
      const dark = theme === "DARK" || (theme === "SYSTEM" && media.matches);
      root.classList.toggle("dark", dark);
      root.dataset.colorMode = dark ? "dark" : "light";
    };
    apply();
    if (theme !== "SYSTEM") return;
    media.addEventListener("change", apply);
    return () => media.removeEventListener("change", apply);
  }, [theme]);
  return <TranslationContext.Provider value={translator}>{children}</TranslationContext.Provider>;
}

export function useTranslation(): Translator {
  return useContext(TranslationContext);
}

export function toColorMode(theme: ThemePreference): "light" | "dark" | "auto" {
  if (theme === "LIGHT") return "light";
  if (theme === "DARK") return "dark";
  return "auto";
}
