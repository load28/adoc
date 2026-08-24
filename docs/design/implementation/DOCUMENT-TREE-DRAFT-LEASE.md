# Document Tree·Draft·Lease 구현 계약

- **문서 ID**: PLAN-16
- **상태**: 구현 기준
- **적용 패키지**: IMP-10

## 책임 경계

`document` domain은 Document metadata·tree position·lifecycle, Draft revision과 Edit Lease 상태 전이를
소유한다. IMP-09 reducer는 Content mutation의 순수 계산만 소유한다. `application`은 IMP-08의
Permission Resolver 결과를 command input으로 받고 PostgreSQL transaction port를 호출한다.
PostgreSQL adapter는 tenant 검증, lock 순서, idempotency, aggregate write와 outbox 원자성을 소유한다.

IMP-10은 Published Version을 읽어 Draft base를 만들 수 있지만 새 Version을 생성하지 않는다.
Reference effect는 reducer 결과에 보존하되 IMP-14가 구현되기 전 ADD/REMOVE_REFERENCE 저장은
`DEPENDENCY_UNAVAILABLE`로 거부한다. Review table이 있으면 Draft mutation·trash에서 active Review를
INVALIDATED로 바꾸며, SSE 전송·Audit projection·purge 실행은 각각 IMP-17·16이 outbox를 소비한다.

## Document와 tree query

Document title은 Unicode trim 뒤 1..500 UTF-8 scalar이며 control character와 bidi override를 거부한다.
Document revision은 title, parent, rank, explicit lifecycle이 바뀔 때 정확히 1 증가한다. current version,
permission revision과 policy revision은 별도 concurrency 축이며 Document revision을 대신하지 않는다.

Workspace마다 `tree_revision bigint`를 둔다. create·rename·move·reorder·trash·restore가 commit될 때
transaction 안에서 정확히 1 증가한다. permission-filtered tree response의 watermark는 이 값이다.
tree query는 ACTIVE row만 먼저 가져온 뒤 필터링하지 않는다. actor의 accessible Document scope를
recursive query 입력에 결합해 접근 불가 node와 hidden child count를 결과 후보에서 만들지 않는다.
접근 가능한 descendant의 ancestor가 접근 불가인 경우 그 descendant도 반환하지 않는다.

## Rank

rank는 ASCII 순서가 값 순서인 alphabet `0-9A-Za-z`, 고정 32자리, PostgreSQL `COLLATE "C"`다.
양 끝은 all-zero·all-z sentinel이며 실제 row에 sentinel을 저장하지 않는다. create·move·reorder는
`afterDocumentId`가 null이면 첫 위치, 값이 있으면 같은 destination parent의 해당 active sibling 바로
뒤를 뜻한다. anchor가 다른 parent·effective trashed·접근 불가면 `DOCUMENT_PARENT_INVALID`다.

두 rank를 base-62 32-digit unsigned 값으로 보고 integer midpoint를 사용한다. 간격이 1이면 destination
parent의 effective active sibling row를 rank 순서로 lock하고 `(i+1) * floor(MAX/(n+1))`에 균등 배치한
뒤 midpoint를 다시 계산한다. rebalance는 Document revision과 tree revision을 바꾸지 않고 outbox를
만들지 않는다. 최종 사용자 command만 target revision과 tree revision을 증가시킨다. unique conflict와
deadlock은 전체 transaction을 새 snapshot으로 최대 3회 재시도한다.

## Create·rename

root create는 active Workspace Member에게 허용하고 생성자에게 해당 Document의 explicit
`EDITOR+manage` grant를 같은 transaction에 bootstrap한다. child create는 parent CONTRIBUTOR 이상을
요구하며 creator bootstrap grant는 동일하다. parent는 ACTIVE이며 effective trashed ancestor가 없어야
한다. empty Content Draft를 자동 생성하지 않는다.

rename은 target CONTRIBUTOR 이상과 expected Document revision을 요구한다. normalize 후 기존 title과
같으면 `NO_EFFECT`다. 생성·rename은 request hash idempotency를 적용하고 `DocumentChanged` outbox에
본문 없이 documentId, action, revision, treeRevision만 기록한다.

## Move preview와 commit

preview는 source EDITOR 이상, destination parent CONTRIBUTOR 이상(root는 active Member), source와
destination의 active ancestry 접근을 요구한다. source·destination이 같은 Workspace인지 확인하고
recursive CTE로 destination ancestry에 source가 없음을 검증한다. source가 effective active가 아니거나
destination이 effective trashed면 실패한다.

preview transaction은 다음 claim의 canonical hash와 random 32-byte token SHA-256을 저장한다.

```text
workspaceId, actorUserId, documentId, expectedDocumentRevision
oldParentId, newParentId, afterDocumentId, beforeDocumentId
sourceAncestryFingerprint, destinationAncestryFingerprint
permissionFingerprint, policyFingerprint, expiresAt(now+5m)
```

응답은 원 token, permission change count, policy change count와 expiresAt만 노출한다. count는 actor가
Manage 권한으로 볼 수 있는 descendant만 포함하며 subject·grant·document title을 포함하지 않는다.

commit은 idempotency row 뒤 preview token hash row를 `FOR UPDATE`하고 actor·workspace·document·expiry,
request destination과 expected revision을 대조한다. Document와 old/new parent를 UUID 순서로 lock한 뒤
cycle, anchors와 모든 fingerprint를 재계산한다. 하나라도 달라지면 `MOVE_PREVIEW_STALE`이고 token은
소비하지 않는다. 성공 시 token을 소비하고 parent·rank·revision, workspace tree revision과
`DocumentMoved` outbox를 원자 반영한다. parent가 같고 rank도 같으면 `NO_EFFECT`다.

## Trash·restore

trash는 target EDITOR, expected Document revision과 reason을 요구한다. target row만 TRASHED로 바꾸고
`trashed_at=server now`, `purge_after=now+30 days`를 저장한다. descendant는 row status를 유지하지만
trashed ancestor CTE 때문에 모든 query·command에서 effective trashed다. subtree의 unexpired Lease를
삭제하고 REQUESTED·APPROVED Review를 INVALIDATED로 바꾼다. `DocumentTrashed` outbox는 root ID와
revision만 담는다.

restore는 명시적 TRASHED root만 허용한다. purge가 시작됐거나 30일이 지났으면 거부한다. 원 parent가
effective active이면 요청 parent가 원 parent와 같을 때 복원할 수 있다. 원 parent가 없거나 effective
trashed이면 caller가 접근 가능한 active destination을 명시해야 한다. target만 ACTIVE로 바꾸고 trash
시간을 제거하며 새 rank를 계산한다. 중첩된 명시적 TRASHED descendant는 복원하지 않는다.

list trash는 명시적 TRASHED root만 `(trashed_at DESC,id DESC)` cursor로 반환한다. permanent purge
endpoint는 IMP-16이 구현하며 IMP-10 handler는 등록하지 않는다.

## Draft

create/get은 Document CONTRIBUTOR 이상과 effective ACTIVE를 요구한다. Document를 lock하고 active
Draft가 있으면 그대로 반환한다. 없으면 current Published Version snapshot 또는 canonical empty Content를
사용해 revision 0 Draft를 만든다. baseVersionId는 snapshot source와 정확히 일치한다. concurrent create는
unique document constraint 승자를 다시 읽어 동일 Draft를 반환한다.

discard는 제품 command로 제공하지 않는다. 임의 Draft 제거는 Review·AI Proposal·local recovery 의미를
잃으므로 Publish(IMP-11) 또는 명시적인 후속 설계 없이는 허용하지 않는다.

Operation save의 If-Match는 Draft revision이다. transaction lock 순서는 Document → Draft → Lease →
active Review다. 다음을 모두 확인한 후에만 IMP-09 reducer를 호출한다.

1. effective ACTIVE, actor CONTRIBUTOR 이상, current permission fingerprint
2. Lease holder user·client instance, token constant-time hash, lease revision과 `expires_at > db now()`
3. Draft expected revision과 모든 Operation `draftRevision`
4. request body canonical hash와 idempotency key

성공 시 normalized Content, schema version, content fingerprint, revision N+1과 updated_by를 한 번 update한다.
REQUESTED·APPROVED Review는 INVALIDATED로 바꾸고 `DraftChanged` outbox에 documentId, N+1,
appliedOperationIds만 기록한다. inverse는 응답에 반환하되 IMP-21의 Undo group 저장 전에는 DB에 별도
저장하지 않는다. reducer 오류·lease 상실·revision conflict는 Draft, Review, outbox를 전혀 변경하지 않는다.

## Edit Lease

Lease TTL은 90초, 권장 heartbeat는 30초이며 모든 비교는 PostgreSQL `clock_timestamp()`를 사용한다.
token은 CSPRNG 32 bytes base64url이고 원문은 acquire·force acquire 응답에서만 전달한다. DB에는 SHA-256만
저장하며 log·outbox·Problem detail에 token·hash를 넣지 않는다.

- acquire: Document CONTRIBUTOR와 Document expected revision을 확인한다. absent·expired면 새 token,
  actor와 clientInstanceId, expiresAt, revision 0으로 upsert한다.
- held by exact actor+client: acquire 재호출은 기존 token을 재노출할 수 없으므로 `EDIT_LEASE_HELD`를
  반환하고 renew endpoint를 사용한다.
- held by other: 일반 acquire는 `EDIT_LEASE_HELD`; force는 target explicit Manage, non-empty reason과
  current Document revision을 요구하고 token을 회전하며 Lease revision을 +1한다.
- renew: If-Match는 Lease revision이다. user·clientInstanceId·token hash가 모두 같고 미만료일 때
  expiresAt을 db now+90초로 바꾸고 revision +1한다. 원 token은 응답에 포함하지 않는다.
- release: 같은 조건에서 `released_at=db now`, expiresAt=db now로 바꾸고 revision +1한다. API에서는
  absent로 보지만 row tombstone은 다음 acquire와 outbox sequence가 단조 증가하도록 유지한다.
  이미 logical absent거나 만료면 `EDIT_LEASE_INVALID`다.

각 상태 변화는 persisted Lease revision+1을 sequence로 `LeaseChanged` outbox에 쓴다. force event에는 이전 holder ID를 넣지 않고 새 holder,
expiry, revision만 넣는다. Membership suspend·remove와 trash는 token 검사 없이 해당 Lease를 삭제할 수
있는 별도 system transition이다.

## API와 오류

모든 cookie-session command는 CSRF와 Idempotency-Key를 요구한다. preview도 token row를 만들기 때문에
CSRF를 요구하지만 사용자 의미 mutation이 아니어서 Idempotency-Key는 요구하지 않는다. If-Match 의미는
endpoint target resource에 따라 Document, Draft, Lease revision으로 고정한다. `clientInstanceId`는
acquire·renew·release·Operation save에 명시적으로 전달해 header token만으로 client identity를 추정하지 않는다.

추가 stable 오류는 `MOVE_PREVIEW_STALE`, `DOCUMENT_EFFECTIVELY_TRASHED`, `DRAFT_EXISTS`,
`EDIT_LEASE_EXPIRED`, `NO_EFFECT`다. 존재를 숨기는 permission failure는 DOCUMENT/DRAFT not found와 같은
404 body를 사용한다. 오류 detail은 currentRevision, leaseExpiresAt, purgeAfter만 allowlist한다.

## PostgreSQL 계약

IMP-10 migration은 다음을 canonical schema와 실제 migration에 동시에 반영한다.

- `documents.rank` 32-char base-62+C collation check
- `workspace_document_revisions(workspace_id PK, tree_revision, updated_at)`
- `document_move_previews(token_hash PK, workspace_id, actor_user_id, document_id, claims_json,
  expires_at, created_at)`과 expiry index
- `edit_leases.client_instance_id uuid NOT NULL`, `released_at timestamptz`; released row는 sequence tombstone

application role은 preview token 원문, lease token 원문을 저장할 수 없다. 모든 tenant reference는 composite
FK를 사용한다. migration은 기존 row가 없다는 전제에 의존하지 않고 기존 rank를 deterministic sibling
순서로 재배치한 뒤 constraint를 검증한다. Draft fingerprint는 canonical Content를 읽을 때 계산하고
mutation write와 API 응답에서 반환하므로 별도 DB 진실 소스를 만들지 않는다.

## 테스트 gate

- rank: 모든 anchor 위치, 32자리 midpoint, full-gap sibling rebalance, permutation determinism
- tree: cross-tenant·cycle·stale preview·anchor drift·permission drift·concurrent move barrier
- trash: ancestor hiding, nested explicit trash, restore destination, lease cleanup, retention boundary
- Draft: concurrent create, stale revision, reducer rollback, idempotent replay, Review invalidation rollback
- Lease: acquire race 단일 승자, expiry/acquire barrier, renew/release/force race, client instance mismatch
- security: permission prefilter, CSRF, token redaction·constant-time compare, request hash reuse
- contract: OpenAPI/generated Rust·TypeScript, migration fresh+upgrade, PostgreSQL 16 integration, root gate
