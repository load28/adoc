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
  type VocabularyConcept,
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
        <LinkButton href={`${base}?mode=draft&panel=ai`} appearance="subtle">
          AI
        </LinkButton>
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
        {panel === "review" && (
          <ReviewPanel
            workspaceId={workspaceId}
            workspaceSlug={workspaceSlug}
            documentId={documentId}
            reviewId={reviewId}
          />
        )}
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
  const [mentionInput, setMentionInput] = useState("");
  const [attachmentIds, setAttachmentIds] = useState<string[]>([]);
  const [uploading, setUploading] = useState(false);
  const [title, setTitle] = useState(detail.discussion.title);
  const [topicLabel, setTopicLabel] = useState("");
  const [topicKind, setTopicKind] = useState<"TEXT" | "DOCUMENT" | "REGION" | "EXTERNAL">("TEXT");
  const [topicValue, setTopicValue] = useState("");
  const [topicBlockId, setTopicBlockId] = useState("");
  const [reason, setReason] = useState("");
  const detailKey = ["discussion-detail", workspaceId, detail.discussion.id] as const;
  const send = useMutation({
    mutationFn: () =>
      api.createMessage(
        workspaceId,
        detail.discussion.id,
        richMessage(message, ids(mentionInput), attachmentIds),
        command(),
      ),
    onSuccess: async () => {
      setMessage("");
      setMentionInput("");
      setAttachmentIds([]);
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
  const updateTitle = useMutation({
    mutationFn: () => api.updateDiscussion(workspaceId, detail.discussion, title.trim(), command()),
    onSuccess: async () => queryClient.invalidateQueries({ queryKey: detailKey }),
  });
  const addTopic = useMutation({
    mutationFn: () =>
      api.addDiscussionTopic(
        workspaceId,
        detail.discussion,
        discussionTopic(topicKind, topicLabel, topicValue, topicBlockId),
        command(),
      ),
    onSuccess: async () => {
      setTopicLabel("");
      setTopicValue("");
      setTopicBlockId("");
      await queryClient.invalidateQueries({ queryKey: detailKey });
    },
  });
  const removeTopic = useMutation({
    mutationFn: (topicId: string) =>
      api.removeDiscussionTopic(workspaceId, detail.discussion, topicId, command()),
    onSuccess: async () => queryClient.invalidateQueries({ queryKey: detailKey }),
  });
  return (
    <section aria-labelledby="discussion-detail-title" className="discussion-detail">
      <h3 id="discussion-detail-title">{detail.discussion.title}</h3>
      {detail.discussion.status === "OPEN" && (
        <form
          className="collaboration-form"
          onSubmit={(event) => {
            event.preventDefault();
            if (title.trim()) updateTitle.mutate();
          }}
        >
          <label htmlFor="discussion-edit-title">토론 제목</label>
          <Textfield
            id="discussion-edit-title"
            value={title}
            onChange={(event) => setTitle(event.currentTarget.value)}
          />
          <Button type="submit" isLoading={updateTitle.isPending}>
            제목 저장
          </Button>
          <MutationStatus mutation={updateTitle} />
        </form>
      )}
      <h4>주제</h4>
      <ul className="resource-list">
        {(detail.discussion.topics ?? []).map((topic) => (
          <li key={topic.id}>
            <Text>{topic.label}</Text>
            <Lozenge>{topic.kind}</Lozenge>
            {detail.discussion.status === "OPEN" && (
              <Button
                appearance="subtle"
                onClick={() => removeTopic.mutate(topic.id)}
                isLoading={removeTopic.isPending}
              >
                주제 제거
              </Button>
            )}
          </li>
        ))}
      </ul>
      {detail.discussion.status === "OPEN" && (
        <form
          className="collaboration-form"
          onSubmit={(event) => {
            event.preventDefault();
            if (topicLabel.trim()) addTopic.mutate();
          }}
        >
          <Inline space="space.050" shouldWrap>
            {(["TEXT", "DOCUMENT", "REGION", "EXTERNAL"] as const).map((kind) => (
              <Button
                key={kind}
                appearance={topicKind === kind ? "primary" : "subtle"}
                onClick={() => setTopicKind(kind)}
              >
                {kind}
              </Button>
            ))}
          </Inline>
          <label htmlFor="discussion-topic">주제 label</label>
          <Textfield
            id="discussion-topic"
            value={topicLabel}
            onChange={(event) => setTopicLabel(event.currentTarget.value)}
          />
          <label htmlFor="discussion-topic-value">
            {topicKind === "TEXT"
              ? "주제 내용"
              : topicKind === "EXTERNAL"
                ? "HTTPS URL"
                : "대상 Document ID"}
          </label>
          <Textfield
            id="discussion-topic-value"
            value={topicValue}
            onChange={(event) => setTopicValue(event.currentTarget.value)}
          />
          {topicKind === "REGION" && (
            <>
              <label htmlFor="discussion-topic-block">대상 Block ID</label>
              <Textfield
                id="discussion-topic-block"
                value={topicBlockId}
                onChange={(event) => setTopicBlockId(event.currentTarget.value)}
              />
            </>
          )}
          <Button type="submit" isLoading={addTopic.isPending}>
            주제 추가
          </Button>
          <MutationStatus mutation={addTopic} />
        </form>
      )}
      <ul className="message-list">
        {detail.messages.map((item) => (
          <MessageRow
            key={item.id}
            workspaceId={workspaceId}
            discussionId={detail.discussion.id}
            message={item}
            editable={detail.discussion.status === "OPEN"}
            onChanged={() => queryClient.invalidateQueries({ queryKey: detailKey })}
          />
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
          <label htmlFor="reply-mentions">Mention 사용자 ID</label>
          <Textfield
            id="reply-mentions"
            value={mentionInput}
            placeholder="쉼표로 구분"
            onChange={(event) => setMentionInput(event.currentTarget.value)}
          />
          <label htmlFor="reply-attachment">첨부파일</label>
          <input
            id="reply-attachment"
            type="file"
            disabled={uploading}
            onChange={(event) => {
              const file = event.currentTarget.files?.[0];
              if (!file) return;
              setUploading(true);
              void uploadReadyFile(workspaceId, file)
                .then((assetId) => setAttachmentIds((current) => [...current, assetId]))
                .finally(() => setUploading(false));
            }}
          />
          {attachmentIds.length > 0 && <Text>첨부 {attachmentIds.length}개 준비됨</Text>}
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

function MessageRow({
  workspaceId,
  discussionId,
  message,
  editable,
  onChanged,
}: Readonly<{
  workspaceId: string;
  discussionId: string;
  message: DiscussionDetail["messages"][number];
  editable: boolean;
  onChanged: () => Promise<unknown>;
}>) {
  const [body, setBody] = useState(contentText(message.body));
  const update = useMutation({
    mutationFn: () =>
      api.updateMessage(
        workspaceId,
        discussionId,
        message.id,
        message.revision,
        richMessage(body, message.mentionUserIds, message.attachmentIds),
        command(),
      ),
    onSuccess: onChanged,
  });
  const redact = useMutation({
    mutationFn: () =>
      api.deleteMessage(workspaceId, discussionId, message.id, message.revision, command()),
    onSuccess: onChanged,
  });
  return (
    <li>
      {message.deletedAt ? (
        <Text>삭제된 메시지</Text>
      ) : (
        <Stack space="space.050">
          <Text>{contentText(message.body)}</Text>
          <Text size="small">
            작성자 {message.authorId}
            {message.editedAt ? ` · 수정 ${message.editedAt}` : ""}
          </Text>
          {message.attachmentIds.map((assetId) => (
            <a
              key={assetId}
              href={`/api/v1/workspaces/${encodeURIComponent(workspaceId)}/files/${encodeURIComponent(assetId)}/content`}
            >
              첨부파일 열기
            </a>
          ))}
          {editable && (
            <>
              <label htmlFor={`message-${message.id}`}>메시지 수정</label>
              <TextArea
                id={`message-${message.id}`}
                value={body}
                onChange={(event) => setBody(event.currentTarget.value)}
              />
              <Inline space="space.050">
                <Button
                  isDisabled={!body.trim()}
                  onClick={() => update.mutate()}
                  isLoading={update.isPending}
                >
                  수정 저장
                </Button>
                <Button
                  appearance="danger"
                  onClick={() => redact.mutate()}
                  isLoading={redact.isPending}
                >
                  메시지 가리기
                </Button>
              </Inline>
              <MutationStatus mutation={update} />
              <MutationStatus mutation={redact} />
            </>
          )}
        </Stack>
      )}
    </li>
  );
}

function ReviewPanel({
  workspaceId,
  workspaceSlug,
  documentId,
  reviewId,
}: Readonly<{
  workspaceId: string;
  workspaceSlug: string;
  documentId: string;
  reviewId?: string;
}>) {
  const draft = useQuery({
    queryKey: ["draft", workspaceId, documentId],
    queryFn: ({ signal }) => api.draft(workspaceId, documentId, signal),
    enabled: !reviewId,
  });
  const policy = useQuery({
    queryKey: ["publish-policy", workspaceId, documentId],
    queryFn: ({ signal }) => api.publishPolicy(workspaceId, documentId, signal),
    enabled: !reviewId,
  });
  const review = useQuery({
    queryKey: ["review", workspaceId, reviewId],
    queryFn: ({ signal }) => api.review(workspaceId, reviewId ?? "", signal),
    enabled: Boolean(reviewId),
  });
  const request = useMutation({
    mutationFn: () => {
      if (!draft.data) throw new Error("초안을 불러오지 못했습니다.");
      return api.requestReview(workspaceId, documentId, draft.data.revision, command());
    },
    onSuccess: (created) => {
      window.location.assign(
        `/w/${encodeURIComponent(workspaceSlug)}/docs/${encodeURIComponent(documentId)}?mode=published&panel=review&review=${encodeURIComponent(created.id)}`,
      );
    },
  });
  if (!reviewId) {
    if (draft.isPending || policy.isPending) return <RoutePending />;
    if (draft.error) return <QueryError error={draft.error} retry={() => void draft.refetch()} />;
    if (policy.error)
      return <QueryError error={policy.error} retry={() => void policy.refetch()} />;
    return (
      <Stack space="space.150">
        <h2>검토 요청</h2>
        <Text>초안 revision {draft.data.revision}</Text>
        <Text>
          {policy.data.mode} · 필요 승인 {policy.data.requiredApprovals}
        </Text>
        <Button appearance="primary" onClick={() => request.mutate()} isLoading={request.isPending}>
          이 revision 검토 요청
        </Button>
        <MutationStatus mutation={request} />
      </Stack>
    );
  }
  if (review.isPending) return <RoutePending />;
  if (review.error) return <QueryError error={review.error} retry={() => void review.refetch()} />;
  return <ReviewDetail workspaceId={workspaceId} review={review.data} />;
}

function ReviewDetail({
  workspaceId,
  review,
}: Readonly<{ workspaceId: string; review: ReviewView }>) {
  const queryClient = useQueryClient();
  const [discussionId, setDiscussionId] = useState("");
  const [cancelReason, setCancelReason] = useState("");
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
  const changes = useMutation({
    mutationFn: () =>
      api.submitReviewDecision(
        workspaceId,
        review.id,
        review.revision,
        { decision: "REQUEST_CHANGES", discussionId: discussionId.trim() },
        command(),
      ),
    onSuccess: async () => queryClient.invalidateQueries({ queryKey: ["review", workspaceId] }),
  });
  const cancel = useMutation({
    mutationFn: () => api.cancelReview(workspaceId, review, cancelReason.trim(), command()),
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
        <Stack space="space.100">
          <Button
            appearance="primary"
            onClick={() => approve.mutate()}
            isLoading={approve.isPending}
          >
            이 revision 승인
          </Button>
          <label htmlFor="review-changes-discussion">변경 요청 Discussion ID</label>
          <Textfield
            id="review-changes-discussion"
            value={discussionId}
            onChange={(event) => setDiscussionId(event.currentTarget.value)}
          />
          <Button
            isDisabled={!discussionId.trim()}
            onClick={() => changes.mutate()}
            isLoading={changes.isPending}
          >
            변경 요청
          </Button>
          <label htmlFor="review-cancel-reason">검토 취소 사유</label>
          <Textfield
            id="review-cancel-reason"
            value={cancelReason}
            onChange={(event) => setCancelReason(event.currentTarget.value)}
          />
          <Button
            appearance="danger"
            isDisabled={!cancelReason.trim()}
            onClick={() => cancel.mutate()}
            isLoading={cancel.isPending}
          >
            검토 취소
          </Button>
        </Stack>
      )}
      <MutationStatus mutation={approve} />
      <MutationStatus mutation={changes} />
      <MutationStatus mutation={cancel} />
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
  const draft = useQuery({
    queryKey: ["draft", workspaceId, documentId],
    queryFn: ({ signal }) => api.draft(workspaceId, documentId, signal),
  });
  const queryClient = useQueryClient();
  const [targetDocumentId, setTargetDocumentId] = useState("");
  const create = useMutation({
    mutationFn: async () => {
      if (!draft.data) throw new Error("초안을 불러오지 못했습니다.");
      const block = draft.data.content.root.children[0];
      if (!block) throw new Error("참조를 연결할 블록이 없습니다.");
      const clientInstanceId = editorClient(documentId);
      const lease = await api.acquireLease(
        workspaceId,
        documentId,
        draft.data.revision,
        clientInstanceId,
        command(),
      );
      if (!lease.token) throw new Error("편집 lease를 받지 못했습니다.");
      return api.createReference(
        workspaceId,
        documentId,
        draft.data.revision,
        {
          sourceRegion: { kind: "BLOCK", blockId: block.id },
          target: { kind: "DOCUMENT", id: targetDocumentId.trim() },
        },
        lease.token,
        clientInstanceId,
        command(),
      );
    },
    onSuccess: async () => {
      setTargetDocumentId("");
      await queryClient.invalidateQueries({ queryKey: ["backlinks", workspaceId] });
    },
  });
  return (
    <Stack space="space.150">
      <h2>이 문서를 참조하는 곳</h2>
      {query.isPending && <RoutePending />}
      {query.error && <QueryError error={query.error} retry={() => void query.refetch()} />}
      {query.data?.items.length === 0 && <Text>항목이 없습니다.</Text>}
      {query.data && (
        <ul className="resource-list">
          {query.data.items.map((item) => (
            <ReferenceMutationRow
              key={item.id}
              workspaceId={workspaceId}
              targetDocumentId={documentId}
              reference={item}
            />
          ))}
        </ul>
      )}
      <form
        className="collaboration-form"
        onSubmit={(event) => {
          event.preventDefault();
          if (targetDocumentId.trim()) create.mutate();
        }}
      >
        <label htmlFor="reference-target-document">참조할 Document ID</label>
        <Textfield
          id="reference-target-document"
          value={targetDocumentId}
          onChange={(event) => setTargetDocumentId(event.currentTarget.value)}
        />
        <Button type="submit" appearance="primary" isLoading={create.isPending}>
          첫 블록에 참조 연결
        </Button>
        <MutationStatus mutation={create} />
      </form>
    </Stack>
  );
}

function ReferenceMutationRow({
  workspaceId,
  targetDocumentId,
  reference,
}: Readonly<{
  workspaceId: string;
  targetDocumentId: string;
  reference: ReferencePage["items"][number];
}>) {
  const client = useQueryClient();
  const remove = useMutation({
    mutationFn: async () => {
      const draft = await api.draft(workspaceId, reference.sourceDocumentId);
      const clientInstanceId = editorClient(reference.sourceDocumentId);
      const lease = await api.acquireLease(
        workspaceId,
        reference.sourceDocumentId,
        draft.revision,
        clientInstanceId,
        command(),
      );
      if (!lease.token) throw new Error("편집 lease를 받지 못했습니다.");
      await api.deleteReference(
        workspaceId,
        reference.sourceDocumentId,
        reference.id,
        draft.revision,
        lease.token,
        clientInstanceId,
        command(),
      );
    },
    onSuccess: async () =>
      client.invalidateQueries({ queryKey: ["backlinks", workspaceId, targetDocumentId] }),
  });
  return (
    <li>
      <Text>
        {reference.snapshot.title || "접근할 수 없는 참조"} · source {reference.sourceDocumentId}
      </Text>
      <Button appearance="danger" onClick={() => remove.mutate()} isLoading={remove.isPending}>
        참조 연결 삭제
      </Button>
      <MutationStatus mutation={remove} />
    </li>
  );
}

export function InboxScreen({
  workspaceId,
  workspaceSlug,
}: Readonly<{ workspaceId: string; workspaceSlug: string }>) {
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
  const markAll = useMutation({
    mutationFn: () => api.markAllInboxRead(workspaceId, new Date().toISOString(), command()),
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
      <Button onClick={() => markAll.mutate()} isLoading={markAll.isPending}>
        현재까지 모두 읽음
      </Button>
      <MutationStatus mutation={markAll} />
      <InboxList
        query={query}
        update={update}
        workspaceId={workspaceId}
        workspaceSlug={workspaceSlug}
      />
    </Screen>
  );
}

function InboxList({
  query,
  update,
  workspaceId,
  workspaceSlug,
}: Readonly<{
  query: ReturnType<typeof useQuery<InboxPage>>;
  update: ReturnType<
    typeof useMutation<unknown, Error, { id: string; action: "read" | "resolve" }>
  >;
  workspaceId: string;
  workspaceSlug: string;
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
              <InboxTargetButton
                workspaceId={workspaceId}
                workspaceSlug={workspaceSlug}
                item={item}
              />
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

function InboxTargetButton({
  workspaceId,
  workspaceSlug,
  item,
}: Readonly<{
  workspaceId: string;
  workspaceSlug: string;
  item: InboxPage["items"][number];
}>) {
  const open = useMutation({
    mutationFn: () => resolveInboxTarget(api, workspaceId, workspaceSlug, item.target),
    onSuccess: (href) => window.location.assign(href),
  });
  return (
    <>
      <Button appearance="subtle" onClick={() => open.mutate()} isLoading={open.isPending}>
        열기
      </Button>
      <MutationStatus mutation={open} />
    </>
  );
}

export function SearchScreen({
  workspaceId,
  workspaceSlug,
  initialQuery = "",
}: Readonly<{ workspaceId: string; workspaceSlug: string; initialQuery?: string }>) {
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
      {submitted && <SearchResults query={query} workspaceSlug={workspaceSlug} />}
    </Screen>
  );
}

function SearchResults({
  query,
  workspaceSlug,
}: Readonly<{ query: ReturnType<typeof useQuery<SearchPage>>; workspaceSlug: string }>) {
  if (query.isPending) return <RoutePending />;
  if (query.error) return <QueryError error={query.error} retry={() => void query.refetch()} />;
  if (query.data.items.length === 0) return <Text>검색 결과가 없습니다.</Text>;
  return (
    <ul className="resource-list">
      {query.data.items.map(({ source }) => (
        <li key={source.stableId}>
          <Stack space="space.050">
            <LinkButton
              appearance="subtle"
              href={`/w/${encodeURIComponent(workspaceSlug)}/docs/${encodeURIComponent(source.documentId)}?mode=${source.kind === "DRAFT" ? "draft" : "published"}`}
            >
              {source.displaySnapshot.title}
            </LinkButton>
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
      <VocabularyList query={query} workspaceId={workspaceId} />
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
  workspaceId,
}: Readonly<{ query: ReturnType<typeof useQuery<VocabularyPage>>; workspaceId: string }>) {
  if (query.isPending) return <RoutePending />;
  if (query.error) return <QueryError error={query.error} retry={() => void query.refetch()} />;
  return (
    <ul className="resource-list">
      {query.data.items.map((item) => (
        <VocabularyRow key={item.id} workspaceId={workspaceId} item={item} />
      ))}
    </ul>
  );
}

function VocabularyRow({
  workspaceId,
  item,
}: Readonly<{ workspaceId: string; item: VocabularyConcept }>) {
  const client = useQueryClient();
  const [term, setTerm] = useState(item.canonicalTerm);
  const [definition, setDefinition] = useState(item.definition);
  const [aliases, setAliases] = useState(
    item.terms
      .filter((value) => value.kind === "SYNONYM")
      .map((value) => value.term)
      .join(", "),
  );
  const [reason, setReason] = useState("");
  const [replacement, setReplacement] = useState("");
  const update = useMutation({
    mutationFn: () =>
      api.writeVocabulary(
        workspaceId,
        {
          canonicalTerm: term.trim(),
          definition: definition.trim(),
          terms: [
            { term: term.trim(), kind: "CANONICAL" },
            ...textValues(aliases).map((value) => ({ term: value, kind: "SYNONYM" as const })),
          ],
        },
        command(),
        item,
      ),
    onSuccess: async () => client.invalidateQueries({ queryKey: ["vocabulary", workspaceId] }),
  });
  const deprecate = useMutation({
    mutationFn: () =>
      api.deprecateVocabulary(
        workspaceId,
        item,
        replacement.trim() || null,
        reason.trim(),
        command(),
      ),
    onSuccess: async () => client.invalidateQueries({ queryKey: ["vocabulary", workspaceId] }),
  });
  return (
    <li>
      <Stack space="space.100">
        <Inline space="space.100">
          <Text weight="semibold">{item.canonicalTerm}</Text>
          <Lozenge appearance={item.status === "ACTIVE" ? "success" : "removed"}>
            {item.status}
          </Lozenge>
        </Inline>
        <Text>{item.definition}</Text>
        <Text size="small">{item.terms.map((value) => value.term).join(" · ")}</Text>
        {item.status === "ACTIVE" && (
          <>
            <label htmlFor={`vocabulary-term-${item.id}`}>표준 용어</label>
            <Textfield
              id={`vocabulary-term-${item.id}`}
              value={term}
              onChange={(event) => setTerm(event.currentTarget.value)}
            />
            <label htmlFor={`vocabulary-definition-${item.id}`}>정의</label>
            <TextArea
              id={`vocabulary-definition-${item.id}`}
              value={definition}
              onChange={(event) => setDefinition(event.currentTarget.value)}
            />
            <label htmlFor={`vocabulary-aliases-${item.id}`}>동의어</label>
            <Textfield
              id={`vocabulary-aliases-${item.id}`}
              value={aliases}
              placeholder="쉼표로 구분"
              onChange={(event) => setAliases(event.currentTarget.value)}
            />
            <Button
              onClick={() => update.mutate()}
              isLoading={update.isPending}
              isDisabled={!term.trim() || !definition.trim()}
            >
              개념 저장
            </Button>
            <label htmlFor={`vocabulary-replacement-${item.id}`}>대체 Concept ID (선택)</label>
            <Textfield
              id={`vocabulary-replacement-${item.id}`}
              value={replacement}
              onChange={(event) => setReplacement(event.currentTarget.value)}
            />
            <label htmlFor={`vocabulary-reason-${item.id}`}>폐기 사유</label>
            <Textfield
              id={`vocabulary-reason-${item.id}`}
              value={reason}
              onChange={(event) => setReason(event.currentTarget.value)}
            />
            <Button
              appearance="danger"
              onClick={() => deprecate.mutate()}
              isLoading={deprecate.isPending}
              isDisabled={!reason.trim()}
            >
              개념 폐기
            </Button>
            <MutationStatus mutation={update} />
            <MutationStatus mutation={deprecate} />
          </>
        )}
      </Stack>
    </li>
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
function richMessage(text: string, mentionUserIds: string[] = [], attachmentIds: string[] = []) {
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
    mentionUserIds,
    attachmentIds,
  };
}

function ids(value: string): string[] {
  return [...new Set(textValues(value))];
}

export function discussionTopic(
  kind: "TEXT" | "DOCUMENT" | "REGION" | "EXTERNAL",
  label: string,
  value: string,
  blockId: string,
): Parameters<ApiClient["addDiscussionTopic"]>[2] {
  const normalizedLabel = label.trim();
  const normalizedValue = value.trim();
  if (!normalizedLabel || !normalizedValue) throw new Error("주제 필수 값을 입력해 주세요.");
  if (kind === "TEXT") return { kind, label: normalizedLabel, text: normalizedValue };
  if (kind === "DOCUMENT") return { kind, label: normalizedLabel, targetId: normalizedValue };
  if (kind === "EXTERNAL") {
    if (!normalizedValue.startsWith("https://")) throw new Error("HTTPS URL만 사용할 수 있습니다.");
    return { kind, label: normalizedLabel, url: normalizedValue };
  }
  if (!blockId.trim()) throw new Error("Region Block ID를 입력해 주세요.");
  return {
    kind,
    label: normalizedLabel,
    targetId: normalizedValue,
    region: { kind: "BLOCK", blockId: blockId.trim() },
  };
}

function textValues(value: string): string[] {
  return value
    .split(",")
    .map((item) => item.trim())
    .filter(Boolean);
}

export function inboxTargetHref(
  workspaceSlug: string,
  target: InboxPage["items"][number]["target"],
): string {
  const base = `/w/${encodeURIComponent(workspaceSlug)}`;
  if (target.kind === "DOCUMENT")
    return `${base}/docs/${encodeURIComponent(target.id)}?mode=published`;
  return `${base}/inbox?targetKind=${encodeURIComponent(target.kind)}&target=${encodeURIComponent(target.id)}`;
}

export async function resolveInboxTarget(
  client: ApiClient,
  workspaceId: string,
  workspaceSlug: string,
  target: InboxPage["items"][number]["target"],
): Promise<string> {
  const base = `/w/${encodeURIComponent(workspaceSlug)}`;
  if (target.kind === "DOCUMENT") return inboxTargetHref(workspaceSlug, target);
  if (target.kind === "WORKSPACE") return `${base}/home`;
  if (target.kind === "FILE") {
    return `/api/v1/workspaces/${encodeURIComponent(workspaceId)}/files/${encodeURIComponent(target.id)}/content`;
  }
  if (target.kind === "DISCUSSION") {
    const detail = await client.discussion(workspaceId, target.id);
    return `${base}/docs/${encodeURIComponent(detail.discussion.documentId)}?mode=published&panel=discussion&discussion=${encodeURIComponent(target.id)}`;
  }
  if (target.kind === "REVIEW") {
    const review = await client.review(workspaceId, target.id);
    return `${base}/docs/${encodeURIComponent(review.documentId)}?mode=published&panel=review&review=${encodeURIComponent(target.id)}`;
  }
  const job = await client.aiJob(workspaceId, target.id);
  if (job.target.kind === "WORKSPACE_QUERY") {
    return `${base}/search?q=${encodeURIComponent(job.target.question)}`;
  }
  if (job.target.kind === "DISCUSSION") {
    const detail = await client.discussion(workspaceId, job.target.discussionId);
    return `${base}/docs/${encodeURIComponent(detail.discussion.documentId)}?mode=draft&panel=ai&job=${encodeURIComponent(job.id)}`;
  }
  return `${base}/docs/${encodeURIComponent(job.target.documentId)}?mode=draft&panel=ai&job=${encodeURIComponent(job.id)}`;
}

function editorClient(documentId: string): string {
  const key = `adoc.editor.client.${documentId}`;
  const current = sessionStorage.getItem(key);
  if (current) return current;
  const value = crypto.randomUUID();
  sessionStorage.setItem(key, value);
  return value;
}

async function uploadReadyFile(workspaceId: string, file: File): Promise<string> {
  const checksum = await sha256(file);
  const upload = await api.createFileUpload(
    workspaceId,
    {
      name: file.name,
      mimeType: file.type || "application/octet-stream",
      size: file.size,
      checksum,
    },
    command(),
  );
  const csrf = command().csrfToken;
  await api.uploadFileBytes(upload.uploadUrl, upload.uploadToken, csrf, file);
  const asset = await api.completeFileUpload(
    workspaceId,
    upload.assetId,
    checksum,
    file.size,
    command(),
  );
  if (asset.status !== "READY") throw new Error(`첨부파일 상태가 ${asset.status}입니다.`);
  return asset.id;
}

async function sha256(file: Blob): Promise<string> {
  const digest = new Uint8Array(await crypto.subtle.digest("SHA-256", await file.arrayBuffer()));
  return [...digest].map((byte) => byte.toString(16).padStart(2, "0")).join("");
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
