# Permission Resolver

- **문서 ID**: SPEC-02
- **상태**: 동결

## Input·output

```text
resolve(workspaceId, userId, documentId)
→ {access, manage, sourceDocumentId, sourceKind, policyRevision}
```

Membership가 없으면 즉시 NO_ACCESS다. Public Viewer는 이 resolver를 사용하지 않고 별도
PublicLinkScope를 사용한다.

## Algorithm

1. current Document부터 root까지 ancestor path를 만든다.
2. 각 위치에서 User explicit Grant를 찾는다. 있으면 그 결과를 반환한다.
3. User Grant가 없으면 속한 Group Grant를 모은다.
4. 하나라도 NO_ACCESS면 NO_ACCESS를 반환한다.
5. 아니면 최고 access와 그 access에 연결된 manage를 병합한다.
6. 명시적 결과가 없으면 Workspace root default를 사용한다.
7. manage는 access=EDITOR가 아니면 false로 정규화한다.

같은 depth에서 여러 Group 최고 access가 같으면 `manage OR`를 사용한다. source 목록은 설명
UI용으로 보존한다.

## Scope query

Search용 PermissionScope는 같은 resolver 의미를 SQL/OpenSearch filter로 compile한다.
결과 비교 golden test로 point resolve와 scope resolve의 동등성을 검증한다.

## 변경

Grant command는 before/after impact count와 policy revision을 요구한다. commit 후 descendant
cache와 index permission projection을 invalidation event로 갱신한다.
