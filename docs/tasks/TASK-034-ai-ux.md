# TASK-034: AI UX 구현

- **상태**: 완료
- **유형**: 구현
- **구현 패키지**: IMP-25
- **시작일**: 2026-08-25
- **완료일**: 2026-08-25
- **커밋**: —

## 목적

권한이 확인된 Context를 사용자가 실행 전에 검토하고, AI Job의 진행·실패를 복구하며,
구조화된 Result와 Proposal을 출처·Diff·선택 적용 경계 안에서 다루는 AI Inspector를 구현한다.

## 범위

- 포함: Context preview, AI Job 생성·목록·상세·취소, Result·Source·Finding 표시, Proposal
  operation 선택·적용·거절, SSE invalidation, stale·권한·접근성 검증
- 제외: AI runtime·Proposal 도메인 재구현, Governance 설정 화면(IMP-26)

## 필수 설계 문서

- `docs/product/features/WRITING-INTELLIGENCE.md`, `docs/domain/writing-intelligence.md`
- `docs/design/ux/KNOWLEDGE-AI-FLOWS.md`, `docs/design/ux/FRONTEND-STATE-ROUTE-CONTRACT.md`
- `docs/design/specs/ai/TASK-CONTEXT-RESULT.md`, `docs/design/specs/ai/JOB-RUNTIME.md`
- `docs/design/specs/document/REGION-OPERATION-DIFF.md`
- `docs/design/api/openapi.yaml`, `docs/design/api/EVENT-CATALOG.md`
- `docs/design/implementation/AI-CONTEXT-RUNTIME-ADAPTERS.md`
- `docs/design/implementation/AI-RESULT-PROPOSAL-RULES.md`
- `docs/design/implementation/AI-UX.md`

## 문서 준비 게이트

- [x] Inspector route·task·context preview와 source 선택 계약 확정
- [x] Job lifecycle·SSE 복구·취소·오류 표시 계약 확정
- [x] Result·Source·Finding·Proposal Diff 표시 계약 확정
- [x] dependency-closed 선택·lease·revision·명시적 적용 계약 확정
- [x] permission·stale·keyboard·접근성 테스트 전략 확정

## 사용자 결정

사용자는 MVP 축소 없이 전체 기능을 구현하고 공개 Atlaskit component와 token만 사용하도록 결정했다.
권장 설계는 별도 승인 없이 자율 확정한다.

## 의사결정

- `panel=ai`와 선택한 job·proposal ID가 공유 가능한 화면 상태의 정본이다.
- Context preview 성공 전에는 Job을 만들지 않으며 preview fingerprint와 exact Draft revision을 함께 보낸다.
- SSE는 Job payload를 직접 적용하지 않고 job·proposal query를 무효화하는 신호로만 사용한다.
- Result operation은 읽기 전용 Diff로 표시하고 Proposal만 사용자의 명시적 operation 선택 뒤 적용한다.
- Proposal 적용은 Editor lease와 client instance를 재사용하고 stale·dependency 오류를 숨기지 않는다.

## 구현 순서

1. PLAN-31과 typed AI API client를 확정한다.
2. Context Inspector와 Job lifecycle 화면을 구현한다.
3. Result·Source·Finding과 Proposal 선택 적용·거절 화면을 구현한다.
4. route·SSE·permission·stale·접근성을 검증한다.
5. root·Compose gate를 통과하고 완료 기록한다.

## 작업 내역

- 2026-08-25: IMP-25 태스크를 등록하고 제품·도메인·AI runtime·Proposal 정본을 확인했다.
- 2026-08-25: PLAN-31에서 route, context, lifecycle, proposal approval, 복구·접근성 계약을 확정하고
  문서 준비 게이트를 통과했다.
- 2026-08-25: Context preview·Job 생성/목록/상세/취소·Result/Source/Finding·Proposal
  선택 적용/거절을 Atlaskit Inspector와 typed API client로 연결했다.
- 2026-08-25: dependency-closed operation 선택과 AI SSE invalidation을 UI domain 함수와 단위
  테스트로 고정했다.
- 2026-08-25: 전체 root gate와 Docker Compose 통합 검증을 통과했다.

## 이슈 및 해결

- OpenAPI를 통해 생성된 Proposal operation과 정본 `DocumentOperation` 타입은 중첩 tuple의 표현이
  달라 UI 선택 함수에서 직접 대입할 수 없었다. 선택 알고리즘의 책임을 `opId·dependsOn` 최소
  인터페이스로 분리해 두 계약이 같은 구조적 불변식을 공유하도록 해결했다.

## 검증

- [x] Context·Job lifecycle·cancel 복구 검증
- [x] Result·Source·Finding 표시 검증
- [x] Proposal 명시적 선택·apply·reject·stale 검증
- [x] route·SSE·permission·a11y·root·Compose gate

## 결과

AI 실행은 permission-safe Context preview와 exact revision을 선행하며, 생성 Operation은 자동 적용되지
않고 사용자 선택·dependency closure·lease·server validation을 거치는 Proposal 경계로 연결됐다.
