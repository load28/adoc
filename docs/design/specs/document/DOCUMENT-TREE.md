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

ASCII base-62 고정 32자리 fractional rank와 PostgreSQL `C` collation을 사용한다. 두 anchor 사이
정수 midpoint 공간이 없으면 destination sibling set만 transaction 안에서 균등 rebalance하며 사용자
의미 event·Document revision을 만들지 않는다. concurrent reorder는 새 snapshot으로 최대 3회 재시도한다.

## Trash

command target만 명시적 TRASHED root로 기록하고 subtree는 trashed ancestor CTE로 함께 숨긴다.
중첩된 명시적 TRASHED descendant는 ancestor restore로 복원하지 않는다. restore는 원 parent가
effective active가 아니면 접근 가능한 root destination을 명시하게 한다.

## Public link

tree 이동은 link scope를 넓히지 않는다. Document 자체가 active Published 상태인 동안만
유효하며 trash 즉시 link access를 차단한다.
