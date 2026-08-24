# TASK-006: Bun 패키지 매니저 채택

- **상태**: 완료
- **유형**: 설계
- **시작일**: 2026-08-25
- **완료일**: 2026-08-25
- **커밋**: —

## 목적

JavaScript·TypeScript workspace의 패키지 매니저를 pnpm에서 Bun으로 변경하고, 구현 전에
관련 아키텍처 결정·구현 계획·검증 계약을 하나의 정본으로 일치시킨다.

## 범위

- 포함: Decision Register, ADR, Technology Selection, Document Map, Design Freeze Report,
  IMP-01 Work Breakdown과 TASK-005의 패키지 매니저 계약 갱신
- 제외: Bun runtime으로의 백엔드 대체, React·TanStack Start·Vite 변경, Rust toolchain 변경,
  애플리케이션 코드와 기능 구현

## 필수 설계 문서

- [x] PROD-08 `product/DECISION-REGISTER.md`
- [x] ARCH-04 `design/architecture/TECHNOLOGY-SELECTION.md`
- [x] ADR-001 `design/adr/ADR-001-monorepo-web-rust.md`
- [x] PLAN-01 `design/implementation/REPOSITORY-STRUCTURE.md`
- [x] PLAN-05 `design/implementation/DESIGN-FREEZE-REPORT.md`
- [x] PLAN-08 `design/implementation/WORK-BREAKDOWN.md`
- [x] TASK-005 `tasks/TASK-005-bootstrap-monorepo-toolchain.md`

## 문서 준비 게이트

- [x] 변경 대상과 대체되는 기존 계약을 식별했다.
- [x] Bun의 적용 경계를 package manager·script runner로 제한했다.
- [x] Rust backend와 Node-compatible TanStack Start 배포 계약은 변경하지 않는다.
- [x] lockfile·frozen install·workspace 검증 조건을 명시한다.
- [x] 사용자 결정 외 미해결 질문이 없다.

## 사용자 결정

### 결정 요청 1: JavaScript 패키지 매니저

- **상황**: TASK-005가 pnpm workspace 구현을 시작하기 직전에 사용자가 Bun 사용을 지시했다.
- **대안과 영향**: pnpm은 기존 계획을 유지하고, Bun은 workspace·lockfile·명령 계약을
  변경한다.
- **권장안**: 사용자 지시에 따라 Bun을 패키지 매니저로 고정하되 제품 runtime 경계와
  혼동하지 않도록 적용 범위를 분리한다.
- **사용자 결정**: Bun 사용, 2026-08-25

## 의사결정

### 결정 1: Bun은 패키지 관리와 workspace 명령을 소유한다

- **상황**: package manager와 production runtime을 같은 결정으로 취급하면 Rust backend와
  TanStack Start 배포 경계가 불필요하게 바뀐다.
- **검토한 대안**: pnpm 유지 / Bun package manager만 사용 / 전체 JavaScript runtime을 Bun으로
  강제.
- **선택과 근거**: Bun을 package manager·workspace script runner로 사용한다. Web production
  runtime은 배포 설계에서 별도로 결정된 Node-compatible output 계약을 유지한다.

## 작업 내역

- 2026-08-25: 사용자의 Bun 사용 결정을 기록하고 영향 문서를 식별했다.
- 2026-08-25: DEC-035와 ADR-009를 추가하고 Technology Selection, Document Map, Design Freeze,
  Work Breakdown과 TASK-005를 Bun 계약으로 일치시켰다.

## 이슈 및 해결

없음.

## 검증

- [x] pnpm 정본 참조가 남아 있지 않음
- [x] DEC·ADR·Technology·Plan·Task 계약 일치
- [x] Document Map과 Design Freeze 수량·snapshot 일치
- [x] Markdown link와 `git diff --check` 통과

## 결과

Bun을 package manager·workspace script runner로 한정해 채택하고 전체 설계를 재동결했다.
