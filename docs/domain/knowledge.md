# Knowledge 도메인

## 1. 책임

문서 위치와 독립적인 지식 연결, Workspace 공통 개념 언어, 사용자 Search와 AI가 공유하는
근거 중심 Retrieval을 소유한다.

## 2. Reference

```text
Reference
├─ id
├─ workspaceId
├─ source
├─ target
├─ createdBy
└─ createdAt

ReferenceEndpoint
├─ DOCUMENT
├─ REGION
├─ DISCUSSION
├─ VOCABULARY_CONCEPT
└─ EXTERNAL_RESOURCE
```

Reference는 기본적으로 관계 의미를 강제하지 않는 연결이다. 도메인상 필요한 관계 종류가
검증되기 전 `related_to`, `implements` 같은 임의 taxonomy를 필수 입력으로 만들지 않는다.

### 불변식

- Source와 내부 Target은 같은 Workspace에 속한다.
- Backlink는 Target 기준 Reference 역조회 결과다.
- Target이 이동해도 안정 identity가 같으면 Reference가 유지된다.
- Region을 더 이상 정확히 해석할 수 없으면 상태를 표시하고 임의 위치로 이동하지 않는다.
- External Resource의 원본 내용은 외부 서비스가 정본이다.

## 3. Vocabulary

```text
Concept
├─ id
├─ workspaceId
├─ canonicalTerm
├─ definition
├─ aliases[]
├─ deprecatedTerms[]
├─ status: ACTIVE | DEPRECATED
└─ revision
```

### 불변식

- 같은 Workspace에서 하나의 개념은 하나의 canonical term을 가진다.
- canonical term, alias와 deprecated term의 충돌은 명시적으로 해결한다.
- AI가 Concept을 생성·변경·폐기할 수 없다. 제안만 만들 수 있다.
- Concept 변경은 사용 중인 Reference와 문서 영향을 조회할 수 있어야 한다.
- History는 Published Document Version과 별도의 Concept revision으로 추적한다.

## 4. Knowledge Unit과 Source

Retrieval의 최소 출력은 내용 문자열이 아니라 출처를 보존한 단위다.

```text
KnowledgeUnit
├─ workspaceId
├─ kind
├─ documentId?
├─ publishedVersionId or draftRevision?
├─ region?
├─ content
├─ visibilityEvidence
└─ indexingMetadata
```

AI 결과가 사용한 `Source`는 KnowledgeUnit의 stable identity와 표시용 snapshot을 연결한다.
현재 내용이 바뀌어도 당시 어떤 근거를 사용했는지 재구성할 수 있어야 한다.

## 5. Retrieval Pipeline

```text
User + Workspace
→ Permission Scope
→ 허용된 Knowledge Index
→ lexical + semantic 후보 검색
→ source diversity·freshness·authority ranking
→ KnowledgeUnit[]
→ task-specific Context Selection
```

사용자 검색, Knowledge Question, AI Writing Context와 Discussion 반영은 같은 후보 검색과
권한 계약을 사용한다. 결과 표시와 Context selection만 작업 목적에 따라 달라진다.

## 6. 권위와 충돌

Context의 기본 신뢰 계층은 다음 원칙을 따른다.

1. 현재 요청에서 사용자가 직접 제공한 정보
2. 명시적으로 선택한 내부 Source와 현재 공식 문서
3. Vocabulary의 공식 정의와 토론에서 명시적으로 확정된 내용
4. 관련 내부 지식 검색 결과
5. 외부 Source와 AI 일반 지식

순서만으로 충돌을 자동 해소하지 않는다. 같은 정책을 다르게 말하는 공식 Source가 있으면
둘을 함께 반환하고 충돌을 명시한다. `최신 Version 우선` 같은 상세 규칙도 제품 결정 없이
코드에 넣지 않는다.

## 7. 인덱스 일관성

Knowledge Index는 정본이 아니라 재구축 가능한 projection이다. 원본 Domain event와 현재
상태에서 다시 만들 수 있어야 한다. 인덱스 지연·실패 시 권한을 우회하거나 오래된 내용을
현재 공식 지식으로 조용히 표시하지 않는다.
