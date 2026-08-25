# TASK-029: Search Projection 통합 검증 회귀 조사

- **상태**: 대기
- **유형**: 조사
- **시작일**: —
- **완료일**: —
- **커밋**: —

## 목적

전체 Compose 통합 검증에서 반복 재현된 Search Projection의
`SEARCH_REQUEST_REJECTED` 원인을 조사하고 구조적으로 해결한다.

## 범위

- 포함: OpenSearch request·mapping·fixture·adapter 경계 진단, 관련 설계 갱신, 구조적 수정과
  전체 Compose 재검증
- 제외: TASK-027 AI Context·Runtime 계약 변경, 특정 테스트만 통과시키는 예외 처리

## 필수 설계 문서

작업 시작 시 검색·projection·OpenSearch·실패·테스트 정본을 감사해 확정한다.

## 문서 준비 게이트

작업 시작 전 작성한다.

## 사용자 결정

없음.

## 의사결정

작업 시작 전 작성한다.

## 작업 내역

- 2026-08-25: TASK-027 전체 Compose 검증에서 독립적으로 반복된 실패를 후속 태스크로
  등록했다.

## 이슈 및 해결

조사 전.

## 검증

- [ ] 관련 정본과 실패 재현 확인
- [ ] 근본 원인과 구조적 해결 검증
- [ ] 전체 Compose 통합 검증 통과
- [ ] root gate와 `git diff --check` 통과

## 결과

대기 중.
