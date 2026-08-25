# Hybrid Retrieval·Source 구현 계약

- **문서 ID**: PLAN-25
- **상태**: 구현 기준
- **구현 패키지**: IMP-19
- **정본**: [Index, Retrieval과 Source](../specs/knowledge/INDEX-RETRIEVAL-SOURCE.md),
  [Permission Resolver](../specs/governance/PERMISSION-RESOLVER.md),
  [Algorithm Catalog](../specs/ALGORITHM-CATALOG.md)

## 1. 책임과 비책임

이 패키지는 현재 Membership·Permission을 검색 전 filter로 compile하고, OpenSearch의 BM25·
kNN 후보를 RRF로 결합해 근거가 보존된 Source를 반환한다. 사용자 Search와 후속 AI Context가
같은 `KnowledgeRetrieval` port를 사용한다.

projection 생성·재구축은 IMP-18, query embedding provider와 AI Context 선택은 IMP-20,
검색 화면은 IMP-24가 소유한다. OpenSearch 장애를 PostgreSQL 본문 검색이나 AI 일반 지식으로
대체하지 않는다.

## 2. 경계 타입

```text
RetrievalRequest = {
  actorId, workspaceId, query,
  queryVector?, includeDrafts,
  limit: 1..30, cursor?, now
}

PermissionKey = {
  documentId, sourceKind,
  scopeToken, ancestryFingerprint, compositeKey
}

ScopedRetrievalQuery = {
  workspaceId, normalizedQuery, queryVector?,
  publishedKeys[], draftKeys[],
  scopeFingerprint, limit, cursor?
}

Source = {
  kind: PUBLISHED | DRAFT,
  stableId, documentId, regionId,
  version?, draftRevision?,
  authority: OFFICIAL | WORKING,
  snapshotHash,
  displaySnapshot: {title, excerpt, updatedAt}
}

SearchPage = {
  items: {source, score}[], nextCursor?,
  indexWatermark, configurationVersion: search-ranking-v1
}
```

`stableId`는 IMP-18 projection ID다. Published는 `version`만, Draft는 `draftRevision`만
채운다. excerpt는 검색 projection body에서 Unicode scalar 기준 최대 500자로 자르며 원문이
더 길면 말줄임표를 붙인다. HTML을 생성하거나 highlight markup을 신뢰하지 않는다.

## 3. 입력 정규화와 vector 계약

query는 NFC 정규화 후 앞뒤 공백 제거와 연속 공백 축약을 수행한다. 빈 문자열, control·
direction override, 500자를 넘는 입력은 `VALIDATION_FAILED`다. limit 기본값은 20이고 최대
30이다. `includeDrafts` 기본값은 true다.

`queryVector`는 caller가 생성한 finite `f32` 배열이다. 값이 하나라도 NaN·infinite이거나
configured mapping dimension과 다르면 validation failure다. vector가 있으면 BM25와 kNN을
모두 실행하고, 없으면 BM25만 실행한다. IMP-20은 같은 port에 embedding을 공급하므로 이
패키지는 provider SDK나 credential을 알지 못한다.

## 4. Permission scope compiler

PostgreSQL repeatable-read snapshot에서 active Membership, user·group grants, Document tree와
각 `permission_revision`을 읽는다. 기존 `compile_permission_scope`로 point resolver와 같은
EffectivePermission을 만든다.

- Published key: access가 VIEWER 이상인 ACTIVE Document
- Draft key: `includeDrafts=true`이고 access가 CONTRIBUTOR 이상인 ACTIVE Document
- `scopeToken`: PLAN-24 `permission_scope`
- `ancestryFingerprint`: root→Document의 `{id,parentId,permissionRevision}` hash
- `compositeKey`: `SHA-256("permission-key:v1:" + scopeToken + ":" + ancestryFingerprint)`

같은 snapshot의 stamp, 정렬된 Document ID·access·fingerprint를 hash해 `scopeFingerprint`를
만든다. Membership이 없거나 Workspace가 active query 범위가 아니면 존재를 숨긴 404다.
resolver N+1 query와 user·group ID를 OpenSearch projection에 저장하는 방식을 금지한다.

## 5. OpenSearch candidate query

모든 request는 `workspace_id`, `deleted=false`, 해당 source read alias와 `permission_key`
terms filter를 candidate query에 포함한다. filter가 없는 BM25·kNN 요청은 만들지 않는다.

Permission key는 source별 4,096개 단위로 분할한다. 각 batch에서 lexical top 100과 vector가
있을 때 kNN top 100을 구한다. `_msearch` 요청도 payload 상한을 위해 최대 16 subquery로
분할한다. batch별 결과를 modality마다 provider score 내림차순·projection ID 오름차순으로
합쳐 global top 100을 선택한다. 이 단계는 권한 후처리가 아니라 이미 exact permission
filter가 적용된 후보 집합의 병합이다.

lexical query는 title `^3`, body `^1`, exact `terms`를 사용한다. vector query도 같은
Workspace·deleted·permission key filter 안에서 실행한다. OpenSearch total hit나 score를
권한 밖 문서와 합산하지 않는다.

## 6. RRF·rerank·dedupe

modality rank는 1부터 시작하고 각 후보에 `1/(60+rank)`를 더한다. `search-ranking-v1`은
다음 고정 weight를 사용한다.

- normalized query와 projection term exact match: `+0.005`
- Published authority: `+0.003`
- Draft authority: `+0.001`
- freshness: `0.002 * 0.5^(ageDays/180)`, 미래 timestamp는 age 0으로 고정

`{documentId,regionId,snapshotHash}`가 같은 후보는 최고 score 하나만 유지한다. 최종 정렬은
score 내림차순, Source stable ID 오름차순이다. 전체 결과 universe는 top 30이다. weight나
top-k 변경은 새 configuration version과 relevance fixture를 요구한다.

## 7. cursor와 index watermark

cursor는 URL-safe base64의 versioned JSON이며 `{version,workspaceId,actorId,queryHash,
scopeFingerprint,indexGeneration,indexWatermark,configurationVersion,offset}`를 가진다. 최대
2 KiB이고 offset은 1..30이다. 다른 actor·Workspace·query·scope·generation·watermark·ranking
version에 사용하면 `SEARCH_CURSOR_EXPIRED`다.

`indexGeneration`은 read alias가 가리키는 두 source generation의 canonical hash다.
`indexWatermark`는 exact permission filter 안의 nondeleted·tombstone projection을 포함한
최대 `outbox_sequence`다. 새 허용 projection write나 alias cutover는 cursor를 만료시킨다.
page는 결과 universe를 결정적으로 다시 계산한 뒤 offset부터 limit만 반환한다.

## 8. Source provenance

Source는 hit의 stable ID, revision, snapshot hash와 표시 snapshot을 그대로 보존한다.
Published·Draft label을 분명히 구분하고 content 전체를 응답하지 않는다. OpenSearch
highlight fragment를 사용하지 않고 projection body에서 deterministic excerpt를 만든다.

AI Result가 Source를 저장하거나 다시 표시하는 동작은 IMP-20·21에서 현재 Permission을 다시
검사한다. 이 패키지의 응답은 request scope snapshot과 exact permission key가 일치한 hit만
포함한다.

## 9. drift repair

정상 후보 query와 별도로 현재 `permission_scope`에는 속하지만 current composite key와 다른
projection을 size 100의 bounded drift probe로 찾는다. 이 probe의 결과는 사용자 응답이나
ranking에 사용하지 않는다. PostgreSQL adapter는 Document별 `SearchRepairRequested.v1`
outbox event와 `OUTBOX_TO_SEARCH` Job을 멱등 생성하고 새 Workspace projection sequence로
current state를 다시 materialize한다.

drift probe 실패는 검색 결과의 안전성에 영향을 주지 않으므로 metric을 남기고 repair만
재시도한다. candidate filter가 exact composite key이므로 실패 중에도 stale permission hit는
반환되지 않는다.

## 10. 실패·복구·관측성

- OpenSearch timeout·429·5xx·alias 전환 404: `SEARCH_UNAVAILABLE`, retryable 503
- cursor binding 불일치: `SEARCH_CURSOR_EXPIRED`, 409
- invalid query·vector·limit: `VALIDATION_FAILED`, 422
- Membership 없음: 존재 비노출 404
- partial `_msearch` failure: 전체 request 실패, 일부 품질 결과 반환 금지
- empty permission scope: OpenSearch 호출 없이 empty page 반환

로그에는 query text, excerpt, embedding, permission key를 남기지 않는다. query hash,
Workspace ID, scope key count, source별 batch count, modality, latency, candidate count,
configuration version, index generation·watermark와 stable error category만 남긴다.

## 11. 모듈·구현 단위

- Knowledge domain: query normalization, permission composite key, RRF·weight·dedupe, excerpt,
  cursor payload validation
- Application: `SearchScopeCompiler`, `HybridSearchIndex`, `SearchDriftRepair`,
  `KnowledgeRetrievalService`
- PostgreSQL adapter: repeatable-read scope snapshot, ancestry fingerprint, repair outbox producer
- OpenSearch adapter: bounded `_msearch`, exact prefilter, watermark·generation, drift probe
- API: `searchKnowledge` route와 stable Problem mapping

HTTP는 vector 없이 같은 service를 호출한다. IMP-20은 provider에서 생성한 vector를 같은
`KnowledgeRetrievalService`에 전달해 query·AI가 candidate·permission·Source 계약을 공유한다.

## 12. 테스트·완료 gate

- unit: Unicode query, finite vector, composite key, RRF tie, weight, dedupe, excerpt, cursor binding
- permission property: point VIEWER/CONTRIBUTOR와 Published/Draft scope key 동등성
- real OpenSearch: Workspace·permission prefilter, BM25, kNN, global merge, RRF, stale fingerprint 0 hit
- drift: mismatch가 사용자 hit 없이 repair Job 하나를 만들고 retry가 dedupe됨
- Source: stable identity, revision exclusivity, snapshot·excerpt, Draft authority
- cursor: query·actor·scope·generation·watermark·version 변경 시 만료
- degraded: OpenSearch 정지 시 503이고 PostgreSQL 본문 fallback 없음
- root: contract generation, format, lint, test, build, Compose PostgreSQL·Redis·OpenSearch gate

IMP-19 완료는 lexical·vector relevance fixture와 denied/cross-workspace/stale fingerprint suite가
실제 OpenSearch에서 통과하고, 같은 retrieval port가 vector 유무 모두 결정적인 Source를
반환할 때 성립한다.
