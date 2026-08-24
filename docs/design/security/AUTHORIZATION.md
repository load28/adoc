# Authorization

- **문서 ID**: SEC-03
- **상태**: 동결

## Enforcement point

HTTP middleware는 session·Membership만 확인한다. 세부 Document 권한은 application use case가
Permission Resolver를 호출한 뒤 repository query에 scope를 전달한다. handler와 UI 조건만으로
권한을 결정하지 않는다.

## Query별 적용

- tree·document: accessible ancestor/target만, hidden child count 없음
- Search·AI retrieval: candidate query 전 PermissionScope filter
- Reference·Backlink: source와 target 표시 시 각각 재검사
- Discussion·Review·Inbox: target content access와 action access 분리
- File: owner reference 중 하나의 접근 또는 exact PublicLinkScope
- History: current Document VIEWER 이상, deleted retention policy 확인

## PublicLinkScope

token hash로 link row를 찾고 active, expiry, Document active, current Version 존재를 검사한다.
scope는 `{documentId,currentVersionId,allowedAssetIds}`로 materialize한다. 일반 API, title
autocomplete와 Workspace bootstrap에 사용할 수 없다.

## Cache

cache key는 workspaceId, userId, documentId, membershipRevision, policyRevision이다. cache miss는
deny가 아니라 resolver 호출이다. invalidation 지연 중 access 확대를 막기 위해 sensitive
command는 cache를 쓰지 않고 current revision을 읽는다.

## TOCTOU

command는 authorization 뒤 target row lock과 policy revision을 transaction 안에서 재검사한다.
긴 AI Job은 시작과 결과 적용 시 모두 권한·revision을 확인한다.

## Document action capability

VIEWER는 Published read, CONTRIBUTOR는 Draft read·Discussion·Review 요청, EDITOR는 edit·AI proposal
적용·Publish command 진입을 허용한다. Permission·PublishPolicy 변경과 다른 subject의 permission
설명은 EDITOR access와 명시적 Manage를 모두 요구한다. reviewer는 reviewer rule 외에도 결정 시점의
effective CONTRIBUTOR 이상을 요구한다. Workspace role은 이 capability를 우회하지 않는다.
