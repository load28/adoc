# Collaboration 도메인

- **문서 ID**: DOM-03
- **상태**: 동결
## 1. 책임

Document를 중심으로 한 Discussion, 특정 Draft revision에 대한 Review, Mention과 사용자의
처리 항목인 Inbox를 소유한다.

## 2. Discussion 모델

```text
Discussion
├─ id
├─ workspaceId
├─ documentId
├─ title
├─ status: OPEN | CLOSED
├─ topics[]
├─ messages[]
├─ createdBy
└─ timestamps

Topic
└─ subject: TEXT | DOCUMENT | REGION | EXTERNAL_REFERENCE

Message
├─ authorId
├─ body
├─ mentions[]
├─ references[]
├─ attachments[]
└─ createdAt
```

## 3. Discussion 불변식

- Discussion은 반드시 같은 Workspace의 Document 하나에 속한다.
- Discussion identity는 Region이나 Topic에 종속되지 않는다.
- Topic 변경은 과거 Message와 Reference를 삭제하지 않는다.
- 닫힌 Discussion은 읽을 수 있고 사람이 다시 열 수 있다.
- AI는 Discussion을 닫을 수 없다.
- Draft가 Publish돼도 관련 Discussion을 보존하고 Published Version의 change context로
  연결할 수 있다.
- 접근 권한을 잃은 Reference의 내용은 노출하지 않고 제한 상태를 표현한다.

## 4. 토론의 문서 반영

```text
사용자 요청
→ Discussion snapshot + Topics + References 구성
→ 합의·미합의·정보 부족 분석
→ DocumentOperation Proposal
→ schema·revision·permission 검증
→ Diff 표시
→ 사용자 승인
→ Draft 적용
```

AI가 의견 수만으로 합의를 추론하거나 미합의 선택지를 임의로 결정하지 않는다. 어떤 Message와
Source가 각 Operation의 근거인지 provenance에 남긴다.

## 5. Review 모델

```text
ReviewRequest
├─ documentId
├─ draftId
├─ requestedRevision
├─ policySnapshot
├─ reviewers
└─ status

ReviewDecision
├─ reviewerId
├─ revision
├─ decision: APPROVED | CHANGES_REQUESTED
├─ discussionId?
└─ decidedAt
```

Review 상태의 최소 전이는 다음과 같다.

```text
REQUESTED
├─ APPROVED when required approvals satisfied
├─ CHANGES_REQUESTED
└─ INVALIDATED when draft revision changes
```

## 6. Review 불변식

- 승인 대상은 추상적인 Draft가 아니라 정확한 revision이다.
- 현재 revision과 다른 승인으로 Publish할 수 없다.
- Review 요청 시점의 유효 PublishPolicy를 snapshot으로 보존한다.
- Reviewer의 Changes Requested는 기존 Discussion을 새로 만들거나 연결한다.
- 정책 변경이 진행 중 Review에 미치는 영향은 상세 설계에서 명시하고 조용히 바꾸지 않는다.

## 7. Inbox

Inbox는 Audit의 사본이 아니라 사용자가 처리해야 할 협업 상태다.

```text
InboxItem
├─ recipientId
├─ kind: MENTION | REVIEW | CHANGES_REQUESTED | PROPOSAL | CONFLICT
├─ target
├─ readAt?
├─ resolvedAt?
└─ createdAt
```

읽음은 내용을 확인했다는 뜻이고 해결은 필요한 대응을 끝냈다는 뜻이다. 둘을 하나의
boolean으로 합치지 않는다. Item은 Document가 아니라 가능한 가장 정확한 Message,
Review, Proposal 또는 Conflict 위치로 이동해야 한다.

## 8. 경계 이벤트

후속 상세 설계가 필요한 도메인 이벤트 후보는 다음과 같다.

- DiscussionCreated, DiscussionClosed, DiscussionReopened
- MessageCreated, UserMentioned
- ReviewRequested, ReviewApproved, ChangesRequested, ReviewInvalidated
- ProposalReady, PublishConflictDetected

이벤트 이름은 상태 변경의 결과를 나타내며 consumer별 UI 문장을 도메인 이벤트 이름으로
저장하지 않는다.
