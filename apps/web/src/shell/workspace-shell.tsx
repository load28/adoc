import { ApiClient } from "@adoc/ui-domain";
import { useMutation, useQuery } from "@tanstack/react-query";
import { Outlet, useLocation } from "@tanstack/react-router";
import {
  Bell,
  BookOpenText,
  ChevronDown,
  FileText,
  Home,
  Languages,
  LogOut,
  Menu,
  MoonStar,
  Search,
  Settings,
  Trash2,
} from "lucide-react";

import { BrandMark } from "../components/product/brand-mark";
import { LinkButton } from "../components/product/legacy";
import { Button } from "../components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "../components/ui/dropdown-menu";
import {
  Sheet,
  SheetContent,
  SheetDescription,
  SheetHeader,
  SheetTitle,
  SheetTrigger,
} from "../components/ui/sheet";
import { DocumentTreeNavigation } from "../workspace/document-tree-navigation";
import { browserCommand } from "./browser-command";
import { useTranslation } from "./product-app-provider";
import { useWorkspaceRealtime } from "../collaboration/workspace-realtime";

const api = new ApiClient();

export function WorkspaceShell({
  id,
  slug,
  name,
}: Readonly<{ id: string; slug: string; name: string }>) {
  useWorkspaceRealtime(id);
  const t = useTranslation();
  const location = useLocation();
  const base = `/w/${encodeURIComponent(slug)}`;
  const links = [
    { label: t("navigation.home"), href: `${base}/home`, icon: Home },
    { label: t("navigation.search"), href: `${base}/search`, icon: Search },
    { label: t("navigation.inbox"), href: `${base}/inbox`, icon: Bell },
    { label: t("navigation.vocabulary"), href: `${base}/vocabulary`, icon: BookOpenText },
    { label: t("navigation.trash"), href: `${base}/trash`, icon: Trash2, separated: true },
    {
      label: t("navigation.settings"),
      href: `${base}/settings/members`,
      icon: Settings,
    },
  ] as const;
  const navigation = (
    <WorkspaceNavigation
      id={id}
      slug={slug}
      name={name}
      pathname={location.pathname}
      links={links}
    />
  );

  return (
    <div className="min-h-svh bg-background text-foreground">
      <a
        href="#main-content"
        className="fixed left-3 top-3 z-[100] -translate-y-20 rounded-md bg-primary px-4 py-2 font-medium text-primary-foreground shadow-lg outline-none transition-transform focus:translate-y-0"
      >
        {t("navigation.skip")}
      </a>
      <header className="sticky top-0 z-40 flex h-14 items-center border-b bg-background/95 px-3 backdrop-blur supports-[backdrop-filter]:bg-background/85 lg:px-4">
        <div className="flex min-w-0 flex-1 items-center gap-2">
          <Sheet>
            <SheetTrigger asChild>
              <Button
                variant="ghost"
                size="icon"
                className="lg:hidden"
                aria-label={t("navigation.expand")}
              >
                <Menu />
              </Button>
            </SheetTrigger>
            <SheetContent side="left">
              <SheetHeader>
                <SheetTitle>{name}</SheetTitle>
                <SheetDescription>{t("navigation.workspace")}</SheetDescription>
              </SheetHeader>
              {navigation}
            </SheetContent>
          </Sheet>
          <LinkButton href="/workspaces" appearance="subtle" className="h-10 gap-2 px-2">
            <BrandMark className="size-7 rounded-md text-xs" />
            <span className="hidden font-semibold tracking-tight sm:inline">Adoc</span>
          </LinkButton>
          <span aria-hidden="true" className="hidden h-5 w-px bg-border sm:block" />
          <LinkButton
            href="/workspaces"
            appearance="subtle"
            className="min-w-0 max-w-64 justify-start px-2"
          >
            <span className="truncate">{name}</span>
            <ChevronDown className="size-4 text-muted-foreground" />
          </LinkButton>
        </div>
        <LinkButton
          href={`${base}/search`}
          appearance="subtle"
          className="mr-1 hidden w-64 justify-start border bg-muted/45 text-muted-foreground md:inline-flex"
        >
          <Search className="size-4" />
          {t("navigation.search")}
          <kbd className="ml-auto rounded border bg-background px-1.5 text-[11px]">⌘K</kbd>
        </LinkButton>
        <AccountMenu />
      </header>
      <div className="flex min-h-[calc(100svh-3.5rem)]">
        <aside
          aria-label={t("navigation.workspace")}
          className="sticky top-14 hidden h-[calc(100svh-3.5rem)] w-66 shrink-0 flex-col overflow-hidden border-r border-sidebar-border bg-sidebar text-sidebar-foreground lg:flex"
        >
          {navigation}
        </aside>
        <div className="min-w-0 flex-1">
          <Outlet />
        </div>
      </div>
    </div>
  );
}

function WorkspaceNavigation({
  id,
  slug,
  name,
  pathname,
  links,
}: Readonly<{
  id: string;
  slug: string;
  name: string;
  pathname: string;
  links: ReadonlyArray<{
    label: string;
    href: string;
    icon: typeof Home;
    separated?: boolean;
  }>;
}>) {
  return (
    <div className="flex min-h-0 flex-1 flex-col px-3 py-3">
      <div className="mb-3 px-2 lg:hidden">
        <p className="truncate text-sm font-semibold">{name}</p>
      </div>
      <nav aria-label="주요 메뉴" className="space-y-1">
        {links.map(({ label, href, icon: Icon, separated }) => (
          <div key={href} className={separated ? "border-t pt-2 mt-2" : undefined}>
            <LinkButton
              href={href}
              appearance="subtle"
              isSelected={
                pathname === href ||
                (href.includes("/settings/") && pathname.includes("/settings/"))
              }
              shouldFitContainer
              className="h-9 gap-3 px-2.5 font-normal"
              aria-current={pathname === href ? "page" : undefined}
            >
              <Icon className="size-4 text-muted-foreground" />
              <span className="truncate">{label}</span>
            </LinkButton>
          </div>
        ))}
      </nav>
      <div className="my-3 h-px bg-sidebar-border" />
      <div className="flex min-h-0 flex-1 flex-col">
        <div className="flex h-8 items-center gap-2 px-2 text-xs font-semibold text-muted-foreground">
          <FileText className="size-3.5" />
          문서
        </div>
        <div className="min-h-0 flex-1 overflow-y-auto overscroll-contain">
          <DocumentTreeNavigation workspaceId={id} workspaceSlug={slug} />
        </div>
      </div>
    </div>
  );
}

function AccountMenu() {
  const preferences = useQuery({
    queryKey: ["preferences"],
    queryFn: ({ signal }) => api.preferences(signal),
  });
  const update = useMutation({
    mutationFn: (kind: "locale" | "theme") => {
      if (!preferences.data) throw new Error("preferences are unavailable");
      return api.updatePreferences(
        preferences.data,
        {
          locale:
            kind === "locale"
              ? preferences.data.locale === "ko"
                ? "en"
                : "ko"
              : preferences.data.locale,
          timezone: preferences.data.timezone,
          theme:
            kind === "theme"
              ? preferences.data.theme === "DARK"
                ? "LIGHT"
                : "DARK"
              : preferences.data.theme,
        },
        browserCommand(),
      );
    },
    onSuccess: () => window.location.reload(),
  });
  const logout = useMutation({
    mutationFn: () => api.logout(browserCommand()),
    onSuccess: () => window.location.assign("/login"),
  });
  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button variant="ghost" size="icon" aria-label="계정 메뉴">
          <span className="grid size-7 place-items-center rounded-full bg-primary/12 text-xs font-semibold text-primary">
            {preferences.data?.locale.toUpperCase() ?? "A"}
          </span>
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end" className="w-56">
        <DropdownMenuLabel>환경 설정</DropdownMenuLabel>
        <DropdownMenuItem onSelect={() => update.mutate("locale")}>
          <Languages />
          언어 · {preferences.data?.locale.toUpperCase() ?? "—"}
        </DropdownMenuItem>
        <DropdownMenuItem onSelect={() => update.mutate("theme")}>
          <MoonStar />
          테마 · {preferences.data?.theme ?? "—"}
        </DropdownMenuItem>
        <DropdownMenuSeparator />
        <DropdownMenuItem
          onSelect={() => logout.mutate()}
          className="text-destructive focus:text-destructive"
        >
          <LogOut />
          로그아웃
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
