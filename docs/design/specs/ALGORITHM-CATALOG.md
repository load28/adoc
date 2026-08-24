# Algorithm Catalog

- **문서 ID**: SPEC-19
- **상태**: 동결

## ALG-001 Effective Permission

```text
resolve(actor, document):
  require ACTIVE membership
  groups := active groups containing actor
  for node in document -> ancestors ordered nearest first:
    grants := grants(node, actor or groups)
    if user grant exists: return user grant
    if any group NO_ACCESS: return NO_ACCESS,false,node
    if group grants exist:
      access := max(access)
      manage := access == EDITOR and any selected EDITOR grant.manage
      return access,manage,node
  return NO_ACCESS,false,null
```

resolver input에는 ancestry revision과 Group membership revision을 포함한다. cache key도 그
fingerprint를 포함하며 Permission·Group·move event가 fingerprint를 무효화한다.

## ALG-002 Document move·rank

transaction에서 Document와 old/new parent를 UUID 순으로 lock한다. recursive CTE로 new parent의
ancestor에 target이 있는지 검사한다. preview token의 document revision, new parent, sibling
anchors, permission/policy fingerprints와 expiry를 검증한다. rank는 C collation의 고정 32자리
base-62 두 anchor 사이 midpoint를 만들고 공간이 없을 때 같은 parent sibling만 deterministic rebalance한다. unique
충돌은 transaction을 다시 읽어 최대 3회 재계산한다.

## ALG-003 Operation apply

```text
validate batch schema and unique opId
topologically sort dependsOn; reject cycle/missing dependency
lock Document -> Draft -> Lease -> active Review
require permission, lease token, expected draft revision
for operation in order:
  resolve Region anchor against current intermediate content
  require targetHash/precondition
  apply pure content reducer; validate Content Schema+limits
persist content and revision N+1 once
invalidate active Review; write outbox/audit; commit
```

batch는 전부 성공하거나 전부 실패한다. 같은 idempotency key의 request hash가 같으면 저장된
response를 반환하고 reducer를 다시 실행하지 않는다.

## ALG-004 Publish

Document→Draft→Review→current Version 순으로 lock한다. current Version이 Draft base와 같은지,
READY File reference, Reference validity, blocking Writing Rule, lease, Effective EDITOR와 Publish
Policy를 검증한다. REVIEW_REQUIRED면 exact Draft revision의 APPROVED Review snapshot을 요구한다.
next number Version·Context·file reference를 삽입하고 Document current_version을 교체한 뒤 Draft를
제거한다. VersionPublished, Audit와 Inbox outbox를 같은 transaction에 쓴다.

## ALG-005 Review threshold

policy snapshot의 reviewer set과 required count를 사용한다. current assignments 중 APPROVED인
서로 다른 active reviewer 수를 센다. 하나라도 CHANGES_REQUESTED면 Review를 즉시 해당 상태로
종료한다. threshold 이상이면 APPROVED, 그 외 REQUESTED다. Membership 제거와 Draft change는
기존 결정 재할당 대신 INVALIDATED를 만든다.

## ALG-006 Proposal partial apply

선택 Operation마다 모든 transitive dependency가 선택됐는지 검사한다. 선택하지 않은 operation이
선택 operation을 필요로 하는 것은 허용한다. base revision과 각 targetHash를 현재 Draft에
검증한다. 하나라도 stale이면 전체 선택을 적용하지 않는다. 성공하면 한 Draft revision·undo
group으로 기록하고 남은 operation은 새 base로 자동 rebase하지 않아 Proposal을 APPLIED 또는
STALE로 종료한다.

## ALG-007 Search retrieval

Workspace와 permission scope filter를 BM25·kNN 모두에 먼저 적용한다. 각 top 100 결과의 rank로
`score += 1/(60+rank)`를 계산한다. configured authority·exact term·freshness weight를 더하고
Document/Region/snapshotHash로 dedupe해 top 30을 반환한다. projection fingerprint 불일치는
제외하고 reindex job을 만든다.

## ALG-008 File reference·GC

Content 또는 Message commit에서 JSON Schema가 허용한 File ID를 추출해 owner reference set을
old/new diff로 갱신한다. Published Version reference는 retention purge만 제거한다. reference가
0이 된 READY asset은 즉시 byte를 지우지 않고 DELETED와 purge_after를 기록한다. purge worker는
row lock 뒤 reference 0을 다시 확인하고 ObjectStorage delete 성공 후 ledger를 완료한다.

## ALG-009 Outbox·consumer

aggregate mutation과 sequence+1 Outbox insert를 같은 PostgreSQL transaction에 둔다. publisher는
`FOR UPDATE SKIP LOCKED`로 claim하고 Redis는 wake-up에만 사용한다. consumer는 `(consumer,eventId)`
receipt를 먼저 선점하고 aggregate sequence가 현재 watermark보다 클 때만 projection을 적용한다.
실패는 backoff+jitter, max attempts 후 DEAD_LETTER다.

## ALG-010 AI Context·Result

명시적 user input→current target→Vocabulary→confirmed Discussion→permission-filtered retrieval→
task opt-in web 순으로 source budget을 배정한다. 조직 사실 claim은 Source ID가 없으면
`INSUFFICIENT_CONTEXT`다. provider output은 JSON parse, AI Result Schema, Operation Schema,
Content dry-run, hard Writing Rule와 Source coverage를 순서대로 검증한다. 검증 실패를 plain text나
다른 model 결과로 대체하지 않는다.

## ALG-011 Region re-anchor

Region은 block ID를 먼저 찾는다. ID가 유지되면 text offset 주변의 contextHash와 quoteHash를
검증하고 offset drift 범위 안에서 exact quote를 탐색한다. 후보가 하나면 anchor를 이동하고,
0개면 `ORPHANED`, 2개 이상이면 `AMBIGUOUS`로 만든다. block이 삭제됐으면 같은 Section 안의
이전·다음 block snapshotHash를 이용해 후보를 제한한다. fuzzy text similarity만으로 자동
연결하지 않으며 Discussion·Reference는 orphan 상태로 보존해 사람이 다시 지정한다.

## ALG-012 Three-way merge

base Published Version, current Published Version, Draft를 block ID sequence로 비교한다. 한쪽만
바꾼 block은 자동 채택하고 양쪽이 같은 canonical content hash로 바뀌면 하나로 합친다. 서로
다른 text range가 겹치지 않고 mark boundary가 같으면 Operation 순서로 합친다. delete-vs-edit,
move-vs-move, 같은 attr의 다른 값, table 구조 변화와 overlapping text는 conflict다. 결과는 새
Draft revision으로만 저장하고 conflict는 `base/current/draft` snapshot과 선택지를 가진다. AI
merge는 동일 conflict input으로 Proposal을 만들 뿐 자동 선택하거나 적용하지 않는다.
