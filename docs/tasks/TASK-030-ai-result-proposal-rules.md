# TASK-030: AI Result·Proposal·Writing Rules 구현

- **상태**: 완료
- **유형**: 구현
- **구현 패키지**: IMP-21
- **시작일**: 2026-08-25
- **완료일**: 2026-08-25
- **커밋**: —

## 목적

AI runtime 결과를 검증 가능한 Result와 사용자가 선택적으로 적용할 수 있는 Proposal로
변환한다. stale Context, Operation dependency, Writing Rule을 적용·Undo 경계에서 강제한다.

## 범위

- 포함: strict AIResult, Source·scope·dry-run·Writing Rule validator, Proposal 생성·조회·부분
  적용·거절, Writing Configuration query·command, migration·OpenAPI·통합 테스트
- 제외: AI 화면(IMP-25), AI의 직접 Publish와 자동 적용

## 필수 설계 문서

- `docs/product/features/WRITING-INTELLIGENCE.md`
- `docs/domain/writing-intelligence.md`
- `docs/domain/knowledge.md`
- `docs/design/specs/ai/TASK-CONTEXT-RESULT.md`
- `docs/design/specs/document/REGION-OPERATION-DIFF.md`
- `docs/design/specs/ALGORITHM-CATALOG.md`
- `docs/design/implementation/CONTENT-OPERATION-REDUCER.md`
- `docs/design/implementation/AI-CONTEXT-RUNTIME-ADAPTERS.md`
- `docs/design/implementation/AI-RESULT-PROPOSAL-RULES.md`
- `docs/design/contracts/ai-contracts.schema.json`
- `docs/design/api/openapi.yaml`
- `docs/design/data/schema.sql`

## 문서 준비 게이트

- [x] PRD의 사람 승인·AI 비직접수정 범위 확인
- [x] Result·Proposal·Writing Rule 용어와 상태 전이 확인
- [x] Source·scope·dependency·stale 불변식 확인
- [x] API·저장·권한·동시성·실패·수명주기 타입 계약 확정
- [x] reducer·lease·idempotency·Outbox 재사용 경계 확정
- [x] 테스트 전략과 완료 조건 확정

## 사용자 결정

사용자는 설계 문서와 AGENTS.md의 구조적 원칙에 따라 권장안을 자율 확정하고 다음 구현
태스크를 자동으로 진행하도록 결정했다.

## 의사결정

- Result 검증은 adapter SQL에 흩뜨리지 않고 Writing Intelligence pure validator가 소유한다.
- bounded REWRITE만 Result Operation으로 남기고 구조 변경은 Proposal로 승격한다.
- 부분 적용은 dependency-closed 집합 전체를 한 Draft revision과 inverse Operation 묶음으로 처리한다.
- 강제 system rule은 override할 수 없고 v1에는 사용자 override 가능한 휴리스틱 rule을 두지 않는다.
- PROHIBITED Vocabulary는 dry-run 결과 본문을 기준으로 차단한다.
- Proposal 적용은 기존 reducer·lease·Reference·File·Review·Outbox transaction primitive를 재사용한다.

## 구현 순서

1. PLAN-27과 영향받는 정본 계약을 확정한다.
2. pure Result validator와 typed 모델을 구현한다.
3. forward migration과 PostgreSQL Result·Proposal repository를 구현한다.
4. Proposal·Writing Configuration application/HTTP 계약을 연결한다.
5. pure·PostgreSQL·contract·Compose gate를 검증한다.

## 작업 내역

- 2026-08-25: IMP-21 구현 태스크를 등록하고 정본 감사를 시작했다.
- 2026-08-25: PLAN-27에서 Result 검증 순서, Proposal 경계, 부분 적용 원자성,
  강제 Writing Rule registry와 API·저장 계약을 확정했다.
- 2026-08-25: AI Result 모델과 pure validator를 Application 계층에 구현하고 Source membership,
  Operation dependency closure, dry-run, 금지 Vocabulary 검사를 하나의 검증 경계로 고정했다.
- 2026-08-25: migration 0021과 PostgreSQL adapter를 추가해 Result 저장, Proposal 조회·거절,
  선택 적용, inverse 묶음, Writing Configuration 변경을 원자 트랜잭션으로 연결했다.
- 2026-08-25: Proposal·Writing Configuration HTTP 계약과 generated Rust·TypeScript 계약을
  갱신하고 기존 Draft reducer·lease·idempotency·Audit·Outbox primitive를 재사용했다.
- 2026-08-25: canonical Content·Operation schema를 AI runtime strict schema에 포함하고 Docker
  빌드에도 동일 계약 파일을 공급했다.
- 2026-08-25: pure test, 실제 PostgreSQL Proposal 통합 테스트, 전체 root gate와 Docker Compose
  통합 gate를 통과했다.

## 이슈 및 해결

- Result validator를 Writing Intelligence 도메인 crate에 처음 배치하자 Document 도메인 의존이
  계층 경계 검사에서 거부됐다. 여러 도메인의 Result·Operation·Vocabulary를 조정하는 책임이므로
  validator를 Application 계층으로 이동해 도메인 간 직접 의존을 제거했다.
- AI runtime이 compile-time에 읽는 canonical schema가 Docker build context에 없어 image build가
  실패했다. Dockerfile이 `docs/design/contracts`를 명시적으로 복사하도록 바꿔 로컬과 컨테이너가
  동일한 정본 계약을 사용하게 했다.

## 검증

- [x] 문서 링크와 정본 경계 확인
- [x] stale·dependency·hard rule 계약 검증
- [x] 실제 PostgreSQL 통합 계약 검증
- [x] generated contract·migration·root·Compose gate

## 결과

AI runtime 결과를 strict Result로 검증하고 사용자가 dependency-closed Operation 집합만 선택해
Draft에 원자 적용하거나 거절할 수 있게 했다. Writing Configuration은 폐쇄형 versioned registry로
관리하며 Proposal 적용은 lease·revision·idempotency·inverse·Audit·Outbox 불변식을 공유한다.
