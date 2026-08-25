# TASK-044: Browser E2E·접근성·시각·호환성 검증

- **상태**: 대기
- **유형**: 품질
- **시작일**: —
- **완료일**: —
- **커밋**: —

## 목적

TEST-09 전체 사용자 여정을 실제 Chromium·Firefox·WebKit에서 실행하고 WCAG·responsive·visual
계약을 검증해 jsdom 수준의 간접 증거를 대체한다.

## 범위

- 포함: Playwright 기반 SSR/CSR E2E, multi-session, dependency failure, axe, keyboard-only, focus,
  compact/wide visual snapshot, ko/en, Chrome/Edge/Firefox/Safari 대응 browser engine matrix
- 제외: 실제 사람 screen-reader 세션과 production credential은 environment evidence로 분리

## 필수 설계 문서

- PROD-04~06·09, UX-02·03·10·12·13·15·16, SEC-02~04
- TEST-01~04·06~09, OPS-02, PLAN-03·34·35 및 이 태스크에서 작성할 browser harness 계약

## 문서 준비 게이트

- [ ] scenario fixture·session·browser·viewport matrix 정의
- [ ] deterministic visual threshold와 font/time/animation 고정 정의
- [ ] keyboard·focus·axe 실패 조건과 수동 evidence 경계 정의
- [ ] CI shard·artifact·failure reproduction 정의

## 사용자 결정

없음.

## 의사결정

구현 시작 시 대안·선택·검증 근거를 기록한다.

## 작업 내역

대기.

## 이슈 및 해결

없음.

## 검증

- [ ] TEST-09 scenario 1:1 browser execution
- [ ] Chromium·Firefox·WebKit wide/compact gate
- [ ] automated a11y·keyboard·visual gate
- [ ] failure screenshot·trace·seed artifact

## 결과

대기.
