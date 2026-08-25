import { ApiClient, ApiProblemError, type DocumentPage } from "@adoc/ui-domain";
import { Button } from "../components/product/legacy";
import { Lozenge } from "../components/product/legacy";
import { Inline, Stack, Text } from "../components/product/legacy";
import { TextArea } from "../components/product/legacy";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";

import { PageHeader } from "../components/product/page";
import { RoutePending, RouteProblem } from "../shell/common-states";
import "./settings-audit.css";

const api = new ApiClient();

export function TrashScreen({ workspaceId }: Readonly<{ workspaceId: string }>) {
  const query = useQuery({
    queryKey: ["trash", workspaceId],
    queryFn: ({ signal }) => api.trashedDocuments(workspaceId, undefined, signal),
  });
  if (query.isPending) return <RoutePending />;
  if (query.error) return <TrashProblem error={query.error} retry={() => void query.refetch()} />;
  return (
    <main className="settings-screen">
      <Stack space="space.250">
        <PageHeader
          eyebrow="RETENTION · 30 DAYS"
          title="휴지통"
          description="복원은 문서를 Workspace root로 되돌립니다. 영구 삭제는 비동기 purge이며 복구할 수 없습니다."
        />
        <TrashList workspaceId={workspaceId} page={query.data} />
      </Stack>
    </main>
  );
}

function TrashList({ workspaceId, page }: Readonly<{ workspaceId: string; page: DocumentPage }>) {
  if (page.items.length === 0) return <Text>휴지통이 비어 있습니다.</Text>;
  return (
    <ul className="settings-list">
      {page.items.map((document) => (
        <TrashRow key={document.id} workspaceId={workspaceId} document={document} />
      ))}
    </ul>
  );
}

function TrashRow({
  workspaceId,
  document,
}: Readonly<{ workspaceId: string; document: DocumentPage["items"][number] }>) {
  const client = useQueryClient();
  const [reason, setReason] = useState("");
  const restore = useMutation({
    mutationFn: () => api.restoreDocument(workspaceId, document.id, document.revision, command()),
    onSuccess: async () => client.invalidateQueries({ queryKey: ["trash", workspaceId] }),
  });
  const purge = useMutation({
    mutationFn: () => {
      if (!reason.trim()) throw new Error("영구 삭제 사유를 입력해 주세요.");
      return api.purgeDocument(
        workspaceId,
        document.id,
        document.revision,
        reason.trim(),
        command(),
      );
    },
    onSuccess: async () => client.invalidateQueries({ queryKey: ["trash", workspaceId] }),
  });
  return (
    <li>
      <Stack space="space.100">
        <Inline space="space.100">
          <Text weight="semibold">{document.title}</Text>
          <Lozenge>{document.status}</Lozenge>
        </Inline>
        <label htmlFor={`purge-reason-${document.id}`}>영구 삭제 사유</label>
        <TextArea
          id={`purge-reason-${document.id}`}
          value={reason}
          onChange={(event) => setReason(event.currentTarget.value)}
        />
        <MutationMessage mutation={restore} />
        <MutationMessage mutation={purge} />
      </Stack>
      <Inline space="space.050" shouldWrap>
        <Button appearance="primary" onClick={() => restore.mutate()} isLoading={restore.isPending}>
          복원
        </Button>
        <Button
          appearance="danger"
          isDisabled={!reason.trim()}
          onClick={() => purge.mutate()}
          isLoading={purge.isPending}
        >
          영구 삭제
        </Button>
      </Inline>
    </li>
  );
}

function TrashProblem({ error, retry }: Readonly<{ error: Error; retry: () => void }>) {
  const problem = error instanceof ApiProblemError ? error.problem : { code: "TRASH_QUERY_FAILED" };
  return (
    <RouteProblem
      code={problem.code}
      correlationId={"correlationId" in problem ? problem.correlationId : undefined}
      onRetry={retry}
    />
  );
}
function MutationMessage({
  mutation,
}: Readonly<{ mutation: { isError: boolean; isSuccess: boolean; error: Error | null } }>) {
  if (mutation.isError) return <div role="alert">{mutation.error?.message}</div>;
  if (mutation.isSuccess) return <div role="status">요청을 처리했습니다.</div>;
  return null;
}
function command() {
  const value = document.cookie
    .split("; ")
    .find((item) => item.startsWith("adoc_csrf="))
    ?.slice("adoc_csrf=".length);
  if (!value) throw new Error("CSRF token is unavailable");
  return { csrfToken: decodeURIComponent(value), idempotencyKey: crypto.randomUUID() };
}
