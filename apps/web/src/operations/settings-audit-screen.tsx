import {
  ApiClient,
  ApiProblemError,
  type AIConfiguration,
  type Group,
  type Invitation,
  type Membership,
  type SettingsSection,
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
}: Readonly<{ workspaceId: string; section: SettingsSection; documentId?: string }>) {
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
        {section === "audit" && <AuditSettings workspaceId={workspaceId} />}
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
  const update = useMutation({
    mutationFn: () => api.updateGroup(workspaceId, group, name.trim(), command()),
    onSuccess: async () => client.invalidateQueries({ queryKey: ["groups", workspaceId] }),
  });
  const remove = useMutation({
    mutationFn: () => api.deleteGroup(workspaceId, group, command()),
    onSuccess: async () => client.invalidateQueries({ queryKey: ["groups", workspaceId] }),
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
  if (!documentId) return <Text>URL의 document query로 관리할 문서를 선택해 주세요.</Text>;
  if (query.isPending) return <RoutePending />;
  if (query.error) return <Problem error={query.error} retry={() => void query.refetch()} />;
  return (
    <Stack space="space.150">
      <Text>
        유효 권한: {query.data.effective.access}
        {query.data.effective.manage ? " · 관리 가능" : ""}
      </Text>
      <ul className="settings-list">
        {query.data.explicitGrants.map((grant) => (
          <li key={grant.id}>
            <Text>
              {grant.subjectKind} · {grant.subjectId}
            </Text>
            <Lozenge>{grant.access}</Lozenge>
          </li>
        ))}
      </ul>
      <PermissionForm
        workspaceId={workspaceId}
        documentId={documentId}
        revision={query.data.revision}
      />
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
  const [access, setAccess] = useState<"VIEWER" | "CONTRIBUTOR" | "EDITOR">("VIEWER");
  const save = useMutation({
    mutationFn: () =>
      api.setDocumentPermission(
        workspaceId,
        documentId,
        crypto.randomUUID(),
        revision,
        { subjectKind: "USER", subjectId, access, manage: false },
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
      <label htmlFor="permission-user">사용자 ID</label>
      <Textfield
        id="permission-user"
        value={subjectId}
        onChange={(event) => setSubjectId(event.currentTarget.value)}
      />
      <Inline space="space.050" shouldWrap>
        {(["VIEWER", "CONTRIBUTOR", "EDITOR"] as const).map((item) => (
          <Button
            key={item}
            appearance={access === item ? "primary" : "subtle"}
            onClick={() => setAccess(item)}
          >
            {item}
          </Button>
        ))}
      </Inline>
      <Button type="submit" appearance="primary" isLoading={save.isPending}>
        권한 추가
      </Button>
      <MutationMessage mutation={save} />
    </form>
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

function AuditSettings({ workspaceId }: Readonly<{ workspaceId: string }>) {
  const query = useQuery({
    queryKey: ["audit", workspaceId],
    queryFn: ({ signal }) => api.auditEvents(workspaceId, undefined, signal),
  });
  if (query.isPending) return <RoutePending />;
  if (query.error) return <Problem error={query.error} retry={() => void query.refetch()} />;
  return (
    <Stack space="space.150">
      <Text>감사 이벤트는 수정할 수 없으며 구조화된 사실만 표시합니다.</Text>
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
            </Stack>
          </li>
        ))}
      </ul>
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
