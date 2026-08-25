# TASK-042: Web API client·화면 계약 완전성

- **상태**: 완료
- **유형**: 구현·품질
- **시작일**: 2026-08-25
- **완료일**: 2026-08-25
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

- [x] OpenAPI를 단일 method 이름·타입 정본으로 사용하는 방식 확정
- [x] 화면 action 중 local action과 HTTP operation 구분 정의
- [x] 누락·orphan·manual drift 실패 조건 정의
- [x] 생성물 갱신과 CI 재현성 정의

## 사용자 결정

없음.

## 의사결정

- OpenAPI에서 client 전체를 생성하는 방식과 handwritten transport를 manifest로 검증하는 방식을
  검토했다. exact revision·lease·idempotency를 묶은 제품 command가 단순 endpoint 호출보다 강한
  경계이므로 handwritten client를 유지하되 모든 operation을 OpenAPI ID로 선언하는 manifest를 선택했다.
- 화면 source를 정규식으로 추론하는 방식과 명시적인 screen/action manifest를 검토했다. alias와
  composite command를 안정적으로 표현하기 위해 SCR별 loader·HTTP action·local action을 닫힌 manifest로
  선언하고 검사기가 실제 client method와 screen module export를 함께 검증하도록 선택했다.

## 작업 내역

- 2026-08-25: TASK-041 완료 뒤 후속 DAG의 네 번째 구현 태스크로 시작했다.
- 2026-08-25: PLAN-39에 OpenAPI operation·typed client command·SCR action·runtime module의 네 집합과
  누락·중복·orphan·manual drift 실패 조건을 고정했다.
- 2026-08-25: 109개 operation을 100개 `ApiClient` method와 navigation·callback·stream·binary
  runtime surface에 전부 연결하고 SCR-01~22의 loader·action·local action manifest를 작성했다.
- 2026-08-25: 계정 preference·logout, Workspace 수명주기, 공개 링크, File·Group·Vocabulary 단건
  command를 typed client와 실제 화면 module에 연결했다.
- 2026-08-25: 누락·orphan·중복·존재하지 않는 owner를 거부하는 재현 가능한 검사기를 root
  `contracts:check`에 편입했다.

## 이슈 및 해결

- Session에 포함된 Workspace summary만 사용해 `listWorkspaces`와 `getWorkspace` 계약이 실제 loader에서
  사라져 있었다. 인증 cookie를 전달하는 server loader가 목록과 단건 endpoint를 명시적으로 호출하도록 했다.
- 로그인·SSE·파일 URL이 화면별 문자열로 조합되어 OpenAPI 변경을 감지할 수 없었다. URL 생성 함수를
  UI-domain 단일 경계로 옮기고 navigation·stream·binary operation owner로 등록했다.
- OpenAPI에는 있지만 Web client에 없는 계정·Workspace 삭제·공개 링크·단건 resource operation이 있었다.
  generated schema 타입과 exact revision header를 사용하는 command를 추가하고 실제 SCR module에 연결했다.

## 검증

- [x] OpenAPI↔client exact diff 0
- [x] SCR action↔route/component exact diff 0
- [x] negative self-test가 누락·중복·orphan을 거부
- [x] generated diff와 root gate clean

## 결과

`bun run check`를 통과했다. coverage 결과는 HTTP operation 109개, public client method 100개,
SCR 22개다. `bun run compose:integration`도 13개 실제 인프라 계약과 backup·restore까지 통과했다.
성능 smoke p95는 API readiness 1.779ms, Web live 0.636ms, SSR login 6.376ms였다.
