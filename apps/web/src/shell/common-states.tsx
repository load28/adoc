import { Button } from "../components/product/legacy";
import { EmptyState } from "../components/product/legacy";
import { InlineMessage } from "../components/product/legacy";
import { Box, Stack } from "../components/product/legacy";
import { Skeleton } from "../components/product/legacy";

import { useTranslation } from "./product-app-provider";

export function RoutePending() {
  const t = useTranslation();
  return (
    <Box
      as="main"
      id="main-content"
      padding="space.400"
      aria-busy="true"
      aria-label={t("common.loading")}
    >
      <Stack space="space.200">
        <Skeleton width="40%" height="32px" />
        <Skeleton width="100%" height="20px" />
        <Skeleton width="80%" height="20px" />
      </Stack>
    </Box>
  );
}

export function RouteEmpty({ action }: Readonly<{ action?: React.ReactNode }>) {
  const t = useTranslation();
  return (
    <main id="main-content">
      <EmptyState
        header={t("workspace.empty")}
        description={t("route.preparing")}
        headingLevel={1}
        primaryAction={action}
      />
    </main>
  );
}

export function RouteProblem({
  code,
  correlationId,
  onRetry,
}: Readonly<{ code: string; correlationId?: string; onRetry?: () => void }>) {
  const t = useTranslation();
  return (
    <Box as="main" id="main-content" padding="space.400">
      <Stack space="space.200">
        <InlineMessage title={t("common.unavailable")} appearance="error">
          <p>{code}</p>
          {correlationId ? <p>{correlationId}</p> : null}
        </InlineMessage>
        {onRetry ? <Button onClick={onRetry}>{t("common.retry")}</Button> : null}
      </Stack>
    </Box>
  );
}
