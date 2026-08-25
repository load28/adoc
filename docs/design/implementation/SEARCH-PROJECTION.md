# Search Projection 구현 계약

- **문서 ID**: PLAN-24
- **상태**: 구현 기준
- **구현 패키지**: IMP-18
- **정본**: [OpenSearch Projection Schema](../data/OPENSEARCH-PROJECTION-SCHEMA.md),
  [Index, Retrieval과 Source](../specs/knowledge/INDEX-RETRIEVAL-SOURCE.md)

## 1. 책임과 비책임

이 패키지는 PostgreSQL 정본을 OpenSearch Published·Draft index로 materialize하고,
실패를 Job Runtime으로 복구하며 generation rebuild를 제공한다. Search query·RRF·Source
응답은 IMP-19가 소유한다. 이 패키지의 query는 contract canary와 rebuild 검증에만
사용한다.

OpenSearch는 정본이 아니다. projection에서 PostgreSQL을 복구하거나 문서 command를
실행하지 않는다. 장애 시 Job은 retry·dead-letter로 이동하고 core Document API는
liveness를 유지한다.

## 2. 경계 타입

```text
SearchSourceKind = PUBLISHED | DRAFT

ProjectionTrigger = {
  outboxEventId, workspaceId, eventType, projectionSequence, occurredAt
}

SearchProjection = {
  projectionSchema: 1,
  workspaceId, documentId, documentStatus,
  sourceKind, sourceRevision, versionNumber?,
  regionId, regionKind, ancestorIds,
  title, body, terms[], embedding?,
  permissionScope, permissionFingerprint,
  snapshotHash, authority, updatedAt,
  outboxSequence, deleted: false
}

SearchTombstone = {
  workspaceId, sourceKind, documentId, ancestorRootId?, outboxSequence
}
```

`outboxSequence`는 기존 field name을 유지하지만 값은 aggregate sequence가 아닌
Workspace별 `projection_sequence`다. Region `_id`는
`SHA-256(workspaceId:sourceKind:documentId:regionId)`다. tombstone marker `_id`는 같은 입력에
reserved region ID `00000000-0000-0000-0000-000000000000`을 사용한다.

## 3. Projection unit과 content 추출

Published는 `documents.current_version_id`가 가리키는 불변 Version만 투영한다. Draft는
현재 `drafts` row만 투영한다. ACTIVE가 아닌 Document, current Version이 없는 Published,
row가 없는 Draft는 tombstone으로 materialize한다.

Content root의 직접 자식 Block 하나가 하나의 Region이다. Region ID는 Block ID를 쓴다.
중첩 Block의 text, code, table cell, toggle summary는 상위 Region body에 document order로 평탄화한다.
Image·File metadata는 이 패키지에서 body로 추출하지 않는다. title은 모든 Region에
복제하되, body와 구분된 field로 ranking 가중치를 적용할 수 있게 한다.

`snapshotHash`는 normalized content snapshot의 SHA-256이다. `sourceRevision`은 Published의
Version number 또는 Draft revision이다. embedding은 IMP-20이 생성하기 전까지 생략한다.

Korean·English lexical field는 `analysis-nori` plugin의 `nori_tokenizer`·`nori_readingform`과
`lowercase`를 사용한다. 공식 OpenSearch image에 해당 analyzer가 내장되어 있지 않으므로
버전을 고정한 파생 image에 plugin을 설치한다. analyzer가 없으면 mapping 생성을
실패시키고 품질이 다른 analyzer로 조용히 대체하지 않는다.

## 4. Permission projection

`permissionScope = SHA-256("scope:v1:" + workspaceId + ":" + documentId)`다.
`permissionFingerprint`는 root→target path의 `{id,parentId,permissionRevision}` 정렬 JSON을
SHA-256한다. 이 fingerprint는 user·group에 독립적이며 문서 조상 권한 상태만
표현한다.

IMP-19 scope compiler는 현재 resolver로 접근 가능한 Document ID를 구한 뒤 각 Document의
scope token·fingerprint 쌍을 만든다. Published query는 VIEWER 이상, Draft query는 CONTRIBUTOR
이상 쌍만 사용한다. `workspace_id AND ((scope=A AND fingerprint=Af) OR ...)`를
BM25·kNN 후보 생성 전에 적용한다.

PermissionChanged·DocumentMoved trigger는 현재 하위 트리의 Published·Draft를 모두
재materialize한다. Membership·Group 변경은 index를 바꾸지 않고 query scope compiler만
바꾸므로 사용자 수에 비례한 fan-out을 금지한다.

## 5. Event·Job 계약

Outbox append는 `workspace_sequences.next_projection_sequence`를 원자적으로 증가시켜
`outbox_events.projection_sequence`을 할당한다. 다음 event에만 같은 transaction으로
`OUTBOX_TO_SEARCH` Job을 만든다.

- DocumentChanged, DocumentMoved, DraftChanged, VersionPublished
- PermissionChanged
- VocabularyChanged
- PurgeChanged

Job payload는 `{outboxEventId}` 한 field만 허용한다. dedupe key는
`search-projection:{outboxEventId}`다. consumer receipt는 `search-projection-v1`을 사용한다.
VocabularyChanged는 terms projection의 Workspace-wide refresh를 예약한다. 현 package에서 terms는
문서 text에 있는 active canonical·synonym 일치를 normalized exact token으로 저장한다.

consumer는 event payload의 content를 신뢰하지 않고 target ID만 폐쇄형 event contract로
파싱한다. 이후 PostgreSQL의 현재 row를 materialize한다. 알 수 없는 event type과
잘못된 payload는 permanent failure다. PostgreSQL·OpenSearch timeout, 429, 5xx는 transient failure다.

## 6. OpenSearch write ordering

새 projection을 쓰기 전 `_update_by_query`로 같은 Workspace·source·Document의
`outbox_sequence <= incomingSequence`인 Region을 `deleted=true`, `body=""`,
`outbox_sequence=incomingSequence`로 tombstone한다. 이후 bulk scripted upsert는 기존
`outbox_sequence <= incomingSequence`일 때만 전체 projection을 교체한다. partial failure는
전체 Job을 retry한다. 같은 sequence retry는 멱등이다. 삭제된 과거 Region을 row에서
즉시 제거하지 않는 이유는, 더 낮은 sequence의 느린 upsert가 없어진 ID를 새로
만드는 ABA race를 막기 위함이다.

tombstone은 Region을 같은 ordering으로 마스킹한 뒤 `deleted=true`인 marker를 같은
scripted ordering으로 쓴다. regular query는 항상 `deleted=false`를 사전 필터한다. Workspace purge는
Workspace filter로, Document purge는 `document_id OR ancestor_ids`로 두 source index에 tombstone을
적용한다.

## 7. Index bootstrap·rebuild

첫 write에서 alias가 없으면 schema 1, generation 1 index를 생성하고 read·write alias를
붙인다. 동시 bootstrap은 deterministic index name과 `resource_already_exists_exception`을
멱등 성공으로 처리한다. mapping은 `dynamic=strict`이며 configured embedding
dimension이 다르면 시작을 거부한다.

Rebuild는 `search_projection_rebuilds` row로 `BUILDING → CATCHING_UP → VALIDATING →
ACTIVE | FAILED`를 저장한다. 하나의 prefix에 활성 rebuild는 하나만 허용한다.

1. repeatable-read transaction에서 Workspace별 watermark와 Published·Draft snapshot을 읽는다.
2. 새 generation에 snapshot을 bulk 적재한다.
3. rebuilding generation을 dual-write 대상으로 등록하고 watermark 이후 outbox를 재생한다.
4. active index와 Workspace·source count, deterministic ID·snapshot hash sample, allow·deny canary를
   비교한다.
5. 하나의 `_aliases` 요청으로 read·write alias를 교체하고 상태를 ACTIVE로 만든다.

실패한 generation은 alias에 연결하지 않는다. 재시도는 새 generation을 만든다.
기존 generation 제거는 alias 교체와 별도 유지보수 작업이며 이 패키지에서 즉시
삭제하지 않는다.

## 8. DDL·모듈 계약

PostgreSQL에 다음을 추가한다.

- `workspace_sequences.next_projection_sequence bigint`
- `outbox_events.projection_sequence bigint`, unique `(workspace_id, projection_sequence)`
- Job kind `OUTBOX_TO_SEARCH`
- `search_projection_rebuilds(id, schema_version, generation, status, snapshot_watermark_json,
  replayed_through_json, error_code, timestamps)`

Rust 경계는 다음으로 분리한다.

- Knowledge: Region extraction, projection identity, scope token·fingerprint
- Application: `SearchProjectionRepository`, `SearchIndex`, `SearchProjectionService`
- PostgreSQL adapter: trigger 해석, current snapshot materialization, rebuild ledger
- OpenSearch adapter: mapping·alias, ordered replace·tombstone, canary
- Job dispatcher: JobKind을 handler에 라우팅하고 runtime은 개별 provider를 알지 못함

## 9. 실패·동시성·복구

- event 중복: consumer receipt와 ordered write로 한 번의 의미만 반영
- event 역순: current-state materialization과 external version으로 최신 상태 보존
- OpenSearch timeout: receipt을 쓰지 않고 Job retry
- PostgreSQL commit 후 worker crash: Job lease expiry·reconcile로 복구
- partial bulk: receipt을 쓰지 않고 idempotent document replacement 재시도
- permission/tree race: trigger sequence 순서와 current subtree snapshot으로 최신 fingerprint 보존
- rebuild race: dual-write 등록 후 catch-up·watermark canary 전에 alias swap 금지

## 10. 관측성·보안

로그와 trace에 `correlationId`, Job ID, outbox ID, projection sequence, source kind,
generation, OpenSearch status category를 남긴다. title·body·term·credential은 로그에 남기지
않는다. metric은 queue age, consumer lag, indexed·deleted region count, retry·dead-letter,
rebuild duration, canary mismatch를 다룬다.

OpenSearch credential은 config secret로만 주입한다. URL·index prefix는 기존 typed config를
사용한다. production은 HTTPS·credential이 없으면 기존 preflight에서 실패한다.

## 11. 테스트·완료 gate

- unit: Content→Region 추출, deterministic ID·scope·fingerprint, trigger parsing
- PostgreSQL integration: projection sequence 단조성, Job·receipt, subtree·purge materialization
- real OpenSearch contract: strict mapping, routing, stale write, duplicate retry, tombstone, partial retry
- rebuild: snapshot+catch-up, dual-write, count·hash·permission canary, atomic alias swap, failed generation isolation
- security: denied scope token으로 hit 0, allowed exact pair로만 hit, Workspace 교차 hit 0
- root: format, lint, test, build, migration seal, Compose PostgreSQL·Redis·OpenSearch gate

IMP-18 완료는 실제 OpenSearch에서 prefilter·ordering·rebuild canary가 통과하고,
OpenSearch를 중지해도 core API health가 성공하는 것을 확인해야 한다.
