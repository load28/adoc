# Index, Retrieval과 Source

- **문서 ID**: SPEC-12
- **상태**: 동결

## Projection document

workspaceId, documentId, version/draftRevision, regionId, kind, title, text, terms, embedding,
permission ancestry fingerprint, updatedAt와 outbox sequence를 가진다. Published와 권한 있는
Draft index를 분리한다.

## Indexing

outbox consumer가 Workspace별 단조 증가 `projectionSequence`를 external version으로
적용한다. consumer는 event snapshot을 바로 인덱싱하지 않고 PostgreSQL의 현재 상태를
다시 materialize한다. 따라서 늦게 도착한 upsert event도 최신 삭제·권한 상태를
되살리지 못한다. 권한·tree 변경은 하위 트리 전체를 재materialize하고 alias
swap으로 zero-downtime reindex한다.

## Retrieval

PermissionScope filter → lexical BM25와 vector kNN 후보 → reciprocal rank fusion → kind·freshness·
authority rerank → dedupe → Source 반환. 권한 filter를 post-filter로 적용하지 않는다.

scope pair는 composite `permissionKey`로 compile하고 4,096개 단위로 분할한다. 각 batch의
후보를 modality별 global top 100으로 다시 줄인 뒤 RRF를 수행하므로 scope 크기에 따라
OpenSearch clause 한도를 바꾸거나 검색 가능한 Document 수를 제한하지 않는다. query vector가
없는 caller는 lexical modality만 실행하며, Hybrid caller는 mapping dimension과 정확히 같은
vector를 제공해야 한다.

## Source

`{kind, stableId, documentId, version?, draftRevision?, regionId, authority, snapshotHash,
displaySnapshot}`.
AI result 저장 시 Source를 복사하되 현재 content를 복사하지 않는다. 표시할 때 현재 permission을
다시 검사한다.

## Degraded mode

OpenSearch outage 시 title·tree의 제한된 PostgreSQL navigation은 유지하지만 semantic search와
AI retrieval은 unavailable로 표시한다. 품질이 낮은 fallback 답변을 만들지 않는다.
