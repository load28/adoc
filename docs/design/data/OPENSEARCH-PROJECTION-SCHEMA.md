# OpenSearch Projection Schema

- **문서 ID**: DATA-09
- **상태**: 동결
- **정본 정책**: [Index, Retrieval과 Source](../specs/knowledge/INDEX-RETRIEVAL-SOURCE.md)

## Index와 alias

물리 index는 `adoc-published-v{schema}-{generation}`과 `adoc-draft-v{schema}-{generation}`을
사용한다. 읽기 alias는 `adoc-published-read`, `adoc-draft-read`이고 쓰기 alias는
`adoc-published-write`, `adoc-draft-write`다. rebuild catch-up 중에는 기존·신규 generation을
모두 갱신하고, 검증 후 두 alias 쌍을 하나의 `_aliases` 요청으로 교체한다.
Workspace별 index를 만들지 않고 routing에 `workspace_id`를 사용하며 모든 query에
`workspace_id` filter를 먼저 적용한다.

## Mapping

```json
{
  "settings": {
    "index": { "number_of_shards": 3, "number_of_replicas": 1 },
    "analysis": {
      "analyzer": {
        "adoc_ko_en": {
          "type": "custom", "tokenizer": "nori_tokenizer",
          "filter": ["lowercase", "nori_readingform"]
        }
      }
    }
  },
  "mappings": {
    "dynamic": "strict",
    "properties": {
      "projection_schema": { "type": "integer" },
      "workspace_id": { "type": "keyword" },
      "document_id": { "type": "keyword" },
      "document_status": { "type": "keyword" },
      "source_kind": { "type": "keyword" },
      "source_revision": { "type": "long" },
      "version_number": { "type": "long" },
      "region_id": { "type": "keyword" },
      "region_kind": { "type": "keyword" },
      "ancestor_ids": { "type": "keyword" },
      "title": { "type": "text", "analyzer": "adoc_ko_en", "fields": { "raw": { "type": "keyword", "ignore_above": 500 } } },
      "body": { "type": "text", "analyzer": "adoc_ko_en" },
      "terms": { "type": "keyword" },
      "embedding": { "type": "knn_vector", "dimension": 1536, "method": { "name": "hnsw", "space_type": "cosinesimil", "engine": "lucene" } },
      "permission_scope": { "type": "keyword" },
      "permission_fingerprint": { "type": "keyword" },
      "snapshot_hash": { "type": "keyword" },
      "authority": { "type": "keyword" },
      "updated_at": { "type": "date" },
      "outbox_sequence": { "type": "long" },
      "deleted": { "type": "boolean" }
    }
  }
}
```

`embedding.dimension`은 provider adapter가 노출하는 configured dimension과 시작 시 일치해야
한다. 불일치하면 worker가 시작하지 않으며 자동 변환이나 padding을 하지 않는다.

## Document identity와 ordering

projection `_id`는 `{workspace_id}:{source_kind}:{document_id}:{region_id}`의 SHA-256이다.
consumer는 같은 `_id`의 현재 `outbox_sequence`보다 큰 event만 scripted update로 적용한다.
tombstone도 같은 ordering을 사용하므로 늦게 도착한 upsert가 삭제나 권한 제거를 되돌리지
못한다. Published와 Draft source는 서로 다른 alias에만 기록한다.

## Permission scope

`permission_scope`에는 `workspace_id`와 `document_id`로 만든 안정적 opaque scope token을
저장한다. `permission_fingerprint`는 root에서 대상까지의
`{document_id, parent_id, permission_revision}` 열을 hash한 값이다. Search scope compiler는 현재
Membership·Group으로 접근 가능한 문서의 `{scope token, ancestry fingerprint}` 쌍을 만들고,
`workspace_id`와 함께 bool filter에 먼저 적용한다. 쌍이 다른 결과는 반환하지 않고
재색인을 예약한다. 검색 후 권한 필터링은 금지한다.

## Query pipeline

1. Workspace·permission scope·status filter를 적용한다.
2. BM25 top 100과 kNN top 100을 별도 수집한다.
3. `k=60` reciprocal rank fusion으로 합친다.
4. authority, exact term, freshness를 deterministic weight로 재정렬한다.
5. 같은 Document·Region·snapshot hash를 제거하고 top 30 Source를 반환한다.

weight와 top-k는 configuration version에 묶고 Search response에 그 version을 기록한다. 실험
변경은 정본을 바꾸지 않으며 offline relevance gate를 통과한 version만 배포한다.

## Rebuild와 장애

전체 rebuild는 PostgreSQL repeatable-read snapshot에서 Workspace별 `projection_sequence`
watermark를 고정하고 새 generation에 적재한 뒤 watermark 이후 outbox를 따라잡는다.
catch-up 중 regular consumer는 활성 generation과 rebuilding generation을 모두 갱신한다. count,
deterministic hash sample, denied·allowed permission canary가 통과해야 alias를 교체한다.
OpenSearch 장애 중 write는 outbox에 유지된다. title·tree 탐색 외 검색과 AI retrieval은
`SEARCH_UNAVAILABLE`로 실패하며 낮은 품질의 PostgreSQL 본문 검색으로 대체하지 않는다.
