# TASK-004: 구현 수준 상세 설계 보강

- **상태**: 완료
- **유형**: 설계
- **시작일**: 2026-08-24
- **완료일**: 2026-08-24
- **커밋**: —

## 목적

현재 전체 제품 설계를 실제 코드를 작성할 수 있는 타입·SQL·화면·상태·검증 수준으로
보강한다. 문서 개수나 설명량이 아니라 구현자가 제품 정책을 새로 결정하지 않아도 되는지를
완료 기준으로 삼는다.

## 범위

- 포함: 구현 공백 감사, 전체 command/query/error catalog, 완전한 OpenAPI·AsyncAPI,
  Content·Operation·AI Result JSON Schema, PostgreSQL DDL·constraint·index, projection schema,
  route·화면·Atlaskit component·editor command matrix, 권한·상태 전이·알고리즘, test fixture·
  contract·E2E 명세, implementation work breakdown과 재동결
- 제외: 애플리케이션 코드, 실제 DB migration 실행, dependency 설치, 외부 계정·인프라 변경

## 필수 설계 문서

- [x] PROD-01~17
- [x] DOM-00~06
- [x] UX-01~12
- [x] ARCH-01~08와 ADR-001~008
- [x] DATA-01~06, API-01~05, SPEC-01~16
- [x] SEC-01~04, PRIV-01, TEST-01~06, OPS-01~07
- [x] 구현 공백 감사와 추가 문서 지도
- [x] 완전한 기계 계약과 DDL
- [x] 화면·component·interaction 상세 계약
- [x] 상태·권한·오류·알고리즘 교차 계약
- [x] 실행 가능한 검증 명세
- [x] 재동결 보고서

## 문서 준비 게이트

- [x] 모든 UI action을 route/API command와 연결했다.
- [x] 모든 command에 권한, expected revision, idempotency, transaction과 error가 있다.
- [x] 모든 저장 invariant가 DDL constraint 또는 application owner로 배정됐다.
- [x] 모든 비동기 흐름에 event, ordering, retry, cancellation과 recovery가 있다.
- [x] 모든 제품 요구사항에 정상·권한·동시성·복구 test가 있다.
- [x] 구현자가 제품 정책을 추정해야 하는 공백이 없다.

## 사용자 결정

현재 새 사용자 결정은 없다. 기존 DEC-000~034를 변경하지 않고 구체화한다. 감사 중 제품
범위·보안·데이터 수명주기·핵심 UX를 바꾸는 선택이 발견되면 해당 부분을 멈추고 사용자에게
상황·대안·권장안을 요청한다.

## 의사결정

### 결정 1: 기존 문서를 구현 계약까지 보강한다

- **상황**: 문서 종류와 아키텍처 경계는 충분하지만 일부 schema가 `additionalProperties`로
  남고 실제 DDL·화면 action 계약이 없어 구현자가 정책을 결정해야 한다.
- **검토한 대안**: 현재 문서로 구현 / 설명 문서 추가 / 기계 계약과 matrix 중심 보강.
- **선택과 근거**: 기존 정본을 유지하고 code generation·DDL·test가 검증할 수 있는 문서만
  추가하거나 보강한다.

## 작업 내역

- 2026-08-24: 사용자의 요청에 따라 구현 수준 상세 설계 보강 태스크를 등록했다.
- 2026-08-24: PLAN-05의 구현 가능 판정을 재검토 상태로 전환했다.
- 2026-08-24: 공백 7개를 감사하고 UX·Data·API·Contract·Spec·Test·Plan 상세 문서 24개와
  deterministic fixture 8개를 작성했다.
- 2026-08-24: API Catalog와 OpenAPI 105 operation을 일치시키고 Content·Operation·AI·Event
  payload를 JSON Schema 정본으로 연결했다.
- 2026-08-24: PostgreSQL 16 clean DB에 최종 DDL을 적용하고 41 table, 91 index, 230 constraint와
  append-only·Manage negative behavior를 검증했다.
- 2026-08-24: 화면 22개, DB invariant 30개, algorithm 12개, 구현 package 28개와 Gherkin
  scenario 15개를 교차 연결하고 전체 설계를 재동결했다.

## 이슈 및 해결

### 이슈 1: 기존 동결 판정이 상세도를 과대평가함

- **증상**: OpenAPI가 일부 endpoint만 포함하고 핵심 JSON payload가 자유 형식이며 실제 DDL과
  화면별 component·action 계약이 없다.
- **근본 원인**: 전체 문서 존재와 구현 가능한 상세도를 같은 완료 조건으로 판단했다.
- **구조적 해결**: UI→API→application→DB→event→test의 양방향 coverage matrix와 기계 검증
  가능한 schema를 새 동결 gate로 추가한다.

### 이슈 2: 계약 구체화 중 상태·schema 조합 불일치

- **증상**: Review의 `OPEN`과 `REQUESTED`, AI terminal state가 문서·DDL 사이에서 달랐고
  Operation Schema의 공통 `additionalProperties:false`가 subtype field까지 거부했다.
- **조사**: Domain state 정본, AsyncAPI event, OpenAPI enum, DDL enum과 Ajv compile을 대조했다.
- **근본 원인**: 설명 모델을 기계 계약으로 옮길 때 공통 object composition과 enum 정본을
  함께 검증하지 않았다.
- **구조적 해결**: Review는 REQUESTED 정본, Job terminal 집합은 단일 catalog로 맞추고 Schema는
  `allOf`+`unevaluatedProperties:false`로 subtype 전체를 검증했다.

## 검증

- [x] Markdown link와 문서 ID
- [x] JSON Schema·OpenAPI·AsyncAPI parse와 reference
- [x] PostgreSQL DDL parse·constraint inventory
- [x] UI action/API/error/test coverage
- [x] `git diff --check`

## 결과

PLAN-09의 7개 구현 공백을 모두 해소했고 PLAN-05를 재동결했다. 다음 구현 태스크는
PLAN-08의 IMP-01부터 시작한다.
