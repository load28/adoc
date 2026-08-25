# TASK-047: AI CLI 교차 플랫폼 테스트 계약 정정

- **상태**: 완료
- **유형**: 결함·설계
- **시작일**: 2026-08-25
- **완료일**: 2026-08-25
- **커밋**: —

## 목적

GitHub Actions Linux에서만 실패한 Codex CLI adapter 테스트의 프로세스 I/O 경쟁 조건을 제거하고,
실제 CLI의 stdin·stdout·result file 계약을 운영체제 스케줄링과 무관하게 검증한다.

## 범위

- 포함: CLI test double의 stdin 수명주기, Linux 재현, adapter 계약 회귀 검사, CI 재검증
- 제외: runtime 제품 동작 변경, provider wire format 변경, AI 제품 정책 변경

## 필수 설계 문서

- [x] PRD: N/A — 제품 범위가 바뀌지 않는다.
- [x] 도메인: N/A — 도메인 의미와 상태 전이가 바뀌지 않는다.
- [x] UX 흐름: N/A — 사용자 화면이 바뀌지 않는다.
- [x] 데이터 모델·상태 전이: N/A — 저장 계약이 바뀌지 않는다.
- [x] API·이벤트 계약: `docs/design/implementation/AI-CONTEXT-RUNTIME-ADAPTERS.md` §7, §12
- [x] 권한·보안: 고정 stdin과 빈 temp root 계약을 그대로 검증한다.
- [x] 실패·복구·동시성: 자식 프로세스의 stdin EOF 전 종료 경쟁 조건을 정의한다.
- [x] 테스트 전략: macOS와 Linux에서 동일한 port 결과를 검증한다.

## 문서 준비 게이트

- [x] 구현 결과를 바꿀 미해결 질문이 없다.
- [x] 정상·예외·권한·동시성 흐름이 정의되어 있다.
- [x] 경계를 넘는 데이터 계약이 구체적으로 정의되어 있다.
- [x] 구현 단위와 테스트 완료 조건을 문서에서 추적할 수 있다.
- [x] 코드 작성 가능: 상세 설계가 stdin EOF와 결과 생성 순서를 소유한다.

## 사용자 결정

없음.

## 의사결정

### 결정 1: test double도 실제 stdin 수명주기 계약을 따른다

- **상황**: 기존 test double은 stdin을 읽지 않고 종료해 Linux에서 부모의 `write_all`과 경쟁했다.
- **검토한 대안**: 제품 adapter에서 `EPIPE`를 무시하면 실제 provider 입력 전달 실패까지 숨긴다. 테스트에
  지연을 넣으면 스케줄러 의존성을 유지한다. test double이 stdin을 EOF까지 소비하면 실제 CLI 계약을
  결정적으로 재현한다.
- **선택과 근거**: test double이 stdin EOF를 확인한 뒤 result file과 JSONL을 생성한다. 제품의
  `AI_CLI_IO_FAILED` 의미는 변경하지 않으며 Linux container와 CI에서 동일 테스트로 확인한다.

## 작업 내역

- 2026-08-25: GitHub Actions run 32843002029의 `AI_CLI_IO_FAILED` 실패를 확인했다.
- 2026-08-25: 자식 test double의 조기 종료와 부모 stdin 쓰기 사이의 경쟁 조건을 근본 원인으로 확인했다.
- 2026-08-25: CLI fixture의 stdin EOF·결과 생성 순서를 상세 설계에 추가했다.
- 2026-08-25: fixture가 stdin EOF를 소비하게 고치고 macOS·Linux container에서 같은 테스트를 통과했다.
- 2026-08-25: root verification 전체를 통과했다.

## 이슈 및 해결

- macOS에서는 통과하지만 Linux에서는 test double이 먼저 종료되어 stdin 쓰기가 `EPIPE`로 실패했다.
  실제 CLI가 소비하는 stdin을 test double이 무시한 계약 불일치가 원인이었다.

## 검증

- [x] macOS targeted adapter test
- [x] Linux container targeted adapter test
- [x] root verification
- [x] clean release candidate와 GitHub Actions 재검증 절차 연결

## 결과

제품 adapter의 I/O 오류 의미를 완화하지 않고 test double을 실제 provider 수명주기에 맞췄다. 운영체제
스케줄링과 무관하게 동일한 `RuntimeResult` 계약을 검증한다.
