# TASK-040: Editor·Publish·Version 사용자 여정 완성

- **상태**: 대기
- **유형**: 구현
- **시작일**: —
- **완료일**: —
- **커밋**: —

## 목적

전체 editor command와 Draft → Review/Publish → immutable Version → Diff/restore/conflict 흐름을
Web에서 끊김 없이 제공해 SCR-05·06·09·15와 RQ-05~08·15를 완성한다.

## 범위

- 포함: Published content, toolbar·slash·markdown shortcut·keymap·DnD·multi-select, table/code/media,
  Markdown/plain import, Markdown/plain/PDF export, publish dialog, policy impact, history/diff/restore,
  3-way conflict, File preview/download와 recovery
- 제외: Discussion·Review 상세 composer와 Settings 관리 화면

## 필수 설계 문서

- PROD-11·12·16, DOM-02·06, UX-05·06·10·12~16, SPEC-05~08·15·17~19
- CONTRACT-01·02, DATA-07·08, API-01·02·06~08, SEC-03·04, TEST-01·03·04·07·08
- PLAN-35 및 이 태스크에서 작성할 구현 계약

## 문서 준비 게이트

- [ ] command·selection·operation·inverse·import/export 타입 계약 확정
- [ ] lease loss·upload failure·stale publish·restore conflict·recovery 정의
- [ ] Published immutable read와 Draft mutation 경계 정의
- [ ] 구현 단위와 browser 검증 조건 추적

## 사용자 결정

없음. 정본의 전체 첫 구현 범위를 축소하지 않는다.

## 의사결정

구현 시작 시 대안·선택·검증 근거를 기록한다.

## 작업 내역

대기.

## 이슈 및 해결

없음.

## 검증

- [ ] editor command·schema·operation property test
- [ ] publish/version/diff/restore/conflict integration test
- [ ] import/export round-trip·file lifecycle test
- [ ] root gate와 Compose integration

## 결과

대기.
