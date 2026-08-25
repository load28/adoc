# Settings·Audit·Public UX 구현 계약

- **문서 ID**: PLAN-32
- **상태**: 구현 기준
- **태스크**: TASK-035 / IMP-26

## 1. Route와 권한

Settings section은 `members`, `groups`, `permissions`, `writing`, `ai`, `audit`의 닫힌 집합이다.
`document`, `subject`, `cursor`는 bounded query로만 읽는다. Admin/Manage 권한 판단은 server query가
정본이며 404·403에서 화면이 감춰진 데이터나 역할을 추측하지 않는다.

## 2. Governance command

Members·Invitation·Groups·Permission은 각 resource revision과 새 idempotency key를 요구한다.
Owner 강등·제거, 마지막 Owner, inherited permission은 server conflict를 그대로 표시한다. Group member
변경과 Permission 변경은 server response 재조회 전 완료로 표시하지 않는다. Permission 설정은
선택 Document가 없으면 안내만 보여 주며 임의 root 권한을 변경하지 않는다.

Writing configuration은 닫힌 baseline registry를 표시한다. AI configuration은 provider·model·동시성·
budget을 한 revision command로 저장하며 credential 값을 표시하거나 받지 않는다. Health와 Usage는
독립 query로 실패 격리한다.

## 3. Audit와 Trash

Audit은 sequence 역순 cursor page를 사용하고 actor/action/target/correlation/occurredAt을 구조화해
표시한다. redacted field는 복원하거나 title을 합성하지 않는다. JSON export는 제공하지 않는다.

Trash restore는 exact Document revision과 nullable parent를 전송한다. purge는 복구 불가 경고,
필수 사유, exact revision을 요구하며 `202 JobReference`를 표시한다. UI는 purge 완료를 낙관 처리하지
않고 목록과 Job 상태를 재조회한다.

## 4. Public Viewer

Public route는 session cookie에 의존하지 않는 `/public/v1/documents/{token}`만 호출한다. token을 log,
analytics, 다른 링크 query 또는 오류 문구에 넣지 않는다. unknown·revoked·expired·trashed·unpublished는
동일한 404 화면이다. 반환된 최신 Published content는 정본 Content renderer로 text·heading·list·code·
table·image/file을 안전하게 표시하고 asset URL은 동일 token 경계 안에서만 구성한다. 편집·토론·검색·
AI·metadata discovery 기능은 렌더링하지 않는다.

## 5. 접근성·검증

설정 navigation과 모든 command는 keyboard로 도달 가능하다. destructive action은 사유 label과 명확한
버튼 이름을 가진다. table이 아닌 resource list로 mobile reflow를 보장한다. automated test는 route
parse, command header, public safe renderer와 404 비구분성을 검증한다. root와 Compose gate를 통과한다.
