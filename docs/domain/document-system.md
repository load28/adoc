# Document System 도메인

- **문서 ID**: DOM-02
- **상태**: 동결
## 1. 책임

Document의 지속되는 identity, 트리 위치, 변경 가능한 Draft, 불변 Published Version과
에디터 Content 모델을 소유한다. Review와 Permission 정책은 소유하지 않고 결과만
입력으로 받는다.

## 2. Aggregate

```text
Document
├─ id
├─ workspaceId
├─ title
├─ parentDocumentId?
├─ position
├─ lifecycle: ACTIVE | TRASHED
├─ currentPublishedVersionId?
└─ draftId?

Draft
├─ id
├─ documentId
├─ basePublishedVersionId?
├─ revision
├─ content
├─ editLease?
└─ saveState

PublishedVersion
├─ id
├─ documentId
├─ versionNumber
├─ contentSnapshot
├─ basedOnVersionId?
├─ publisherId
├─ publishedAt
└─ changeContext
```

Document identity와 Content를 분리한다. 제목·위치가 바뀌고 여러 Version이 생겨도 같은
Document다.

## 3. 상태 전이

```text
Published vN
  → Create Draft(base=vN, revision=1)
  → Edit / Apply Operations(revision + 1)
  → Review(optional, exact revision)
  → Validate base == current published
      ├─ same: Publish
      └─ changed: Conflict → Resolve → 새 revision → Review 재검증
  → Published vN+1
  → Draft 제거
```

첫 Publish 전에는 `basePublishedVersionId`와 current Published가 모두 없을 수 있다.

## 4. 불변식

- 한 Document에는 활성 Draft가 최대 하나다.
- Draft revision은 Content가 의미 있게 바뀔 때 단조 증가한다.
- 한 시점에 하나의 유효한 Edit Lease만 존재한다.
- PublishedVersion의 Content와 change context는 생성 후 변경하지 않는다.
- Version number는 Document 안에서 단조 증가하고 중복되지 않는다.
- Publish는 현재 Draft snapshot 하나를 원자적으로 Version으로 만든다.
- Draft base가 current Published와 다르면 충돌 해결 전 Publish할 수 없다.
- 과거 복원은 해당 snapshot을 기반으로 새 Draft를 만들 뿐 과거 Version을 수정하지 않는다.
- Tree에 cycle이 생길 수 없고 position은 같은 parent 안에서 일관된 순서를 만든다.
- 휴지통 전환은 Version과 Audit 보존 정책을 우회한 실제 삭제가 아니다.

## 5. Document Content

```text
DocumentContent
└─ Block[]
   ├─ stable blockId
   ├─ blockType
   ├─ semantic attributes
   ├─ inline content or child blocks
   └─ references
```

Block Type은 닫힌 전역 분기 하나에 고정하지 않는다. 공통 계약과 type별 schema,
validation, renderer, serializer와 operation handler를 조합해 확장할 수 있어야 한다.

## 6. Region

```text
DocumentRegion
├─ documentId
├─ anchor version or draft revision
├─ start anchor
├─ end anchor
└─ semantic fallback
```

단순 문자 offset만으로 identity를 만들지 않는다. Block identity와 구조적 anchor를 우선하고,
편집 후 정확히 유지할 수 없으면 `RESOLVED`, `MOVED`, `AMBIGUOUS`, `ORPHANED` 같은 해석
상태를 명시해야 한다. 조용히 다른 내용을 가리키면 안 된다.

## 7. Document Operation

모든 직접·AI 변경은 동일한 operation 계약으로 표현할 수 있어야 한다.

```text
DocumentOperation
├─ operationId
├─ targetDocumentId
├─ expectedRevision
├─ target Region or Block
├─ kind
├─ payload
└─ provenance
```

최소 kind 후보는 Block 삽입·삭제·이동·변환, Text Range 교체, 속성 변경과 Reference
연결·해제다. 적용 시 expectedRevision, 권한, schema와 대상 존재를 검증한다. 부분 실패 시
전체 작업의 원자성 계약은 상세 설계에서 작업 종류별로 확정한다.

## 8. Edit Lease와 복구

Lease는 동시 타이핑을 막는 도메인 계약이지 무기한 소유권이 아니다. 만료, 명시적 반환,
연결 종료와 회수 규칙을 상세 설계해야 한다. Lease가 만료돼도 저장된 Draft와 아직 서버에
전달되지 않은 로컬 입력의 복구 가능성을 별도로 다룬다.

## 9. 삭제와 복원

```text
ACTIVE → TRASHED → ACTIVE
                  또는
TRASHED → PURGING → PERMANENTLY_DELETED
```

TRASHED 상태는 30일 동안 복구할 수 있다. 영구 삭제는 child Documents, Published Versions,
Discussions, References, File Assets와 Audit redaction 영향을 먼저 계산한다. purge lease와
step ledger로 재시도하며 UI 한 건의 삭제 동작이 연쇄 삭제를 암묵적으로 결정하지 않는다.
