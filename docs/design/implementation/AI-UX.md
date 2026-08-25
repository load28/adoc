# AI UX 구현 계약

- **문서 ID**: PLAN-31
- **상태**: 구현 기준
- **태스크**: TASK-034 / IMP-25

## 1. 소유권과 화면 상태

Document route의 `panel=ai`가 Inspector를 소유한다. `job`과 `proposal` query parameter는 공유 가능한
선택 상태이며, instruction·source 선택·operation 선택은 URL에 기록하지 않는 session UI 상태다.
Inspector는 문서 편집기를 대체하지 않고 같은 Document·Draft query 정본과 lease 경계를 재사용한다.

## 2. Context preview와 실행

사용자는 이름 있는 Task kind와 instruction을 선택한다. 화면은 current Draft revision, Document target,
외부 web 비활성 기본값을 사용해 preview를 요청한다. preview는 포함·제외 Source, authority,
omission, input estimate와 만료 시각을 표시한다. 사용자가 optional source를 제외하면 새 preview를
만들어 fingerprint를 갱신한다. 권한 밖 Source의 제목이나 개수는 추측하지 않는다.

Job 생성은 유효한 preview fingerprint와 같은 input·exact revision을 사용한다. preview 만료,
revision 충돌 또는 입력 변경 뒤에는 실행을 막고 preview를 다시 만든다.

## 3. Job lifecycle과 복구

목록과 상세 query는 Workspace·job ID로 분리한다. `QUEUED`, `RUNNING`, `CANCEL_REQUESTED`는 진행 상태,
나머지는 terminal 상태다. 취소는 exact job revision과 새 idempotency key를 사용하며 성공을
낙관 확정하지 않는다. SSE `AI_JOB_CHANGED`와 `PROPOSAL_CHANGED`는 query invalidation만 발생시킨다.
stream 단절은 Job을 취소하지 않으며 HTTP 상세 재조회로 복구한다.

## 4. Result·Source·Proposal

Result는 status, finding severity, claim certainty, uncertainty, conflict와 사용된 Source ID를 구조화해
표시한다. 생성 텍스트를 HTML로 주입하지 않는다. Source는 preview snapshot metadata와 연결하며
권한 상실 시 제한 상태만 표시한다.

Proposal Diff는 operation ID·kind·target과 dependency를 읽기 전용 목록으로 표시한다. `OPEN`이면서
base revision이 current Draft와 같을 때만 적용 가능하다. 사용자는 operation을 명시적으로 선택한다.
선택은 client에서 dependency closure를 안내하되 server 검증을 대체하지 않는다. 적용 command는
Draft lease token, client instance, base revision과 operation ID를 전송한다. reject는 proposal revision과
필수 사유를 전송한다. 적용·거절 이후 query를 무효화하고 server 상태를 다시 읽는다.

## 5. 실패·보안·접근성

permission denial은 404 계약을 그대로 사용하며 Source 정보를 노출하지 않는다. stale, dependency,
quota, provider, timeout, cancellation은 stable problem code와 correlation ID를 표시한다. action은
keyboard로 도달 가능하고 checkbox label은 operation 의미를 포함한다. 진행 상태는 `role=status`,
실패는 `role=alert`로 알리며 focus를 강제로 이동하지 않는다.

## 6. 검증

- API client test: preview→create, exact revision cancel, lease-bound apply, revision-bound reject
- pure state test: terminal 분류, dependency closure, SSE invalidation mapping
- component test: no-direct-apply, source disclosure, stale disable, accessible labels
- root gate와 Docker Compose integration gate
