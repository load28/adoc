# PublishPolicy

- **문서 ID**: SPEC-03
- **상태**: 동결

## Model

`DIRECT | REVIEW_REQUIRED(requiredApprovals>=1, reviewerRule)`이다. reviewerRule은 explicit
User·Group이고 실제 요청 시 active Member USER 집합으로 snapshot한다.

## Resolution

Document부터 root 방향의 가장 가까운 override, 없으면 Workspace DIRECT를 사용한다. Review
요청 시 policy revision과 reviewer snapshot을 저장한다. 이후 policy 변화가 active Review를
조용히 바꾸지 않으며 UI에 outdated policy를 표시한다.

## Publish gate

- DIRECT: EDITOR와 current revision이면 가능
- REVIEW_REQUIRED: required count 이상의 current-revision approval, changes requested 0
- Draft content revision 변화: 모든 active decision invalidated
- reviewer Membership·access 상실: 해당 approval invalidated

## Commands

`SetPolicy`, `RemoveOverride`, `RequestReview`, `Approve`, `RequestChanges`, `CancelReview`는
expected policy/Draft/Review revision을 각각 요구한다.
