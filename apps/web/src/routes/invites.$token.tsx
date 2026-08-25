import { ApiClient, ApiProblemError } from "@adoc/ui-domain";
import { Button } from "../components/product/legacy";
import { InlineMessage } from "../components/product/legacy";
import { useMutation } from "@tanstack/react-query";
import { createFileRoute, redirect } from "@tanstack/react-router";
import { CalendarClock, ShieldCheck, Users } from "lucide-react";

import { BrandMark } from "../components/product/brand-mark";
import { useTranslation } from "../shell/product-app-provider";
import { browserCommand } from "../shell/browser-command";
import { loadInvitationPreview, loadShellBootstrap } from "../shell/server-bootstrap";

export const Route = createFileRoute("/invites/$token")({
  loader: async ({ params }) => {
    const bootstrap = await loadShellBootstrap();
    if (!bootstrap.authenticated) {
      throw redirect({
        to: "/login",
        search: { returnTo: `/invites/${encodeURIComponent(params.token)}` },
      });
    }
    return loadInvitationPreview({ data: { token: params.token } });
  },
  component: InvitationScreen,
});

function InvitationScreen() {
  const t = useTranslation();
  const preview = Route.useLoaderData();
  const { token } = Route.useParams();
  const accept = useMutation({
    mutationFn: () => new ApiClient().acceptInvitation(token, browserCommand()),
    onSuccess: () => window.location.assign(`/w/${encodeURIComponent(preview.workspaceSlug)}/home`),
  });
  const code =
    accept.error instanceof ApiProblemError ? accept.error.problem.code : "INVITATION_INVALID";
  return (
    <main
      id="main-content"
      className="flex min-h-svh items-center justify-center bg-sidebar px-5 py-10"
    >
      <div className="w-full max-w-xl rounded-xl border bg-card p-6 shadow-sm sm:p-8">
        <div className="mb-7 flex items-center gap-3">
          <BrandMark />
          <span className="font-semibold tracking-tight">Adoc</span>
        </div>
        <p className="text-xs font-medium text-primary">WORKSPACE INVITATION</p>
        <h1 className="mt-2">{t("invitation.title")}</h1>
        <p className="mt-2 text-sm leading-6 text-muted-foreground">
          초대 내용을 확인한 뒤 Workspace에 참여하세요.
        </p>
        <dl className="my-7 divide-y rounded-lg border bg-muted/20">
          <div className="flex items-center gap-3 px-4 py-3">
            <Users aria-hidden="true" className="size-4 text-muted-foreground" />
            <dt className="w-24 text-sm text-muted-foreground">Workspace</dt>
            <dd className="min-w-0 truncate text-sm font-medium">{preview.workspaceName}</dd>
          </div>
          <div className="flex items-center gap-3 px-4 py-3">
            <ShieldCheck aria-hidden="true" className="size-4 text-muted-foreground" />
            <dt className="w-24 text-sm text-muted-foreground">역할</dt>
            <dd className="text-sm font-medium">
              {t(preview.role === "ADMIN" ? "invitation.roleAdmin" : "invitation.roleMember")}
            </dd>
          </div>
          <div className="flex items-center gap-3 px-4 py-3">
            <CalendarClock aria-hidden="true" className="size-4 text-muted-foreground" />
            <dt className="w-24 text-sm text-muted-foreground">만료</dt>
            <dd className="text-sm font-medium">{new Date(preview.expiresAt).toLocaleString()}</dd>
          </div>
        </dl>
        <Button
          appearance="primary"
          isLoading={accept.isPending}
          isDisabled={accept.isPending}
          className="h-11 w-full"
          onClick={() => accept.mutate()}
        >
          {t("invitation.accept")}
        </Button>
        {accept.error ? (
          <div className="mt-4">
            <InlineMessage appearance="error" title={t("common.unavailable")}>
              <p>{code}</p>
            </InlineMessage>
          </div>
        ) : null}
      </div>
    </main>
  );
}
