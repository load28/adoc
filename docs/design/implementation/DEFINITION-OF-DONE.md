# Definition of Done

- **문서 ID**: PLAN-03
- **상태**: 동결

## 설계

- 요구사항 ID와 정본 설계가 연결돼 있다.
- 새 정책·상태·error·event가 문서와 schema에 먼저 반영됐다.
- 구조적 해결이며 특정 fixture·화면을 위한 예외가 없다.

## 구현

- module dependency, typed ID, tenant scope와 expected revision을 지킨다.
- permission, idempotency, transaction, outbox와 recovery 계약을 우회하지 않는다.
- public `@atlaskit` package/token만 사용하고 병행 UI 체계가 없다.
- ko/en, responsive, loading·empty·error·denied·recovery 상태가 있다.

## 검증

- unit, integration, contract, E2E, concurrency, security, a11y와 performance test가 통과한다.
- unauthorized exposure, Version mutation, stale approval Publish와 invalid AI apply가 0이다.
- OpenAPI·AsyncAPI·migration·generated client diff가 clean하다.

## 운영

- metric·trace·redacted log·alert와 runbook이 있다.
- backup restore, rollback과 deletion lifecycle을 실제 환경에서 검증했다.
- dependency license·SBOM·vulnerability gate가 통과한다.

## 기록

Task의 작업 내역, 의사결정, issue·root cause, verification과 변경 문서를 완료한다. 사용자
요청 전 commit하지 않으며 AI attribution을 기록하지 않는다.
