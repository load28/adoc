# TASK-050: CI 명시 실행 경계

- **상태**: 완료
- **유형**: 설계·구현·운영
- **시작일**: 2026-08-25
- **완료일**: 2026-08-25
- **커밋**: —

## 목적

Git push와 pull request가 원격 CI를 자동 실행하지 않게 한다. 개발자는 로컬 전체 CI를 단일
명령으로 명시 실행하고, GitHub Actions도 운영자가 수동 호출했을 때만 같은 품질 범위를
검증하게 한다.

## 범위

- 포함: CI 트리거, 로컬 전체 CI 진입점, 공급망 계약 검사, CI/CD·릴리스 정본 문서
- 제외: 품질 게이트 자체의 완화, 배포 자동화, 제품·도메인·데이터·API 변경

## 필수 설계 문서

- [x] 관련 PRD: N/A — 제품 범위와 사용자 가치는 바뀌지 않는다.
- [x] 관련 도메인 문서: N/A — 도메인 불변식과 상태 전이는 바뀌지 않는다.
- [x] UX 흐름: N/A — 제품 화면을 변경하지 않는다.
- [x] 데이터 모델·상태 전이: N/A — 애플리케이션 데이터를 변경하지 않는다.
- [x] API·이벤트 계약: N/A — 런타임 경계를 변경하지 않는다.
- [x] 권한·보안: `docs/design/operations/CI-CD.md`
- [x] 실패·복구·동시성: `docs/design/operations/CI-CD.md`
- [x] 테스트 전략: `docs/design/operations/CI-CD.md`
- [x] 릴리스 계약: `docs/design/operations/RELEASE-RUNBOOK.md`,
  `docs/design/implementation/FULL-ACCEPTANCE-RELEASE.md`

## 문서 준비 게이트

- [x] 구현 결과를 바꿀 미해결 질문이 없다.
- [x] 정상·예외·권한·동시성 흐름이 정의되어 있다.
- [x] 경계를 넘는 데이터 계약이 구체적으로 정의되어 있다.
- [x] 구현 단위와 테스트 완료 조건을 문서에서 추적할 수 있다.
- [x] 코드 작성 가능 여부와 근거를 기록했다.

코드 작성 가능. OPS-02가 로컬 명령, 원격 수동 트리거, 실패 의미와 계약 검사를 소유한다.

## 사용자 결정

### 결정 1: CI 실행 방식

- **상황**: push·pull request가 GitHub Actions를 자동 실행하고 있다.
- **대안과 영향**: 자동 트리거 유지, 일부 이벤트만 제외, 모든 자동 트리거 제거가 가능하다.
- **권장안**: 로컬은 단일 명령, GitHub는 `workflow_dispatch`만 허용한다.
- **사용자 결정**: 2026-08-25 모든 자동 실행을 금지하고 양쪽 모두 명시 실행하도록 결정했다.

## 의사결정

### 결정 1: 실행 정책과 품질 게이트를 분리한다

- **상황**: CI 실행 비용 정책을 바꾸되 검증 범위를 약화하면 안 된다.
- **검토한 대안**: 기존 `ci`를 부분 검사로 유지하면 이름과 동작이 어긋난다. GitHub 전용
  스크립트는 로컬과 원격 검증 범위를 분기시킨다.
- **선택과 근거**: `ci:local`을 전체 로컬 게이트의 정본으로 두고 공급망 검사에서 필수
  단계와 원격 `workflow_dispatch` 전용 트리거를 함께 검증한다.

## 작업 내역

- 2026-08-25: 태스크를 등록하고 OPS-02·OPS-06·PLAN-34의 실행 정책을 먼저 갱신했다.
- 2026-08-25: GitHub workflow trigger를 `workflow_dispatch` 단일 event로 제한했다.
- 2026-08-25: `ci:local`에 repository·audit·readiness·Compose·browser gate를 순서대로 연결했다.
- 2026-08-25: 공급망 검사에 trigger와 로컬 전체 CI 단계·순서의 negative self-test를 추가했다.
- 2026-08-25: 로컬 전체 CI와 Agent Browser 화면 검증을 수행하고 임시 자원을 정리했다.

## 이슈 및 해결

### Cargo advisory DB 잠금 실패

- **증상**: 제한된 실행 환경에서 `cargo audit`가 사용자 Cargo 디렉터리의 잠금 파일을 만들지
  못해 로컬 CI가 중단됐다.
- **조사**: 실패 위치가 코드·의존성 결과가 아니라 read-only 경로의 advisory DB lock임을
  확인했다.
- **근본 원인**: 검증 프로세스의 사용자 Cargo cache 쓰기 권한이 제한됐다.
- **구조적 해결**: 명령을 우회하거나 감사를 생략하지 않고 정상 로컬 권한에서 동일한
  `bun run ci:local` 전체 명령을 재실행해 통과했다.

## 검증

- [x] 문서 링크와 정본 경계 확인
- [x] PRD·도메인·상세 설계 간 모순 확인
- [x] 공급망 계약 self-test
- [x] 로컬 전체 CI

## 결과

GitHub Actions 자동 event trigger를 제거했다. 로컬 전체 CI, 공급망 negative self-test,
Compose 인수·복구, 54개 브라우저 테스트와 Agent Browser 실제 화면 검증이 모두 통과했다.
