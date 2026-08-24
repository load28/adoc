# Region, Operation과 Diff

- **문서 ID**: SPEC-06
- **상태**: 동결

## Region

`DOCUMENT`, `BLOCK(blockId)`, `BLOCK_RANGE(startId,endId)`, `SECTION(headingId)`,
`TEXT_RANGE(blockId,fromAnchor,toAnchor,quoteHash)`를 지원한다. text anchor는 주변 text hash와
offset affinity를 함께 가져 편집 후 재위치한다. 해석 불가하면 `ORPHANED`이지 임의 위치로
붙이지 않는다.

`DOCUMENT`는 root children 전체다. `BLOCK_RANGE`의 두 Block은 같은 parent의 연속 구간이어야 한다.
`SECTION`은 heading부터 같은 parent에서 다음 같거나 높은 level heading 직전까지다. text offset은
Browser와 같은 UTF-16 code unit이며 surrogate pair 내부 offset은 유효하지 않다.

## Operation

insertBlock, deleteBlock, moveBlock, replaceText, setBlockAttrs, setMarks, replaceRegion,
addReference, removeReference를 지원한다. 각 Operation은 opId, scope Region, precondition과
payload를 가진다. replaceText는 plain string이 아니라 inline content를 받아 mark와 hard break를
손실 없이 표현한다. Reference 두 연산은 같은 referenceId·sourceRegion·target snapshot을 가져
서로 완전한 역연산이 된다.

## Validation·apply

base Draft revision, schema, scope containment, target existence, permission, File state와
operation dependency DAG를 검증한다. transaction apply는 all-or-nothing이 기본이며 partial
Proposal apply는 독립 component만 새 command로 보낸다.

## Diff

block ID 정렬을 먼저 맞추고 move를 delete+insert와 구분한다. text는 mark-aware diff,
table은 row·cell identity diff를 사용한다. 3-way merge는 base/current/draft Operation을
교차해 disjoint scope만 자동 병합한다.

## Undo

즉시 AI Rewrite는 apply 전에 inverse Operation을 생성한다. Undo는 applied revision이
current이거나 inverse precondition이 여전히 참일 때만 수행한다.
