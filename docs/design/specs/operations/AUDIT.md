# Audit

- **문서 ID**: SPEC-16
- **상태**: 동결

## Event contract

`{id,workspaceId,sequence,actor:{kind,id?},action,target:{kind,id},before?,after?,metadata,
occurredAt,correlationId}`. UI 문장을 저장하지 않는다.

## 대상 action

action은 다음 closed vocabulary를 사용한다.

`WORKSPACE_CREATED`, `WORKSPACE_UPDATED`, `WORKSPACE_DELETION_SCHEDULED`, `WORKSPACE_RESTORED`,
`WORKSPACE_PURGED`, `MEMBER_INVITED`, `MEMBER_ADDED`, `MEMBER_ROLE_CHANGED`, `MEMBER_REMOVED`,
`GROUP_CREATED`, `GROUP_UPDATED`, `GROUP_DELETED`, `GROUP_MEMBER_ADDED`, `GROUP_MEMBER_REMOVED`,
`PERMISSION_CHANGED`, `PUBLISH_POLICY_CHANGED`, `DOCUMENT_CREATED`, `DOCUMENT_RENAMED`,
`DOCUMENT_MOVED`, `DOCUMENT_TRASHED`, `DOCUMENT_RESTORED`, `DOCUMENT_PURGED`, `DRAFT_CREATED`,
`VERSION_PUBLISHED`, `PUBLIC_LINK_CREATED`, `PUBLIC_LINK_REVOKED`, `DISCUSSION_CREATED`,
`DISCUSSION_CLOSED`, `DISCUSSION_REOPENED`, `REVIEW_REQUESTED`, `REVIEW_APPROVED`,
`REVIEW_CHANGES_REQUESTED`, `VOCABULARY_CREATED`, `VOCABULARY_UPDATED`, `VOCABULARY_DEPRECATED`,
`FILE_DELETED`, `AI_PROPOSAL_APPLIED`, `SECURITY_ACTION_RECORDED`.

## 제외

key stroke, autosave heartbeat, panel open, search, AI review 실행 자체는 영구 Audit이 아니다.
보안 access log와 product analytics는 별도 sink다.

## Atomicity

domain command와 같은 PostgreSQL transaction에 append한다. actor display name은 현재 UI에서
resolve하고 event에는 stable ID와 당시 actor kind만 필수로 둔다.

## Redaction

영구 delete 시 target ID·action·actor·time은 보존하고 title, content, email, file name과
before/after sensitive field를 tombstone migration으로 제거한다. retention credential도 Audit row를
삭제할 수 없고 `before`, `after`, `metadata`와 `redactedAt`만 변경할 수 있다.
