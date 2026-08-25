import { beginGoogleLoginUrl, canonicalReturnTo } from "@adoc/ui-domain";
import { createFileRoute, redirect } from "@tanstack/react-router";
import { ArrowRight, CheckCircle2, FileCheck2, MessageSquareText } from "lucide-react";

import { BrandMark } from "../components/product/brand-mark";
import { LinkButton } from "../components/product/legacy";
import { useTranslation } from "../shell/product-app-provider";
import { loadShellBootstrap } from "../shell/server-bootstrap";

export const Route = createFileRoute("/login")({
  validateSearch: (search) =>
    typeof search.returnTo === "string"
      ? { returnTo: canonicalReturnTo(search.returnTo) }
      : { returnTo: undefined },
  loader: async () => {
    const bootstrap = await loadShellBootstrap();
    if (bootstrap.authenticated) throw redirect({ to: "/workspaces" });
  },
  component: LoginScreen,
});

function LoginScreen() {
  const t = useTranslation();
  const returnTo = canonicalReturnTo(Route.useSearch().returnTo);
  return (
    <main
      id="main-content"
      className="grid min-h-svh lg:grid-cols-[minmax(0,1.15fr)_minmax(28rem,0.85fr)]"
    >
      <section className="relative hidden overflow-hidden border-r bg-sidebar px-12 py-10 lg:flex lg:flex-col">
        <div className="flex items-center gap-3">
          <BrandMark />
          <span className="text-base font-semibold tracking-tight">Adoc</span>
        </div>
        <div className="my-auto max-w-xl py-16">
          <p className="mb-4 text-sm font-medium text-primary">팀의 생각이 공식 지식이 되는 곳</p>
          <h1 className="text-[2.5rem] leading-[1.18] tracking-[-0.04em]">
            흩어진 생각을 모아,
            <br />
            신뢰할 수 있는 문서로.
          </h1>
          <p className="mt-6 max-w-lg text-base leading-7 text-muted-foreground">
            초안부터 검토, 발행까지 하나의 흐름에서 협업하고 결정의 맥락을 오래 남기세요.
          </p>
          <ol className="mt-10 grid gap-3" aria-label="문서 작성 흐름">
            {[
              [MessageSquareText, "Draft", "팀의 생각과 근거를 안전하게 모읍니다."],
              [CheckCircle2, "Review", "변경과 토론을 확인하고 사람이 승인합니다."],
              [FileCheck2, "Publish", "검증된 버전을 조직의 공식 지식으로 남깁니다."],
            ].map(([Icon, label, description]) => (
              <li
                key={String(label)}
                className="flex items-center gap-4 rounded-lg border bg-card/65 px-4 py-3"
              >
                <span className="grid size-9 place-items-center rounded-md bg-primary/10 text-primary">
                  <Icon className="size-4" aria-hidden="true" />
                </span>
                <span>
                  <strong className="block text-sm font-semibold">{label as string}</strong>
                  <span className="text-sm text-muted-foreground">{description as string}</span>
                </span>
              </li>
            ))}
          </ol>
        </div>
        <p className="text-xs text-muted-foreground">Draft → Review → Publish</p>
      </section>
      <section className="flex min-h-svh items-center justify-center px-5 py-10 sm:px-8">
        <div className="w-full max-w-[26rem]">
          <div className="mb-10 flex items-center gap-3 lg:hidden">
            <BrandMark />
            <span className="text-base font-semibold tracking-tight">Adoc</span>
          </div>
          <div className="rounded-xl border bg-card p-6 shadow-sm sm:p-8">
            <p className="mb-2 text-xs font-medium text-muted-foreground">TEAM KNOWLEDGE</p>
            <h1>Adoc에 로그인</h1>
            <p className="mt-2 text-sm leading-6 text-muted-foreground">
              {t("auth.loginDescription")}
            </p>
            <LinkButton
              href={beginGoogleLoginUrl(returnTo)}
              appearance="primary"
              className="mt-7 h-11 w-full justify-center text-sm"
            >
              <span
                aria-hidden="true"
                className="grid size-5 place-items-center rounded-full bg-primary-foreground text-xs font-bold text-primary"
              >
                G
              </span>
              {t("auth.login")}
              <ArrowRight aria-hidden="true" className="ml-auto size-4" />
            </LinkButton>
            <p className="mt-5 text-center text-xs leading-5 text-muted-foreground">
              로그인하면 조직이 허용한 Workspace와 문서에만 접근합니다.
            </p>
          </div>
        </div>
      </section>
    </main>
  );
}
