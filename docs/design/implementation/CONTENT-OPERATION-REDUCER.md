# Content·Operation reducer 구현 계약

- **문서 ID**: PLAN-15
- **상태**: 구현 기준
- **적용 패키지**: IMP-09

## 책임 경계

`document` domain은 version 1 Content의 semantic validation, Region resolve, Operation batch 검증,
순수 apply와 inverse 생성을 소유한다. 같은 규칙을 `packages/editor-schema`가 TypeScript로 구현하고
두 구현은 같은 canonical fixture를 실행한다. JSON Schema는 wire shape의 정본이며 domain validator는
Schema로 표현할 수 없는 ID·tree·mark·limit 불변식을 보강한다.

IMP-09는 Draft·Lease·권한·transaction을 알지 않는다. 호출자는 Content, base revision, operation batch와
현재 Reference snapshot을 전달하고 성공 결과를 한 번 저장한다. IMP-12는 permission·lease·revision lock 뒤 reducer를 호출한다.
IMP-14는 reducer가 반환한 Reference effect를 같은 transaction에 반영한다. IMP-15는 resulting Content의
asset reference set을 검사한다. IMP-21·23은 이 공개 계약 외 별도 mutation 의미를 만들지 않는다.

## Content model과 semantic validation

Content는 `schemaVersion=1`과 root children으로 구성한다. 모든 `id` 보유 node는 Content 전체에서
중복되지 않는다. node 종류별 허용 child는 CONTRACT-01과 같으며 다음 추가 불변식을 적용한다.

- 최대 node 수 50,000, container depth 32, 전체 logical text UTF-8 10 MiB
- table은 모든 row의 effective column 수가 같고 colspan·rowspan이 grid를 겹치거나 범위를 넘지 않음
- orderedList만 `start`, taskList의 direct item만 boolean `checked`를 가지며 나머지는 해당 attr 부재
- mark는 canonical kind 순서이며 kind별 최대 하나, subscript와 superscript는 동시에 존재하지 않음
- link는 `https`, `http`, `mailto`만 허용하고 credential·control character가 없음
- asset ID는 Content에서 존재만 검증하며 READY·tenant 검증은 IMP-15 port가 소유

wire object의 원본 key 순서나 text node 분할은 의미가 아니다. reducer는 인접한 동일 mark text를
합치고 빈 text를 제거하며 mark를 canonical 순서로 정렬한다. hardBreak는 유지한다. 이 normalize 뒤
동일한 Content는 같은 canonical JSON과 SHA-256 lowercase fingerprint를 가진다.

## Region 해석

```text
DOCUMENT                         root children 전체
BLOCK(id)                        정확한 node 하나
BLOCK_RANGE(start,end)           같은 parent의 start..=end 연속 children
SECTION(heading)                 heading부터 다음 sibling의 level<=heading.level 직전
TEXT_RANGE(block,from,to,quote)  한 text-bearing node의 inline slice
```

text-bearing node는 paragraph·heading·toggle summary다. codeBlock은 mark가 없는 별도 text field이므로
TEXT_RANGE 대상이 아니며 `SET_BLOCK_ATTRS`로 text를 바꿀 수 없다. inline logical text는 text 값과
hardBreak의 `\n`을 이어 붙인다. offset은 UTF-16 code unit이고 surrogate pair 내부, from>to,
Content 끝 초과는 invalid다. `contextHash`는 anchor 앞뒤 각 최대 32 UTF-16 code unit을
`before + 0x00 + after`로 합친 SHA-256이고 `quoteHash`는 선택 logical text의 SHA-256이다.

현재 exact Draft revision에 적용할 때 anchor offset·context·quote가 모두 맞아야 `RESOLVED`다.
재위치는 별도 `reanchor` 함수만 수행한다. 같은 block의 offset ±256 UTF-16 범위에서 exact quote와
하나 이상의 contextHash가 일치하는 후보를 찾고, 두 context가 일치하는 후보를 우선한다. 최고 점수
후보 1개는 `MOVED`, 0개는 `ORPHANED`, 복수는 `AMBIGUOUS`다. apply는
`RESOLVED`만 받고 reanchor 결과를 조용히 적용하지 않는다.

## Batch graph와 공통 precondition

batch size는 1..500이다. 모든 opId는 서로 다르고 `dependsOn`은 같은 batch의 opId만 가리킨다.
Kahn topological sort는 ready opId의 UUID byte 순서를 사용해 입력 순서와 무관한 유일한 실행 순서를
만든다. cycle·self dependency·missing dependency는 batch 전체 validation 실패다.

모든 `precondition.draftRevision`은 호출자의 base revision과 같아야 한다. operation의 `scope`는
payload target과 다음처럼 일치해야 한다.

- insert: parent null이면 DOCUMENT, 아니면 BLOCK(parentId)
- delete·move·set attrs: BLOCK(blockId)
- replace text·set marks: payload range와 byte-for-byte 동일한 TEXT_RANGE
- replace region: payload region과 동일하며 DOCUMENT·BLOCK·BLOCK_RANGE·SECTION만 허용
- add reference: sourceRegion과 동일, remove reference: 저장 snapshot의 sourceRegion과 동일

`targetHash`가 있으면 operation 직전 intermediate Content에서 resolve한 scope의 canonical hash와
일치해야 한다. Reference remove는 입력 snapshot hash를 사용한다. 하나라도 불일치하면
`PRECONDITION_FAILED`이며 원본 Content·effect 집합을 전혀 바꾸지 않는다.

## Operation 의미

- INSERT_BLOCK: parent가 허용하는 child index에 새 subtree를 삽입한다. payload node는 Block뿐 아니라
  listItem·tableRow·tableCell을 포함하며 기존 ID와 하나라도 겹치면 실패다.
- DELETE_BLOCK: subtree를 제거한다. root나 존재하지 않는 node는 대상이 될 수 없다.
- MOVE_BLOCK: subtree identity를 유지해 새 parent/index로 옮긴다. 자기 descendant 이동과 invalid child를
  거부한다. 같은 parent 이동 index는 제거 후 배열 기준이다.
- REPLACE_TEXT: exact inline slice를 supplied inline content로 교체하고 normalize한다.
- SET_BLOCK_ATTRS: type별 mutable allowlist만 patch한다. `SET(value)`와 `REMOVE`를 구분하며
  id·type·children·text·items·rows·cells는 금지다.
- SET_MARKS: text node 경계를 split해 ADD·REMOVE·REPLACE 후 normalize한다. hardBreak에는 mark를 붙이지 않는다.
- REPLACE_REGION: resolved block sequence를 supplied blocks로 원자 교체한다. TEXT_RANGE는 허용하지 않는다.
- ADD_REFERENCE·REMOVE_REFERENCE: 입력 Reference snapshot에 absent/present exact match를 검사하고
  Content를 변경하지 않은 채 typed Reference effect를 반환한다.

각 operation 적용 직후 affected subtree의 local shape를, batch 끝에는 전체 Content semantic contract를
검증한다. batch 성공 시 Content가 실제로 달라졌거나 Reference effect가 하나 이상이어야 한다.
완전히 동일한 mutation은 `NO_EFFECT`로 거부해 Draft revision의 의미 있는 증가 불변식을 지킨다.

## Inverse와 Undo group

reducer는 각 operation 직전 snapshot으로 inverse를 만든 뒤 최종 실행 순서의 역순으로 반환한다.
inverse opId는 UUIDv5 namespace `ad0c0000-0000-5000-8000-000000000009`에
`forwardOpId + ":inverse"` UTF-8을 넣어 결정적으로 만든다. inverse의 draftRevision은 base+1이며
dependsOn은 직전 inverse opId를 가리켜 실행 순서를 고정한다. targetHash는 forward 결과에서 inverse를
순서대로 dry-run하며 각 inverse가 실행되기 직전의 scope hash로 계산한다.

insert↔delete, move↔원래 parent/index, replace text↔이전 inline slice, attrs↔이전 attr 값·부재,
marks↔이전 inline slice, replace region↔이전 blocks, add reference↔remove reference다. attr inverse는
기존 값이 있으면 SET, 없으면 REMOVE를 사용해 null과 부재를 혼동하지 않는다. inverse batch는 forward 결과와 revision base+1에 적용했을 때 normalize된 원본과
같아야 한다. 이후 변경으로 hash가 달라지면 Undo는 실패하며 임의 rebase하지 않는다.

## 결과와 오류

```text
ReducerResult
  content
  contentFingerprint
  appliedOperationIds[]
  inverseOperations[]
  referenceEffects[]

OperationError
  SCHEMA_INVALID | CONTENT_INVALID | BATCH_INVALID | DEPENDENCY_INVALID
  REGION_NOT_FOUND | REGION_AMBIGUOUS | PRECONDITION_FAILED
  TARGET_CONFLICT | NO_EFFECT | LIMIT_EXCEEDED
```

오류는 stable category와 실패 opId만 노출하고 Content·quote·target payload를 포함하지 않는다.
domain apply는 panic·부분 결과·fallback을 반환하지 않는다. Rust와 TypeScript는 같은 fixture에서
result fingerprint, applied ID 순서, inverse와 error category가 정확히 같아야 한다.

## 검증 gate

- CONTRACT-01·02 positive·negative corpus의 Rust·TypeScript 동일 판정
- 9 kind와 5 Region의 허용·금지 matrix, UTF-16 한글·emoji boundary와 reanchor 상태
- dependency graph permutation·cycle·missing property
- apply determinism, atomic rollback, normalize idempotency, apply→inverse round trip property
- duplicate ID·depth·node·text·table grid·mark·URL limit negative corpus
- generated Rust·TypeScript 계약, repository root gate
