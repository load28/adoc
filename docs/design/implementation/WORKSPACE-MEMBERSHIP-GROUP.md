# Workspace·Membership·Group 구현 계약

- **문서 ID**: PLAN-13
- **상태**: 구현 기준
- **적용 패키지**: IMP-07

## 책임 경계

`governance` domain은 Workspace, Membership, Invitation과 Group의 값·상태 전이·불변식을
소유한다. application service는 인증된 `SessionPrincipal`과 repository·clock·random·invitation
key port를 조합한다. PostgreSQL은 tenant transaction과 idempotency·outbox를 소유하고 Axum은
UUID·header·cookie·CSRF·problem response 변환만 담당한다.

IMP-07은 Workspace 수준의 role과 관리 capability까지만 구현한다. Document access와
PublishPolicy는 IMP-08, invitation delivery worker는 IMP-17의 durable job runner, purge 실행과
Audit 정본은 IMP-16이 소유한다. 이 태스크는 이후 consumer가 사용할 Membership revision과
outbox event를 원자적으로 남긴다.

## Domain model

```text
Workspace
  id, slug, name, status, default_publish_mode, revision,
  created_by, created_at, updated_at, delete_after

Membership
  workspace_id, user_id, role(MEMBER|ADMIN|OWNER),
  status(ACTIVE|SUSPENDED|REMOVED), revision, joined_at, removed_at

Invitation
  id, workspace_id, normalized_email, role(MEMBER|ADMIN),
  token_hash, token_key_id, expires_at, accepted_at, revoked_at, revision

Group
  id, workspace_id, name, member_ids, revision, deleted_at
```

`WorkspaceName`과 `GroupName`은 trim 뒤 1~200 Unicode scalar이고 slug는
`^[a-z0-9][a-z0-9-]{1,62}$`를 만족한다. slug는 name을 ASCII 소문자·hyphen으로 정규화하고
의미 있는 문자가 없으면 `workspace`를 사용한다. 충돌은 무작위 suffix를 붙여 재시도하되
DB unique constraint가 최종 판정한다. email은 검증 후 Unicode case-fold가 아니라 Google verified
email과 같은 ASCII lowercase contract를 사용한다.

상태 전이는 다음만 허용한다.

```text
Workspace: ACTIVE → DELETION_SCHEDULED → ACTIVE
                         └──────────────→ PURGING → DELETED  (IMP-16)
Membership: ACTIVE ↔ SUSPENDED, ACTIVE|SUSPENDED → REMOVED
Invitation: PENDING → ACCEPTED | REVOKED | EXPIRED
Group: ACTIVE → DELETED
```

삭제된 Membership과 Group을 되살리지 않는다. 같은 사용자를 다시 초대하면 새 Membership
identity를 만들지 않고 기존 `(workspace_id,user_id)` row를 ACTIVE로 전환하며 joined/revision을
갱신한다. 삭제된 Group 이름은 재사용할 수 있다.

## Capability와 정보 노출

- 모든 query와 command는 active Membership을 transaction 입구에서 확인한다.
- MEMBER는 Workspace·Member·Group query를 사용할 수 있다.
- ADMIN은 Workspace 수정, invitation과 Group, MEMBER·ADMIN 제거를 관리한다.
- OWNER는 ADMIN capability에 더해 Owner 승격·강등과 Workspace 삭제 예약·취소를 수행한다.
- ADMIN은 OWNER의 role을 바꾸거나 제거할 수 없다.
- 대상 Workspace나 Membership을 볼 수 없는 actor에게는 `WORKSPACE_NOT_FOUND`를 반환해 존재를
  구분하지 않는다.
- Workspace role은 Document content access를 부여하지 않는다.

role 변경·제거는 actor와 target Membership을 같은 transaction에서 잠근다. Owner 수는
`SELECT ... WHERE role='OWNER' AND status='ACTIVE' FOR UPDATE`로 직렬화한다. 결과 active Owner가
0이면 `LAST_OWNER`로 전체 command를 취소한다. 자기 자신을 강등·제거하는 것도 같은 규칙으로
허용한다.

## Invitation capability

초대 token은 다음 48-byte payload의 base64url 표현이다.

```text
invitation UUID(16) || HMAC-SHA256(token_hash_pepper,
  "adoc-invitation-v1" || invitation_id || workspace_id || email || expires_at)(32)
```

DB에는 token 전체의 SHA-256 hash와 `token_key_id`만 저장한다. 범용 token pepper 안에서도
`adoc-invitation-v1` label로 protocol을 분리하며 current key로만 발급한다. 수락은 저장된 token
hash를 constant-time으로 비교하므로 key rotation과 독립적이다. delivery 재생성은 저장된 key ID의
current·previous key를 사용하며 previous key 유지 기간은 invitation TTL 7일보다 길어야 한다.
원문 token은 log·analytics·API resource·outbox에 저장하지 않는다.

초대 생성 transaction은 invitation과 `InvitationDeliveryRequested.v1` outbox event를 함께
commit한다. event에는 invitation ID만 포함한다. IMP-17 worker는 row를 읽고 같은 key ID로 link를
재생성해 `Mailer` port에 invitation ID를 delivery idempotency key로 전달한다. 따라서 DB commit과
외부 mail 사이 장애에도 token 원문 저장이나 유실 없이 재시도할 수 있다.

수락 transaction은 token hash row와 현재 사용자를 잠근 뒤 다음 순서로 처리한다.

1. pending·미만료·미폐기와 verified email 일치를 검사한다.
2. 불일치 계정이면 token을 소비하지 않고 `INVITATION_INVALID`를 반환한다.
3. Membership을 ACTIVE로 insert 또는 재활성화한다.
4. invitation을 ACCEPTED로 전이하고 outbox event를 append한다.
5. 같은 token 재호출은 기존 Membership을 반환하는 멱등 성공으로 처리한다.

## Group invariant

Group 생성 입력의 member ID는 중복 제거 후 정렬한다. 모든 member는 같은 Workspace의 active
Membership이어야 하며 하나라도 아니면 전체 command를 `GROUP_MEMBER_INVALID`로 취소한다.
추가·제거는 Group row를 잠그고 expected revision을 확인한 뒤 membership set과 Group revision을
같은 transaction에서 변경한다.

Group 삭제는 active PermissionGrant가 있으면 `GROUP_IN_USE`와 reference count를 반환한다.
그 외에는 `deleted_at`과 revision을 갱신하고 group_members를 제거한다. 물리 삭제하지 않아
Audit·event consumer가 identity를 추적할 수 있다.

## Repository transaction과 idempotency

```text
create_workspace:
  user → workspace slug candidate → workspace → owner membership → outbox

change_membership:
  workspace → actor membership → target membership → active owners → sessions → outbox

write_invitation:
  workspace → actor membership → invitation/email → outbox

write_group:
  workspace → actor membership → group → member IDs ascending → grants → outbox
```

모든 command는 canonical request hash와 `(workspace, actor, operation, idempotency key)` receipt를
사용한다. Workspace 생성은 생성 전 workspace ID가 없으므로 제안 UUID를 idempotency namespace로
쓰지 않는다. 별도 `user_command_receipts`의 `(user,createWorkspace,key)`를 사용해 재시도 시 같은
Workspace를 반환한다. invitation accept도 같은 user receipt를 사용한다. Workspace 내부 command는
기존 `idempotency_keys`를 사용한다.

expected revision은 정확히 일치해야 한다. Group member 추가·제거도 Group revision을 요구한다.
idempotency replay는 최초 status와 response를 그대로 반환하고 다른 request hash면
`IDEMPOTENCY_KEY_REUSED`다.

## Session·cache invalidation

role 변경, suspend와 remove는 target user의 active session을 같은 transaction에서 모두 revoke한다.
outbox의 `MembershipChanged.v1`에는 before·after role/status와 revision을 포함한다. IMP-08 cache
consumer는 이 revision으로 stale permission entry를 무효화한다. revoke와 event 중 하나만 성공하는
상태를 허용하지 않는다.

Workspace 삭제 예약은 상태를 `DELETION_SCHEDULED`, `delete_after=now+30d`로 바꾸고 모든 신규
invitation·group mutation을 차단한다. 기존 Member의 read는 유예 기간에 유지한다. 취소는 purge
lease가 시작되기 전 ACTIVE로 되돌린다. 실제 접근 차단·purge는 IMP-16 transaction이 수행한다.

## HTTP 계약

IMP-07은 OpenAPI의 다음 operation을 모두 연결한다.

```text
listWorkspaces, createWorkspace, getWorkspace, updateWorkspace,
scheduleWorkspaceDeletion, cancelWorkspaceDeletion,
listMembers, inviteMember, listInvitations, revokeInvitation, acceptInvitation,
updateMemberRole, removeMember,
listGroups, createGroup, getGroup, updateGroup, deleteGroup,
addGroupMember, removeGroupMember
```

모든 command는 `X-CSRF-Token`과 exact Origin 검증을 요구한다. create와 accept는 `If-Match`가 없고
나머지 mutable target command는 quoted decimal `If-Match`를 요구한다. `204` 응답에는 body를
넣지 않는다. JSON은 OpenAPI generated type과 동일한 camelCase를 사용한다.

오류 mapping은 `AUTH_REQUIRED` 401, 권한 부족 403, tenant-safe not found 404, revision·last Owner·
state·name conflict 409, invalid member 422, dependency failure 503이다. PostgreSQL constraint 이름을
안정적인 domain error로 mapping하고 provider·SQL 원문은 노출하지 않는다.

## 검증 gate

- domain unit: 이름·slug·role capability·상태 전이·invitation token negative corpus
- application fake: idempotent replay, email mismatch non-consume, last Owner, role authority, tenant denial
- PostgreSQL 16: concurrent last Owner change, slug/name collision, invitation one-shot, group member tenant
  mismatch, revision race, session revoke와 outbox atomicity
- HTTP security: 모든 command의 missing/wrong CSRF·Origin, invalid UUID·If-Match·key, IDOR 404
- contract: OpenAPI generated Rust·TypeScript와 response corpus
- repository root gate와 실제 Compose integration
