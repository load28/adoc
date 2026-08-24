# Writing Intelligence 도메인

## 1. 책임

사용자 의도를 제한된 AI Task로 변환하고, 권한이 확인된 Context와 Writing Policy를
구성하며, Runtime 결과를 구조화된 Proposal로 검증한다. 특정 모델의 능력이나 Prompt
문자열이 제품 의미를 소유하지 않는다.

## 2. 계층

```text
User Intent
→ AITask
→ ContextBuilder
→ WritingPolicy
→ AIRequest
→ AIJob
→ AIRuntime
→ Structured AIResult
→ Validation
→ Application Command or Proposal
```

## 3. AI Task

```text
AITask
├─ COMPOSE: raw thoughts → Draft content proposal
├─ REWRITE: target Region → operations
├─ REVIEW: content → findings
├─ DISCUSSION_APPLY: discussion → analysis + operations
├─ MERGE: base/current/incoming → merge proposal
└─ KNOWLEDGE_QUERY: question → answer + sources
```

자유 형식 Agent가 애플리케이션을 조작하게 하지 않는다. 새 능력은 이름 있는 Task와 입력,
출력, 권한, 검증, 실패 계약으로 추가한다.

## 4. Context

```text
AIContext
├─ requester and Permission Scope evidence
├─ current Document or target Region
├─ selected Discussion and Topics
├─ explicit References
├─ retrieved KnowledgeUnit[]
├─ Vocabulary Concepts
├─ Writing Rules
└─ user-provided facts and intent
```

### 불변식

- Permission Scope 밖의 내용은 ContextBuilder 입력에도 들어오지 않는다.
- Context는 Job마다 격리하고 다른 사용자 요청과 session state를 공유하지 않는다.
- 각 Context item은 source kind, identity, revision/version과 authority를 가진다.
- 사용자가 제외한 선택적 Source는 자동 검색이 다시 넣지 않는다.
- 누락, 충돌과 접근 불가를 내용 없음과 구분한다.

## 5. Writing Policy

Writing Policy는 다음 순서로 합성한다.

```text
Global Safety and Accuracy Rules
+ Language Rules
+ Workspace Rules
+ Vocabulary Policy
+ Task-specific Rules
```

하위 규칙이 상위 정확성·권한·사람 승인 원칙을 무효화할 수 없다. 규칙 충돌은 명시적인
precedence와 진단으로 다룬다.

### 한국어 규칙 후보

현재 대화에서 나온 후보는 검증 전 설계 입력이다.

- 한 문장에 가능한 하나의 핵심 명제
- 주어와 서술어 및 의존 성분의 불필요한 거리 축소
- 중첩 관형절과 여러 논리 전환의 동시 처리 제한
- 필요한 전제와 핵심을 세부사항보다 먼저 배치
- 정의되지 않은 팀 용어와 동일 개념의 여러 이름 탐지

이 후보를 코드나 Prompt의 강제 규칙으로 넣기 전에 별도 조사 태스크에서 근거, 적용 범위,
탐지 가능성, 오탐, 수정 전략과 평가 자료를 확정한다.

## 6. 결과 계약

```text
AIResult
├─ taskType
├─ status
├─ analysis?
├─ operations?
├─ findings?
├─ answer?
├─ sources[]
├─ uncertainties[]
├─ conflicts[]
└─ runtimeMetadata
```

문장만 출력하고 애플리케이션이 추측해 parse하는 계약을 만들지 않는다. schema 검증에 실패한
결과는 Draft에 부분 적용하지 않는다.

## 7. 적용 정책

- `COMPOSE`: 일반 Draft content proposal로 열고 사람이 즉시 편집 가능
- 좁은 `REWRITE`: 현재 Editor가 요청했고 target과 revision이 일치하면 적용 후 Undo 가능
- 광범위 `REWRITE`, `DISCUSSION_APPLY`, `MERGE`: Proposal + Diff + 명시적 승인
- `REVIEW`: Finding과 수정안을 표시하되 기본적으로 Publish 차단 없음
- `KNOWLEDGE_QUERY`: Source가 없는 조직 사실을 답으로 단정하지 않음

작업 크기 경계는 미확정이며 UX·보안 상세 설계 없이 코드에서 글자 수 등 휴리스틱으로
결정하지 않는다.

## 8. Runtime 경계

```text
AIRuntime.execute(AIRequest) → AIResult stream/final
```

현재 구현 대상은 서버에서 구독형 CLI를 실행하는 `CliRuntime`이다. Runtime은 인증, process,
streaming, timeout과 cancellation을 처리하지만 제품 Context를 스스로 검색하거나 DB를
수정하지 않는다. 동시성 제한, Job queue, sandbox, 공급자 약관과 운영 격리는 구현 전
상세 설계의 필수 조건이다.
