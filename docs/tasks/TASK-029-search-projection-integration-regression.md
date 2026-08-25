# TASK-029: Search Projection 통합 검증 회귀 조사

- **상태**: 완료
- **유형**: 조사
- **시작일**: 2026-08-25
- **완료일**: 2026-08-25
- **커밋**: —

## 목적

전체 Compose 통합 검증에서 반복 재현된 Search Projection의
`SEARCH_REQUEST_REJECTED` 원인을 조사하고 구조적으로 해결한다.

## 범위

- 포함: OpenSearch request·mapping·fixture·adapter 경계 진단, 관련 설계 갱신, 구조적 수정과
  전체 Compose 재검증
- 제외: TASK-027 AI Context·Runtime 계약 변경, 특정 테스트만 통과시키는 예외 처리

## 필수 설계 문서

- [x] 제품·도메인: `product/features/KNOWLEDGE.md`, `domain/knowledge.md`
- [x] 검색 계약: `design/adr/ADR-003-search-projection.md`,
  `design/specs/knowledge/INDEX-RETRIEVAL-SOURCE.md`,
  `design/data/OPENSEARCH-PROJECTION-SCHEMA.md`
- [x] 구현·실패 계약: `design/implementation/SEARCH-PROJECTION.md`,
  `design/architecture/TRANSACTION-EVENT-JOB.md`,
  `design/operations/OBSERVABILITY-SLO.md`
- [x] 검증 계약: `design/quality/TEST-STRATEGY.md`,
  `design/quality/CONCURRENCY-RECOVERY-TESTS.md`
- [x] 제품·API·DDL 변경: N/A — 기존 Search Projection provider 계약의 회귀를 복구한다.

## 문서 준비 게이트

- [x] 재현 대상은 실제 OpenSearch의 첫 projection Job으로 고정했다.
- [x] 동일 artifact를 격리 환경에서 실행해 코드 결함과 환경 결함을 구분한다.
- [x] analyzer·cluster·mapping·projection 전체 계약을 실제 OpenSearch에서 확인한다.
- [x] 특정 fixture나 status를 우회하는 production code 변경을 금지한다.
- [x] 수정 후 targeted Search Projection과 전체 Compose gate를 모두 검증한다.

## 사용자 결정

사용자는 AGENTS.md의 구조적 해결 원칙에 따라 구현 가능한 다음 태스크를 자동으로 진행하도록
결정했다.

## 의사결정

### 결정 1: 동일 artifact의 격리 재현으로 환경 실패를 판정한다

- **상황**: 실패 직전 host 여유 공간이 116MiB까지 줄고 Docker overlay가 read-only가 됐으며,
  TASK-028 정리 뒤에는 같은 코드와 fixture를 다시 검증할 수 있다.
- **검토한 대안**: 4xx를 임의 재시도로 변경 / 테스트 fixture 우회 / 동일 artifact를 새
  PostgreSQL·Redis·OpenSearch 세대와 전체 Compose에서 재실행.
- **선택과 근거**: 격리 세대에서 plugin·cluster를 확인하고 동일 test image를 실행한 뒤 전체
  Compose를 재실행한다. 두 gate가 통과하면 환경 실패로 판정하고 production code를 바꾸지
  않는다.

## 작업 내역

- 2026-08-25: TASK-027 전체 Compose 검증에서 독립적으로 반복된 실패를 후속 태스크로
  등록했다.
- 2026-08-25: TASK-025·PLAN-24와 OpenSearch adapter·통합 fixture를 감사하고 문서 준비
  게이트를 통과했다.
- 2026-08-25: 새 OpenSearch 3.3.2 세대에서 `analysis-nori` 설치와 green cluster를 확인하고,
  동일 TASK-027 test image의 Search Projection 계약이 통과했다.
- 2026-08-25: TASK-028 정리 후 전체 Compose 통합 gate를 재실행해 AI를 포함한 모든 adapter
  계약, backup checksum과 degraded health를 통과했다.

## 이슈 및 해결

- 이전 두 실패는 host disk 고갈과 Docker overlay read-only 상태에서 발생했다. TASK-028이
  재생성 가능한 image·build cache를 정리한 뒤 동일 artifact의 targeted·전체 검증이 모두
  통과했으므로 환경 용량 실패로 판정했다. Search adapter와 fixture는 변경하지 않았다.

## 검증

- [x] 관련 정본과 실패 조건 확인
- [x] 격리된 실제 OpenSearch targeted 계약 통과
- [x] 전체 Compose 통합 검증 통과
- [x] root gate와 `git diff --check` 통과

## 결과

디스크 고갈 상태를 TASK-028에서 제거한 뒤 동일 Search Projection artifact가 격리 환경과 전체
Compose에서 모두 통과했다. 제품 코드나 테스트에 예외를 추가하지 않고 환경 실패를 분리했다.
