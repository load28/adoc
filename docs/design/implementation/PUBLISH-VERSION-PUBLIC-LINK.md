# Publish·Version·Public Link 구현 계약

- **문서 ID**: PLAN-17
- **상태**: 구현 기준
- **구현 패키지**: IMP-11

## 책임과 경계

Document domain은 PublishedVersion의 불변 snapshot, Document별 단조 version number, Draft base
충돌과 restore 의미를 소유한다. Application은 권한·PublishPolicy·Lease 결과를 조합한다.
PostgreSQL adapter는 Version 생성, current pointer 전환, Draft 종료, context와 outbox를 하나의
transaction으로 보장한다.

Review 승인 계산은 IMP-13이다. REVIEW_REQUIRED 정책은 이 태스크에서 fail closed다. File asset의
READY·exact reference 검증과 public byte delivery는 IMP-15다. 현재 content에 File owner가 없으면
일반 문서만 발행할 수 있으며 파일 검증을 성공으로 추정하지 않는다.

## PublishedVersion 계약

```text
PublishedVersion
  id, documentId, number
  content, schemaVersion, contentFingerprint
  basedOnVersionId?
  publisherId, summary, publishedAt
  sourceDraftRevision
  reviewSnapshot, discussionIds
```

Version과 VersionContext는 생성 뒤 UPDATE·DELETE를 거부한다. `content_fingerprint`와
`based_on_version_id`를 PublishedVersion row에 저장해 snapshot identity와 3-way base를 별도 JSON
해석 없이 비교한다. `(document_id,number)`와 `(workspace_id,id)`는 유일하다.

## Publish command

입력은 expected Draft revision, idempotency key, 1~1000자의 trim된 summary와 선택적인
`clientInstanceId + leaseToken` 쌍이다. 둘 중 하나만 전달하면 validation 실패다.

transaction lock 순서는 Workspace idempotency → Document → Draft → EditLease → effective
PublishPolicy → Review snapshot이다. source EDITOR와 effective active를 current row에서 재검증한다.
Draft `base_version_id`와 Document `current_version_id`가 다르면 `PUBLISH_BASE_STALE`이며
`baseVersionId`, `currentVersionId`, `draftId`를 안전한 conflict metadata로 반환한다.

unexpired active Lease가 있으면 actor·client·constant-time token hash가 모두 일치해야 한다. Lease가
없거나 expired/released면 token 없이 발행할 수 있다. DIRECT policy만 진행한다. Content schema와
fingerprint를 재검증하고 지원되지 않는 File·Reference owner가 있으면 dependency 오류로 거부한다.

성공 시 `max(number)+1`을 Document lock 아래 계산하고 Version·empty review context를 insert한다.
Document current pointer와 revision을 증가시키고 Draft를 삭제하며 Lease를 logical release한다.
`VersionPublished.v1`과 `DocumentChanged.v1` outbox, idempotency response를 같은 transaction에 기록한다.

## History·detail·diff

history와 Version detail은 current Document VIEWER scope를 먼저 적용한다. history cursor는
`(number DESC,id DESC)`이며 50개 제한이다. Version detail은 snapshot·context를 반환한다.

diff는 두 Version이 같은 workspace·Document인지 확인한 뒤 IMP-09의 stable node ID와
`DocumentOperation[]`을 단일 diff 계약으로 사용한다. 최소 보장은 `DOCUMENT` scope의
`REPLACE_REGION`으로 root block snapshot을 교체하는 구조 diff이며, 문자열 diff나 별도
change 언어를 만들지 않는다. 동일 Version 또는 동일 fingerprint는 empty operations다.

## 과거 Version 복원

restore command는 EDITOR, effective active, expected Document revision과 idempotency key를 요구한다.
Document를 lock하고 active Draft가 없음을 확인한다. 선택 Version content를 current schema validator로
읽고 `base_version_id=selectedVersionId`, revision 0인 새 Draft를 만든다. current Version pointer와 과거
Version은 변경하지 않는다. 현재 Published와 다른 snapshot에서 시작하므로 Publish 전 3-way conflict
해결이 필요하며 이를 조용히 current base로 바꾸지 않는다.

## Public capability

list/create/revoke는 Document EDITOR+Manage와 current Published 존재를 요구한다. create 입력의
`expiresAt`은 server time보다 미래이고 최대 365일이다. 원 token은 CSPRNG 32 bytes base64url이며
SHA-256만 저장한다. idempotency replay는 최초 token을 다시 노출하지 않으므로 replay response는 같은
command ledger에 암호화 없는 원문을 저장하지 않고 `PUBLIC_LINK_TOKEN_ALREADY_ISSUED`로 fail closed한다.

anonymous route는 session middleware와 분리한다. token 형식 검증 뒤 hash lookup 한 번으로 revoked,
expired, Document status와 current Version을 검사한다. 모든 실패는 동일 404와 동일 Problem body다.
응답은 title과 current Version content·number·publishedAt만 포함한다. Workspace id·name·Document id,
tree, permission, author email과 history를 포함하지 않는다. Publish 직후 같은 link는 새 current Version을
읽는다. trash·revoke·expiry는 즉시 차단한다.

asset route는 IMP-15 전까지 등록하지 않는다. 이후 `PublicLinkScope{documentId,currentVersionId,
allowedAssetIds}`를 snapshot content에서 materialize하고 exact asset만 전달한다.

## 동시성·실패·관측성

- concurrent Publish: Document lock 뒤 한 command만 next number를 생성하고 나머지는 stale revision이다.
- Publish vs save: Draft row revision과 Lease lock으로 snapshot 경계를 직렬화한다.
- Publish vs trash: Document lock 승자가 상태·revision을 결정하고 패자는 conflict다.
- public read vs revoke/publish: 한 DB statement의 MVCC snapshot으로 old 또는 new의 완전한 상태만 읽는다.
- 모든 command는 request hash idempotency와 transaction outbox를 사용한다.
- metric은 outcome code만 기록하고 token·content·summary·title을 label/log에 넣지 않는다.

## 검증 계약

- DB trigger로 Version·context UPDATE/DELETE가 거부된다.
- barrier race로 concurrent Publish 단일 승자와 중복 number 부재를 검증한다.
- stale base, stale Draft revision, foreign tenant Version, active foreign Lease를 거부한다.
- restore가 Version/current pointer를 바꾸지 않고 active Draft와 충돌하는지 검증한다.
- token 원문 비저장, revoke·expiry·trash·unpublished 동일 404, latest pointer 전환을 검증한다.
