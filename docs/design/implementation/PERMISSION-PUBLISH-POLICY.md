# Permission·PublishPolicy 구현 계약

- **문서 ID**: PLAN-14
- **상태**: 구현 기준
- **적용 패키지**: IMP-08

## 책임 경계

`governance` domain은 access 순서, User·Group precedence, Manage와 PublishPolicy 유효성을 소유한다.
application의 `PermissionService`는 하나의 순수 policy compiler로 point와 scope를 계산한다.
PostgreSQL repository는 같은 snapshot의 tree·subject·grant를 공급하고 permission·policy command의
lock·revision·idempotency·outbox를 소유한다. Redis는 point query의 선택적 read-through cache일 뿐
정본이나 authorization 가용성 조건이 아니다.

IMP-08은 기존 Document row와 tree를 읽지만 생성·이동·trash를 구현하지 않는다. IMP-10은 이
서비스의 bootstrap·sensitive point port를 사용한다. public link는 IMP-11, event consumer와 SSE는
IMP-17, OpenSearch scope projection은 IMP-18, 화면은 IMP-22·26이 소유한다.

## Domain model

```text
Access = NO_ACCESS < VIEWER < CONTRIBUTOR < EDITOR

PermissionGrant
  id, document_id, subject(USER|GROUP), access, manage, revision

EffectivePermission
  access, manage, source_document_id?, evidence_grant_ids[]

PermissionScope
  workspace_id, user_id, accessible_document_ids[], fingerprint

PublishPolicy
  document_id, mode(DIRECT|REVIEW_REQUIRED), required_approvals,
  reviewer_rule(ANY_EDITOR|USERS|GROUPS), inherited_from_document_id?, revision
```

`manage=true`는 `access=EDITOR`에서만 유효하다. `NO_ACCESS`는 명시적인 deny이고 grant 부재와
다르다. subject ID와 evidence grant ID는 중복 제거 후 UUID byte 순서로 정렬한다. fingerprint는
compiler version, Workspace access revision, actor Membership revision, group membership과 선택된
evidence를 canonical byte encoding한 SHA-256 lowercase hex다.

## 단일 precedence compiler

Repository는 대상 Document에서 root까지의 경로를 leaf→root 순서로 반환한다. 각 node에는 해당
User grant와 사용자가 속한 active Group grant만 포함한다. compiler는 가장 가까운 node부터 다음을
적용하고 처음 결정된 node에서 멈춘다.

1. User grant가 있으면 access와 manage를 그대로 선택한다.
2. User grant가 없고 Group `NO_ACCESS`가 하나라도 있으면 `NO_ACCESS`, `manage=false`를 선택한다.
3. deny가 없으면 Group grant 중 가장 높은 access를 선택한다. 동률 grant의 evidence를 모두 남기고
   선택 access가 `EDITOR`일 때 해당 동률 grant 중 `manage=true`가 하나라도 있으면 manage다.
4. 적용 가능한 grant가 없으면 부모로 이동한다.
5. root까지 없으면 `NO_ACCESS`, source와 evidence가 없는 결과다.

User grant는 같은 node의 Group deny보다 우선하지만 더 가까운 node의 Group 결정은 먼 조상의 User
grant보다 우선한다. inactive Membership, deleted Group과 inactive Group member는 입력 집합에서
제외한다. compiler는 저장소나 HTTP 타입에 의존하지 않는 순수 함수다.

point와 scope는 같은 compiler를 호출한다. point는 한 ancestry snapshot을, scope는 Workspace의
`PURGING`이 아닌 Document·grant·group snapshot을 한 번에 읽어 parent-before-child로 계산한다.
scope의 각 Document 결과는 같은 snapshot의 point 결과와 byte-for-byte 같아야 한다. scope는
`access != NO_ACCESS`인 정확한 Document ID 집합이며 downstream repository는 이 집합을 query 입력에
포함한다. 결과 조회 후 권한 필터링은 금지한다.

## Action capability

- `VIEWER`: Published content와 유효 PublishPolicy query
- `CONTRIBUTOR`: VIEWER + Draft read, Discussion 참여, Review 요청
- `EDITOR`: CONTRIBUTOR + edit·AI proposal 적용·Publish command 진입
- `manage`: Permission·PublishPolicy 변경과 다른 subject explanation

Workspace ADMIN·OWNER도 content access와 manage를 우회하지 않는다. reviewer는 Draft를 읽어야 하므로
승인 시점에 최소 CONTRIBUTOR가 필요하다. `ANY_EDITOR`는 effective EDITOR인 active Member,
`USERS`는 지정 active User 중 effective CONTRIBUTOR 이상, `GROUPS`는 지정 active Group의 Member 중
effective CONTRIBUTOR 이상을 후보로 만든다. policy 저장 시 subject 존재와 현재 후보 수를 검사하고
Review 요청·결정 시 IMP-13이 다시 계산한다.

## Permission query와 explanation

`getDocumentPermissions`는 actor의 sensitive point 결과가 manage일 때만 explicit current-node grant와
actor effective result를 반환한다. `explainEffectivePermission`은 subject USER가 actor 자신이면
actor가 대상에 최소 VIEWER인 경우 허용한다. 다른 User나 Group 설명은 actor manage가 필요하다.
권한이 없으면 tenant와 Document 존재를 구분하지 않는 `DOCUMENT_NOT_FOUND`를 반환한다.

explanation step은 leaf→root 순서다. 선택 node 전에는 `NO_GRANT`, 선택 node에는
`USER_GRANT|GROUP_DENY|GROUP_MAX`, 그 뒤 조상에는 `INHERITED`를 기록한다. Effective 결과에는 실제
evidence ID만 포함하며 title·Group membership·다른 grant payload는 설명 응답에 노출하지 않는다.

## Permission command와 last manager

set·delete는 다음 lock 순서를 사용한다.

```text
Workspace access revision → Document ancestry root→leaf → affected subtree parent→child
→ actor Membership → subject row → grants by document,subject → idempotency receipt
```

actor 권한은 cache 없이 transaction snapshot에서 재계산한다. `If-Match`는 target Document의
`permission_revision`과 일치해야 한다. set은 path의 `grantId`를 identity로 사용한다. 이미 존재하는
ID가 다른 document·subject를 가리키면 tenant-safe not-found이며 같은 document·subject의 다른 ID가
있으면 stable conflict다. User subject는 active Membership, Group subject는 active Group이어야 한다.

변경을 메모리 snapshot에 먼저 적용해 target을 root로 하는 `PURGING` 아닌 전체 subtree를 다시
계산한다. 단 하나의 Document라도 effective manager인 active User가 0명이 되면
`PERMISSION_LAST_MANAGER`로 전체 transaction을 취소한다. Group manager grant는 active Group member를
User로 확장해 판정한다. IMP-10의 Document 생성 transaction만 creator `EDITOR+manage` bootstrap
method를 사용하며 일반 API의 manage·last-manager 검사를 우회할 수 없다.

성공 시 grant와 Document `permission_revision`, Workspace `permission_revision`, outbox
`PermissionChanged.v1`을 같은 transaction에 반영한다. event는 affected root, Workspace revision과
before·after grant를 포함한다. Audit 정본 projection은 IMP-16이 이 event를 소비한다.

## PublishPolicy 상속과 command

가장 가까운 explicit policy override를 leaf→root로 선택한다. override가 없으면 Workspace
`default_publish_mode`를 사용한다. 기본 DIRECT 결과는 approvals 0, `ANY_EDITOR`, inherited source null이다.
응답 `revision`은 source row revision이 아니라 target Document의 `policy_revision`이다. 따라서 다른
조상의 변경과 무관하게 target command의 optimistic concurrency 의미가 안정적이다.

DIRECT는 `required_approvals=0`, reviewer rule `ANY_EDITOR`만 허용한다. REVIEW_REQUIRED는 approvals
1..20이고 reviewer rule로 계산한 현재 distinct 후보 수보다 클 수 없다. USERS·GROUPS ID는
1..100개이며 정렬·중복 제거한다. 모든 subject는 같은 Workspace의 active entity여야 한다.

set은 sensitive manage point와 target policy revision을 transaction에서 확인한다. explicit row를
upsert하고 target `policy_revision`, Workspace `policy_revision`, `PublishPolicyChanged.v1` outbox를
원자 반영한다. 정책 override 제거 API는 현재 계약에 없으므로 지원하지 않는다. IMP-10 tree 이동은
Workspace permission·policy revision을 모두 증가시켜 상속 cache와 projection을 무효화한다.

## Revision stamp와 Redis cache

`workspace_access_revisions(workspace_id, permission_revision, policy_revision)`가 cache와 projection의
정본 stamp다. DB trigger가 Membership·Group member·PermissionGrant 변화에 permission revision을,
Document tree·status 변화에는 두 revision을, PublishPolicy 변화에는 policy revision을 증가시킨다.
Document row의 `permission_revision`과 `policy_revision`은 해당 API command의 local If-Match다.

point cache key는 다음 값의 versioned encoding이다.

```text
adoc:permission:v1:{workspace}:{user}:{document}:
  {workspacePermissionRevision}:{workspacePolicyRevision}:{membershipRevision}
```

value는 EffectivePermission과 fingerprint만 포함하고 TTL은 5분이다. cache hit도 schema·fingerprint를
검증한다. miss, timeout, 연결 실패, decode 실패와 stale key는 PostgreSQL resolver로 fallback한다.
cache 오류는 access를 허용하거나 `DEPENDENCY_UNAVAILABLE`로 바꾸지 않는다. sensitive command와
scope compiler는 Redis를 사용하지 않는다. revision이 바뀌면 새 key만 읽으므로 consumer 지연 중에도
권한 확대가 발생하지 않으며 구 key는 TTL로 제거된다.

## Idempotency·오류·HTTP

permission·policy command는 기존 `(workspace,actor,operation,key)` receipt와 canonical request hash를
사용한다. replay는 최초 status·response를 반환하고 hash가 다르면 `IDEMPOTENCY_KEY_REUSED`다.
constraint·domain 오류는 다음 stable problem으로 변환한다.

- tenant·Document 비노출: `DOCUMENT_NOT_FOUND` 404
- manage 없음: query에서는 비노출 404, visible command action 부족은 `PERMISSION_DENIED` 403
- subject·manage/access·policy 입력: 422 stable code
- revision·grant identity·last manager: 409 stable code
- PostgreSQL 불가: `DEPENDENCY_UNAVAILABLE` 503, 내부 원문 비노출

IMP-08은 `getDocumentPermissions`, `setDocumentPermission`, `deleteDocumentPermission`,
`explainEffectivePermission`, `getPublishPolicy`, `setPublishPolicy`를 연결한다. 모든 command는 shared
CSRF·exact Origin·Idempotency·If-Match validator를 사용한다. generated Rust·TypeScript type 외의
임의 wire model을 만들지 않는다.

## 검증 gate

- domain property: precedence 전체 matrix, UUID 순서 무관성, point/scope equivalence
- PostgreSQL 16: cross-tenant subject, deferred constraint, revision race, subtree last manager,
  Workspace stamp·outbox·receipt atomicity
- Redis: valid hit, revision miss, corrupt value, unavailable fallback, TTL
- HTTP: self/manager explanation, IDOR 404, 모든 command의 CSRF·Origin·If-Match와 safe problem metadata
- contract: OpenAPI generated Rust·TypeScript와 negative corpus
- repository root gate와 실제 PostgreSQL·Redis Compose integration
