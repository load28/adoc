# TASK-007: asdf 로컬 툴체인 표준화

- **상태**: 완료
- **유형**: 설계
- **시작일**: 2026-08-25
- **완료일**: 2026-08-25
- **커밋**: —

## 목적

개발자가 사용하는 Rust·Bun·Node 설치와 버전 선택을 asdf로 통일하고, manifest·lockfile·CI
버전과 로컬 실행 환경이 달라지는 문제를 구조적으로 방지한다.

## 범위

- 포함: Decision Register, ADR, Technology Selection, Document Map, Design Freeze Report,
  IMP-01과 TASK-005의 로컬 toolchain 계약 갱신
- 제외: production container의 package manager, 애플리케이션 runtime 변경, dependency 선택,
  애플리케이션 기능 구현

## 필수 설계 문서

- [x] PROD-08 `product/DECISION-REGISTER.md`
- [x] ARCH-04 `design/architecture/TECHNOLOGY-SELECTION.md`
- [x] ADR-001 `design/adr/ADR-001-monorepo-web-rust.md`
- [x] ADR-009 `design/adr/ADR-009-bun-package-manager.md`
- [x] PLAN-05 `design/implementation/DESIGN-FREEZE-REPORT.md`
- [x] PLAN-08 `design/implementation/WORK-BREAKDOWN.md`
- [x] TASK-005 `tasks/TASK-005-bootstrap-monorepo-toolchain.md`

## 문서 준비 게이트

- [x] 적용 대상을 로컬 개발 toolchain 설치·선택으로 한정했다.
- [x] `.tool-versions`를 로컬 버전의 단일 진실 소스로 정의했다.
- [x] Cargo·Bun lockfile과 CI의 정확한 버전 고정은 유지한다.
- [x] 사용자 결정 외 미해결 질문이 없다.

## 사용자 결정

### 결정 요청 1: 로컬 도구 설치 방식

- **상황**: TASK-005 검증 중 Rust toolchain 설치가 필요해졌고 사용자가 asdf 사용을 지시했다.
- **대안과 영향**: 도구별 설치는 환경마다 절차가 달라지고, asdf는 저장소 단위 버전과 설치
  명령을 통일한다.
- **권장안**: 사용자 지시에 따라 로컬 도구 설치와 버전 선택을 asdf로 고정한다.
- **사용자 결정**: asdf 사용, 2026-08-25

## 의사결정

### 결정 1: `.tool-versions`가 로컬 toolchain 정본이다

- **상황**: global 도구 버전이나 도구별 설치 절차에 의존하면 clean bootstrap을 재현할 수 없다.
- **검토한 대안**: global 도구 사용 / 도구별 manager 사용 / asdf local version 통일.
- **선택과 근거**: Rust·Bun·Node의 정확한 버전을 root `.tool-versions`에 기록하고 `asdf install`로
  설치한다. `rust-toolchain.toml`, `packageManager`, engine과 CI 버전은 같은 값을 검증한다.

## 작업 내역

- 2026-08-25: 사용자의 asdf 설치 결정을 기록하고 영향 문서를 갱신했다.

## 이슈 및 해결

없음.

## 검증

- [x] DEC·ADR·Technology·Plan·Task 계약 일치
- [x] Document Map 130개와 Design Freeze snapshot 일치
- [x] `git diff --check` 통과

## 결과

로컬 개발 toolchain은 asdf와 root `.tool-versions`를 사용하도록 설계를 재동결했다.
