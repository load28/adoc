# AI Context·Runtime Adapter 구현 계약

- **문서 ID**: PLAN-26
- **상태**: 구현 기준
- **구현 패키지**: IMP-20
- **정본**: [Writing Intelligence](../../domain/writing-intelligence.md),
  [AI Task, Context와 Result](../specs/ai/TASK-CONTEXT-RESULT.md),
  [AI Job과 Runtime](../specs/ai/JOB-RUNTIME.md),
  [Integration Architecture](../architecture/INTEGRATION-ARCHITECTURE.md)

## 1. 책임과 비책임

이 패키지는 이름 있는 AI Task를 검증하고, 현재 권한 안의 Source snapshot으로 Context를
구성하며, provider-neutral `AIRuntime`과 `EmbeddingRuntime` port를 Codex CLI와 OpenAI
adapter로 구현한다. Context 선택과 runtime 실행은 애플리케이션 DB나 도구를 AI에 제공하지
않는다.

AIResult의 도메인 검증·Proposal 생성·적용은 IMP-21, AI 화면은 IMP-25가 소유한다. 이
패키지는 provider가 반환한 JSON을 schema-valid runtime result로만 저장하며 Draft를 수정하지
않는다. 외부 Web Context는 별도 allowlisted fetch adapter가 구현될 때까지 활성화를 거부한다.

## 2. 확정 결정

### 2.1 Context Inspector는 preview fingerprint로 admission과 결합한다

검토한 대안은 Job 생성 뒤 Context 표시, 서버측 preview session, 무상태 재구성이었다. Job
생성 뒤 표시는 사용자가 시작 전에 Source를 조정한다는 UX를 위반하고, preview session은
별도 만료 저장소와 정리 경계를 만든다. 따라서 preview와 admission이 같은 Context Builder를
호출한다. preview는 canonical artifact의 fingerprint를 반환하고 create request는 동일 Task,
include·exclude와 fingerprint를 제출한다. admission이 current snapshot에서 다시 계산한 값과
다르면 `AI_CONTEXT_STALE`로 새 preview를 요구한다.

### 2.2 Context 본문 snapshot은 AI 전용 정본 row에 저장한다

검토한 대안은 실행 때 최신 본문 재조회, Job payload 본문 저장, Source metadata에 본문을
섞는 방식이었다. 최신 재조회는 사용자가 확인한 Context와 실행 Context를 바꾸고, 범용 Job
payload는 본문 저장을 금지하며, metadata 혼합은 redaction과 수명주기를 모호하게 한다.
`ai_context_sources`에 bounded `snapshot_text`와 source revision·permission evidence를 명시적으로
저장한다. generic `jobs.payload_json`에는 `aiJobId`만 둔다.

### 2.3 embedding은 별도 capability이고 lexical retrieval은 자동 fallback이 아니다

OpenAI adapter는 `text-embedding-3-small`과 configured dimensions로 embedding을 생성한다.
Codex CLI profile은 embedding capability가 없으므로 registry가 semantic retrieval을 필수로
요구하지 않는 Task에서 vector 없이 같은 Hybrid Retrieval port를 호출한다. 이는 provider
실패 뒤 조용히 모델을 바꾸는 fallback이 아니라 admission 전에 표시되는 capability 차이다.
OpenAI embedding 실패 시 lexical만 재시도하지 않고 `AI_PROVIDER_UNAVAILABLE`로 실패한다.

### 2.4 runtime은 tools·저장·대화를 사용하지 않는 단발 실행이다

OpenAI Responses는 `store=false`, `tools=[]`, `tool_choice=none`, `truncation=disabled`와 strict
JSON Schema를 사용한다. Codex CLI는 job별 빈 temp root에서 `exec --ephemeral
--ignore-user-config --sandbox read-only --skip-git-repo-check --output-schema`로 실행한다. 두
adapter 모두 application repository·DB·Redis·ObjectStorage credential을 받지 않는다.

## 3. Task registry

```text
TaskDefinition = {
  kind,
  allowedTargetKinds,
  minimumAccess,
  contextRecipe,
  timeoutClass,
  outputSchemaId,
  applicationPolicy,
  evaluationSetVersion,
  semanticRetrieval: REQUIRED | OPTIONAL | DISABLED
}
```

registry는 compile-time closed map이다. Task kind와 target 조합, instruction 10,000자, expected
revision, 외부 Web 사용 가능 여부를 prompt 생성 전 검증한다. `REWRITE`, `REVIEW`, `COMPOSE`,
`CONFLICT_MERGE`는 Document 또는 Region, `DISCUSSION_APPLY`는 Discussion,
`KNOWLEDGE_QUERY`는 Workspace Query만 허용한다. target Document 계열은 CONTRIBUTOR,
Workspace Query는 active Member를 요구한다.

Task별 result schema는 CONTRACT-03 `result`를 기반으로 `taskKind`를 const로 고정하고 사용하지
않는 operations·findings·claims 배열을 `maxItems: 0`으로 제한한다. runtime prompt 문자열이
Task 의미나 출력 형태를 새로 결정하지 않는다.

## 4. Context preview·artifact 타입

```text
ContextSelection = {
  includeSourceIds[], excludeSourceIds[]
}

ContextSource = {
  sourceId, kind, stableId,
  documentId?, regionId?, version?, draftRevision?,
  authority, includeReason,
  snapshotHash, snapshotText,
  permissionKey?, sourceRevision,
  retrievedAt?, included
}

AIContextArtifact = {
  schemaVersion: 1,
  task, taskDefinitionVersion,
  sources[], writingRuleVersion, vocabularyRevision,
  permissionScopeFingerprint,
  artifactFingerprint, estimatedInputUnits
}

AIContextPreview = {
  artifactFingerprint, expiresAt,
  sourcesWithoutSnapshotText[], omissions[], estimatedInputUnits
}
```

`sourceId`는 source kind·stable ID·snapshot hash에서 생성한 deterministic UUID다. 같은 snapshot은
preview와 admission에서 같은 ID를 가진다. exclude는 optional Source에만 적용한다. current
target, 명시적 user input과 mandatory Writing Rule은 제외할 수 없다. include ID가 current
권한·snapshot에 없거나 include와 exclude가 겹치면 validation failure다.

fingerprint는 content를 포함한 canonical artifact의 SHA-256이다. preview 유효 시간은 5분이며
시간 경과, target revision, permission scope, writing rule, vocabulary, Source snapshot 중 하나라도
바뀌면 재계산 결과가 달라져 admission을 거부한다.

## 5. Context Builder pipeline

Context Builder는 외부 provider·OpenSearch 호출을 PostgreSQL transaction 밖에 두는 세 단계로
실행한다. 각 DB 단계는 repeatable-read snapshot을 사용한다.

1. DB snapshot A에서 Membership, target permission·revision과 direct Source 후보 identity를 읽는다.
2. transaction 밖에서 Task query embedding과 IMP-19 retrieval을 실행한다.
3. DB snapshot B에서 A의 revision·permission stamp가 여전히 같은지 재검사하고 direct·retrieved
   stable ID의 exact content, Reference, Discussion, Vocabulary와 Writing Rule을 materialize한다.
4. 명시적 include·exclude를 적용하고 authority·reason·stable ID 순으로 정렬한다.
5. source 수·개별 text·전체 artifact limit을 적용하고 canonical fingerprint를 계산한다.

A와 B 사이 state가 바뀌면 결과를 섞지 않고 bounded 1회 처음부터 재시도한다. 다시 바뀌면
`AI_CONTEXT_STALE`다. admission write transaction은 B의 모든 revision·permission stamp를 다시
조건으로 검사한 뒤 Context와 두 Job을 commit한다.

권한 밖, 삭제, stale Source는 후보나 preview count에 포함하지 않는다. 명시적으로 요청한
Source가 접근 불가·변경됨은 generic omission code로 표시해 존재를 노출하지 않는다. retrieved
Source는 사용자가 제외한 stable ID를 다시 넣지 않는다. 상충 공식 Source는 함께 보존한다.

`snapshot_text`는 plain text canonicalizer 결과다. HTML·Markdown 명령 또는 provider message로
변환하지 않는다. 단일 Source 64KiB, Source 200개, artifact 4MiB를 hard limit으로 둔다.
`estimatedInputUnits`는 UTF-8 byte count를 보수적인 token upper bound로 사용하고 configured
`max_context_tokens` 이하로 자른다. mandatory Source가 한도를 넘으면 silent truncation 없이
`AI_CONTEXT_LIMIT_EXCEEDED`다. optional Source는 authority·include reason·stable ID 순으로
결정적으로 제외하고 omission을 남긴다.

## 6. Runtime port

```text
RuntimeRequest = {
  jobId, taskKind, model,
  policyArtifact, contextArtifact,
  outputSchema, timeout, maxOutputBytes
}

RuntimeEvent = {
  phase: STARTED | GENERATING | FINALIZING,
  providerSequence, progress?
}

RuntimeResult = {
  providerRequestId?, model,
  outputJson,
  usage: {inputUnits, outputUnits, estimatedMicrounits?},
  latencyMs
}

AIRuntime.execute(request, EventSink, Cancellation) -> RuntimeResult
AIRuntime.health() -> ProviderHealth
AIRuntime.capabilities() -> RuntimeCapabilities

EmbeddingRuntime.embed(normalizedText, dimensions, Cancellation) -> Vector + usage
```

event에는 generated text·Operation·Source content를 넣지 않는다. output은 2MiB를 넘으면
`AI_OUTPUT_LIMIT_EXCEEDED`다. cancellation은 시작 전, provider I/O 중, 결과 commit 직전에
확인한다. timeout·cancel과 terminal Job에 도착한 late result는 commit하지 않는다.

## 7. Codex CLI adapter

adapter는 job마다 `0700` temp directory를 만들고 output schema를 `0400` file로 기록한다.
working directory는 빈 temp root이며 application path를 전달하지 않는다. stdin에는 고정
policy와 bounded canonical request JSON을 직접 전달한다. shell interpolation과 별도 입력 파일
읽기 tool은 사용하지 않는다. network는 Codex sandbox에만 의존하지 않고 배포 container network
policy로 차단한다.

stdout JSONL과 stderr는 각각 최대 64KiB까지만 진단용으로 소비하고 저장·로그하지 않는다.
애플리케이션 lifecycle event는 provider 출력이 아니라 adapter의 시작·최종화 경계에서 만든다.
final output은 `--output-last-message`의 2MiB 제한 file에서 읽는다. timeout 또는 cancellation은
process group에 TERM을 보내고 `kill_grace` 뒤 KILL한다. nonzero exit와 schema-invalid JSON은
stable code로 변환한다. temp directory는 성공·실패 모두 제거한다.

## 8. OpenAI adapter

Responses request는 `POST /v1/responses`에 service credential을 Authorization header로만
전달한다. body는 model, developer policy, user task와 canonical Context artifact,
`text.format={type:"json_schema",name,strict:true,schema}`, `store:false`, `tools:[]`,
`tool_choice:"none"`, `truncation:"disabled"`, `max_output_tokens:32768`을 포함한다. 응답 `output`
배열을 순회해 completed assistant의 `output_text` 하나만 허용하고 refusal·incomplete·tool call을
명시적 실패로 변환한다. usage와 response ID만 보존한다.

Embedding request는 `POST /v1/embeddings`에 model, normalized input, `encoding_format:"float"`,
configured dimensions를 보낸다. 빈·비finite·dimension mismatch vector는 provider contract
failure다. HTTP 408·409·429·5xx는 transient, 인증·schema·capability 오류는 permanent다.
provider response body와 request content는 log하지 않는다.

공식 OpenAI API 형식은 2026-08-25에 [Responses create](https://developers.openai.com/api/reference/resources/responses/methods/create)와
[Embeddings create](https://developers.openai.com/api/reference/resources/embeddings/methods/create)를
확인했다. provider wire parser fixture로 변경을 감지한다.

## 9. Job admission·execution·저장

`createAIJob` transaction은 idempotency reservation, current Context 재구성·fingerprint 검증,
Workspace·User concurrency와 monthly budget reservation, `ai_jobs`, included
`ai_context_sources`, generic `jobs(kind=AI_RUNTIME,payload={aiJobId})`, 사용자 대상
`AIJobChanged` Outbox를 함께
commit한다. credential·prompt·Source text는 generic Job과 Outbox에 넣지 않는다.

worker는 generic Job claim 뒤 AI Job을 `RUNNING`으로 조건 전이하고, 현재 Membership·target
permission·expected revision·각 source permission evidence를 재검사한다. 저장된 snapshot을
canonical artifact로 조립해 runtime을 실행한다. success는 raw output JSON과 usage를
`ai_results`에 저장하고 AI Job을 `SUCCEEDED`로 만든다. IMP-21 전까지 validation summary는
`RUNTIME_SCHEMA_VALID`만 기록하며 Proposal을 만들지 않는다.

generic Job과 AI Job terminal 전이는 같은 transaction에서 commit한다. retry는 AI Job을
`QUEUED`로 되돌리지 않고 `RUNNING` attempt 상태를 유지하며 generic Job 재claim이 같은 AI Job을
재실행한다. 이미 result가 있으면 멱등 success다. cancel command는 두 row를 한 transaction에서
`CANCEL_REQUESTED`로 만들고 worker cancellation poll이 provider future 또는 process를 중단한다.

## 10. 실패·복구·관측성

- invalid Task·target·selection: `VALIDATION_FAILED`, 422
- preview fingerprint·revision 변경: `AI_CONTEXT_STALE`, 409
- context hard limit: `AI_CONTEXT_LIMIT_EXCEEDED`, 422
- quota·concurrency 부족: `AI_QUOTA_EXCEEDED`, 429 + retry/reset
- provider unconfigured·health/capability failure: `AI_PROVIDER_UNAVAILABLE`, 503
- provider timeout: AI Job `TIMED_OUT`, generic Job terminal success 처리
- user cancellation: 두 Job `CANCELLED`, late result 폐기
- transient provider failure: generic Job bounded retry, AI Job result 없음
- schema/refusal/output limit: AI Job `FAILED`, retry하지 않음

metric은 task kind, provider kind, bounded result, latency·usage bucket, Context source count·omission
count만 label로 사용한다. trace는 request→AI Job→generic Job attempt→provider request ID를 연결한다.
instruction, query, model output, Source content·ID, provider error body는 log·span·SSE에 넣지 않는다.

## 11. 모듈·구현 단위

- `writing_intelligence`: Task registry, Context·Source·runtime type, canonical fingerprint와 limits
- `application::ai`: preview/admission, Context Builder orchestration, runtime execution service
- `adapters::postgres::ai`: source materialization, job/context/result·usage transaction
- `adapters::ai_runtime`: Codex CLI·OpenAI Responses·OpenAI Embeddings adapter
- `operations`: generic `AI_RUNTIME` Job kind
- `worker`: configured runtime 생성과 AI handler registry 연결
- `api`: context preview, create/list/get/cancel AI Job과 stable Problem mapping

## 12. 테스트·완료 gate

- domain: Task-target matrix, deterministic source ID·artifact hash, exclusion, budget·ordering
- Context: point/scope equivalence, denied/cross-workspace 0 Source, expected revision·fingerprint stale
- same-port suite: 두 runtime의 structured output, usage, refusal, malformed, oversized, timeout·cancel
- CLI security: empty root, read-only input, bounded stdout/stderr, TERM→KILL, temp cleanup
- OpenAI wire: store false, tools none, strict schema, response array parser, embedding dimensions
- Job: admission atomicity, generic payload 비노출, duplicate attempt, cancel/late result terminal guard
- real PostgreSQL·OpenSearch: direct·Reference·Vocabulary·retrieved Source coverage와 non-leak
- root: contract generation, migration seal/check, format, lint, test, build, Compose gate

IMP-20 완료는 같은 Context artifact와 output schema가 두 runtime adapter suite에서 동일한
`RuntimeResult` 의미를 만들고, 실제 permission-safe retrieval의 Source가 preview·저장·worker
재검사 전 구간에서 권한 밖 내용을 노출하지 않을 때 성립한다.
