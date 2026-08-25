# TASK-042: Web API client·화면 계약 완전성

- **상태**: 대기
- **유형**: 구현·품질
- **시작일**: —
- **완료일**: —
- **커밋**: —

## 목적

OpenAPI operation, API client method, SCR primary action과 실제 Web 호출의 집합 차이를 0으로 만들어
화면 구현 누락이 다시 숨겨지지 않게 한다.

## 범위

- 포함: generated typed client 또는 동등한 단일 생성 경계, operation/action manifest, route loader와
  mutation mapping, unused·missing·manual drift 검사
- 제외: 각 도메인 action의 UI 자체와 operation별 backend assertion

## 필수 설계 문서

- PROD-09, UX-13·15, API-01·02·06~08, TEST-01·08, PLAN-06·35
- 이 태스크에서 작성할 client generation·coverage 구현 계약

## 문서 준비 게이트

- [ ] OpenAPI를 단일 method 이름·타입 정본으로 사용하는 방식 확정
- [ ] 화면 action 중 local action과 HTTP operation 구분 정의
- [ ] 누락·orphan·manual drift 실패 조건 정의
- [ ] 생성물 갱신과 CI 재현성 정의

## 사용자 결정

없음.

## 의사결정

구현 시작 시 대안·선택·검증 근거를 기록한다.

## 작업 내역

대기.

## 이슈 및 해결

없음.

## 검증

- [ ] OpenAPI↔client exact diff 0
- [ ] SCR action↔route/component exact diff 0
- [ ] negative self-test가 누락·중복·orphan을 거부
- [ ] generated diff와 root gate clean

## 결과

대기.
