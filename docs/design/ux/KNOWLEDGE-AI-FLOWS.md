# Knowledge와 AI 흐름

- **문서 ID**: UX-08
- **상태**: 동결

## Search

query 입력 → lexical·semantic 통합 결과 → kind·updatedAt filter → snippet의 matching Region
강조 → 문서 이동. index 지연은 `최근 변경이 아직 반영 중`으로 표시하고 권한 scope를
완화하지 않는다.

## Source 보기

AI answer와 Proposal의 각 claim은 Source chip을 가진다. 선택 시 version·region snapshot과
현재 위치를 함께 보여주고 변경·삭제·권한 상실 상태를 구분한다.

## Context Inspector

Task 시작 전 현재 Draft, 선택 Region, Discussion, Reference, Vocabulary, retrieved source와
외부 web 여부를 목록으로 보여준다. 사용자는 optional source를 제외하고 명시적 source를
추가할 수 있으나 권한 밖 대상은 추가할 수 없다.

## Job

queued position → running phase → streaming progress → validating → ready/failed/cancelled를
표시한다. stream disconnect는 job을 취소하지 않고 query로 상태를 복구한다. quota 초과는
관리자 설정과 reset 시각을 보여준다.

## Proposal

Diff는 block·operation 단위, Source와 rule finding을 연결한다. stale Draft이면 적용 버튼을
비활성화하고 rebase 또는 재실행을 요구한다. 일부 적용은 dependency가 없는 Operation만
허용한다.
