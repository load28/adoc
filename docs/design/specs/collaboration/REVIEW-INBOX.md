# Review와 Inbox

- **문서 ID**: SPEC-10
- **상태**: 동결

## Review state

`REQUESTED → {APPROVED|CHANGES_REQUESTED|CANCELLED|INVALIDATED}`. 복수 reviewer decision은
assignment별 append하고 aggregate status는 policy와 decision으로 계산한다.

Draft content revision, reviewer Membership·access 상실은 affected approval을 INVALIDATED로
바꾼다. 정책 변경은 snapshot을 바꾸지 않고 outdated flag만 만든다.

## Commands

RequestReview, SubmitApproval, RequestChanges(discussionId), CancelReview. reviewer 자신만 자신의
decision을 제출하며 requester가 대신 승인할 수 없다.

## Inbox projection

source key 예: `review:{reviewId}:{reviewerId}`, `mention:{messageId}:{userId}`. outbox replay가
같은 item을 upsert한다. target permission 상실 시 item은 제목을 redacted하고 resolve action만
허용한다.

## User state

MarkRead와 Resolve는 별도 command다. source domain이 완료되면 system resolvedAt을 설정할 수
있지만 user readAt을 대신 설정하지 않는다.
