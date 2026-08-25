# Design Freeze Report

- **문서 ID**: PLAN-05
- **상태**: 동결
- **판정일**: 2026-08-24
- **대상 결정 snapshot**: DEC-000~036

## 판정

전체 제품 설계는 구현 준비 상태다. 구현자는 제품 정책, 상태 전이, 권한, payload, 저장 제약,
화면 행동과 검증 방식을 새로 결정하지 않고 IMP-01부터 dependency DAG에 따라 작업할 수 있다.
이 판정은 전체 범위 구현을 승인하는 문서 gate이며 MVP나 부분 출시 승인이 아니다.

## Gate 결과

| Gate | 결과 | 근거 |
|---|---|---|
| 사용자 차단 결정 | 통과 | Decision Register 대기 0, DEC-000~036 유지 |
| 전체 문서 상태 | 통과 | Document Map mapped artifact 130개 동결 |
| 제품·도메인 | 통과 | PROD-01~17, DOM-00~06, RQ-01~20 traceability |
| 화면·디자인 | 통과 | SCR-01~22, UX-17 조사, UX-18 원칙, UX-19 Story, shadcn matrix, route/state, keymap |
| HTTP·event | 통과 | Catalog/OpenAPI 105 operation 1:1, AsyncAPI 3.1 |
| Payload type | 통과 | Content·Operation·AI·Event JSON Schema 2020-12 |
| 저장·동시성 | 통과 | PostgreSQL DDL, DB invariant 30개, lock order |
| 권한·보안 | 통과 | action matrix, point/scope resolver, pre-filter contract |
| 상태·알고리즘 | 통과 | aggregate transition, 핵심 algorithm 12개 |
| 검증 | 통과 | fixture corpus, contract coverage, Gherkin 15개 |
| 구현 분해 | 통과 | module port, config reference, IMP-01~28 DAG |

## 자동·실행 검증 evidence

- OpenAPI 3.1: Redocly validator 통과, 105 operation·79 path, duplicate 0.
- AsyncAPI 3.1: AsyncAPI CLI validator 통과, external Event Schema 해석.
- JSON Schema: Ajv draft 2020-12 compile 통과, format validator 포함.
- Fixture: positive 4개 승인, negative 4개 거부.
- PostgreSQL 16: clean DB에 41 table, 91 index, 230 constraint 전체 적용.
- DB behavior: Published Version update와 VIEWER+Manage insert가 예상대로 거부됨.
- Gherkin: 15개 scenario 전용 parser 통과.
- 문서: mapped path·ID, Markdown relative link, YAML·JSON parse, external fragment와
  `git diff --check` 통과.

## 정본 일관성

- Workspace role은 Document content access를 우회하지 않는다.
- Published Version은 불변이고 restore·merge는 Draft revision을 만든다.
- Draft 변경은 REQUESTED·APPROVED Review를 INVALIDATED로 만든다.
- Search·AI는 candidate 생성 전에 Permission Scope를 적용한다.
- PostgreSQL·ObjectStorage만 복구 정본이고 Redis·OpenSearch는 재구축 가능하다.
- 좁은 Rewrite 외 AI 변경은 Proposal·Diff·사람 승인을 거친다.
- 공개 link는 단일 최신 Published Document와 exact embedded File만 제공한다.
- UI는 Tailwind CSS와 저장소 소유 shadcn/ui New York source를 단일 기반으로 사용한다.

## 구현 중 금지되는 우회

permission 사후 필터링, owner context table 직접 write, Version update, stale revision 덮어쓰기,
WebSocket·CRDT 추가, 개인 AI subscription credential 저장, provider silent fallback, custom design
token·병행 component library와 Public Viewer의 Workspace API 재사용을 금지한다.

## 재검토 trigger

제품 범위, 권한 precedence, retention, AI provider 계약, Content·Operation schema, public link 경계,
React/UI foundation major version, 배포 topology 또는 SLO가 바뀌면 관련 설계 task에서 정본, ADR,
schema, test와 이 보고서를 함께 다시 연다.

Bun의 적용 범위가 package manager·workspace script runner를 넘어 production runtime으로
바뀌는 경우에도 ADR-009와 배포 설계를 다시 연다.

로컬 toolchain manager나 `.tool-versions`의 정본 지위가 바뀌는 경우 ADR-010과 clean
bootstrap 검증을 다시 연다.
