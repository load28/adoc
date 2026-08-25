# TASK-039: Workspace·Document Tree 사용자 여정 완성

- **상태**: 대기
- **유형**: 구현
- **시작일**: —
- **완료일**: —
- **커밋**: —

## 목적

로그인 뒤 Workspace 생성·초대 수락·전환과 Document Tree 생성·탐색·변경이 Web에서 끝까지
동작하도록 SCR-01~04와 RQ-01·02·04를 완성한다.

## 범위

- 포함: invitation preview/accept, Workspace create/switch/home, permission-scoped tree, Document
  create/rename/move/sort/trash 진입, loading·empty·denied·conflict·responsive 상태
- 제외: 본문 편집·Publish와 Settings의 상세 Governance mutation

## 필수 설계 문서

- PROD-05·10·11, DOM-01·02, UX-01~04·10·12~15, SPEC-01·02·04·18·19
- DATA-07·08, API-01·02·06~08, SEC-02·03, TEST-01·03·04·07·08
- PLAN-35 및 이 태스크에서 작성할 구현 계약

## 문서 준비 게이트

- [ ] route loader·action·cache·permission·revision 계약을 상세 설계에 고정
- [ ] invitation·tree의 정상·empty·denied·stale·복구 흐름 정의
- [ ] 필요한 API client method와 exact test ID 정의
- [ ] 구현 가능 여부와 문서 근거 기록

## 사용자 결정

없음. 기존 동결 설계와 Atlaskit 공개 component 정책을 따른다.

## 의사결정

구현 시작 시 대안·선택·검증 근거를 기록한다.

## 작업 내역

대기.

## 이슈 및 해결

없음.

## 검증

- [ ] SCR-01~04 component·route integration test
- [ ] cross-tenant·permission·stale revision negative test
- [ ] ko/en·compact·keyboard·recovery 상태
- [ ] root gate와 Compose integration

## 결과

대기.
