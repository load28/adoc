import type { DocumentContent } from "@adoc/contracts";
import {
  ApiClient,
  ApiProblemError,
  type DiscussionDetail,
  type DiscussionPage,
  type InboxPage,
  type ReferencePage,
  type ReviewView,
  type SearchPage,
  type VersionPage,
  type VocabularyPage,
} from "@adoc/ui-domain";
import Button from "@atlaskit/button/default/button";
import LinkButton from "@atlaskit/button/link";
import Lozenge from "@atlaskit/lozenge";
import { Box, Inline, Stack, Text } from "@atlaskit/primitives";
import TextArea from "@atlaskit/textarea";
import Textfield from "@atlaskit/textfield";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";

import { RoutePending, RouteProblem } from "../shell/common-states";
import "./collaboration-knowledge.css";

const api = new ApiClient();

export function DocumentCollaborationPanel({
  workspaceId,
  workspaceSlug,
  documentId,
  panel,
  discussionId,
  reviewId,
}: Readonly<{
  workspaceId: string;
  workspaceSlug: string;
  documentId: string;
  panel: "discussion" | "review" | "history" | "references";
  discussionId?: string;
  reviewId?: string;
}>) {
  const base = `/w/${encodeURIComponent(workspaceSlug)}/docs/${encodeURIComponent(documentId)}`;
  return (
    <aside className="collaboration-panel" aria-label="문서 협업">
      <Inline space="space.050" shouldWrap>
        {(["discussion", "review", "history", "references"] as const).map((name) => (
          <LinkButton
            key={name}
            href={`${base}?mode=published&panel=${name}`}
            appearance={panel === name ? "primary" : "subtle"}
          >
            {panelLabel(name)}
          </LinkButton>
        ))}
      </Inline>
      <Box paddingBlockStart="space.200">
        {panel === "discussion" && (
          <DiscussionPanel
            workspaceId={workspaceId}
            workspaceSlug={workspaceSlug}
            documentId={documentId}
            discussionId={discussionId}
          />
        )}
        {panel === "review" && <ReviewPanel workspaceId={workspaceId} reviewId={reviewId} />}
        {panel === "history" && <HistoryPanel workspaceId={workspaceId} documentId={documentId} />}
        {panel === "references" && (
          <ReferencePanel workspaceId={workspaceId} documentId={documentId} />
        )}
      </Box>
    </aside>
  );
}

function DiscussionPanel({
  workspaceId,
  workspaceSlug,
  documentId,
  discussionId,
}: Readonly<{
  workspaceId: string;
  workspaceSlug: string;
  documentId: string;
  discussionId?: string;
}>) {
  const client = useQueryClient();
  const listKey = ["discussion", workspaceId, documentId] as const;
  const list = useQuery({
    queryKey: listKey,
    queryFn: ({ signal }) => api.discussions(workspaceId, documentId, undefined, signal),
  });
  const detail = useQuery({
    queryKey: ["discussion-detail", workspaceId, discussionId],
    queryFn: ({ signal }) => api.discussion(workspaceId, discussionId ?? "", undefined, signal),
    enabled: Boolean(discussionId),
  });
  const [title, setTitle] = useState("");
  const [message, setMessage] = useState("");
  const create = useMutation({
    mutationFn: () =>
      api.createDiscussion(
        workspaceId,
        documentId,
        {
          title,
          message: richMessage(message),
          topics: [{ kind: "TEXT", label: title, text: message }],
        },
        command(),
      ),
    onSuccess: async () => {
      setTitle("");
      setMessage("");
      await client.invalidateQueries({ queryKey: listKey });
    },
  });
  if (list.isPending) return <RoutePending />;
  if (list.error) return <QueryError error={list.error} retry={() => void list.refetch()} />;
  return (
    <Stack space="space.200">
      <h2>토론</h2>
      <DiscussionList page={list.data} workspaceSlug={workspaceSlug} documentId={documentId} />
      {discussionId && detail.data && (
        <DiscussionDetailView workspaceId={workspaceId} detail={detail.data} />
      )}
      <form
        className="collaboration-form"
        onSubmit={(event) => {
          event.preventDefault();
          if (title.trim() && message.trim()) create.mutate();
        }}
      >
        <label htmlFor="discussion-title">제목</label>
        <Textfield
          id="discussion-title"
          value={title}
          onChange={(event) => setTitle(event.currentTarget.value)}
        />
        <label htmlFor="discussion-message">첫 메시지</label>
        <TextArea
          id="discussion-message"
          value={message}
          onChange={(event) => setMessage(event.currentTarget.value)}
        />
        <Button type="submit" appearance="primary" isLoading={create.isPending}>
          토론 만들기
        </Button>
        <MutationStatus mutation={create} />
      </form>
    </Stack>
  );
}

function DiscussionList({
  page,
  workspaceSlug,
  documentId,
}: Readonly<{ page: DiscussionPage; workspaceSlug: string; documentId: string }>) {
  if (page.items.length === 0) return <Text>아직 토론이 없습니다.</Text>;
  return (
    <ul className="resource-list">
      {page.items.map((item) => (
        <li key={item.id}>
          <LinkButton
            appearance="subtle"
            href={`/w/${encodeURIComponent(workspaceSlug)}/docs/${encodeURIComponent(documentId)}?mode=published&panel=discussion&discussion=${encodeURIComponent(item.id)}`}
          >
            {item.title}
          </LinkButton>
          <Lozenge appearance={item.status === "OPEN" ? "inprogress" : "default"}>
            {item.status}
          </Lozenge>
        </li>
      ))}
    </ul>
  );
}

function DiscussionDetailView({
  workspaceId,
  detail,
}: Readonly<{ workspaceId: string; detail: DiscussionDetail }>) {
  const queryClient = useQueryClient();
  const [message, setMessage] = useState("");
  const [reason, setReason] = useState("");
  const detailKey = ["discussion-detail", workspaceId, detail.discussion.id] as const;
  const send = useMutation({
    mutationFn: () =>
      api.createMessage(workspaceId, detail.discussion.id, richMessage(message), command()),
    onSuccess: async () => {
      setMessage("");
      await queryClient.invalidateQueries({ queryKey: detailKey });
    },
  });
  const status = useMutation({
    mutationFn: () =>
      api.changeDiscussionStatus(
        workspaceId,
        detail.discussion.id,
        detail.discussion.revision,
        detail.discussion.status === "OPEN" ? "close" : "reopen",
        reason,
        command(),
      ),
    onSuccess: async () => {
      setReason("");
      await queryClient.invalidateQueries({ queryKey: ["discussion", workspaceId] });
      await queryClient.invalidateQueries({ queryKey: detailKey });
    },
  });
  return (
    <section aria-labelledby="discussion-detail-title" className="discussion-detail">
      <h3 id="discussion-detail-title">{detail.discussion.title}</h3>
      <ul className="message-list">
        {detail.messages.map((item) => (
          <li key={item.id}>
            <Text>{item.deletedAt ? "삭제된 메시지" : contentText(item.body)}</Text>
            <Text size="small">작성자 {item.authorId}</Text>
          </li>
        ))}
      </ul>
      {detail.discussion.status === "OPEN" && (
        <form
          className="collaboration-form"
          onSubmit={(event) => {
            event.preventDefault();
            if (message.trim()) send.mutate();
          }}
        >
          <label htmlFor="reply-message">답글</label>
          <TextArea
            id="reply-message"
            value={message}
            onChange={(event) => setMessage(event.currentTarget.value)}
          />
          <Button type="submit" appearance="primary" isLoading={send.isPending}>
            보내기
          </Button>
          <MutationStatus mutation={send} />
        </form>
      )}
      <form
        className="collaboration-form"
        onSubmit={(event) => {
          event.preventDefault();
          if (reason.trim()) status.mutate();
        }}
      >
        <label htmlFor="discussion-reason">
          {detail.discussion.status === "OPEN" ? "종료 사유" : "다시 여는 이유"}
        </label>
        <Textfield
          id="discussion-reason"
          value={reason}
          onChange={(event) => setReason(event.currentTarget.value)}
        />
        <Button type="submit" isLoading={status.isPending}>
          {detail.discussion.status === "OPEN" ? "토론 닫기" : "토론 다시 열기"}
        </Button>
        <MutationStatus mutation={status} />
      </form>
    </section>
  );
}

function ReviewPanel({
  workspaceId,
  reviewId,
}: Readonly<{ workspaceId: string; reviewId?: string }>) {
  const review = useQuery({
    queryKey: ["review", workspaceId, reviewId],
    queryFn: ({ signal }) => api.review(workspaceId, reviewId ?? "", signal),
    enabled: Boolean(reviewId),
  });
  if (!reviewId)
    return <Text>검토 링크를 선택하면 정확한 초안 revision의 결정을 확인할 수 있습니다.</Text>;
  if (review.isPending) return <RoutePending />;
  if (review.error) return <QueryError error={review.error} retry={() => void review.refetch()} />;
  return <ReviewDetail workspaceId={workspaceId} review={review.data} />;
}

function ReviewDetail({
  workspaceId,
  review,
}: Readonly<{ workspaceId: string; review: ReviewView }>) {
  const queryClient = useQueryClient();
  const approve = useMutation({
    mutationFn: () =>
      api.submitReviewDecision(
        workspaceId,
        review.id,
        review.revision,
        { decision: "APPROVE", discussionId: null },
        command(),
      ),
    onSuccess: async () => queryClient.invalidateQueries({ queryKey: ["review", workspaceId] }),
  });
  return (
    <Stack space="space.150">
      <h2>검토</h2>
      <Inline space="space.100">
        <Lozenge appearance={review.status === "APPROVED" ? "success" : "inprogress"}>
          {review.status}
        </Lozenge>
        <Text>초안 revision {review.draftRevision}</Text>
      </Inline>
      {review.policyOutdated && <div role="alert">요청 뒤 발행 정책이 변경됐습니다.</div>}
      <ul className="resource-list">
        {review.assignments.map((assignment) => (
          <li key={assignment.reviewerId}>
            <Text>{assignment.reviewerId}</Text>
            <Lozenge>{assignment.decision}</Lozenge>
          </li>
        ))}
      </ul>
      {review.status === "REQUESTED" && (
        <Button appearance="primary" onClick={() => approve.mutate()} isLoading={approve.isPending}>
          이 revision 승인
        </Button>
      )}
      <MutationStatus mutation={approve} />
    </Stack>
  );
}

function HistoryPanel({
  workspaceId,
  documentId,
}: Readonly<{ workspaceId: string; documentId: string }>) {
  const query = useQuery({
    queryKey: ["versions", workspaceId, documentId],
    queryFn: ({ signal }) => api.versions(workspaceId, documentId, undefined, signal),
  });
  return (
    <PagedResource
      title="버전 기록"
      query={query}
      render={(page: VersionPage) =>
        page.items.map((item) => ({
          id: item.id,
          title: `v${item.number} · ${item.summary}`,
          meta: item.publishedAt,
        }))
      }
    />
  );
}

function ReferencePanel({
  workspaceId,
  documentId,
}: Readonly<{ workspaceId: string; documentId: string }>) {
  const query = useQuery({
    queryKey: ["backlinks", workspaceId, documentId],
    queryFn: ({ signal }) => api.backlinks(workspaceId, documentId, undefined, signal),
  });
  return (
    <PagedResource
      title="이 문서를 참조하는 곳"
      query={query}
      render={(page: ReferencePage) =>
        page.items.map((item) => ({
          id: item.id,
          title: item.snapshot.title || "접근할 수 없는 참조",
          meta: item.createdAt,
        }))
      }
    />
  );
}

export function InboxScreen({ workspaceId }: Readonly<{ workspaceId: string }>) {
  const [status, setStatus] = useState<"UNREAD" | "ACTIONABLE" | "RESOLVED" | "ALL">("ACTIONABLE");
  const key = ["inbox", workspaceId, status] as const;
  const client = useQueryClient();
  const query = useQuery({
    queryKey: key,
    queryFn: ({ signal }) => api.inbox(workspaceId, status, undefined, signal),
  });
  const update = useMutation({
    mutationFn: ({ id, action }: { id: string; action: "read" | "resolve" }) =>
      api.updateInboxItem(workspaceId, id, action, command()),
    onSuccess: async () => client.invalidateQueries({ queryKey: ["inbox", workspaceId] }),
  });
  return (
    <Screen title="받은 편지함">
      <Inline space="space.050" shouldWrap>
        {(["ACTIONABLE", "UNREAD", "RESOLVED", "ALL"] as const).map((value) => (
          <Button
            key={value}
            appearance={status === value ? "primary" : "subtle"}
            onClick={() => setStatus(value)}
          >
            {value}
          </Button>
        ))}
      </Inline>
      <InboxList query={query} update={update} />
    </Screen>
  );
}

function InboxList({
  query,
  update,
}: Readonly<{
  query: ReturnType<typeof useQuery<InboxPage>>;
  update: ReturnType<
    typeof useMutation<unknown, Error, { id: string; action: "read" | "resolve" }>
  >;
}>) {
  if (query.isPending) return <RoutePending />;
  if (query.error) return <QueryError error={query.error} retry={() => void query.refetch()} />;
  if (query.data.items.length === 0) return <Text>처리할 항목이 없습니다.</Text>;
  return (
    <ul className="resource-list">
      {query.data.items.map((item) => (
        <li key={item.id}>
          <Stack space="space.050">
            <Text weight="semibold">{item.kind}</Text>
            <Text>
              {item.target.kind} · {item.target.id}
            </Text>
            <Inline space="space.050">
              {!item.readAt && (
                <Button onClick={() => update.mutate({ id: item.id, action: "read" })}>읽음</Button>
              )}
              {!item.resolvedAt && (
                <Button
                  appearance="primary"
                  onClick={() => update.mutate({ id: item.id, action: "resolve" })}
                >
                  해결
                </Button>
              )}
            </Inline>
          </Stack>
        </li>
      ))}
    </ul>
  );
}

export function SearchScreen({
  workspaceId,
  initialQuery = "",
}: Readonly<{ workspaceId: string; initialQuery?: string }>) {
  const [input, setInput] = useState(initialQuery);
  const [submitted, setSubmitted] = useState(initialQuery);
  const query = useQuery({
    queryKey: ["search", workspaceId, submitted],
    queryFn: ({ signal }) => api.search(workspaceId, submitted, undefined, signal),
    enabled: submitted.length > 0,
  });
  return (
    <Screen title="검색">
      <form
        className="search-form"
        onSubmit={(event) => {
          event.preventDefault();
          setSubmitted(input.trim());
        }}
      >
        <Textfield
          aria-label="검색어"
          value={input}
          onChange={(event) => setInput(event.currentTarget.value)}
        />
        <Button type="submit" appearance="primary">
          검색
        </Button>
      </form>
      {submitted && <SearchResults query={query} />}
    </Screen>
  );
}

function SearchResults({ query }: Readonly<{ query: ReturnType<typeof useQuery<SearchPage>> }>) {
  if (query.isPending) return <RoutePending />;
  if (query.error) return <QueryError error={query.error} retry={() => void query.refetch()} />;
  if (query.data.items.length === 0) return <Text>검색 결과가 없습니다.</Text>;
  return (
    <ul className="resource-list">
      {query.data.items.map(({ source }) => (
        <li key={source.stableId}>
          <Stack space="space.050">
            <Text weight="semibold">{source.displaySnapshot.title}</Text>
            <Text>{source.displaySnapshot.excerpt}</Text>
            <Inline space="space.100">
              <Lozenge appearance={source.authority === "OFFICIAL" ? "success" : "inprogress"}>
                {source.authority}
              </Lozenge>
              <Text size="small">{source.kind}</Text>
            </Inline>
          </Stack>
        </li>
      ))}
    </ul>
  );
}

export function VocabularyScreen({ workspaceId }: Readonly<{ workspaceId: string }>) {
  const key = ["vocabulary", workspaceId] as const;
  const queryClient = useQueryClient();
  const query = useQuery({
    queryKey: key,
    queryFn: ({ signal }) => api.vocabulary(workspaceId, undefined, signal),
  });
  const [term, setTerm] = useState("");
  const [definition, setDefinition] = useState("");
  const write = useMutation({
    mutationFn: () =>
      api.writeVocabulary(
        workspaceId,
        { canonicalTerm: term, definition, terms: [{ term, kind: "CANONICAL" }] },
        command(),
      ),
    onSuccess: async () => {
      setTerm("");
      setDefinition("");
      await queryClient.invalidateQueries({ queryKey: key });
    },
  });
  return (
    <Screen title="용어집">
      <VocabularyList query={query} />
      <form
        className="collaboration-form"
        onSubmit={(event) => {
          event.preventDefault();
          if (term.trim() && definition.trim()) write.mutate();
        }}
      >
        <label htmlFor="canonical-term">표준 용어</label>
        <Textfield
          id="canonical-term"
          value={term}
          onChange={(event) => setTerm(event.currentTarget.value)}
        />
        <label htmlFor="concept-definition">정의</label>
        <TextArea
          id="concept-definition"
          value={definition}
          onChange={(event) => setDefinition(event.currentTarget.value)}
        />
        <Button type="submit" appearance="primary" isLoading={write.isPending}>
          개념 추가
        </Button>
        <MutationStatus mutation={write} />
      </form>
    </Screen>
  );
}

function VocabularyList({
  query,
}: Readonly<{ query: ReturnType<typeof useQuery<VocabularyPage>> }>) {
  if (query.isPending) return <RoutePending />;
  if (query.error) return <QueryError error={query.error} retry={() => void query.refetch()} />;
  return (
    <ul className="resource-list">
      {query.data.items.map((item) => (
        <li key={item.id}>
          <Stack space="space.050">
            <Inline space="space.100">
              <Text weight="semibold">{item.canonicalTerm}</Text>
              <Lozenge appearance={item.status === "ACTIVE" ? "success" : "removed"}>
                {item.status}
              </Lozenge>
            </Inline>
            <Text>{item.definition}</Text>
            <Text size="small">{item.terms.map((term) => term.term).join(" · ")}</Text>
          </Stack>
        </li>
      ))}
    </ul>
  );
}

function Screen({ title, children }: Readonly<{ title: string; children: React.ReactNode }>) {
  return (
    <main className="resource-screen">
      <Stack space="space.250">
        <h1>{title}</h1>
        {children}
      </Stack>
    </main>
  );
}

function PagedResource<T>({
  title,
  query,
  render,
}: Readonly<{
  title: string;
  query: ReturnType<typeof useQuery<T>>;
  render: (page: T) => { id: string; title: string; meta: string }[];
}>) {
  if (query.isPending) return <RoutePending />;
  if (query.error) return <QueryError error={query.error} retry={() => void query.refetch()} />;
  const items = render(query.data);
  return (
    <Stack space="space.150">
      <h2>{title}</h2>
      {items.length === 0 ? (
        <Text>항목이 없습니다.</Text>
      ) : (
        <ul className="resource-list">
          {items.map((item) => (
            <li key={item.id}>
              <Text weight="semibold">{item.title}</Text>
              <Text size="small">{item.meta}</Text>
            </li>
          ))}
        </ul>
      )}
    </Stack>
  );
}

function QueryError({ error, retry }: Readonly<{ error: Error; retry: () => void }>) {
  const problem =
    error instanceof ApiProblemError
      ? error.problem
      : { code: "QUERY_FAILED", message: error.message };
  return (
    <RouteProblem
      code={problem.code}
      correlationId={"correlationId" in problem ? problem.correlationId : undefined}
      onRetry={retry}
    />
  );
}
function MutationStatus({
  mutation,
}: Readonly<{ mutation: { isError: boolean; isSuccess: boolean; error: Error | null } }>) {
  if (mutation.isError) return <div role="alert">{mutation.error?.message}</div>;
  if (mutation.isSuccess) return <div role="status">저장했습니다.</div>;
  return null;
}
function panelLabel(panel: string) {
  return (
    (
      { discussion: "토론", review: "검토", history: "버전", references: "참조" } as Record<
        string,
        string
      >
    )[panel] ?? panel
  );
}
function command() {
  const csrfToken = document.cookie
    .split("; ")
    .find((value) => value.startsWith("adoc_csrf="))
    ?.slice("adoc_csrf=".length);
  if (!csrfToken) throw new Error("CSRF token is unavailable");
  return { csrfToken: decodeURIComponent(csrfToken), idempotencyKey: crypto.randomUUID() };
}
function richMessage(text: string) {
  return {
    body: {
      schemaVersion: 1,
      root: {
        type: "doc" as const,
        children: [
          {
            id: crypto.randomUUID(),
            type: "paragraph" as const,
            children: [{ type: "text" as const, text }],
          },
        ],
      },
    } as DocumentContent,
    mentionUserIds: [],
    attachmentIds: [],
  };
}
function contentText(content: unknown): string {
  const visit = (value: unknown): string => {
    if (!value || typeof value !== "object") return "";
    const record = value as Record<string, unknown>;
    return `${typeof record.text === "string" ? record.text : ""}${Array.isArray(record.children) ? record.children.map(visit).join("") : ""}`;
  };
  if (!content || typeof content !== "object") return "내용 없음";
  return visit((content as Record<string, unknown>).root) || "내용 없음";
}
