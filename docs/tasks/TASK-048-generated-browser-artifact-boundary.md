# TASK-048: 브라우저 생성 산출물 검사 경계 정정

- **상태**: 완료
- **유형**: 결함·설계
- **시작일**: 2026-08-25
- **완료일**: 2026-08-25
- **커밋**: —

## 목적

브라우저 검증이 만든 Git 비추적 evidence를 소스 포매터가 다시 검사해 root gate를 깨뜨리는 경계
오류를 제거한다.

## 범위

- 포함: `artifacts/`의 생성 산출물 소유권, Biome 입력 경계, root gate 재실행
- 제외: 브라우저 evidence 내용 변경, Playwright reporter 변경, Git 추적 정책 변경

## 필수 설계 문서

- [x] PRD·도메인·UX·데이터·API·권한: N/A — 제품 계약이 바뀌지 않는다.
- [x] 실패·복구: `docs/design/implementation/BROWSER-QUALITY-GATES.md` §5
- [x] 테스트 전략: 생성 산출물이 존재하는 상태에서도 root source gate가 재실행 가능해야 한다.

## 문서 준비 게이트

- [x] 구현 결과를 바꿀 미해결 질문이 없다.
- [x] 생성 evidence와 source file의 소유권 경계가 정의되어 있다.
- [x] Biome exclusion과 evidence validator 책임을 추적할 수 있다.
- [x] 코드 작성 가능: Browser Quality Gates가 정본이다.

## 사용자 결정

없음.

## 의사결정

- 생성 evidence를 매번 포맷하거나 삭제하는 방식은 검증 도구의 원본 출력을 바꾸고 실행 순서 의존성을
  남긴다. `artifacts/`는 전용 검증기와 release evidence gate가 검사하고 source formatter에서는 제외한다.

## 작업 내역

- 2026-08-25: `bun run check`가 ignored Playwright JSON 두 파일의 포맷으로 실패하는 것을 확인했다.
- 2026-08-25: Browser Quality Gates에 생성 evidence의 검사 소유권을 정의했다.
- 2026-08-25: Biome source 입력에서 `artifacts/`를 제외하고 기존 evidence가 있는 상태에서 format과
  root gate를 통과했다.

## 이슈 및 해결

- `.gitignore`의 `artifacts/`가 Biome 입력에는 포함되어 이전 브라우저 실행 여부에 따라 root gate 결과가
  달라졌다. 생성물과 소스의 도구 경계가 분리되지 않은 것이 원인이다.

## 검증

- [x] 생성 browser evidence가 있는 상태의 format gate
- [x] root verification
- [x] clean release candidate와 GitHub Actions 재검증 절차 연결

## 결과

브라우저 원본 evidence는 전용 validator가 검사하고 source formatter는 추적 소스만 검사한다. 이전
브라우저 실행 여부와 무관하게 root gate를 재실행할 수 있다.
