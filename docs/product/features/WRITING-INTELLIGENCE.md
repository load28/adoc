# Writing Intelligence 요구사항

- **문서 ID**: PROD-15
- **상태**: 동결

## Task 유형

Raw Thoughts Compose, scoped Rewrite, Writing Review, Discussion Apply, Conflict Merge와
grounded Knowledge Query를 지원한다. 각 Task는 input schema, scope, required Context,
output schema와 적용 정책을 가진다.

## Context

현재 Draft, 선택 Region, Discussion, Reference, Vocabulary, 관련 Published 지식과 Writing
Rules를 authority와 provenance와 함께 구성한다. Permission Scope 밖의 대상은 후보 생성
단계부터 제외한다. 사용자가 Context를 확인·추가·제외할 수 있어야 한다.

## 결과와 적용

- 현재 Region의 제한적 rewrite는 expected revision 검증 후 적용하고 Undo를 제공한다.
- 다중 Region, 전체 문서, Discussion Apply와 Merge는 Proposal·Diff 승인을 요구한다.
- Runtime text를 직접 저장하지 않고 schema-valid AIResult와 Document Operation을 검증한다.
- 근거 부족·충돌·불확실성은 결과 상태로 표시한다.

## Writing Rules

조직 용어, 금칙어와 근거 정확성은 강제한다. 문체, 가독성과 표현 선호는 이유를 표시하는
권고다. 기본 한국어 Rule은 versioned baseline이며 Workspace가 추가·재정의한다.

## Runtime

로컬·자체 호스팅은 Codex CLI adapter를 지원한다. 다중 사용자 운영은 Provider port 뒤
OpenAI Responses API를 사용한다. 개인 구독 credential을 공용 서버에 저장하지 않는다.
Provider 실패 시 다른 모델로 조용히 전환하지 않는다.
