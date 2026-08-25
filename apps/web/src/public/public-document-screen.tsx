import type { DocumentContent } from "@adoc/contracts";
import { ApiClient, publicFileContentUrl } from "@adoc/ui-domain";
import { Stack, Text } from "@atlaskit/primitives";
import { useQuery } from "@tanstack/react-query";

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
    <main className="public-document">
      <article>
        <Stack space="space.250">
          <header>
            <h1>{query.data.title}</h1>
            <Text>
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
    <main className="public-document">
      <Stack space="space.150">
        <h1>문서를 찾을 수 없습니다</h1>
        <Text>링크가 만료됐거나 더 이상 공개되지 않는 문서입니다.</Text>
      </Stack>
    </main>
  );
}
