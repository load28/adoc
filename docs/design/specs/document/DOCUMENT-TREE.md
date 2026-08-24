# Document Tree

- **문서 ID**: SPEC-04
- **상태**: 동결

## Commands

`CreateDocument(parentId?, title, rankAfter?)`, `RenameDocument`, `MoveDocument`,
`ReorderDocument`, `TrashDocument`, `RestoreDocument`, `PurgeDocument`.

## Move transaction

same Workspace, parent active, no cycle를 recursive CTE로 검사한다. source·destination sibling
rank를 계산하고 Permission·PublishPolicy before/after impact를 preview token으로 먼저 만든다.
commit은 token hash와 expected document revision을 요구한다.

## Ordering

lexicographic fractional rank를 사용한다. rank 길이 threshold를 넘으면 sibling set만
background rebalance하며 사용자 의미 event를 만들지 않는다. concurrent reorder는 expected
parent revision conflict로 재시도한다.

## Trash

subtree는 논리적으로 함께 숨기되 각 child status를 즉시 rewrite하지 않고 trashed ancestor를
query한다. restore는 원 parent가 없으면 접근 가능한 root destination을 명시하게 한다.

## Public link

tree 이동은 link scope를 넓히지 않는다. Document 자체가 active Published 상태인 동안만
유효하며 trash 즉시 link access를 차단한다.
