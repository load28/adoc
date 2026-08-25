import { ApiClient, ApiProblemError, type WorkspaceView } from "@adoc/ui-domain";
import { Button } from "../components/product/legacy";
import { EmptyState } from "../components/product/legacy";
import { InlineMessage } from "../components/product/legacy";
import { Textfield } from "../components/product/legacy";
import { useMutation } from "@tanstack/react-query";
import { createFileRoute, redirect } from "@tanstack/react-router";
import { ArrowRight, Building2, Plus, Settings2 } from "lucide-react";
import { useState } from "react";

import { BrandMark } from "../components/product/brand-mark";
import { PageFrame, PageHeader, SectionHeader } from "../components/product/page";
import { useTranslation } from "../shell/product-app-provider";
import { loadShellBootstrap, loadWorkspaceList } from "../shell/server-bootstrap";
import { LinkButton } from "../components/product/legacy";
import { browserCommand } from "../shell/browser-command";

export const Route = createFileRoute("/workspaces")({
  loader: async () => {
    const bootstrap = await loadShellBootstrap();
    if (!bootstrap.authenticated || !bootstrap.session)
      throw redirect({ to: "/login", search: { returnTo: "/workspaces" } });
    return loadWorkspaceList();
  },
  component: WorkspaceList,
});

function WorkspaceList() {
  const workspaces = Route.useLoaderData();
  const t = useTranslation();
  const [name, setName] = useState("");
  const create = useMutation({
    mutationFn: () => new ApiClient().createWorkspace(name.trim(), browserCommand()),
    onSuccess: (workspace) =>
      window.location.assign(`/w/${encodeURIComponent(workspace.slug)}/home`),
  });
  const error = create.error instanceof ApiProblemError ? create.error.problem.code : undefined;
  return (
    <main id="main-content" className="min-h-svh bg-background">
      <header className="flex h-14 items-center border-b px-5 sm:px-8">
        <div className="flex items-center gap-3">
          <BrandMark className="size-7 rounded-md text-xs" />
          <span className="font-semibold tracking-tight">Adoc</span>
        </div>
      </header>
      <PageFrame className="max-w-6xl">
        <PageHeader
          eyebrow="YOUR ORGANIZATION"
          title={t("workspace.list")}
          description="팀의 문서와 권한이 분리되는 작업 공간을 선택하세요."
        />
        <div className="grid items-start gap-8 lg:grid-cols-[minmax(0,1fr)_22rem]">
          <section>
            <SectionHeader
              title="참여 중인 Workspace"
              description={`${workspaces.length}개의 Workspace에 참여하고 있습니다.`}
            />
            {workspaces.length === 0 ? (
              <EmptyState
                header={t("workspace.empty")}
                description="오른쪽 양식에서 첫 Workspace를 만들 수 있습니다."
                headingLevel={2}
              />
            ) : (
              <ul className="grid list-none gap-3 p-0">
                {workspaces.map((workspace) => (
                  <WorkspaceRow key={workspace.id} workspace={workspace} />
                ))}
              </ul>
            )}
          </section>
          <section className="rounded-xl border bg-card p-5 shadow-xs lg:sticky lg:top-8">
            <div className="mb-5 flex size-10 items-center justify-center rounded-lg bg-primary/10 text-primary">
              <Plus aria-hidden="true" className="size-5" />
            </div>
            <h2>{t("workspace.create")}</h2>
            <p className="mt-1 text-sm leading-6 text-muted-foreground">
              문서, 멤버와 권한이 독립된 새 작업 공간을 만듭니다.
            </p>
            <form
              className="mt-5 grid gap-4"
              onSubmit={(event) => {
                event.preventDefault();
                if (name.trim().length > 0 && name.trim().length <= 200) create.mutate();
              }}
            >
              <div className="grid gap-2">
                <label htmlFor="workspace-name">{t("workspace.name")}</label>
                <Textfield
                  id="workspace-name"
                  value={name}
                  maxLength={200}
                  placeholder="예: 제품 개발팀"
                  onChange={(event) => setName(event.currentTarget.value)}
                />
              </div>
              <Button
                type="submit"
                appearance="primary"
                isLoading={create.isPending}
                isDisabled={create.isPending || name.trim().length === 0}
                className="w-full"
              >
                {t("workspace.create")}
              </Button>
              {error ? (
                <InlineMessage appearance="error" title={t("common.unavailable")}>
                  <p>{error}</p>
                </InlineMessage>
              ) : null}
            </form>
          </section>
        </div>
      </PageFrame>
    </main>
  );
}

function WorkspaceRow({ workspace }: Readonly<{ workspace: WorkspaceView }>) {
  const [name, setName] = useState(workspace.name);
  const [reason, setReason] = useState("");
  const update = useMutation({
    mutationFn: () =>
      new ApiClient().updateWorkspace(workspace, { name: name.trim() }, browserCommand()),
    onSuccess: () => window.location.reload(),
  });
  const schedule = useMutation({
    mutationFn: () =>
      new ApiClient().scheduleWorkspaceDeletion(workspace, reason.trim(), browserCommand()),
    onSuccess: () => window.location.reload(),
  });
  const cancel = useMutation({
    mutationFn: () => new ApiClient().cancelWorkspaceDeletion(workspace, browserCommand()),
    onSuccess: () => window.location.reload(),
  });
  return (
    <li className="overflow-hidden rounded-lg border bg-card transition-colors hover:border-ring/35">
      <div className="flex items-center gap-4 p-4">
        <span className="grid size-10 shrink-0 place-items-center rounded-lg bg-secondary text-secondary-foreground">
          <Building2 aria-hidden="true" className="size-5" />
        </span>
        <div className="min-w-0 flex-1">
          <LinkButton
            href={`/w/${workspace.slug}/home`}
            appearance="subtle"
            className="h-auto max-w-full justify-start p-0 text-[15px] font-semibold hover:bg-transparent"
          >
            <span className="truncate">{workspace.name}</span>
            <ArrowRight aria-hidden="true" className="size-4 text-muted-foreground" />
          </LinkButton>
          <p className="mt-1 text-[13px] text-muted-foreground">
            {workspace.status === "DELETION_SCHEDULED" ? "삭제 예약됨" : "활성 Workspace"}
          </p>
        </div>
        <details className="group relative">
          <summary className="grid size-9 cursor-pointer list-none place-items-center rounded-md text-muted-foreground outline-none hover:bg-accent focus-visible:ring-2 focus-visible:ring-ring [&::-webkit-details-marker]:hidden">
            <Settings2 aria-hidden="true" className="size-4" />
            <span className="sr-only">{workspace.name} 관리</span>
          </summary>
          <div className="mt-3 grid gap-4 border-t pt-4 sm:min-w-[28rem]">
            <div className="grid gap-2">
              <label htmlFor={`workspace-name-${workspace.id}`}>Workspace 이름</label>
              <div className="flex gap-2">
                <Textfield
                  id={`workspace-name-${workspace.id}`}
                  value={name}
                  maxLength={200}
                  onChange={(event) => setName(event.currentTarget.value)}
                />
                <Button
                  onClick={() => update.mutate()}
                  isLoading={update.isPending}
                  isDisabled={!name.trim() || name.trim() === workspace.name}
                >
                  이름 저장
                </Button>
              </div>
            </div>
            <div className="rounded-lg border border-destructive/20 bg-destructive/5 p-4">
              <p className="text-sm font-semibold text-destructive">Danger zone</p>
              {workspace.status === "DELETION_SCHEDULED" ? (
                <Button
                  className="mt-3"
                  onClick={() => cancel.mutate()}
                  isLoading={cancel.isPending}
                >
                  삭제 예약 취소
                </Button>
              ) : (
                <div className="mt-3 grid gap-2 sm:grid-cols-[1fr_auto]">
                  <Textfield
                    aria-label={`${workspace.name} 삭제 사유`}
                    placeholder="삭제 예약 사유"
                    value={reason}
                    onChange={(event) => setReason(event.currentTarget.value)}
                  />
                  <Button
                    appearance="danger"
                    onClick={() => schedule.mutate()}
                    isLoading={schedule.isPending}
                    isDisabled={!reason.trim()}
                  >
                    삭제 예약
                  </Button>
                </div>
              )}
            </div>
          </div>
        </details>
      </div>
    </li>
  );
}
