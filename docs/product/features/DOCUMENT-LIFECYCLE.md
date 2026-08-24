# Document 생명주기 요구사항

- **문서 ID**: PROD-11
- **상태**: 동결

## Tree와 identity

Document는 자유로운 단일 tree에 속한다. 생성, rename, move, reorder, trash와 restore를
지원하며 cycle과 cross-workspace parent를 금지한다. tree는 navigation·상속만 소유하고
의미 관계는 Reference가 소유한다.

## Draft와 저장

- Document당 active Draft는 최대 하나다.
- Draft는 base Published Version과 monotonic revision을 가진다.
- 본문은 한 번에 한 Edit Lease holder만 변경한다.
- 모든 save는 expected revision과 idempotency key를 요구한다.
- network failure 시 Local Recovery Buffer를 유지하고 재연결 후 revision을 비교한다.

## Review와 Publish

- Review는 정확한 revision에 고정된다. 내용 revision 변화는 모든 approval을 무효화한다.
- Publish는 current Published Version과 Draft base를 비교한다.
- base가 stale이면 3-way conflict를 해결하기 전 Publish하지 않는다.
- 성공 transaction은 immutable Version, current pointer, Draft 종료와 outbox event를 함께
  기록한다.

## History와 복원

Version Diff, publisher, publishedAt, change summary, Review snapshot과 관련 Discussion을
조회한다. 과거 복원은 그 snapshot을 base로 새 Draft를 만들며 과거 row를 수정하지 않는다.

## 삭제

trash 후 30일 동안 복구한다. 영구 삭제 전 descendant, Reference, File과 public link 영향을
표시한다. 영구 삭제는 content와 projection을 제거하고 최소 Audit tombstone만 남긴다.
