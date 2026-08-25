# AI Result·Proposal·Writing Rules 구현 계약

- **문서 ID**: PLAN-27
- **상태**: 구현 기준
- **구현 패키지**: IMP-21
- **정본**: [Writing Intelligence](../../domain/writing-intelligence.md),
  [AI Task, Context와 Result](../specs/ai/TASK-CONTEXT-RESULT.md),
  [Region·Operation·Diff](../specs/document/REGION-OPERATION-DIFF.md),
  [Algorithm Catalog](../specs/ALGORITHM-CATALOG.md)

## 1. 책임과 비책임

이 패키지는 provider JSON을 typed `AIResult`로 검증하고, Document 변경이 필요한 결과를
`Proposal`로 보존하며, 사용자의 명시적 선택만 기존 Draft mutation 경계로 적용한다. 검증과
적용은 Source provenance, Operation dependency, Draft revision, Edit Lease와 강제 Writing Rule을
한 경계에서 지킨다.

AI 화면은 IMP-25, provider 실행은 PLAN-26, 일반 Draft 편집 reducer는 PLAN-15·16이 소유한다.
AI는 Draft나 Published Version을 직접 수정하지 않는다. 한국어 문체 휴리스틱은 TEST-05의
근거와 detector threshold가 별도 확정되기 전까지 강제 규칙으로 구현하지 않는다.

## 2. Result 정규화와 검증 순서

runtime schema는 CONTRACT-03의 `result`와 같은 필드·제한을 사용한다. runtime 완료 transaction은
다음 순서를 중간 성공 없이 실행한다.
canonical Content·Operation schema는 compile-time asset으로 runtime output schema에 합성하며,
container builder에도 같은 정본 경로만 복사한다.

1. strict JSON→typed `AIResult` 역직렬화와 task kind 일치
2. `usedSourceIds`, Finding·Claim·Conflict의 모든 Source ID가 저장 Context의 included Source인지 확인
3. 상태별 의미 검증: `READY`만 Operation을 가질 수 있고, `NO_CHANGE`는 Operation이 없어야 하며,
   조직 사실 Claim은 Source가 없으면 `INSUFFICIENT_CONTEXT`여야 함
4. 모든 Operation의 precondition revision·scope가 Task target 안인지 확인
5. current Draft와 Reference snapshot으로 기존 reducer 전체 dry-run
6. dry-run 결과 본문에 baseline 강제 규칙과 현재 PROHIBITED Vocabulary 적용
7. Result·validation summary·선택적 Proposal과 AI Job 성공을 한 transaction에 저장

어느 단계든 실패하면 Result와 Proposal을 저장하지 않는다. AI Job은 `FAILED`와 stable
`AI_RESULT_*` code로 종료한다. provider 원문·본문·금지어를 error나 log에 넣지 않는다.

## 3. 적용 정책과 Proposal 경계

`REVIEW`, `KNOWLEDGE_QUERY`는 Proposal을 만들지 않는다. `COMPOSE`, `DISCUSSION_APPLY`,
`CONFLICT_MERGE`는 READY Operation이 있으면 항상 Proposal을 만든다. `REWRITE` 중 Region target이고
모든 Operation이 같은 target 안의 `REPLACE_TEXT|SET_MARKS`이면 bounded result로 남겨 기존 Draft
operation API에서 사용자가 적용한다. 그 외 REWRITE는 Proposal을 만든다. 글자 수나 Operation 수
같은 휴리스틱으로 경계를 바꾸지 않는다.

Proposal 생성 시 current Draft revision이 Task expected revision과 같아야 한다. 다르면 AI Job을
`AI_RESULT_STALE`로 실패시킨다. OPEN Proposal은 job owner와 현재 target Contributor에게만 보이며,
권한이 없으면 404로 은폐한다.

## 4. 부분 적용과 원자성

apply request의 `operationIds`는 비어 있지 않고 중복이 없어야 한다. 생략하면 전체 Operation을
선택한다. 선택 집합은 Proposal의 부분집합이고 모든 transitive dependency를 포함해야 한다.
선택되지 않은 dependent Operation은 허용한다.

transaction lock 순서는 Proposal→Document→Draft→Lease다. Proposal이 OPEN인지, If-Match Draft
revision과 Proposal base revision이 current Draft와 같은지, lease token·client instance·Contributor
권한이 유효한지 확인한다. 선택 Operation을 기존 reducer에 한 batch로 전달한다. Reference·File
동기화, Review invalidation, DraftChanged·ProposalApplied Outbox와 Audit, idempotency receipt,
Proposal `APPLIED` 전이를 모두 같은 transaction에 기록한다.

성공 응답의 `inverseOperations` 전체가 한 undo group이다. 일부만 적용해도 Proposal은 APPLIED로
종료하며 나머지를 자동 rebase하거나 다시 OPEN으로 유지하지 않는다. stale이면 전체를 거부하고
Draft를 바꾸지 않으며 Proposal을 STALE로 종료한다.

reject는 Proposal revision If-Match를 사용하고 OPEN→REJECTED만 허용한다. apply와 reject 모두
같은 key·request hash는 저장 응답을 재생하고, 다른 body는 `IDEMPOTENCY_KEY_REUSED`다.

## 5. Writing Rule registry

baseline ID는 `writing-rules-v1`이다. 다음 system rule은 비활성화하거나 완화할 수 없다.

- `system.result.contract`: typed Result·task/status 의미
- `system.source.membership`: Result의 모든 provenance가 included Context Source에 속함
- `system.operation.scope`: Operation이 Task target과 expected Draft revision을 벗어나지 않음
- `system.operation.dry_run`: current reducer와 Content Schema를 통과함
- `vocabulary.prohibited`: ACTIVE Vocabulary의 PROHIBITED term이 결과 본문에 없음

Workspace override는 알려진 비강제 rule ID만 받을 수 있다. v1에는 override 가능한 rule이 없으므로
`overrides`는 빈 배열만 유효하다. 이는 아직 근거가 없는 한국어 휴리스틱을 설정 JSON으로 몰래
도입하는 것을 막는다. get은 row가 없으면 revision 0의 baseline 기본값을 반환한다. update는 Admin,
If-Match와 idempotency key를 요구하고 알려지지 않은 baseline·rule·중복 ID를 거부한다.

## 6. 저장 계약

`ai_results.validation_json`은 validator version, writing rule version, vocabulary revision,
검증 단계와 Proposal 여부만 보존한다. `proposals`는 owner, revision, rule/provenance version,
validation summary, applied Operation IDs를 보존한다. Operation 본문은 AI Result와 Proposal에만 있고
Outbox·Audit에는 ID·revision만 둔다.

Proposal 상태 전이는 `OPEN→APPLIED|REJECTED|STALE|CANCELLED` 단방향이다. terminal row는 삭제하지
않고 AI Job retention을 따른다. 하나의 Job은 최대 하나의 Proposal을 가진다.

## 7. API와 오류 계약

- `GET /workspaces/{workspaceId}/proposals/{proposalId}`
- `POST /workspaces/{workspaceId}/proposals/{proposalId}/apply`
- `POST /workspaces/{workspaceId}/proposals/{proposalId}/reject`
- `GET|PUT /workspaces/{workspaceId}/writing-configuration`

apply는 `If-Match`, `Idempotency-Key`, `Lease-Token`, `X-Client-Instance`를 요구한다. reject와
configuration update는 앞의 두 command header를 요구한다. stable 오류는
`AI_RESULT_INVALID`, `AI_RESULT_STALE`, `AI_RESULT_RULE_BLOCKED`, `PROPOSAL_NOT_FOUND`,
`PROPOSAL_STALE`, `PROPOSAL_STATE_INVALID`, `PROPOSAL_DEPENDENCY_INVALID`,
`WRITING_CONFIGURATION_INVALID`다.

## 8. 검증 전략

pure test는 schema/status/source/scope/dependency/rule을 경계값으로 검증한다. PostgreSQL 통합
test는 Result+Proposal 원자 생성, permission 은폐, full·partial apply, stale zero-mutation,
lease, idempotency replay, reject와 configuration revision을 검증한다. migration manifest,
generated contract, root check와 Compose integration을 모두 통과해야 완료한다.
