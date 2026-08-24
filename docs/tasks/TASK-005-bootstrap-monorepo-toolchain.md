# TASK-005: 모노레포·툴체인 기반 구현

- **상태**: 완료
- **유형**: 구현
- **구현 패키지**: IMP-01
- **시작일**: 2026-08-25
- **완료일**: 2026-08-25
- **커밋**: —

## 목적

전체 제품 구현이 동일한 언어·빌드·의존성·모듈 경계 위에서 진행되도록 Rust와
TypeScript 모노레포의 실행 가능한 기반을 만든다. 이후 구현 패키지가 임의의 디렉터리,
중복 도메인 로직 또는 금지된 의존 방향을 추가하지 못하도록 경계를 자동 검증한다.

## 범위

- 포함: Cargo workspace, Bun workspace, asdf 기반 Rust·Node·Bun toolchain pinning, root command,
  `apps`·`crates`·`packages`·`infra` 기준 구조, API·worker·web의 최소 bootstrap,
  format·lint·typecheck·test 명령, Rust·TypeScript 의존 방향 검사와 CI foundation
- 제외: 도메인 로직, HTTP endpoint, 화면 UI, 계약 code generation, runtime configuration,
  telemetry, DB migration, Docker Compose service와 외부 dependency 연동

## 산출물

- root Cargo·Bun workspace와 재현 가능한 toolchain·lockfile
- `apps/web`, `apps/api`, `apps/worker`의 compile 가능한 최소 entry point
- PLAN-01에 정의된 `crates/*`, `packages/*`, `infra/*` 경계의 workspace skeleton
- Rust와 TypeScript의 format·lint·typecheck·unit test root command
- forbidden dependency edge와 dependency cycle을 실패시키는 자동 검사
- 로컬·CI가 같은 명령을 사용하는 최소 CI workflow

빈 디렉터리를 보존하기 위한 의미 없는 placeholder는 만들지 않는다. 아직 구현하지 않는
module은 workspace membership과 경계 검사에 필요한 최소 manifest·compile unit만 둔다.

## 필수 설계 문서

- [x] PROD-05 `product/IMPLEMENTATION-SCOPE.md`
- [x] PROD-06 `product/NON-FUNCTIONAL-REQUIREMENTS.md`
- [x] ARCH-03 `design/architecture/MODULE-ARCHITECTURE.md`
- [x] ARCH-04 `design/architecture/TECHNOLOGY-SELECTION.md`
- [x] ADR-001 `design/adr/ADR-001-monorepo-web-rust.md`
- [x] PLAN-01 `design/implementation/REPOSITORY-STRUCTURE.md`
- [x] PLAN-02 `design/implementation/IMPLEMENTATION-PLAN.md`
- [x] PLAN-03 `design/implementation/DEFINITION-OF-DONE.md`
- [x] PLAN-04 `design/implementation/RISK-REGISTER.md`
- [x] PLAN-05 `design/implementation/DESIGN-FREEZE-REPORT.md`
- [x] PLAN-08 `design/implementation/WORK-BREAKDOWN.md`
- [x] OPS-02 `design/operations/CI-CD.md`
- [x] TEST-01 `design/quality/TEST-STRATEGY.md`
- [x] 도메인 문서: N/A — 이 태스크는 도메인 동작을 구현하지 않고 의존 경계만 고정한다.
- [x] UX 흐름: N/A — web은 화면 없이 build 가능한 application shell까지만 만든다.
- [x] 데이터·API·권한 계약: N/A — 생성과 구현은 각각 IMP-02, IMP-04, IMP-08에서 수행한다.

## 문서 준비 게이트

- [x] 구현 결과를 바꿀 미해결 제품·아키텍처 질문이 없다.
- [x] repository layout, module ownership과 금지 의존 방향이 정의되어 있다.
- [x] 구현 산출물과 IMP-01 완료 gate를 추적할 수 있다.
- [x] 실패 조건은 compile·lint·dependency 검사 결과로 판정할 수 있다.
- [x] 문서상 코드 작성이 가능하다. 실제 구현 시작 시 필수 문서를 다시 확인한다.

## 사용자 결정

JavaScript package manager는 사용자의 2026-08-25 결정에 따라 Bun으로 변경했고 TASK-006,
DEC-035와 ADR-009에 기록했다. 나머지 TanStack Start·React 18.2·TypeScript·Vite,
Rust·Axum·Tokio·Tower 모노레포와 main 직접 작업은 DEC-012·015 및 ADR-001에서 확정됐다.

## 의사결정

### 결정 1: 실행 가능한 최소 skeleton만 만든다

- **상황**: 전체 디렉터리를 미리 채우면 후속 도메인 태스크의 책임과 코드가 섞일 수 있다.
- **검토한 대안**: 디렉터리만 생성 / 전체 module placeholder 생성 / bootstrap에 필요한
  compile unit과 manifest만 생성.
- **선택과 근거**: workspace와 경계 검사를 실행하는 데 필요한 최소 compile unit만 만든다.
  후속 IMP가 소유하는 domain·adapter 구현은 선행하지 않는다.

### 결정 2: 의존 경계를 자동 검사한다

- **상황**: 문서에만 의존 방향을 적으면 구현이 늘어날수록 우회 import와 cycle을 놓친다.
- **검토한 대안**: code review만 사용 / package별 수동 검사 / root CI에서 구조 검사.
- **선택과 근거**: Cargo와 package dependency graph를 root 명령과 CI에서 검사한다. PLAN-01의
  forbidden edge를 위반하면 build 성공 여부와 관계없이 실패시킨다.

### 결정 3: 정확한 도구 버전은 저장소에서 고정한다

- **상황**: library version은 제품 정책이 아니지만 재현 가능한 build에는 정확한 버전이 필요하다.
- **검토한 대안**: global latest 사용 / 문서에 버전 복제 / toolchain file·manifest·lockfile 고정.
- **선택과 근거**: 호환성이 검증된 버전을 `rust-toolchain.toml`, package manager 선언과
  lockfile에 고정한다. 설계 문서에는 중복 기록하지 않는다.

## 구현 순서

1. 현재 main과 기존 변경을 확인하고 사용자 변경을 보존한다.
2. Rust·Node·Bun 실행 환경과 TanStack Start·React 18.2 호환성을 확인한다.
3. root `.tool-versions`, workspace·toolchain·lockfile과 공통 명령을 구성한다.
4. PLAN-01 경계에 맞춰 최소 compile unit을 구성한다.
5. format·lint·typecheck·test와 forbidden dependency 검사를 연결한다.
6. 동일한 root 명령을 CI foundation에 연결한다.
7. clean checkout 상당 조건에서 bootstrap과 전체 gate를 검증한다.

## 이슈 및 해결

### 이슈 1: 로컬 toolchain 설치 정본이 없었음

- **증상**: 고정한 Rust 1.90.0 검증 과정에서 도구별 installer가 사용됐고 로컬 PATH와 실제
  선택 버전을 한 곳에서 재현할 수 없었다.
- **조사**: asdf plugin·현재 버전, `rust-toolchain.toml`, Bun·Node 버전과 CI setup을 대조했다.
- **근본 원인**: 기존 설계는 버전 pinning을 요구했지만 로컬 version manager를 지정하지 않았다.
- **구조적 해결**: TASK-007·DEC-036·ADR-010으로 asdf를 정본화하고 `.tool-versions`, manifest,
  toolchain file, CI와 실제 runtime을 `check-toolchains.mjs`에서 함께 검증한다.

### 이슈 2: 생성 전 Route Tree로 TypeScript 검사가 실패함

- **증상**: 최초 typecheck에서 TanStack Router의 `routeTree.gen.ts`가 없어 route 타입을 해석하지
  못했다.
- **조사**: TanStack Start 공식 file-based route 생성 순서와 Vite build 결과를 확인했다.
- **근본 원인**: 생성물이 필요한 검사를 generator보다 먼저 실행했다.
- **구조적 해결**: 공식 Vite plugin으로 Route Tree를 생성해 저장하고 이후 typecheck·build에서
  생성물 일관성을 검증한다. 생성 파일의 수동 편집은 금지한다.

### 이슈 3: 라이선스 검사 정책에 private root와 실제 SPDX가 섞임

- **증상**: 초기 검사에서 배포하지 않는 private workspace와 허용 목록에 없던 permissive SPDX
  항목 때문에 실패했다.
- **조사**: 전체 JavaScript dependency graph의 선언 license와 Cargo package license를 확인했다.
- **근본 원인**: 배포 대상과 private workspace를 구분하지 않았고 실제 transitive license
  inventory 없이 allowlist를 시작했다.
- **구조적 해결**: private package를 배포 검사에서 제외하고 실제 의존 graph의 permissive SPDX만
  명시적으로 허용한다. 새 license는 자동으로 실패해 검토를 요구한다.

## 검증

- [x] Rust workspace 전체 format·lint·build·test 통과
- [x] TypeScript workspace 전체 format·lint·typecheck·build·test 통과
- [x] web·API·worker 최소 artifact가 clean bootstrap에서 생성됨
- [x] forbidden Rust dependency fixture가 검사에서 거부됨
- [x] forbidden TypeScript dependency fixture가 검사에서 거부됨
- [x] dependency cycle 검사 통과
- [x] lockfile 변경 없는 재실행과 CI 명령 일치 확인
- [x] repository boundary·license·secret scan 통과
- [x] `git diff --check` 통과

## 완료 조건

- IMP-01의 `clean bootstrap·forbidden edge test` gate를 통과한다.
- IMP-02~05가 별도 구조 결정을 만들지 않고 현재 workspace에 구현을 추가할 수 있다.
- 새 제품 정책이나 후속 IMP의 도메인 구현이 포함되지 않는다.
- 작업 내역, 이슈의 근본 원인과 모든 검증 결과를 이 문서에 기록한다.

## 작업 내역

- 2026-08-24: 사용자의 요청에 따라 IMP-01 구현 태스크를 대기 상태로 등록했다.
- 2026-08-25: 사용자의 구현 시작 지시에 따라 태스크를 진행 중으로 전환했다.
- 2026-08-25: main 브랜치와 기존 문서 변경을 보존한 상태에서 로컬 Rust·Node·pnpm
  toolchain을 확인했다.
- 2026-08-25: 사용자가 package manager를 Bun으로 변경해 TASK-006에서 DEC-035·ADR-009와
  영향 문서를 먼저 갱신했다. 로컬 Bun 1.3.13을 확인했다.
- 2026-08-25: 사용자가 로컬 설치를 asdf로 통일해 TASK-007에서 DEC-036·ADR-010과 영향
  문서를 먼저 갱신했다.
- 2026-08-25: asdf로 Rust 1.90.0을 설치하고 Rust·Bun·Node 버전을 `.tool-versions`와 자동
  검증으로 고정했다.
- 2026-08-25: Cargo 15개 package, Bun 5개 workspace와 web·API·worker bootstrap을 구성했다.
- 2026-08-25: Rust·JavaScript forbidden edge와 cycle 검사, format·lint·typecheck·test,
  secret·license 검사와 GitHub Actions CI를 연결했다.
- 2026-08-25: 별도 임시 디렉터리에서 `bun ci`부터 SSR·CSR build까지 clean bootstrap을
  재실행해 lockfile과 전체 gate를 검증했다.

## 결과

IMP-01의 clean bootstrap과 forbidden edge gate를 통과했다. 다음 구현 package는 IMP-02이며
별도 TASK에서 계약 생성 기반을 구현한다.
