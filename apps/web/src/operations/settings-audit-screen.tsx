import {
  ApiClient,
  ApiProblemError,
  type AIConfiguration,
  type Group,
  type Invitation,
  type Membership,
  type PermissionGrant,
  type PublishPolicy,
  type SettingsSection,
  type SettingsSearch,
} from "@adoc/ui-domain";
import Button from "@atlaskit/button/default/button";
import Lozenge from "@atlaskit/lozenge";
import { Inline, Stack, Text } from "@atlaskit/primitives";
import Textfield from "@atlaskit/textfield";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect, useState } from "react";

import { RoutePending, RouteProblem } from "../shell/common-states";
import "./settings-audit.css";

const api = new ApiClient();

export function SettingsAuditScreen({
  workspaceId,
  section,
  documentId,
  search,
}: Readonly<{
  workspaceId: string;
  section: SettingsSection;
  documentId?: string;
  search: SettingsSearch;
}>) {
  return (
    <main className="settings-screen">
      <Stack space="space.250">
        <h1>설정 · {sectionLabel(section)}</h1>
        {section === "members" && <MembersSettings workspaceId={workspaceId} />}
        {section === "groups" && <GroupsSettings workspaceId={workspaceId} />}
        {section === "permissions" && (
          <PermissionSettings workspaceId={workspaceId} documentId={documentId} />
        )}
        {section === "writing" && <WritingSettings workspaceId={workspaceId} />}
        {section === "ai" && <AISettings workspaceId={workspaceId} />}
        {section === "audit" && <AuditSettings workspaceId={workspaceId} initial={search} />}
      </Stack>
    </main>
  );
}

function MembersSettings({ workspaceId }: Readonly<{ workspaceId: string }>) {
  const client = useQueryClient();
  const members = useQuery({
    queryKey: ["members", workspaceId],
    queryFn: ({ signal }) => api.members(workspaceId, signal),
  });
  const invitations = useQuery({
    queryKey: ["invitations", workspaceId],
    queryFn: ({ signal }) => api.invitations(workspaceId, undefined, signal),
  });
  const [email, setEmail] = useState("");
  const [role, setRole] = useState<"MEMBER" | "ADMIN">("MEMBER");
  const invite = useMutation({
    mutationFn: () => api.inviteMember(workspaceId, email.trim(), role, command()),
    onSuccess: async () => {
      setEmail("");
      await client.invalidateQueries({ queryKey: ["invitations", workspaceId] });
    },
  });
  if (members.isPending || invitations.isPending) return <RoutePending />;
  if (members.error) return <Problem error={members.error} retry={() => void members.refetch()} />;
  if (invitations.error)
    return <Problem error={invitations.error} retry={() => void invitations.refetch()} />;
  return (
    <Stack space="space.200">
      <ResourceHeading title="구성원" count={members.data.length} />
      <ul className="settings-list">
        {members.data.map((member) => (
          <MemberRow key={member.userId} workspaceId={workspaceId} member={member} />
        ))}
      </ul>
      <h2>초대</h2>
      <form
        className="settings-form"
        onSubmit={(event) => {
          event.preventDefault();
          if (email.trim()) invite.mutate();
        }}
      >
        <label htmlFor="invite-email">이메일</label>
        <Textfield
          id="invite-email"
          type="email"
          value={email}
          onChange={(event) => setEmail(event.currentTarget.value)}
        />
        <Inline space="space.050">
          <Button
            appearance={role === "MEMBER" ? "primary" : "subtle"}
            onClick={() => setRole("MEMBER")}
          >
            Member
          </Button>
          <Button
            appearance={role === "ADMIN" ? "primary" : "subtle"}
            onClick={() => setRole("ADMIN")}
          >
            Admin
          </Button>
        </Inline>
        <Button type="submit" appearance="primary" isLoading={invite.isPending}>
          초대 보내기
        </Button>
        <MutationMessage mutation={invite} />
      </form>
      <ul className="settings-list">
        {invitations.data.items.map((item) => (
          <InvitationRow key={item.id} workspaceId={workspaceId} invitation={item} />
        ))}
      </ul>
    </Stack>
  );
}

function MemberRow({ workspaceId, member }: Readonly<{ workspaceId: string; member: Membership }>) {
  const client = useQueryClient();
  const update = useMutation({
    mutationFn: () =>
      api.updateMemberRole(
        workspaceId,
        member,
        member.role === "MEMBER" ? "ADMIN" : "MEMBER",
        command(),
      ),
    onSuccess: async () => client.invalidateQueries({ queryKey: ["members", workspaceId] }),
  });
  const remove = useMutation({
    mutationFn: () => api.removeMember(workspaceId, member, command()),
    onSuccess: async () => client.invalidateQueries({ queryKey: ["members", workspaceId] }),
  });
  return (
    <li>
      <Stack space="space.050">
        <Text weight="semibold">{member.userId}</Text>
        <Inline space="space.100">
          <Lozenge>{member.role}</Lozenge>
          <Lozenge>{member.status}</Lozenge>
        </Inline>
      </Stack>
      <Inline space="space.050" shouldWrap>
        {member.role !== "OWNER" && (
          <>
            <Button onClick={() => update.mutate()} isLoading={update.isPending}>
              역할 전환
            </Button>
            <Button
              appearance="danger"
              onClick={() => remove.mutate()}
              isLoading={remove.isPending}
            >
              제거
            </Button>
          </>
        )}
        <MutationMessage mutation={update} />
        <MutationMessage mutation={remove} />
      </Inline>
    </li>
  );
}

function InvitationRow({
  workspaceId,
  invitation,
}: Readonly<{ workspaceId: string; invitation: Invitation }>) {
  const client = useQueryClient();
  const revoke = useMutation({
    mutationFn: () => api.revokeInvitation(workspaceId, invitation, command()),
    onSuccess: async () => client.invalidateQueries({ queryKey: ["invitations", workspaceId] }),
  });
  return (
    <li>
      <Stack space="space.050">
        <Text weight="semibold">{invitation.email}</Text>
        <Text>
          {invitation.role} · {invitation.status}
        </Text>
      </Stack>
      {invitation.status === "PENDING" && (
        <Button onClick={() => revoke.mutate()} isLoading={revoke.isPending}>
          초대 취소
        </Button>
      )}
    </li>
  );
}

function GroupsSettings({ workspaceId }: Readonly<{ workspaceId: string }>) {
  const client = useQueryClient();
  const query = useQuery({
    queryKey: ["groups", workspaceId],
    queryFn: ({ signal }) => api.groups(workspaceId, signal),
  });
  const [name, setName] = useState("");
  const create = useMutation({
    mutationFn: () => api.createGroup(workspaceId, name.trim(), command()),
    onSuccess: async () => {
      setName("");
      await client.invalidateQueries({ queryKey: ["groups", workspaceId] });
    },
  });
  if (query.isPending) return <RoutePending />;
  if (query.error) return <Problem error={query.error} retry={() => void query.refetch()} />;
  return (
    <Stack space="space.200">
      <form
        className="settings-form"
        onSubmit={(event) => {
          event.preventDefault();
          if (name.trim()) create.mutate();
        }}
      >
        <label htmlFor="group-name">그룹 이름</label>
        <Textfield
          id="group-name"
          value={name}
          onChange={(event) => setName(event.currentTarget.value)}
        />
        <Button type="submit" appearance="primary" isLoading={create.isPending}>
          그룹 만들기
        </Button>
        <MutationMessage mutation={create} />
      </form>
      <ul className="settings-list">
        {query.data.map((group) => (
          <GroupRow key={group.id} workspaceId={workspaceId} group={group} />
        ))}
      </ul>
    </Stack>
  );
}

function GroupRow({ workspaceId, group }: Readonly<{ workspaceId: string; group: Group }>) {
  const client = useQueryClient();
  const [name, setName] = useState(group.name);
  const [userId, setUserId] = useState("");
  const update = useMutation({
    mutationFn: () => api.updateGroup(workspaceId, group, name.trim(), command()),
    onSuccess: async () => client.invalidateQueries({ queryKey: ["groups", workspaceId] }),
  });
  const remove = useMutation({
    mutationFn: () => api.deleteGroup(workspaceId, group, command()),
    onSuccess: async () => client.invalidateQueries({ queryKey: ["groups", workspaceId] }),
  });
  const member = useMutation({
    mutationFn: (action: "add" | "remove") =>
      api.changeGroupMember(workspaceId, group, userId.trim(), action, command()),
    onSuccess: async () => {
      setUserId("");
      await client.invalidateQueries({ queryKey: ["groups", workspaceId] });
    },
  });
  return (
    <li>
      <div className="settings-inline-field">
        <Textfield
          aria-label={`${group.name} 그룹 이름`}
          value={name}
          onChange={(event) => setName(event.currentTarget.value)}
        />
        <Text size="small">구성원 {group.memberIds.length}명</Text>
      </div>
      <Inline space="space.050">
        <Button onClick={() => update.mutate()} isLoading={update.isPending}>
          저장
        </Button>
        <Button appearance="danger" onClick={() => remove.mutate()} isLoading={remove.isPending}>
          삭제
        </Button>
      </Inline>
      <label htmlFor={`group-member-${group.id}`}>구성원 사용자 ID</label>
      <Textfield
        id={`group-member-${group.id}`}
        value={userId}
        onChange={(event) => setUserId(event.currentTarget.value)}
      />
      <Inline space="space.050">
        <Button
          isDisabled={!userId.trim()}
          onClick={() => member.mutate("add")}
          isLoading={member.isPending}
        >
          구성원 추가
        </Button>
        <Button
          isDisabled={!userId.trim()}
          onClick={() => member.mutate("remove")}
          isLoading={member.isPending}
        >
          구성원 제거
        </Button>
      </Inline>
      {group.memberIds.length > 0 && <Text size="small">현재: {group.memberIds.join(" · ")}</Text>}
      <MutationMessage mutation={member} />
    </li>
  );
}

function PermissionSettings({
  workspaceId,
  documentId,
}: Readonly<{ workspaceId: string; documentId?: string }>) {
  const query = useQuery({
    queryKey: ["permissions", workspaceId, documentId],
    queryFn: ({ signal }) => api.documentPermissions(workspaceId, documentId ?? "", signal),
    enabled: Boolean(documentId),
  });
  const policy = useQuery({
    queryKey: ["publish-policy", workspaceId, documentId],
    queryFn: ({ signal }) => api.publishPolicy(workspaceId, documentId ?? "", signal),
    enabled: Boolean(documentId),
  });
  if (!documentId) return <Text>URL의 document query로 관리할 문서를 선택해 주세요.</Text>;
  if (query.isPending || policy.isPending) return <RoutePending />;
  if (query.error) return <Problem error={query.error} retry={() => void query.refetch()} />;
  if (policy.error) return <Problem error={policy.error} retry={() => void policy.refetch()} />;
  return (
    <Stack space="space.150">
      <Text>
        유효 권한: {query.data.effective.access}
        {query.data.effective.manage ? " · 관리 가능" : ""}
      </Text>
      <ul className="settings-list">
        {query.data.explicitGrants.map((grant) => (
          <PermissionGrantRow
            key={grant.id}
            workspaceId={workspaceId}
            documentId={documentId}
            collectionRevision={query.data.revision}
            grant={grant}
          />
        ))}
      </ul>
      <PermissionForm
        workspaceId={workspaceId}
        documentId={documentId}
        revision={query.data.revision}
      />
      <PermissionExplanationForm workspaceId={workspaceId} documentId={documentId} />
      <PublishPolicyForm workspaceId={workspaceId} documentId={documentId} initial={policy.data} />
    </Stack>
  );
}

function PermissionForm({
  workspaceId,
  documentId,
  revision,
}: Readonly<{ workspaceId: string; documentId: string; revision: number }>) {
  const client = useQueryClient();
  const [subjectId, setSubjectId] = useState("");
  const [subjectKind, setSubjectKind] = useState<"USER" | "GROUP">("USER");
  const [access, setAccess] = useState<"NO_ACCESS" | "VIEWER" | "CONTRIBUTOR" | "EDITOR">("VIEWER");
  const [manage, setManage] = useState(false);
  const save = useMutation({
    mutationFn: () =>
      api.setDocumentPermission(
        workspaceId,
        documentId,
        crypto.randomUUID(),
        revision,
        { subjectKind, subjectId, access, manage },
        command(),
      ),
    onSuccess: async () =>
      client.invalidateQueries({ queryKey: ["permissions", workspaceId, documentId] }),
  });
  return (
    <form
      className="settings-form"
      onSubmit={(event) => {
        event.preventDefault();
        if (subjectId) save.mutate();
      }}
    >
      <Inline space="space.050">
        {(["USER", "GROUP"] as const).map((kind) => (
          <Button
            key={kind}
            appearance={subjectKind === kind ? "primary" : "subtle"}
            onClick={() => setSubjectKind(kind)}
          >
            {kind}
          </Button>
        ))}
      </Inline>
      <label htmlFor="permission-user">Subject ID</label>
      <Textfield
        id="permission-user"
        value={subjectId}
        onChange={(event) => setSubjectId(event.currentTarget.value)}
      />
      <Inline space="space.050" shouldWrap>
        {(["NO_ACCESS", "VIEWER", "CONTRIBUTOR", "EDITOR"] as const).map((item) => (
          <Button
            key={item}
            appearance={access === item ? "primary" : "subtle"}
            onClick={() => setAccess(item)}
          >
            {item}
          </Button>
        ))}
      </Inline>
      <Button appearance={manage ? "primary" : "subtle"} onClick={() => setManage(!manage)}>
        권한 관리 {manage ? "허용" : "미허용"}
      </Button>
      <Button type="submit" appearance="primary" isLoading={save.isPending}>
        권한 추가
      </Button>
      <MutationMessage mutation={save} />
    </form>
  );
}

function PermissionGrantRow({
  workspaceId,
  documentId,
  collectionRevision,
  grant,
}: Readonly<{
  workspaceId: string;
  documentId: string;
  collectionRevision: number;
  grant: PermissionGrant;
}>) {
  const client = useQueryClient();
  const remove = useMutation({
    mutationFn: () =>
      api.deleteDocumentPermission(
        workspaceId,
        documentId,
        grant.id,
        collectionRevision,
        command(),
      ),
    onSuccess: async () =>
      client.invalidateQueries({ queryKey: ["permissions", workspaceId, documentId] }),
  });
  return (
    <li>
      <Stack space="space.050">
        <Text>
          {grant.subjectKind} · {grant.subjectId}
        </Text>
        <Inline space="space.050">
          <Lozenge>{grant.access}</Lozenge>
          {grant.manage && <Lozenge appearance="inprogress">MANAGE</Lozenge>}
        </Inline>
      </Stack>
      <Button appearance="danger" onClick={() => remove.mutate()} isLoading={remove.isPending}>
        명시 권한 삭제
      </Button>
      <MutationMessage mutation={remove} />
    </li>
  );
}

function PermissionExplanationForm({
  workspaceId,
  documentId,
}: Readonly<{ workspaceId: string; documentId: string }>) {
  const [subjectKind, setSubjectKind] = useState<"USER" | "GROUP">("USER");
  const [subjectId, setSubjectId] = useState("");
  const [submitted, setSubmitted] = useState("");
  const explanation = useQuery({
    queryKey: ["permission-explanation", workspaceId, documentId, subjectKind, submitted],
    queryFn: ({ signal }) =>
      api.explainDocumentPermission(workspaceId, documentId, subjectKind, submitted, signal),
    enabled: Boolean(submitted),
  });
  return (
    <Stack space="space.100">
      <h2>유효 권한 설명</h2>
      <Inline space="space.050">
        {(["USER", "GROUP"] as const).map((kind) => (
          <Button
            key={kind}
            appearance={subjectKind === kind ? "primary" : "subtle"}
            onClick={() => {
              setSubjectKind(kind);
              setSubmitted("");
            }}
          >
            {kind}
          </Button>
        ))}
      </Inline>
      <Textfield
        aria-label="설명할 Subject ID"
        value={subjectId}
        onChange={(event) => setSubjectId(event.currentTarget.value)}
      />
      <Button isDisabled={!subjectId.trim()} onClick={() => setSubmitted(subjectId.trim())}>
        계산 근거 확인
      </Button>
      {explanation.isPending && submitted && <RoutePending />}
      {explanation.error && (
        <Problem error={explanation.error} retry={() => void explanation.refetch()} />
      )}
      {explanation.data && (
        <Stack space="space.050">
          <Text>
            {explanation.data.effective.access} · fingerprint {explanation.data.fingerprint}
          </Text>
          <ul className="settings-list">
            {explanation.data.steps.map((step) => (
              <li key={`${step.documentId}:${step.decision}`}>
                <Text>
                  {step.documentId} · {step.decision}
                </Text>
              </li>
            ))}
          </ul>
        </Stack>
      )}
    </Stack>
  );
}

function PublishPolicyForm({
  workspaceId,
  documentId,
  initial,
}: Readonly<{ workspaceId: string; documentId: string; initial: PublishPolicy }>) {
  const client = useQueryClient();
  const [mode, setMode] = useState<PublishPolicy["mode"]>(initial.mode);
  const [approvals, setApprovals] = useState(String(initial.requiredApprovals));
  const [reviewerKind, setReviewerKind] = useState<"ANY_EDITOR" | "USERS" | "GROUPS">(
    initial.reviewerRule.kind,
  );
  const [reviewerIds, setReviewerIds] = useState(
    initial.reviewerRule.kind === "USERS"
      ? initial.reviewerRule.userIds.join(", ")
      : initial.reviewerRule.kind === "GROUPS"
        ? initial.reviewerRule.groupIds.join(", ")
        : "",
  );
  const save = useMutation({
    mutationFn: () =>
      api.setPublishPolicy(
        workspaceId,
        documentId,
        initial,
        {
          mode,
          requiredApprovals: mode === "DIRECT" ? 0 : Number(approvals),
          reviewerRule: reviewerRule(reviewerKind, reviewerIds),
        },
        command(),
      ),
    onSuccess: async () =>
      client.invalidateQueries({ queryKey: ["publish-policy", workspaceId, documentId] }),
  });
  return (
    <Stack space="space.100">
      <h2>발행 정책</h2>
      <Inline space="space.050">
        {(["DIRECT", "REVIEW_REQUIRED"] as const).map((value) => (
          <Button
            key={value}
            appearance={mode === value ? "primary" : "subtle"}
            onClick={() => setMode(value)}
          >
            {value}
          </Button>
        ))}
      </Inline>
      {mode === "REVIEW_REQUIRED" && (
        <>
          <label htmlFor="publish-approvals">필요 승인 수</label>
          <Textfield
            id="publish-approvals"
            type="number"
            min={1}
            max={20}
            value={approvals}
            onChange={(event) => setApprovals(event.currentTarget.value)}
          />
          <Inline space="space.050">
            {(["ANY_EDITOR", "USERS", "GROUPS"] as const).map((kind) => (
              <Button
                key={kind}
                appearance={reviewerKind === kind ? "primary" : "subtle"}
                onClick={() => setReviewerKind(kind)}
              >
                {kind}
              </Button>
            ))}
          </Inline>
          {reviewerKind !== "ANY_EDITOR" && (
            <Textfield
              aria-label="Reviewer ID 목록"
              placeholder="쉼표로 구분"
              value={reviewerIds}
              onChange={(event) => setReviewerIds(event.currentTarget.value)}
            />
          )}
        </>
      )}
      <Button appearance="primary" onClick={() => save.mutate()} isLoading={save.isPending}>
        발행 정책 저장
      </Button>
      <MutationMessage mutation={save} />
    </Stack>
  );
}

function WritingSettings({ workspaceId }: Readonly<{ workspaceId: string }>) {
  const client = useQueryClient();
  const query = useQuery({
    queryKey: ["writing-config", workspaceId],
    queryFn: ({ signal }) => api.writingConfiguration(workspaceId, signal),
  });
  const save = useMutation({
    mutationFn: () =>
      api.updateWritingConfiguration(workspaceId, query.data?.revision ?? -1, command()),
    onSuccess: async () => client.invalidateQueries({ queryKey: ["writing-config", workspaceId] }),
  });
  if (query.isPending) return <RoutePending />;
  if (query.error) return <Problem error={query.error} retry={() => void query.refetch()} />;
  return (
    <Stack space="space.150">
      <Text>기준 규칙: {query.data.baselineVersion}</Text>
      <Text>이 버전은 닫힌 규칙 registry를 사용하며 임의 override를 허용하지 않습니다.</Text>
      <Button appearance="primary" onClick={() => save.mutate()} isLoading={save.isPending}>
        기준 규칙 확인 저장
      </Button>
      <MutationMessage mutation={save} />
    </Stack>
  );
}

function AISettings({ workspaceId }: Readonly<{ workspaceId: string }>) {
  const client = useQueryClient();
  const config = useQuery({
    queryKey: ["ai-config", workspaceId],
    queryFn: ({ signal }) => api.aiConfiguration(workspaceId, signal),
  });
  const health = useQuery({
    queryKey: ["ai-health", workspaceId],
    queryFn: ({ signal }) => api.aiProviderHealth(workspaceId, signal),
  });
  const now = new Date();
  const from = `${now.getUTCFullYear()}-${String(now.getUTCMonth() + 1).padStart(2, "0")}-01`;
  const to = now.toISOString().slice(0, 10);
  const usage = useQuery({
    queryKey: ["ai-usage", workspaceId, from, to],
    queryFn: ({ signal }) => api.aiUsage(workspaceId, from, to, signal),
  });
  if (config.isPending || health.isPending || usage.isPending) return <RoutePending />;
  if (config.error) return <Problem error={config.error} retry={() => void config.refetch()} />;
  if (health.error) return <Problem error={health.error} retry={() => void health.refetch()} />;
  if (usage.error) return <Problem error={usage.error} retry={() => void usage.refetch()} />;
  return (
    <AIConfigurationForm
      workspaceId={workspaceId}
      initial={config.data}
      health={`${health.data.provider} · ${health.data.status}`}
      usage={`${usage.data.jobCount} jobs · ${usage.data.inputTokens + usage.data.outputTokens} tokens`}
      onSaved={() => client.invalidateQueries({ queryKey: ["ai-config", workspaceId] })}
    />
  );
}

function AIConfigurationForm({
  workspaceId,
  initial,
  health,
  usage,
  onSaved,
}: Readonly<{
  workspaceId: string;
  initial: AIConfiguration;
  health: string;
  usage: string;
  onSaved: () => Promise<unknown>;
}>) {
  const [model, setModel] = useState(initial.model);
  const [userLimit, setUserLimit] = useState(String(initial.userConcurrencyLimit));
  const [workspaceLimit, setWorkspaceLimit] = useState(String(initial.workspaceConcurrencyLimit));
  const [budget, setBudget] = useState(String(initial.monthlyBudgetMicrounits));
  useEffect(() => {
    setModel(initial.model);
    setUserLimit(String(initial.userConcurrencyLimit));
    setWorkspaceLimit(String(initial.workspaceConcurrencyLimit));
    setBudget(String(initial.monthlyBudgetMicrounits));
  }, [initial]);
  const save = useMutation({
    mutationFn: () =>
      api.updateAIConfiguration(
        workspaceId,
        {
          provider: initial.provider,
          model,
          userConcurrencyLimit: Number(userLimit),
          workspaceConcurrencyLimit: Number(workspaceLimit),
          monthlyBudgetMicrounits: Number(budget),
        },
        initial.revision,
        command(),
      ),
    onSuccess: onSaved,
  });
  return (
    <form
      className="settings-form"
      onSubmit={(event) => {
        event.preventDefault();
        save.mutate();
      }}
    >
      <Text>Provider 상태: {health}</Text>
      <Text>이번 달 사용량: {usage}</Text>
      <label htmlFor="ai-model">모델</label>
      <Textfield
        id="ai-model"
        value={model}
        onChange={(event) => setModel(event.currentTarget.value)}
      />
      <label htmlFor="ai-user-limit">사용자 동시 실행</label>
      <Textfield
        id="ai-user-limit"
        type="number"
        value={userLimit}
        onChange={(event) => setUserLimit(event.currentTarget.value)}
      />
      <label htmlFor="ai-workspace-limit">Workspace 동시 실행</label>
      <Textfield
        id="ai-workspace-limit"
        type="number"
        value={workspaceLimit}
        onChange={(event) => setWorkspaceLimit(event.currentTarget.value)}
      />
      <label htmlFor="ai-budget">월 예산 microunits</label>
      <Textfield
        id="ai-budget"
        type="number"
        value={budget}
        onChange={(event) => setBudget(event.currentTarget.value)}
      />
      <Button type="submit" appearance="primary" isLoading={save.isPending}>
        AI 설정 저장
      </Button>
      <MutationMessage mutation={save} />
    </form>
  );
}

function AuditSettings({
  workspaceId,
  initial,
}: Readonly<{ workspaceId: string; initial: SettingsSearch }>) {
  const [action, setAction] = useState(initial.action ?? "");
  const [actorUserId, setActorUserId] = useState(initial.actor ?? "");
  const [targetKind, setTargetKind] = useState(initial.targetKind ?? "");
  const [from, setFrom] = useState(initial.from ?? "");
  const [to, setTo] = useState(initial.to ?? "");
  const [filter, setFilter] = useState({
    action: initial.action ?? "",
    actorUserId: initial.actor ?? "",
    targetKind: initial.targetKind ?? "",
    from: initial.from ?? "",
    to: initial.to ?? "",
  });
  const [selected, setSelected] = useState<string>();
  const query = useQuery({
    queryKey: ["audit", workspaceId, filter],
    queryFn: ({ signal }) =>
      api.auditEvents(workspaceId, undefined, signal, {
        action: filter.action || undefined,
        actorUserId: filter.actorUserId || undefined,
        targetKind: filter.targetKind || undefined,
        from: filter.from || undefined,
        to: filter.to || undefined,
      }),
  });
  if (query.isPending) return <RoutePending />;
  if (query.error) return <Problem error={query.error} retry={() => void query.refetch()} />;
  return (
    <Stack space="space.150">
      <Text>감사 이벤트는 수정할 수 없으며 구조화된 사실만 표시합니다.</Text>
      <form
        className="settings-form"
        onSubmit={(event) => {
          event.preventDefault();
          const next = { action, actorUserId, targetKind, from, to };
          setFilter(next);
          replaceAuditSearch(next);
        }}
      >
        <label htmlFor="audit-action">Action</label>
        <Textfield
          id="audit-action"
          value={action}
          placeholder="DOCUMENT_MOVED"
          onChange={(event) => setAction(event.currentTarget.value.trim().toUpperCase())}
        />
        <label htmlFor="audit-actor">Actor User ID</label>
        <Textfield
          id="audit-actor"
          value={actorUserId}
          onChange={(event) => setActorUserId(event.currentTarget.value)}
        />
        <label htmlFor="audit-target-kind">Target kind</label>
        <Textfield
          id="audit-target-kind"
          value={targetKind}
          placeholder="DOCUMENT"
          onChange={(event) => setTargetKind(event.currentTarget.value.trim().toUpperCase())}
        />
        <label htmlFor="audit-from">시작 시각 (RFC 3339)</label>
        <Textfield
          id="audit-from"
          value={from}
          placeholder="2026-08-01T00:00:00Z"
          onChange={(event) => setFrom(event.currentTarget.value)}
        />
        <label htmlFor="audit-to">종료 시각 (RFC 3339)</label>
        <Textfield
          id="audit-to"
          value={to}
          placeholder="2026-08-25T23:59:59Z"
          onChange={(event) => setTo(event.currentTarget.value)}
        />
        <Inline space="space.050">
          <Button type="submit" appearance="primary">
            필터 적용
          </Button>
          <Button
            onClick={() => {
              setAction("");
              setActorUserId("");
              setTargetKind("");
              setFrom("");
              setTo("");
              setFilter({ action: "", actorUserId: "", targetKind: "", from: "", to: "" });
              replaceAuditSearch({
                action: "",
                actorUserId: "",
                targetKind: "",
                from: "",
                to: "",
              });
            }}
          >
            초기화
          </Button>
        </Inline>
      </form>
      <ul className="settings-list">
        {query.data.items.map((event) => (
          <li key={event.id}>
            <Stack space="space.050">
              <Text weight="semibold">{event.action}</Text>
              <Text>
                {event.actor.kind} · {event.target.kind} {event.target.id}
              </Text>
              <Text size="small">
                sequence {event.sequence} · {event.occurredAt} · {event.correlationId}
              </Text>
              <Button appearance="subtle" onClick={() => setSelected(event.id)}>
                상세 보기
              </Button>
              {selected === event.id && (
                <Stack space="space.050">
                  {event.redactedAt && <Text>redacted {event.redactedAt}</Text>}
                  <AuditFields title="변경 전" fields={event.before} />
                  <AuditFields title="변경 후" fields={event.after} />
                  <AuditFields title="Metadata" fields={event.metadata} />
                </Stack>
              )}
            </Stack>
          </li>
        ))}
      </ul>
    </Stack>
  );
}

function AuditFields({
  title,
  fields,
}: Readonly<{
  title: string;
  fields?: Record<string, string | number | boolean | null> | null;
}>) {
  const entries = Object.entries(fields ?? {});
  if (entries.length === 0) return <Text size="small">{title}: 없음</Text>;
  return (
    <Stack space="space.025">
      <Text weight="semibold">{title}</Text>
      {entries.map(([key, value]) => (
        <Text size="small" key={key}>
          {key}: {String(value)}
        </Text>
      ))}
    </Stack>
  );
}

function ResourceHeading({ title, count }: Readonly<{ title: string; count: number }>) {
  return (
    <Inline space="space.100">
      <h2>{title}</h2>
      <Lozenge>{count}</Lozenge>
    </Inline>
  );
}
function Problem({ error, retry }: Readonly<{ error: Error; retry: () => void }>) {
  const problem =
    error instanceof ApiProblemError ? error.problem : { code: "SETTINGS_QUERY_FAILED" };
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
  if (mutation.isSuccess) return <div role="status">저장했습니다.</div>;
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

export function reviewerRule(
  kind: "ANY_EDITOR" | "USERS" | "GROUPS",
  value: string,
): PublishPolicy["reviewerRule"] {
  const ids = [
    ...new Set(
      value
        .split(",")
        .map((item) => item.trim())
        .filter(Boolean),
    ),
  ];
  if (kind === "USERS") return { kind, userIds: ids };
  if (kind === "GROUPS") return { kind, groupIds: ids };
  return { kind };
}

function replaceAuditSearch(filter: {
  action: string;
  actorUserId: string;
  targetKind: string;
  from: string;
  to: string;
}) {
  const url = new URL(window.location.href);
  const values = {
    action: filter.action,
    actor: filter.actorUserId,
    targetKind: filter.targetKind,
    from: filter.from,
    to: filter.to,
  };
  for (const [key, value] of Object.entries(values)) {
    if (value) url.searchParams.set(key, value);
    else url.searchParams.delete(key);
  }
  window.history.replaceState(null, "", `${url.pathname}${url.search}`);
}
function sectionLabel(section: SettingsSection) {
  return (
    {
      members: "구성원",
      groups: "그룹",
      permissions: "권한",
      writing: "Writing",
      ai: "AI",
      audit: "감사 로그",
    } as Record<SettingsSection, string>
  )[section];
}
