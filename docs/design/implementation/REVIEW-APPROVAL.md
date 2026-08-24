# Review·Approval 구현 계약

- **문서 ID**: PLAN-19
- **상태**: 구현 기준
- **구현 패키지**: IMP-13

## 1. 책임과 경계

이 패키지는 정확한 Draft revision과 요청 시점 PublishPolicy에 묶인 Review aggregate를 구현한다.
Review 요청·조회·결정·취소, reviewer와 requester Inbox projection, Draft 변경 invalidation, Publish
approval gate를 소유한다. Discussion 내용 작성은 IMP-12, File·Reference·Writing Rule 검증은 각 후속
패키지가 소유하며 미구현 dependency가 실제 content에 있으면 fail closed한다.

## 2. Review snapshot

Review 요청 transaction은 Document → Draft → 기존 active Review 순으로 잠근다. 유효
PublishPolicy가 `REVIEW_REQUIRED`인지 확인한 뒤 다음 canonical JSON을 저장한다.

```text
policySnapshot
├─ sourceDocumentId?
├─ policyRevision
├─ requiredApprovals
├─ reviewerRule
└─ reviewerIds[]        UUID 오름차순, 중복 없음
```

`ANY_EDITOR`는 요청 시점에 Document EDITOR인 active member를, `USERS`와 `GROUPS`는 rule로 선택된
active member 중 Document VIEWER 이상인 사용자를 snapshot한다. 요청자는 reviewer 후보에서 제외한다.
남은 후보가 threshold보다 적으면 Review를 만들지 않는다. Assignment는 snapshot reviewer마다 하나씩
생성하고 이후 정책 변경으로 재할당하지 않는다.

## 3. 상태와 revision

```text
REQUESTED
├─ APPROVED             서로 다른 active approval 수가 threshold 이상
├─ CHANGES_REQUESTED    하나 이상의 current decision이 변경 요청
├─ CANCELLED            requester 또는 current Editor
└─ INVALIDATED          Draft revision·reviewer 자격 상실

APPROVED
├─ REQUESTED            reviewer가 approval을 PENDING으로 바꿔 threshold 미달
├─ CHANGES_REQUESTED    reviewer가 변경 요청
└─ INVALIDATED          Draft revision·reviewer 자격 상실
```

Review `revision`은 aggregate command마다 한 번 증가한다. Assignment current row는 조회 projection이며,
각 결정은 immutable `review_decision_revisions`에 `(review_id, reviewer_id, revision)`으로 append한다.
같은 reviewer의 동일한 현재 결정 재전송은 idempotency replay 외에는 새 command로 기록한다.
Review의 `draft_id`는 삭제 가능한 Draft row에 대한 소유 FK가 아니라 승인 대상을 식별하는 immutable
snapshot 값이다. 따라서 Publish로 Draft row가 제거돼도 Review와 결정 이력은 함께 삭제되지 않는다.
Document당 unique active Review 제약은 `REQUESTED`에만 적용한다.

## 4. Command 계약

- `RequestReview`: CONTRIBUTOR, expected Draft revision, active Review 없음, review-required policy.
- `SubmitDecision`: assigned reviewer 본인, active Membership, target Document VIEWER 이상, expected Review
  revision. `REQUEST_CHANGES`는 같은 Document의 접근 가능한 Discussion ID를 반드시 요구한다.
- `CancelReview`: requester 또는 current Document EDITOR, expected Review revision, REQUESTED 상태.
- `GetReview`: target Document VIEWER 이상. 접근 불가와 다른 Workspace는 같은 `REVIEW_NOT_FOUND`다.

모든 mutation은 Workspace idempotency reservation, aggregate lock, state·revision 검사, current projection,
decision history, Inbox, outbox, idempotency response를 하나의 PostgreSQL transaction에 기록한다.

## 5. Policy 변화와 Publish gate

정책 변경은 기존 snapshot을 수정하지 않는다. 조회 시 snapshot의 policy revision·source와 현재 유효
정책을 비교해 `policyOutdated`를 계산한다. 새 Review 요청은 기존 Review가 APPROVED지만 outdated인 경우
그 Review를 INVALIDATED로 종료한 뒤 새 aggregate를 만든다.

Publish의 `REVIEW_REQUIRED` 경로는 Document → Draft → Review → current Version 순으로 잠근다. 같은
Document·Draft ID·Draft revision의 APPROVED Review가 있고 policy snapshot이 현재 정책과 같으며 모든
승인 reviewer가 여전히 active Membership과 VIEWER 이상인지 재검증한다. 성공하면 전체 Review와 결정
snapshot을 `version_context.review_snapshot_json`에 저장한다. 하나라도 어긋나면 publish는
`PUBLISH_REVIEW_REQUIRED`로 원자적으로 거부한다.

## 6. Inbox와 outbox

- 요청: `review:{reviewId}:{reviewerId}` source key로 reviewer `REVIEW_REQUESTED` item을 upsert한다.
- 결정: requester에게 `review-decision:{reviewId}:{reviewerId}:{assignmentRevision}` item을 만든다.
- 종료: reviewer request item을 resolve하되 read 상태는 바꾸지 않는다.
- 재결정: aggregate가 다시 REQUESTED면 해당 reviewer request item을 reopen한다.

Review aggregate는 `ReviewChanged.v1`, Inbox aggregate는 `InboxChanged.v1`을 commit 전 append한다.
payload는 review ID, document ID, draft revision, aggregate revision, action만 포함하고 문서 내용과
사용자 작성 사유는 포함하지 않는다.

## 7. 동시성·실패 계약

- Draft operation과 Review request가 경쟁하면 같은 Draft lock으로 직렬화하며 request는 잠근 revision만
  snapshot한다.
- Draft mutation은 REQUESTED·APPROVED Review를 잠그고 INVALIDATED·`resolved_at`·revision·Inbox·outbox를
  같은 transaction에서 갱신한다.
- 서로 다른 reviewer 결정은 Review row lock으로 직렬화하고 잠금 뒤 threshold를 다시 계산한다.
- reviewer 자격 상실은 decision command와 publish에서 fail closed하며 Review를 INVALIDATED로 종료한다.
- policy snapshot, assignment, history, Inbox 중 하나라도 실패하면 transaction 전체를 rollback한다.

## 8. 구현·검증 단위

1. Review domain type·threshold reducer와 snapshot validation
2. DDL migration·immutable decision history·canonical OpenAPI
3. Application service·PostgreSQL repository·HTTP route
4. Draft invalidation·Publish review snapshot 연결
5. request/edit race, concurrent threshold, stale revision, policy outdated, permission loss, Inbox dedupe 통합 테스트

완료 gate는 `bun run check`와 Docker PostgreSQL·Redis `bun run compose:integration` 통과다.
