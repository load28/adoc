import { createFileRoute } from "@tanstack/react-router";
import { BookOpen, FilePlus2, MessageSquareText, Search } from "lucide-react";

import { PageFrame, PageHeader, SectionHeader } from "../components/product/page";
import { useTranslation } from "../shell/product-app-provider";

export const Route = createFileRoute("/w/$workspaceSlug/home")({
  component: WorkspaceHome,
});

function WorkspaceHome() {
  const t = useTranslation();
  return (
    <main id="main-content">
      <PageFrame>
        <PageHeader
          eyebrow="WORKSPACE OVERVIEW"
          title={t("navigation.home")}
          description="팀의 문서, 검토와 지식 작업을 한 곳에서 이어가세요."
        />
        <section>
          <SectionHeader
            title="문서 작업 시작하기"
            description="왼쪽 문서 영역에서 새 문서를 만들거나 기존 문서를 선택하세요."
          />
          <div className="grid gap-4 md:grid-cols-3">
            {[
              [
                FilePlus2,
                t("workspace.createDocument"),
                "정리되지 않은 생각에서 첫 초안을 시작합니다.",
              ],
              [MessageSquareText, t("navigation.inbox"), "검토 요청과 언급된 토론을 확인합니다."],
              [Search, t("navigation.search"), "발행된 지식과 근거를 Workspace 안에서 찾습니다."],
            ].map(([Icon, title, description]) => (
              <article key={String(title)} className="rounded-lg border bg-card p-5">
                <span className="grid size-9 place-items-center rounded-md bg-primary/10 text-primary">
                  <Icon aria-hidden="true" className="size-4" />
                </span>
                <h2 className="mt-4 text-[15px]">{title as string}</h2>
                <p className="mt-1.5 text-sm leading-6 text-muted-foreground">
                  {description as string}
                </p>
              </article>
            ))}
          </div>
        </section>
        <section className="mt-10 rounded-lg border border-dashed bg-muted/20 px-6 py-10 text-center">
          <BookOpen aria-hidden="true" className="mx-auto size-6 text-muted-foreground" />
          <h2 className="mt-3">{t("workspace.documents")}</h2>
          <p className="mx-auto mt-1 max-w-md text-sm leading-6 text-muted-foreground">
            문서 트리에서 문서를 선택하면 최신 발행 버전, 초안과 협업 맥락을 함께 볼 수 있습니다.
          </p>
        </section>
      </PageFrame>
    </main>
  );
}
