# TASK-041: Collaboration·Knowledge·Governance 사용자 여정 완성

- **상태**: 대기
- **유형**: 구현
- **시작일**: —
- **완료일**: —
- **커밋**: —

## 목적

현재 일부 action만 연결된 Collaboration·Knowledge·AI·Settings 화면을 정본 primary action과
상태 전이에 맞춰 완성해 SCR-07·08·10~14·16~21과 RQ-02·03·09~16을 충족한다.

## 범위

- 포함: Topic/Message/reference/attachment, Review request·approve·changes request·cancel, Inbox
  navigation, Reference mutation/backlink, Vocabulary lifecycle, AI 6 task/context/source/proposal,
  Group membership, Permission delete/explain·PublishPolicy, Writing Rules, Audit filter/detail
- 제외: editor command 자체와 실제 browser matrix·운영 infrastructure

## 필수 설계 문서

- PROD-10·13~16, DOM-01·03~06, UX-04·07·08·10·12~15, SPEC-02·03·09~19
- CONTRACT-03·04, DATA-07~09, API-01~08, SEC-03·04, TEST-01·03~05·07·08
- PLAN-35 및 이 태스크에서 작성할 구현 계약

## 문서 준비 게이트

- [ ] 화면별 primary action과 aggregate transition을 1:1로 고정
- [ ] permission scope가 query/search/AI/file보다 먼저 적용됨을 정의
- [ ] stale·cancel·retry·invalid proposal·denied source 복구 흐름 정의
- [ ] 구현 단위와 exact test ID 추적

## 사용자 결정

없음. 정본의 사람 승인과 AI 비자율 원칙을 따른다.

## 의사결정

구현 시작 시 대안·선택·검증 근거를 기록한다.

## 작업 내역

대기.

## 이슈 및 해결

없음.

## 검증

- [ ] 각 화면 action·state component/integration test
- [ ] cross-tenant·permission·stale·idempotency negative test
- [ ] event·Inbox·Audit·Outbox 일치 test
- [ ] root gate와 Compose integration

## 결과

대기.
