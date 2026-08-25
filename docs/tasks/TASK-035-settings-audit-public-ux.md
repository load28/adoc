# TASK-035: Settings·Audit·Public UX 구현

- **상태**: 완료
- **유형**: 구현
- **구현 패키지**: IMP-26
- **시작일**: 2026-08-25
- **완료일**: 2026-08-25
- **커밋**: —

## 목적

Workspace 운영자가 구성원·그룹·권한·Writing·AI 설정과 Audit·Trash를 안전한 revision command로
관리하고, 외부 사용자는 capability token으로 최신 Published 문서만 읽는 운영 UX를 구현한다.

## 범위

- 포함: Members·Invitation, Groups, Document Permission, Writing·AI settings, Audit, Trash
  restore·purge, Public Viewer, loading·empty·404·responsive·접근성
- 제외: 도메인 command 재구현, 배포 hardening(IMP-27)

## 필수 설계 문서

- `docs/domain/workspace-governance.md`, `docs/domain/document-system.md`, `docs/domain/operations.md`
- `docs/design/ux/WORKSPACE-PERMISSION-FLOWS.md`, `docs/design/ux/SCREEN-BEHAVIOR-SPECS.md`
- `docs/design/specs/AUTHORIZATION-MATRIX.md`, `docs/design/specs/operations/AUDIT.md`
- `docs/design/specs/document/DOCUMENT-TREE.md`, `docs/design/specs/document/PUBLISH-VERSION.md`
- `docs/design/api/openapi.yaml`, `docs/design/api/ENDPOINT-COVERAGE.md`
- `docs/design/implementation/SETTINGS-AUDIT-PUBLIC-UX.md`

## 문서 준비 게이트

- [x] settings section·query·권한 gate와 command revision 계약 확정
- [x] membership·group·permission·configuration 상태·실패 계약 확정
- [x] Audit disclosure·cursor와 Trash restore·purge 계약 확정
- [x] Public capability·latest Published·동일 404·asset 계약 확정
- [x] responsive·keyboard·접근성·통합 검증 전략 확정

## 사용자 결정

사용자는 공개 링크에서 최신 Published 문서의 viewer만 제공하고 나머지 공개 기능은 제공하지
않도록 결정했다. Atlaskit 공개 component와 token을 그대로 사용한다.

## 의사결정

- Settings route section과 `document·subject·cursor`만 공유 가능한 선택 상태로 둔다.
- 모든 mutable settings는 server revision과 새 idempotency key를 사용하고 결과를 낙관 확정하지 않는다.
- Audit은 구조화 event를 표시하고 권한 없는 target title·before/after를 합성하지 않는다.
- purge는 복구 불가능성과 비동기 Job을 명시하며 exact revision·필수 사유를 요구한다.
- Public Viewer는 capability를 URL 밖으로 전파하지 않고 모든 실패를 동일한 문서 없음 상태로 표시한다.

## 구현 순서

1. PLAN-32와 typed governance·operations·public API client를 확정한다.
2. Members·Groups·Permission·Writing·AI settings를 구현한다.
3. Audit·Trash·Public Viewer를 구현한다.
4. permission·revision·404·responsive·접근성을 검증한다.
5. root·Compose gate를 통과하고 완료 기록한다.

## 작업 내역

- 2026-08-25: IMP-26 태스크를 등록하고 governance·operations·public 정본을 확인했다.
- 2026-08-25: PLAN-32에서 section ownership, command, disclosure, destructive action, public capability
  계약을 확정하고 문서 준비 게이트를 통과했다.
- 2026-08-25: Members·Invitation·Groups·Document Permission·Writing·AI·Audit settings를
  typed API와 revision command에 연결했다.
- 2026-08-25: Trash restore·reason-bound purge와 credential-free Public Viewer·safe Content
  renderer를 구현했다.
- 2026-08-25: public failure 비구분성과 settings query allowlist를 단위 테스트로 고정하고 root·
  Docker Compose gate를 통과했다.

## 이슈 및 해결

- Public Content의 List·Table·File wire field를 일반 children/name 구조로 가정한 불일치가 있었다.
  정본 schema의 `items`, `rows/cells`, `caption` 구조를 직접 사용하고 semantic table renderer로 수정했다.

## 검증

- [x] Members·Groups·Permission command·revision 검증
- [x] Writing·AI configuration·health·usage 검증
- [x] Audit·Trash restore/purge 검증
- [x] Public 404·content·asset·a11y·root·Compose gate

## 결과

운영 화면의 mutable command는 revision·idempotency 경계를 공유하고, Audit disclosure·복구 불가능한
purge·credential-free Public Viewer가 정본 보안 정책대로 구현됐다.
