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
import { Box, Stack, Text } from "@atlaskit/primitives";
import { Outlet, useLocation } from "@tanstack/react-router";

import { useTranslation } from "./product-app-provider";
import { useWorkspaceRealtime } from "../collaboration/workspace-realtime";

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
