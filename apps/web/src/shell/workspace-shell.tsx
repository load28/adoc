import { ApiClient } from "@adoc/ui-domain";
import Button from "@atlaskit/button/default/button";
import LinkButton from "@atlaskit/button/link";
import {
  Main,
  Root,
  SideNav,
  SideNavBody,
  SideNavHeader,
  SideNavToggleButton,
  TopNav,
  TopNavEnd,
  TopNavStart,
} from "@atlaskit/navigation-system";
import { Box, Inline, Stack, Text } from "@atlaskit/primitives";
import { useMutation, useQuery } from "@tanstack/react-query";
import { Outlet, useLocation } from "@tanstack/react-router";

import { useTranslation } from "./product-app-provider";
import { useWorkspaceRealtime } from "../collaboration/workspace-realtime";
import { DocumentTreeNavigation } from "../workspace/document-tree-navigation";
import { browserCommand } from "./browser-command";

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
    [t("navigation.home"), `${base}/home`],
    [t("navigation.search"), `${base}/search`],
    [t("navigation.inbox"), `${base}/inbox`],
    [t("navigation.vocabulary"), `${base}/vocabulary`],
    [t("navigation.trash"), `${base}/trash`],
    [t("navigation.settings"), `${base}/settings/members`],
  ] as const;

  return (
    <Root
      skipLinksLabel={t("navigation.skip")}
      skipLinksTriggerLabel={t("navigation.skip")}
      defaultSideNavCollapsed={false}
    >
      <TopNav>
        <TopNavStart
          sideNavToggleButton={
            <SideNavToggleButton
              collapseLabel={t("navigation.collapse")}
              expandLabel={t("navigation.expand")}
            />
          }
        >
          <Text weight="bold">{t("app.name")}</Text>
        </TopNavStart>
        <TopNavEnd label={t("navigation.workspace")} showMoreButtonLabel={t("navigation.more")}>
          <AccountActions />
          <LinkButton href="/workspaces" appearance="subtle">
            {name}
          </LinkButton>
        </TopNavEnd>
      </TopNav>
      <SideNav label={t("navigation.workspace")}>
        <SideNavHeader>
          <Box padding="space.200">
            <Text weight="bold">{name}</Text>
          </Box>
        </SideNavHeader>
        <SideNavBody>
          <Box padding="space.100">
            <Stack space="space.050">
              {links.map(([label, href]) => (
                <LinkButton
                  key={href}
                  href={href}
                  appearance="subtle"
                  isSelected={location.pathname === href}
                  shouldFitContainer
                >
                  {label}
                </LinkButton>
              ))}
              <DocumentTreeNavigation workspaceId={id} workspaceSlug={slug} />
            </Stack>
          </Box>
        </SideNavBody>
      </SideNav>
      <Main id="main-content" skipLinkLabel={t("navigation.skip")}>
        <Outlet />
      </Main>
    </Root>
  );
}

function AccountActions() {
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
  if (!preferences.data) return null;
  return (
    <Inline space="space.050">
      <Button appearance="subtle" onClick={() => update.mutate("locale")}>
        {preferences.data.locale.toUpperCase()}
      </Button>
      <Button appearance="subtle" onClick={() => update.mutate("theme")}>
        {preferences.data.theme}
      </Button>
      <Button appearance="subtle" onClick={() => logout.mutate()}>
        로그아웃
      </Button>
    </Inline>
  );
}
