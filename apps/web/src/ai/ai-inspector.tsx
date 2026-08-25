import {
  ApiClient,
  ApiProblemError,
  type AIContextPreview,
  type AIContextRequest,
  type AIJob,
  type Proposal,
  selectProposalOperation,
} from "@adoc/ui-domain";
import { Button } from "../components/product/legacy";
import { LinkButton } from "../components/product/legacy";
import { Checkbox } from "../components/product/legacy";
import { Lozenge } from "../components/product/legacy";
import { Inline, Stack, Text } from "../components/product/legacy";
import { TextArea } from "../components/product/legacy";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useMemo, useState } from "react";

import { RoutePending, RouteProblem } from "../shell/common-states";
import "./ai-inspector.css";

const api = new ApiClient();
export const aiTaskKinds = [
  "COMPOSE",
  "REWRITE",
  "REVIEW",
  "DISCUSSION_APPLY",
  "CONFLICT_MERGE",
  "KNOWLEDGE_QUERY",
] as const;

export function AIInspector({
  workspaceId,
  workspaceSlug,
  documentId,
  jobId,
  proposalId,
}: Readonly<{
  workspaceId: string;
  workspaceSlug: string;
  documentId: string;
  jobId?: string;
  proposalId?: string;
}>) {
  const draft = useQuery({
    queryKey: ["draft", workspaceId, documentId],
    queryFn: ({ signal }) => api.draft(workspaceId, documentId, signal),
  });
  if (draft.isPending) return <RoutePending />;
  if (draft.error) return <AIProblem error={draft.error} retry={() => void draft.refetch()} />;
  return (
    <aside className="ai-inspector" aria-label="AI 도우미">
      <Stack space="space.250">
        <h2>AI 도우미</h2>
        <ContextRunner
          workspaceId={workspaceId}
          documentId={documentId}
          revision={draft.data.revision}
        />
        <JobBrowser
          workspaceId={workspaceId}
          workspaceSlug={workspaceSlug}
          documentId={documentId}
          jobId={jobId}
        />
        {proposalId && (
          <ProposalReview
            workspaceId={workspaceId}
            documentId={documentId}
            proposalId={proposalId}
            draftRevision={draft.data.revision}
          />
        )}
      </Stack>
    </aside>
  );
}

function ContextRunner({
  workspaceId,
  documentId,
  revision,
}: Readonly<{ workspaceId: string; documentId: string; revision: number }>) {
  const queryClient = useQueryClient();
  const [kind, setKind] = useState<(typeof aiTaskKinds)[number]>("REVIEW");
  const [instruction, setInstruction] = useState("");
  const [discussionId, setDiscussionId] = useState("");
  const [excluded, setExcluded] = useState<string[]>([]);
  const input = useMemo<AIContextRequest>(
    () => ({
      kind,
      target: taskTarget(kind, documentId, discussionId, instruction),
      expectedRevision: revision,
      externalWebEnabled: false,
      ...(instruction.trim() ? { instruction: instruction.trim() } : {}),
      includeSourceIds: [],
      excludeSourceIds: excluded,
    }),
    [discussionId, documentId, excluded, instruction, kind, revision],
  );
  const preview = useMutation({
    mutationFn: () => api.previewAIContext(workspaceId, input),
  });
  const create = useMutation({
    mutationFn: () => {
      if (!preview.data) throw new Error("먼저 Context를 확인해 주세요.");
      if (new Date(preview.data.expiresAt).getTime() <= Date.now())
        throw new Error("Context 확인 결과가 만료됐습니다.");
      return api.createAIJob(
        workspaceId,
        { ...input, contextFingerprint: preview.data.artifactFingerprint },
        revision,
        browserCommand(),
      );
    },
    onSuccess: async () => queryClient.invalidateQueries({ queryKey: ["ai-jobs", workspaceId] }),
  });
  return (
    <section aria-labelledby="ai-context-title">
      <Stack space="space.150">
        <h3 id="ai-context-title">Context Inspector</h3>
        <Inline space="space.050" shouldWrap>
          {aiTaskKinds.map((value) => (
            <Button
              key={value}
              appearance={kind === value ? "primary" : "subtle"}
              onClick={() => {
                setKind(value);
                preview.reset();
              }}
            >
              {taskLabel(value)}
            </Button>
          ))}
        </Inline>
        <label htmlFor="ai-instruction">요청</label>
        <TextArea
          id="ai-instruction"
          value={instruction}
          onChange={(event) => {
            setInstruction(event.currentTarget.value);
            preview.reset();
          }}
        />
        {kind === "DISCUSSION_APPLY" && (
          <>
            <label htmlFor="ai-discussion-id">Discussion ID</label>
            <TextArea
              id="ai-discussion-id"
              value={discussionId}
              onChange={(event) => {
                setDiscussionId(event.currentTarget.value);
                preview.reset();
              }}
            />
          </>
        )}
        <Inline space="space.100" shouldWrap>
          <Button
            onClick={() => preview.mutate()}
            isLoading={preview.isPending}
            isDisabled={!taskInputReady(kind, discussionId, instruction)}
          >
            Context 확인
          </Button>
          <Button
            appearance="primary"
            isDisabled={!preview.data}
            isLoading={create.isPending}
            onClick={() => create.mutate()}
          >
            작업 실행
          </Button>
        </Inline>
        {preview.data && (
          <ContextPreview
            preview={preview.data}
            excluded={excluded}
            onExcludedChange={(next) => {
              setExcluded(next);
              preview.reset();
            }}
          />
        )}
        <MutationMessage mutation={preview} />
        <MutationMessage mutation={create} success="AI 작업을 대기열에 추가했습니다." />
      </Stack>
    </section>
  );
}

function ContextPreview({
  preview,
  excluded,
  onExcludedChange,
}: Readonly<{
  preview: AIContextPreview;
  excluded: string[];
  onExcludedChange: (ids: string[]) => void;
}>) {
  return (
    <div className="ai-card">
      <Stack space="space.100">
        <Text weight="semibold">예상 입력 {preview.estimatedInputUnits} units</Text>
        <Text size="small">유효 시간 {new Date(preview.expiresAt).toLocaleString()}</Text>
        <ul className="ai-list">
          {preview.sources.map((source) => {
            const checked = !excluded.includes(source.sourceId);
            return (
              <li key={source.sourceId}>
                <Checkbox
                  isChecked={checked}
                  label={`${source.title ?? "제한된 Source"} · ${source.authority}`}
                  onChange={() =>
                    onExcludedChange(
                      checked
                        ? [...excluded, source.sourceId]
                        : excluded.filter((id) => id !== source.sourceId),
                    )
                  }
                />
              </li>
            );
          })}
        </ul>
        {preview.omissions.length > 0 && <Text>제외 사유: {preview.omissions.join(" · ")}</Text>}
      </Stack>
    </div>
  );
}

function JobBrowser({
  workspaceId,
  workspaceSlug,
  documentId,
  jobId,
}: Readonly<{
  workspaceId: string;
  workspaceSlug: string;
  documentId: string;
  jobId?: string;
}>) {
  const jobs = useQuery({
    queryKey: ["ai-jobs", workspaceId],
    queryFn: ({ signal }) => api.aiJobs(workspaceId, undefined, signal),
  });
  const job = useQuery({
    queryKey: ["ai-job", workspaceId, jobId],
    queryFn: ({ signal }) => api.aiJob(workspaceId, jobId ?? "", signal),
    enabled: Boolean(jobId),
  });
  if (jobs.isPending) return <RoutePending />;
  if (jobs.error) return <AIProblem error={jobs.error} retry={() => void jobs.refetch()} />;
  const base = `/w/${encodeURIComponent(workspaceSlug)}/docs/${encodeURIComponent(documentId)}?mode=draft&panel=ai`;
  return (
    <section aria-labelledby="ai-jobs-title">
      <Stack space="space.150">
        <h3 id="ai-jobs-title">작업 기록</h3>
        {jobs.data.items.length === 0 ? (
          <Text>AI 작업이 없습니다.</Text>
        ) : (
          <ul className="ai-list">
            {jobs.data.items.map((item) => (
              <li key={item.id}>
                <LinkButton appearance="subtle" href={`${base}&job=${encodeURIComponent(item.id)}`}>
                  {taskLabel(item.kind)} · {item.status}
                </LinkButton>
              </li>
            ))}
          </ul>
        )}
        {jobId && job.isPending && <RoutePending />}
        {job.error && <AIProblem error={job.error} retry={() => void job.refetch()} />}
        {job.data && <JobDetail workspaceId={workspaceId} job={job.data} base={base} />}
      </Stack>
    </section>
  );
}

function JobDetail({
  workspaceId,
  job,
  base,
}: Readonly<{ workspaceId: string; job: AIJob; base: string }>) {
  const queryClient = useQueryClient();
  const cancel = useMutation({
    mutationFn: () => api.cancelAIJob(workspaceId, job.id, job.revision, browserCommand()),
    onSuccess: async () =>
      queryClient.invalidateQueries({ queryKey: ["ai-job", workspaceId, job.id] }),
  });
  return (
    <div className="ai-card">
      <Stack space="space.100">
        <Inline space="space.100">
          <Lozenge appearance={job.status === "SUCCEEDED" ? "success" : "inprogress"}>
            {job.status}
          </Lozenge>
          <Text>sequence {job.sequence}</Text>
        </Inline>
        {isActiveJob(job.status) && (
          <Button onClick={() => cancel.mutate()} isLoading={cancel.isPending}>
            취소 요청
          </Button>
        )}
        {job.errorCode && <div role="alert">{job.errorCode}</div>}
        {job.result && <ResultView result={job.result} />}
        {job.proposalId && (
          <LinkButton
            appearance="primary"
            href={`${base}&job=${encodeURIComponent(job.id)}&proposal=${encodeURIComponent(job.proposalId)}`}
          >
            Proposal 검토
          </LinkButton>
        )}
        <MutationMessage mutation={cancel} success="취소를 요청했습니다." />
      </Stack>
    </div>
  );
}

function ResultView({ result }: Readonly<{ result: NonNullable<AIJob["result"]> }>) {
  return (
    <Stack space="space.100">
      <Text weight="semibold">결과: {result.status}</Text>
      {result.findings.map((finding) => (
        <div className="ai-finding" key={finding.findingId}>
          <Lozenge appearance={finding.severity === "BLOCKING" ? "removed" : "moved"}>
            {finding.severity}
          </Lozenge>
          <Text>{finding.reason}</Text>
        </div>
      ))}
      {result.claims.map((claim) => (
        <div
          className="ai-card"
          key={`${claim.certainty}:${claim.sourceIds.join(",")}:${claim.text}`}
        >
          <Text>{claim.text}</Text>
          <Text size="small">
            {claim.certainty} · Source {claim.sourceIds.join(", ") || "없음"}
          </Text>
        </div>
      ))}
      {result.uncertainties.map((item) => (
        <Text key={item}>불확실: {item}</Text>
      ))}
      {result.conflicts.map((item) => (
        <Text key={item.description}>충돌: {item.description}</Text>
      ))}
    </Stack>
  );
}

function ProposalReview({
  workspaceId,
  documentId,
  proposalId,
  draftRevision,
}: Readonly<{
  workspaceId: string;
  documentId: string;
  proposalId: string;
  draftRevision: number;
}>) {
  const queryClient = useQueryClient();
  const proposal = useQuery({
    queryKey: ["proposal", workspaceId, proposalId],
    queryFn: ({ signal }) => api.proposal(workspaceId, proposalId, signal),
  });
  const [selected, setSelected] = useState<string[]>([]);
  const [reason, setReason] = useState("");
  const apply = useMutation({
    mutationFn: async () => {
      if (!proposal.data || selected.length === 0) throw new Error("적용할 변경을 선택해 주세요.");
      const clientInstanceId = editorClient(documentId);
      const lease = await api.acquireLease(
        workspaceId,
        documentId,
        draftRevision,
        clientInstanceId,
        browserCommand(),
      );
      if (!lease.token) throw new Error("편집 lease token을 받지 못했습니다.");
      return api.applyProposal(
        workspaceId,
        proposalId,
        draftRevision,
        lease.token,
        clientInstanceId,
        selected,
        browserCommand(),
      );
    },
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ["proposal", workspaceId, proposalId] });
      await queryClient.invalidateQueries({ queryKey: ["draft", workspaceId, documentId] });
    },
  });
  const reject = useMutation({
    mutationFn: () => {
      if (!proposal.data || !reason.trim()) throw new Error("거절 사유를 입력해 주세요.");
      return api.rejectProposal(
        workspaceId,
        proposalId,
        proposal.data.revision,
        reason.trim(),
        browserCommand(),
      );
    },
    onSuccess: async () =>
      queryClient.invalidateQueries({ queryKey: ["proposal", workspaceId, proposalId] }),
  });
  if (proposal.isPending) return <RoutePending />;
  if (proposal.error)
    return <AIProblem error={proposal.error} retry={() => void proposal.refetch()} />;
  const stale = proposal.data.baseRevision !== draftRevision;
  return (
    <section aria-labelledby="proposal-title">
      <Stack space="space.150">
        <h3 id="proposal-title">Proposal Diff</h3>
        <Inline space="space.100">
          <Lozenge>{proposal.data.status}</Lozenge>
          <Text>base revision {proposal.data.baseRevision}</Text>
        </Inline>
        {stale && <div role="alert">현재 Draft와 기준 revision이 달라 다시 실행해야 합니다.</div>}
        <ul className="ai-list">
          {proposal.data.operations.map((operation) => (
            <OperationChoice
              key={operation.opId}
              operation={operation}
              operations={proposal.data.operations}
              selected={selected}
              onChange={setSelected}
              disabled={stale || proposal.data.status !== "OPEN"}
            />
          ))}
        </ul>
        <Button
          appearance="primary"
          isDisabled={stale || proposal.data.status !== "OPEN" || selected.length === 0}
          isLoading={apply.isPending}
          onClick={() => apply.mutate()}
        >
          선택한 변경 적용
        </Button>
        <label htmlFor="proposal-reject-reason">거절 사유</label>
        <TextArea
          id="proposal-reject-reason"
          value={reason}
          onChange={(event) => setReason(event.currentTarget.value)}
        />
        <Button
          isDisabled={proposal.data.status !== "OPEN" || !reason.trim()}
          isLoading={reject.isPending}
          onClick={() => reject.mutate()}
        >
          Proposal 거절
        </Button>
        <MutationMessage mutation={apply} success="선택한 변경을 적용했습니다." />
        <MutationMessage mutation={reject} success="Proposal을 거절했습니다." />
      </Stack>
    </section>
  );
}

type ProposalOperation = Proposal["operations"][number];

function OperationChoice({
  operation,
  operations,
  selected,
  onChange,
  disabled,
}: Readonly<{
  operation: ProposalOperation;
  operations: ProposalOperation[];
  selected: string[];
  onChange: (ids: string[]) => void;
  disabled: boolean;
}>) {
  const checked = selected.includes(operation.opId);
  return (
    <li>
      <Checkbox
        isDisabled={disabled}
        isChecked={checked}
        label={`${operation.kind} · ${scopeLabel(operation.scope)}${operation.dependsOn?.length ? ` · 선행 ${operation.dependsOn.length}개` : ""}`}
        onChange={() => onChange(selectProposalOperation(operations, selected, operation.opId))}
      />
    </li>
  );
}

function AIProblem({ error, retry }: Readonly<{ error: Error; retry: () => void }>) {
  const problem =
    error instanceof ApiProblemError
      ? error.problem
      : { code: "AI_QUERY_FAILED", message: error.message };
  return <RouteProblem code={problem.code} correlationId={problem.correlationId} onRetry={retry} />;
}

function MutationMessage({
  mutation,
  success = "처리했습니다.",
}: Readonly<{
  mutation: { isError: boolean; isSuccess: boolean; error: Error | null };
  success?: string;
}>) {
  if (mutation.isError) return <div role="alert">{mutation.error?.message}</div>;
  if (mutation.isSuccess) return <div role="status">{success}</div>;
  return null;
}

function taskLabel(kind: string) {
  return (
    (
      {
        COMPOSE: "초안 작성",
        REWRITE: "다시 쓰기",
        REVIEW: "문서 검토",
        DISCUSSION_APPLY: "토론 반영",
        CONFLICT_MERGE: "충돌 병합",
        KNOWLEDGE_QUERY: "지식 질문",
      } as Record<string, string>
    )[kind] ?? kind
  );
}

export function taskTarget(
  kind: (typeof aiTaskKinds)[number],
  documentId: string,
  discussionId: string,
  instruction: string,
): AIContextRequest["target"] {
  if (kind === "DISCUSSION_APPLY") {
    return { kind: "DISCUSSION", discussionId: discussionId.trim() };
  }
  if (kind === "KNOWLEDGE_QUERY") {
    return { kind: "WORKSPACE_QUERY", question: instruction.trim() };
  }
  return { kind: "DOCUMENT", documentId };
}

export function taskInputReady(
  kind: (typeof aiTaskKinds)[number],
  discussionId: string,
  instruction: string,
): boolean {
  if (kind === "DISCUSSION_APPLY") return discussionId.trim().length > 0;
  if (kind === "KNOWLEDGE_QUERY") return instruction.trim().length > 0;
  return true;
}

function isActiveJob(status: AIJob["status"]) {
  return status === "QUEUED" || status === "RUNNING" || status === "CANCEL_REQUESTED";
}

function scopeLabel(scope: ProposalOperation["scope"]) {
  if (scope.kind === "DOCUMENT") return "문서 전체";
  if (scope.kind === "BLOCK") return `블록 ${scope.blockId}`;
  if (scope.kind === "SECTION") return `섹션 ${scope.headingId}`;
  if (scope.kind === "BLOCK_RANGE") return `블록 범위 ${scope.startBlockId}–${scope.endBlockId}`;
  return `텍스트 범위 ${scope.blockId}`;
}

function browserCommand() {
  const value = document.cookie
    .split("; ")
    .find((item) => item.startsWith("adoc_csrf="))
    ?.slice("adoc_csrf=".length);
  if (!value) throw new Error("CSRF token is unavailable");
  return { csrfToken: decodeURIComponent(value), idempotencyKey: crypto.randomUUID() };
}

function editorClient(documentId: string) {
  const key = `adoc.editor.client.${documentId}`;
  const value = sessionStorage.getItem(key) ?? crypto.randomUUID();
  sessionStorage.setItem(key, value);
  return value;
}
