# Audit

- **문서 ID**: SPEC-16
- **상태**: 동결

## Event contract

`{id,workspaceId,sequence,actor:{kind,id?},action,target:{kind,id},before?,after?,metadata,
occurredAt,correlationId}`. UI 문장을 저장하지 않는다.

## 대상 action

Workspace·Member·Group·Role, Permission·PublishPolicy, Document create/move/trash/restore/purge,
Draft create, Publish, Discussion create/close/reopen, Review request/decision, Vocabulary change,
File delete, public link create/revoke, AI Proposal apply와 security action.

## 제외

key stroke, autosave heartbeat, panel open, search, AI review 실행 자체는 영구 Audit이 아니다.
보안 access log와 product analytics는 별도 sink다.

## Atomicity

domain command와 같은 PostgreSQL transaction에 append한다. actor display name은 현재 UI에서
resolve하고 event에는 stable ID와 당시 actor kind만 필수로 둔다.

## Redaction

영구 delete 시 target ID·action·actor·time은 보존하고 title, content, email, file name과
before/after sensitive field를 tombstone migration으로 제거한다.
