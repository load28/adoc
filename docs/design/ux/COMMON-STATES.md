# 공통 UI 상태

- **문서 ID**: UX-03
- **상태**: 동결

## 상태 우선순위

`denied/not-found → deleted → blocking error → loading → empty → ready` 순으로 하나의 주 상태를
선택한다. stale data가 있으면 content를 유지하고 inline freshness 상태를 표시한다.

## Loading

route skeleton은 최종 layout과 같은 크기를 사용한다. 300ms 미만 요청에는 spinner를
깜빡이지 않는다. command는 button pending과 중복 submit 방지를 함께 적용한다.

## Empty

데이터 부재와 filter 결과 부재를 구분한다. 사용자가 수행 가능한 단일 primary action만
제시하고 권한 없는 action은 비활성화 대신 숨긴다.

## Error

오류는 `code`, 사람용 message, correlation ID와 가능한 recovery action을 표시한다.
retry-safe command만 재시도 버튼을 제공한다. validation, conflict, quota, dependency outage와
unknown을 구분한다.

## Optimistic UI

read/unread, panel preference 같은 가역적 low-risk action만 optimistic하게 처리한다.
Permission, Publish, delete, Proposal apply와 Version 생성은 server commit 뒤 성공으로
표시한다.

## Denied와 existence

Workspace 내부 UI는 이미 알려진 대상의 permission loss를 설명할 수 있다. public 또는
cross-tenant 경계에서는 denied와 not-found를 같은 상태로 표현한다.

## Offline·reconnect

Draft typing은 local recovery 상태를 표시한다. 다른 command는 offline queue에 넣지 않고
재시도를 요청한다. SSE gap은 query refresh로 복구한다.
