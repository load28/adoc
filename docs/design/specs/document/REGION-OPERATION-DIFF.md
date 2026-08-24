# Region, Operation과 Diff

- **문서 ID**: SPEC-06
- **상태**: 동결

## Region

`BLOCK(blockId)`, `BLOCK_RANGE(startId,endId)`, `SECTION(headingId)`,
`TEXT_RANGE(blockId,fromAnchor,toAnchor,quoteHash)`를 지원한다. text anchor는 주변 text hash와
offset affinity를 함께 가져 편집 후 재위치한다. 해석 불가하면 `ORPHANED`이지 임의 위치로
붙이지 않는다.

## Operation

insertBlock, deleteBlock, moveBlock, replaceText, setBlockAttrs, setMarks, replaceRegion,
addReference, removeReference를 지원한다. 각 Operation은 opId, scope Region, precondition과
payload를 가진다.

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
