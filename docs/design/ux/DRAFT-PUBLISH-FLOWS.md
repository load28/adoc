# Draft·Publish 흐름

- **문서 ID**: UX-06
- **상태**: 동결

## Draft 생성

Published 문서에서 Edit → active Draft가 있으면 재사용 → 없으면 current Version을 base로
생성 → lease 획득. Published Version이 없으면 empty content schema에서 시작한다.

## Review

Review 요청 전에 pending upload, invalid Reference와 blocking Writing Rule을 검사한다.
reviewer·required approvals와 revision을 확인하고 요청한다. Draft 내용 변경 시 invalidated
badge와 이유를 즉시 표시한다.

## Publish

1. current revision과 policy 충족 여부를 확인한다.
2. change summary와 public link 영향 preview를 표시한다.
3. base Version이 current인지 검사한다.
4. 같으면 Publish transaction을 수행한다.
5. 다르면 base/current/draft 3-way Diff로 이동한다.

## Conflict

block 단위 automatic merge 가능한 영역과 사람 판단이 필요한 conflict를 구분한다. AI Merge
Proposal은 별도 선택지이며 자동 적용하지 않는다. 해결 결과는 새 Draft revision으로
저장한 뒤 다시 Review가 필요하다.

## History와 restore

Version timeline에서 두 Version을 비교한다. Restore는 선택 Version으로 새 Draft를 만드는
동작임을 명확히 알리고 active Draft가 있으면 replace가 아니라 별도 confirmation과 archive
정책을 적용한다.
