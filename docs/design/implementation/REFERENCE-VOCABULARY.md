# Reference·Vocabulary 구현 계약

- **문서 ID**: PLAN-20
- **상태**: 구현 기준
- **구현 패키지**: IMP-14

## 1. 책임과 경계

이 패키지는 Workspace 내부 지식 연결과 공통 용어의 정본을 구현한다. Reference 생성·삭제·Backlink,
Vocabulary 조회·생성·수정·폐기, revision·idempotency·outbox를 소유한다. Search projection은 IMP-18,
AI proposal 적용은 IMP-21, 영구 삭제와 Audit projection은 IMP-16이 소유한다.

## 2. Reference aggregate

Reference source는 Draft Document와 정확한 source Region이다. 생성·삭제 command는 source Document
CONTRIBUTOR, active edit lease, expected Draft revision을 요구한다. Reference row 변경과 Draft의
`ADD_REFERENCE`·`REMOVE_REFERENCE` operation은 하나의 transaction과 하나의 Draft revision으로
처리한다. 별도 API와 editor operation은 같은 application command를 호출한다.
삭제는 `deleted_at` tombstone으로 표현해 동일 command replay에 필요한 inverse 입력을 보존하며,
Backlink와 현재 Draft projection에서는 즉시 제외한다. 물리 삭제는 IMP-16이 소유한다.

Target 계약은 다음과 같다.

- `DOCUMENT`: 같은 Workspace의 visible Document UUID
- `REGION`: 같은 Workspace의 visible Document UUID와 target Region
- `DISCUSSION`: source 사용자가 볼 수 있는 Discussion UUID
- `VOCABULARY`: 같은 Workspace의 Concept UUID
- `EXTERNAL`: 정규화된 HTTPS URL

내부 target은 존재와 VIEWER permission을 먼저 검증한다. External URL은 credential·fragment를
금지한다. Snapshot은 표시 title과 canonical SHA-256 hash를 생성 시 저장하며 target 변화로 덮어쓰지
않는다. Reference identity는 source·target 자연키가 아니라 client가 제공한 UUID다.

## 3. Backlink와 비공개성

Backlink query는 target Document VIEWER를 먼저 확인한 뒤, PermissionScope에 포함된 source Document의
Reference만 SQL predicate에서 조회한다. 허용되지 않은 row는 count·cursor 계산 전에 제외한다.
cursor는 `(created_at,id)` keyset이며 다른 target·Workspace cursor는 validation error다.

## 4. Vocabulary aggregate

Concept는 canonical term, definition, terms, status, revision을 가진다. 생성·수정·폐기는 Workspace
ADMIN·OWNER만 수행한다. ACTIVE에서 DEPRECATED로만 전이하며 폐기에는 reason이 필요하다. replacement는
선택 사항이며 같은 Workspace의 다른 ACTIVE Concept만 허용한다.

모든 term은 표시 원문을 보존한다. uniqueness key는 Unicode NFC → Unicode default lowercase → 모든
Unicode whitespace 단일 ASCII space → trim 순으로 만든다. canonical term은 terms에 정확히 하나의
`CANONICAL`로 존재해야 한다. 한 Concept 안의 중복과 Workspace 전체 canonical·synonym·prohibited
충돌을 하나의 unique index로 거부한다.

## 5. History와 transaction

Concept 변경 전 상태는 immutable `vocabulary_concept_revisions`에 append한다. Concept current row와
term rows를 잠그고 expected revision을 검사한 뒤 history append, current projection 교체, outbox,
idempotency response를 한 transaction에 기록한다. Reference와 Vocabulary 모두 target/source 존재를
권한 검사 이후가 아니라 저장 전 동일 transaction에서 잠가 TOCTOU를 막는다.

## 6. API·오류·이벤트

모든 mutation은 CSRF와 idempotency key를 요구한다. Reference mutation은 If-Match와 lease token·client
instance를 함께 요구한다. 주요 오류는 `REFERENCE_NOT_FOUND`, `REFERENCE_TARGET_INVALID`,
`VOCABULARY_NOT_FOUND`, `VOCABULARY_TERM_CONFLICT`, `VOCABULARY_STATE_INVALID`다.

성공 transaction은 `ReferenceChanged.v1` 또는 `VocabularyChanged.v1` outbox를 aggregate sequence와 함께
append한다. payload에는 stable ID·revision·action만 포함하고 definition·external URL은 넣지 않는다.

## 7. 구현·검증 단위

1. Knowledge domain normalization·state reducer
2. canonical DDL·OpenAPI·generated contract
3. Application service·PostgreSQL repository·HTTP route
4. Draft operation·lease·revision 연결과 permission-safe Backlink
5. target permission·term uniqueness·history immutability·tenant isolation 통합 테스트

완료 gate는 `bun run check`와 Docker PostgreSQL·Redis `bun run compose:integration` 통과다.
