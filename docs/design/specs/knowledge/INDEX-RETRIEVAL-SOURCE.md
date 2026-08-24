# Index, Retrieval과 Source

- **문서 ID**: SPEC-12
- **상태**: 동결

## Projection document

workspaceId, documentId, version/draftRevision, regionId, kind, title, text, terms, embedding,
permission ancestry fingerprint, updatedAt와 outbox sequence를 가진다. Published와 권한 있는
Draft index를 분리한다.

## Indexing

outbox consumer가 latest aggregate sequence만 적용한다. delete·permission change tombstone이
upsert보다 낮은 sequence면 무시한다. alias swap으로 zero-downtime reindex한다.

## Retrieval

PermissionScope filter → lexical BM25와 vector kNN 후보 → reciprocal rank fusion → kind·freshness·
authority rerank → dedupe → Source 반환. 권한 filter를 post-filter로 적용하지 않는다.

## Source

`{kind, stableId, documentId?, version?, regionId?, authority, snapshotHash, displaySnapshot}`.
AI result 저장 시 Source를 복사하되 현재 content를 복사하지 않는다. 표시할 때 현재 permission을
다시 검사한다.

## Degraded mode

OpenSearch outage 시 title·tree의 제한된 PostgreSQL navigation은 유지하지만 semantic search와
AI retrieval은 unavailable로 표시한다. 품질이 낮은 fallback 답변을 만들지 않는다.
