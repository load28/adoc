# Database Invariant와 Transaction Matrix

- **문서 ID**: DATA-08
- **상태**: 동결
- **정본 DDL**: [schema.sql](schema.sql)

## 책임 규칙

`DB`는 단일 row·key·동일 tenant 참조·불변 이력을 강제한다. `Application`은 권한 계산,
그래프 cycle, JSON Schema와 상태 전이를 강제한다. 둘에 걸친 규칙은 transaction 안에서
application이 검증하고 DB constraint가 가능한 부분을 재검증한다.

## 불변식

| ID | 불변식 | 최종 소유자 | DB 장치 | 실패 코드 |
|---|---|---|---|---|
| DBI-001 | tenant row의 참조는 같은 Workspace만 연결 | DB | composite FK | `DATA_TENANT_MISMATCH` |
| DBI-002 | active Membership은 Workspace·User당 하나 | DB | primary key+status | `MEMBERSHIP_EXISTS` |
| DBI-003 | Workspace에는 active Owner가 최소 한 명 | Application | owner 변경 transaction lock | `LAST_OWNER` |
| DBI-004 | Group Member는 같은 Workspace의 active Member | DB+Application | composite FK+status check | `GROUP_MEMBER_INVALID` |
| DBI-005 | Permission subject는 active User 또는 Group | DB | deferred constraint trigger | `PERMISSION_SUBJECT_INVALID` |
| DBI-006 | `can_manage` grant는 `EDITOR` access만 허용 | DB | check | `PERMISSION_MANAGE_REQUIRES_EDITOR` |
| DBI-007 | Document parent는 같은 Workspace이고 self가 아님 | DB | composite FK+check | `DOCUMENT_PARENT_INVALID` |
| DBI-008 | Document tree에는 cycle이 없음 | Application | recursive CTE under tree lock | `DOCUMENT_TREE_CYCLE` |
| DBI-009 | active sibling rank는 중복되지 않음 | DB | partial unique index | `DOCUMENT_RANK_CONFLICT` |
| DBI-010 | Document당 Draft는 최대 하나 | DB | unique document FK | `DRAFT_EXISTS` |
| DBI-011 | Draft mutation은 expected revision과 일치하고 정확히 +1 | Application | conditional update | `REVISION_CONFLICT` |
| DBI-012 | Edit Lease token·holder·revision이 일치한 편집만 허용 | Application | lease row lock | `EDIT_LEASE_INVALID` |
| DBI-013 | Published Version number는 Document 안에서 단조 증가 | DB+Application | document lock+unique | `VERSION_NUMBER_CONFLICT` |
| DBI-014 | Published Version과 Version Context는 불변 | DB | append-only trigger | `IMMUTABLE_RESOURCE` |
| DBI-015 | current version은 같은 Document에서 publish된 Version | Application | publish transaction | `CURRENT_VERSION_INVALID` |
| DBI-016 | Review Required는 approval 수가 1 이상 | DB | check | `PUBLISH_POLICY_INVALID` |
| DBI-017 | Draft revision 변화는 REQUESTED·APPROVED Review를 INVALIDATED로 변경 | Application | same draft transaction | `REVIEW_INVALIDATION_FAILED` |
| DBI-018 | Document당 active Review(REQUESTED·APPROVED)는 최대 하나 | DB | partial unique index | `REVIEW_ALREADY_OPEN` |
| DBI-019 | Approval은 Review snapshot revision에만 유효 | Application | review+draft lock | `REVIEW_STALE` |
| DBI-020 | Message edit 전 본문은 Revision에 보존 | Application | same transaction | `MESSAGE_HISTORY_FAILED` |
| DBI-021 | Inbox event 재처리는 동일 item을 만들지 않음 | DB | source key unique | `INBOX_DUPLICATE` |
| DBI-022 | active Vocabulary term은 Workspace에서 유일 | DB | unique index | `VOCABULARY_TERM_CONFLICT` |
| DBI-023 | AI Job terminal state에는 completion time이 존재 | DB | check | `AI_JOB_STATE_INVALID` |
| DBI-024 | Proposal은 dependency-closed 선택과 base revision이 일치할 때 한 번만 적용 | Application | proposal→document→draft→lease lock | `PROPOSAL_STALE`, `PROPOSAL_DEPENDENCY_INVALID` |
| DBI-025 | READY File만 새 Reference를 생성할 수 있음 | Application | asset row lock | `FILE_NOT_READY` |
| DBI-026 | 참조 중인 File은 purge하지 않음 | Application | asset+reference lock | `FILE_STILL_REFERENCED` |
| DBI-027 | Audit sequence는 Workspace 안에서 유일·단조 증가 | DB+Application | sequence row lock+unique | `AUDIT_SEQUENCE_CONFLICT` |
| DBI-028 | Audit Event는 불변 | DB | append-only trigger | `IMMUTABLE_RESOURCE` |
| DBI-029 | aggregate event sequence는 유일·단조 증가 | DB+Application | aggregate lock+unique | `EVENT_SEQUENCE_CONFLICT` |
| DBI-030 | 같은 idempotency key는 같은 request hash만 재사용 | Application+DB | primary key+hash compare | `IDEMPOTENCY_KEY_REUSED` |
| DBI-031 | access cache stamp는 Membership·Group·Grant·tree·policy 변화마다 단조 증가 | DB | revision row+trigger | `EVENT_SEQUENCE_CONFLICT` |
| DBI-032 | Job terminal state는 active로 돌아가지 않고 lease owner만 결과를 확정 | DB+Application | conditional transition+terminal check | `JOB_STATE_INVALID` |
| DBI-033 | consumer·Outbox Event당 side effect는 한 번만 의미를 가짐 | DB | consumer receipt primary key | `CONSUMER_DUPLICATE` |
| DBI-034 | Stream sequence는 Workspace에서 유일·단조이고 Event row는 불변 | DB+Application | sequence row lock+unique+update trigger | `STREAM_SEQUENCE_CONFLICT` |
| DBI-035 | Browser Event audience는 producer transaction에서 구조화 | DB | audience kind/id/access check | `EVENT_AUDIENCE_INVALID` |
| DBI-036 | AI Context fingerprint·Source snapshot과 runtime Job은 한 admission에서 결합 | DB+Application | context hash check+AI/runtime Job FK+transaction | `AI_CONTEXT_STALE` |

## Command transaction

| Transaction | Isolation·lock order | 같은 transaction의 write | Commit 이후 |
|---|---|---|---|
| Workspace role 변경 | `READ COMMITTED`; Workspace → memberships by user ID | Membership, Audit, Outbox | permission cache invalidation |
| Document 생성·이동 | `READ COMMITTED`; Workspace → parent → document | Document, Audit, Outbox | tree projection update |
| Draft mutation | `READ COMMITTED`; Document → Draft → Lease → active Review | Draft, Review invalidation, Audit 대상 event, Outbox | SSE revision notification |
| Publish | `READ COMMITTED`; Document → Draft → Review → next Version | Published Version, Context, current version, Draft removal, Review, Audit, Outbox | index·file reference projection |
| Permission 변경 | `READ COMMITTED`; Document ancestry root→leaf → Grant | Grant, Audit, Outbox | scope/index rebuild job |
| Proposal 적용 | `READ COMMITTED`; Proposal → Document → Draft → Lease | Draft, Proposal, Review invalidation, Audit, Outbox | SSE revision notification |
| Trash·restore | `READ COMMITTED`; tree root→descendant | Documents, Audit, Outbox | purge schedule/index tombstone |
| Permanent purge | `SERIALIZABLE`; purge target → references → immutable content | ledger, dependent rows, tombstone Audit·Outbox | object delete·index removal |
| Outbox→Stream delivery | `READ COMMITTED`; Job → Outbox → receipt → Workspace sequence | Stream Event, receipt, Outbox published, Job terminal | Redis Stream wake publish |
| AI Job admission | `REPEATABLE READ`; Workspace → Membership → target → AI configuration | AI Job, Context Source, generic Runtime Job, Outbox | Redis wake signal |
| AI Runtime completion | `READ COMMITTED`; generic Job → AI Job → Result → Usage | AI Result, usage reconciliation, 두 Job terminal, Outbox | user SSE wake |

모든 command는 transaction 시작 전에 idempotency row를 선점한다. PostgreSQL deadlock은 동일
command identity로 최대 3회 재시도한다. expected revision, 권한, validation 실패는 재시도하지
않는다. 외부 API, ObjectStorage, Redis와 OpenSearch 호출은 transaction 안에서 수행하지 않는다.

## 삭제 예외

불변 table 삭제는 `adoc_retention` DB role만 수행한다. 일반 application role에는 해당 role의
상속과 `SET ROLE` 권한을 부여하지 않는다. purge worker는 별도 credential을 사용하고
`purge_ledger`의 대상·시작·완료를 남긴다.
