import type { DocumentContent } from "@adoc/contracts";
import { ApiClient, publicFileContentUrl } from "@adoc/ui-domain";
import { Stack, Text } from "../components/product/legacy";
import { useQuery } from "@tanstack/react-query";

import { BrandMark } from "../components/product/brand-mark";
import { RoutePending } from "../shell/common-states";
import { ContentRenderer } from "../document/content-renderer";
import "./public-document.css";

const api = new ApiClient();

export function PublicDocumentScreen({ token }: Readonly<{ token: string }>) {
  const query = useQuery({
    queryKey: ["public-document", token],
    queryFn: ({ signal }) => api.publicDocument(token, signal),
    retry: false,
  });
  if (query.isPending) return <RoutePending />;
  if (query.error) return <PublicNotFound />;
  return (
    <main id="main-content" className="public-document">
      <article>
        <Stack space="space.250">
          <header>
            <div className="mb-8 flex items-center gap-2.5 text-sm font-semibold tracking-tight">
              <BrandMark className="size-7 rounded-md text-xs" />
              Adoc
            </div>
            <p className="mb-2 text-xs font-medium text-muted-foreground">PUBLISHED DOCUMENT</p>
            <h1>{query.data.title}</h1>
            <Text className="mt-2 block">
              v{query.data.versionNumber} · {new Date(query.data.publishedAt).toLocaleString()}
            </Text>
          </header>
          <ContentRenderer
            content={query.data.content as DocumentContent}
            assetUrl={(assetId) => publicFileContentUrl(token, assetId)}
          />
        </Stack>
      </article>
    </main>
  );
}

function PublicNotFound() {
  return (
    <main id="main-content" className="public-document">
      <Stack space="space.150" className="min-h-[70svh] justify-center">
        <BrandMark />
        <h1>문서를 찾을 수 없습니다</h1>
        <Text>링크가 만료됐거나 더 이상 공개되지 않는 문서입니다.</Text>
      </Stack>
    </main>
  );
}
