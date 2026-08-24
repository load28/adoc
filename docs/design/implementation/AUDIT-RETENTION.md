# Audit·Retention 구현 계약

- **문서 ID**: PLAN-22
- **상태**: 구현 기준
- **구현 패키지**: IMP-16

## 1. 책임과 경계

Operations domain은 Audit Event와 Purge Run을 소유한다. Audit은 domain command가 발생시킨 중요한 사실의
구조화된 불변 기록이다. access log·analytics·키 입력·autosave heartbeat는 Audit이 아니다. Retention은
Document·Workspace의 접근 차단, domain row 제거, ObjectStorage byte 제거와 최소 tombstone을 단계적으로
조정한다. OpenSearch와 backup의 실제 삭제 실행은 각 adapter가 구현되는 후속 패키지가 소유하되 purge
outbox와 ledger 경계는 지금 고정한다.

## 2. Audit event와 append

`AuditEventInput`은 `workspace_id`, `actor`, `action`, `target`, optional `before/after`, scalar-only `metadata`,
`occurred_at`, `correlation_id`를 가진다. actor는 USER 또는 SYSTEM이며 USER에는 stable user ID가 필수다.
target은 kind와 stable ID를 가진다. action은 임의 UI 문장이 아니라 Operations domain의 closed vocabulary다.

`append_audit_event(tx,input)`은 `workspace_sequences` row를 `FOR UPDATE`로 잠그고 현재 sequence를 할당한 뒤
1 증가시킨다. event ID는 UUIDv7이고 같은 transaction에 insert한다. sequence·insert 실패는 caller mutation과
함께 rollback한다. before/after에는 상태·role·parent·policy처럼 판정에 필요한 비민감 구조만 저장한다.
title·content·email·filename·token·checksum은 금지한다.

영구 action vocabulary의 정본은 SPEC-16이다. 현재 존재하는 command는 해당 action을 즉시 연결하고, 아직
구현되지 않은 AI Proposal과 security command는 각 후속 태스크에서 같은 primitive를 사용한다. 고빈도 Draft
autosave·lease·Inbox read는 제외한다.

## 3. Audit query와 redaction

`GET /workspaces/{workspaceId}/audit-events`는 active ADMIN·OWNER만 호출한다. cursor는 `(sequence,id)`를
서명하지 않은 base64url 내부 표현으로 전달하되 tenant와 권한을 먼저 확인하고 descending keyset pagination을
사용한다. page size는 50, 최대 100이다. response는 actor·action·target·before·after·metadata·occurredAt·
correlationId를 반환한다. 존재하지 않거나 권한 없는 Workspace는 같은 not-found 의미를 사용한다.

Purge redaction은 Audit row를 삭제하지 않는다. retention credential이 target의 title/content/email/file name과
민감 before/after/metadata key를 빈 tombstone 구조로 교체하고 `redacted_at`을 기록한다. sequence·actor stable
ID·action·target ID·time·correlation은 유지한다. append-only trigger는 retention role의 redaction update만
허용하고 일반 update/delete를 거부한다.

## 4. Purge aggregate와 단계

`purge_ledger`는 target `(kind,id)`당 하나이며 `status`, `step`, `attempt`, `lease_owner`, `lease_until`,
`last_error_code`, `started_at`, `updated_at`, `completed_at`, `result_hash`를 가진다. status는 PENDING, RUNNING,
RETRY, COMPLETED다. worker claim은 PENDING·RETRY와 만료된 RUNNING을 `FOR UPDATE SKIP LOCKED`로 잡는다.

단계는 다음 단조 순서를 따른다.

1. `ACCESS_REVOKED`: target을 PURGING으로 바꾸고 Public link·Session·queued Job 접근을 차단한다.
2. `OBJECTS_CAPTURED`: 삭제될 storage key를 `purge_object_deletions`에 idempotent insert한다.
3. `DOMAIN_PURGED`: retention transaction으로 dependent domain row와 immutable history를 제거한다.
4. `OBJECTS_PURGED`: key별 ObjectStorage delete를 완료한다. 없는 byte도 성공이다.
5. `AUDIT_REDACTED`: target Audit의 민감 구조를 tombstone으로 바꾼다.
6. `COMPLETED`: result hash와 완료 시각을 기록하고 PURGE_CHANGED outbox를 남긴다.

각 단계 commit 뒤 다음 단계로 이동한다. 실패는 attempt를 증가시키고 low-cardinality error code와 backoff
시각을 기록한다. side effect와 단계 전이는 멱등해야 한다.

## 5. Document purge

명시적 purge와 retention worker 모두 TRASHED root, `purge_after <= now`, expected revision, subtree 안의 별도
trash root를 포함한 exact subtree를 잠근다. claim transaction이 target을 PURGING으로 바꾼 뒤에는 Restore가
`DOCUMENT_STATE_INVALID`로 실패한다. 30일 전 요청은 `PURGE_NOT_ELIGIBLE`이다.

Object capture는 subtree의 Draft·PublishedVersion·Message가 참조한 FileAsset 중 purge 뒤 reference 0이 되는
asset만 대상으로 한다. Domain purge는 reference·review·discussion·draft·version·permission·policy·public link·
document를 FK 순서로 제거한다. 다른 owner reference가 남은 FileAsset과 byte는 보존한다. root Audit은
DOCUMENT_PURGED tombstone으로 남는다.

## 6. Workspace purge

Workspace는 DELETION_SCHEDULED이고 `delete_after <= now`일 때 claim한다. ACCESS_REVOKED에서 PURGING으로
바꾸고 queued/cancel-requested Job을 cancel한다. Session은 사용자 전역이므로 revoke하지 않고 모든 tenant
query가 Workspace status를 선검사해 접근을 차단한다. 모든 tenant storage key를 capture한 뒤 tenant domain row를 제거한다. Workspace와 Audit minimum은 hard-delete하지 않고 Workspace를 DELETED로,
Membership은 REMOVED로 축소해 deletion ledger의 FK anchor를 유지한다. 사용자 identity는 다른 Workspace가
참조할 수 있으므로 삭제하지 않는다.

Workspace 삭제 취소는 claim 전만 허용한다. PURGING 이후에는 부분 복구하지 않는다. backup restore 절차는
COMPLETED ledger를 먼저 적용해야 하며 TASK-023은 이를 검증 가능한 ledger 결과 hash로 제공한다.

## 7. Port·worker·권한

`AuditRepository`는 list만 public application port로 제공한다. append는 PostgreSQL transaction primitive여서
domain repository transaction 안에서만 호출한다. `RetentionRepository`는 claim/advance/fail만 worker
application service에 제공한다. ObjectStorage port는 PLAN-21을 재사용한다.

API process는 일반 DB credential만 사용한다. worker는 일반 connection과 별도 retention connection을 받는다.
production은 retention URL의 `current_user=adoc_retention`을 preflight에서 강제한다. development/test에서는
명시적 superuser fixture와 retention transaction marker를 함께 요구한다. marker가 없는 일반 transaction은
superuser fixture에서도 immutable history 변경을 거부한다. API에서 임의 purge step advance나 Audit mutation
endpoint를 제공하지 않는다.

## 8. 검증

완료 gate는 concurrent Audit append의 gap 없는 sequence, 일반 role update/delete 차단, domain command rollback
원자성, admin-only cursor query, Restore 대 claim barrier, step crash/resume, shared File reference 보존, byte delete
retry, Workspace tombstone, `bun run check`, `bun run compose:integration`이다.
