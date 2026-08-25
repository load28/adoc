# TASK-052: Web SSR runtime bundle 경계 복구

- **상태**: 완료
- **유형**: 결함·설계·구현·운영
- **시작일**: 2026-08-26
- **완료일**: 2026-08-26
- **커밋**: —

## 목적

TASK-051 이후 Web Docker image가 production SSR external dependency를 해석하지 못해 종료되는
결함을 해결한다. 개발 SSR과 경량 production runtime이 서로 다른 dependency bundling 계약을
명시적으로 갖도록 복구한다.

## 범위

- 포함: Vite SSR dependency bundling mode, Web image runtime 검증, 관련 구현 정본
- 제외: UI·제품 기능·API·도메인·데이터 변경, Google OIDC 설정 변경

## 필수 설계 문서

- [x] 관련 PRD: N/A — 제품 범위와 동작을 변경하지 않는 build 결함이다.
- [x] 관련 도메인 문서: N/A — 도메인 의미와 상태 전이를 변경하지 않는다.
- [x] UX 흐름: N/A — 화면·흐름을 변경하지 않는다.
- [x] 데이터 모델·API·이벤트·권한·동시성: N/A — 경계 계약을 변경하지 않는다.
- [x] 실패·복구: `PLAN-24`, `PLAN-28`
- [x] 테스트 전략: `PLAN-24`, production build와 Compose Web health

## 문서 준비 게이트

- [x] 구현 결과를 바꿀 미해결 질문이 없다.
- [x] 정상·예외·권한·동시성 흐름이 정의되어 있다.
- [x] 경계를 넘는 데이터 계약이 구체적으로 정의되어 있다.
- [x] 구현 단위와 테스트 완료 조건을 문서에서 추적할 수 있다.
- [x] 코드 작성 가능 여부와 근거를 기록했다.

코드 작성 가능. PLAN-24의 self-contained SSR artifact 계약을 PLAN-28의 build mode별 Vite 설정과
Compose health 검증으로 구체화한다.

## 사용자 결정

없음.

## 의사결정

### 결정 1: production SSR만 dependency를 bundle한다

- **상황**: 개발 SSR에서 CommonJS React를 inline하면 `module is not defined`가 발생하고,
  production SSR에서 externalize하면 `node_modules`가 없는 runtime image가 시작하지 못한다.
- **검토한 대안**: runtime image에 전체 `node_modules`를 복사하면 단순하지만 image 크기와 공급망
  surface가 커진다. 모든 mode를 inline하면 개발 SSR의 CommonJS 해석이 다시 깨진다.
- **선택과 근거**: Vite `build` command에서만 `ssr.noExternal: true`를 사용한다. 개발 server는
  package externalization을 유지하고 production은 PLAN-24대로 self-contained artifact를 만든다.

## 작업 내역

- 2026-08-26: `adoc-web-1` 종료 상태와 로그를 확인했다.
- 2026-08-26: production SSR output이 `react`와 `react/jsx-runtime`을 external import로 남기며
  Web runtime image에는 `node_modules`가 없는 것을 확인했다.
- 2026-08-26: Vite command별 SSR bundling을 분리하고 production server artifact의 bare UI
  package import가 0임을 확인했다.
- 2026-08-26: Web image를 재빌드하고 Compose health와 `/login` HTTP 응답을 확인했다.

## 이슈 및 해결

- **증상**: Web container가 `Cannot find package 'react'`로 exit 1 했다.
- **조사**: Compose log, Dockerfile runtime copy 경계와 SSR output import를 대조했다.
- **근본 원인**: 개발 SSR 보정 과정에서 production self-contained bundle 계약까지 제거됐다.
- **구조적 해결**: build command에만 dependency inline을 적용한다.

## 검증

- [x] 문서 링크와 정본 경계 확인
- [x] PRD·도메인·상세 설계 간 모순 확인
- [x] typecheck·25 tests·production build 통과
- [x] SSR output external React import 0
- [x] Compose Web container healthy, `/login` HTTP 200

## 결과

개발 SSR은 package externalization을 유지하고 production SSR만 dependency를 bundle하도록 Vite
계약을 분리했다. 경량 Web runtime image를 유지하면서 Compose Web 시작 실패를 해결했다.
