# Workspace와 Governance 도메인

- **문서 ID**: DOM-01
- **상태**: 동결
## 1. 책임

Workspace는 모든 제품 데이터의 최상위 소유·보안 경계다. Governance는 누가 Workspace에
속하고 각 Document에서 무엇을 할 수 있는지, Publish에 어떤 검토가 필요한지를 결정한다.

## 2. 모델

```text
Workspace
├─ id
├─ settings
└─ status

Membership
├─ workspaceId
├─ userId
├─ role: MEMBER | ADMIN
└─ status: ACTIVE | REMOVED

PermissionGrant
├─ workspaceId
├─ documentId
├─ subject: USER | GROUP
├─ access: NO_ACCESS | VIEWER | CONTRIBUTOR | EDITOR
└─ capabilities

PublishPolicy
├─ documentId
├─ mode: DIRECT | REVIEW_REQUIRED
├─ reviewers
└─ requiredApprovals
```

Group과 capability의 구체 구조는 상세 설계 전 별도 결정이 필요하지만 PermissionGrant가
User ID 하나에만 고정되면 안 된다.

## 3. 불변식

- Workspace에 속하지 않은 사용자는 그 Workspace의 어떤 대상에도 접근할 수 없다.
- 모든 도메인 entity는 하나의 workspaceId로 추적 가능해야 한다.
- Workspace 경계를 넘는 Reference, 파일 공유와 Retrieval 결과를 만들지 않는다.
- Admin 여부만으로 개별 Document access를 암묵적으로 부여하지 않는다. 관리 우회가
  필요한 정책은 별도 capability로 명시한다.
- Effective Permission 계산은 모든 소비자에게 동일한 결과를 반환해야 한다.
- 접근할 수 없는 대상의 제목, 존재 여부와 metadata도 노출하지 않는다.
- PublishPolicy는 현재 Document에서 가장 가까운 명시적 설정을 상속한다.

## 4. Permission Resolution

```text
resolve(userId, documentId)
  1. Active Membership 확인
  2. Document에서 root까지 조상 경로 확인
  3. User와 소속 Group의 가장 가까운 명시적 Grant 수집
  4. 확정된 precedence 정책 적용
  5. EffectivePermission 반환
```

반환 모델의 최소 의미는 다음과 같다.

```text
EffectivePermission
├─ access
├─ capabilities
├─ resolvedAtDocumentId
└─ evidenceGrantIds
```

근거 Grant를 함께 반환해야 권한 변경과 문서 이동의 영향을 설명하고 Audit할 수 있다.

## 5. 접근 수준의 의미

- `NO_ACCESS`: 대상의 존재와 내용에 접근 불가
- `VIEWER`: Published Version 읽기
- `CONTRIBUTOR`: Viewer + Draft 읽기 + Discussion 생성·참여
- `EDITOR`: Contributor + Draft 생성·편집 + AI 문서 작업 + 정책에 따른 Publish

`Manage Permission`, `Request Review`, `Approve`, `Publish`를 access 계층과 어떻게 결합할지는
보안 상세 설계에서 확정한다. 코드가 임의 조합을 먼저 만들 수 없다.

## 6. Tree 이동

Document 이동은 단순 navigation 변경이 아니다. 새 조상에서 Permission과 PublishPolicy를
상속하므로 다음 순서를 보장한다.

```text
이동 요청
→ 현재 위치와 목적지 권한 확인
→ 이동 전·후 Effective Permission 영향 계산
→ 필요한 사용자 확인 또는 정책 적용
→ 원자적 이동
→ Permission cache·index 무효화
→ Audit Event
```

## 7. 확정된 정책

- Group 생성·멤버 관리·Group Permission을 전체 구현에 포함한다.
- 현재 Document에서 root 방향으로 가장 가까운 명시적 Grant를 사용한다.
- 같은 위치에서는 개인 Grant가 Group Grant보다 우선한다.
- 개인 Grant가 없으면 Group의 명시적 `NO_ACCESS`가 우선한다.
- Group deny가 없으면 Group access 중 가장 높은 수준을 사용한다.
- `Manage`는 최소 `EDITOR` access를 요구한다.
- Workspace Admin도 Document 내용 access를 우회하지 않는다.
