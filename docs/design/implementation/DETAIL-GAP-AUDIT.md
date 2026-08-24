# 구현 상세도 공백 감사

- **문서 ID**: PLAN-09
- **상태**: 동결
- **감사 기준**: UI action → API → application port → transaction·storage → event → test를
  구현자가 새 정책 없이 추적할 수 있는가

## 결론

기존 문서의 구현 공백 7개를 모두 보강했다. 130개 mapped artifact가 제품→화면→API→port→
transaction·storage→event→test 경로를 제공하며 TASK-004의 새 coverage gate를 통과했다.

## G-01 API completeness

- **발견**: OpenAPI operation 30개만 존재한다. Member 변경·제거, Group 변경, tree query,
  rename·trash·restore, lease heartbeat·release, Version·Diff·restore, Discussion Message·Topic,
  Inbox, Vocabulary, Reference, AI Context·Proposal, File complete·download, Audit와 공개 link
  revoke 등이 없다.
- **해결 정본**: API-02, API-06~08.
- **완료 evidence**: Catalog 105개와 OpenAPI operationId 105개가 중복·차이 없이 1:1 대응한다.

## G-02 Payload specificity

- **발견**: Content, Operation, Message, AI target·result와 Event payload가 자유 형식 object다.
- **해결 정본**: CONTRACT-01~04. OpenAPI·AsyncAPI는 이 schema를 external `$ref`로 사용한다.
- **완료 evidence**: unconstrained payload 0, JSON Schema 2020-12 compile, positive·negative fixture
  8개 validation을 통과했다.

## G-03 Persistence executability

- **발견**: logical table 설명만 있고 enum, foreign key, unique, check, index, append-only trigger와
  transaction lock 순서가 실제 SQL로 고정되지 않았다.
- **해결 정본**: DATA-07~09.
- **완료 evidence**: PostgreSQL 16 clean DB에 41 table, 91 index, 230 constraint와 trigger를
  적용했고 immutable Version·Manage check negative test를 통과했다.

## G-04 UI executability

- **발견**: 화면 목록은 있으나 screen별 loader, permission, exact action, Atlaskit import,
  responsive frame, query invalidation과 error recovery가 한 계약으로 연결되지 않는다.
- **해결 정본**: UX-13~16.
- **완료 evidence**: SCR-01~22의 loader·action·완료 state를 exact Operation ID와 stable error
  action에 연결했다.

## G-05 Cross-domain behavior

- **발견**: 상태 전이·authorization·Permission merge·tree move·Region re-anchor·3-way merge·
  purge 알고리즘이 여러 문서에 분산돼 exhaustive 검증이 어렵다.
- **해결 정본**: SPEC-17~19.
- **완료 evidence**: mutable aggregate transition, authorization matrix, DB invariant 30개와 핵심
  algorithm 12개를 정본으로 연결했다.

## G-06 Verification executability

- **발견**: acceptance 설명은 있으나 deterministic fixture ID, error expectation과 실행 가능한
  Given/When/Then이 없다.
- **해결 정본**: TEST-07~09.
- **완료 evidence**: RQ-01~20 traceability, operation test template, deterministic fixture와
  Gherkin 15개 scenario를 연결했고 전용 parser를 통과했다.

## G-07 Implementation decomposition

- **발견**: workstream 순서만 있고 실제 module port, config key, migration·handler·test 산출물
  단위가 없다.
- **해결 정본**: PLAN-06~08.
- **완료 evidence**: port signature, runtime config와 IMP-01~28 dependency·산출물·gate가 있다.

## 사용자 결정 필요 여부

현재 공백은 기존 DEC-000~034를 타입과 구현 계약으로 구체화하는 범위다. 제품 정책을
바꾸는 새 결정은 발견되지 않았다.
