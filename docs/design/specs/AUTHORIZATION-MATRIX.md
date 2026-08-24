# Authorization Matrix

- **문서 ID**: SPEC-18
- **상태**: 동결
- **정본 계산**: [Permission Resolver](governance/PERMISSION-RESOLVER.md)

## Workspace capability

| Capability | Member | Admin | Owner |
|---|---:|---:|---:|
| Workspace 사용·Search·Inbox | ✓ | ✓ | ✓ |
| Member 초대·제거, Group·Vocabulary·Writing·AI 설정 | — | ✓ | ✓ |
| Audit 조회 | — | ✓ | ✓ |
| Workspace 설정 변경 | — | ✓ | ✓ |
| Owner 지정·Workspace 삭제 예약·취소 | — | — | ✓ |

Workspace role은 Document content access를 부여하지 않는다. Admin과 Owner도 Effective Document
Permission이 없으면 title, content, Discussion, Review, Reference, File과 Search hit를 볼 수 없다.

## Document capability

| Action | Viewer | Contributor | Editor | `can_manage` 추가 필요 |
|---|---:|---:|---:|---:|
| Published·Version·Backlink 읽기 | ✓ | ✓ | ✓ | — |
| Draft 읽기·편집, Reference, Discussion·Message | — | ✓ | ✓ | — |
| Review 요청 | — | ✓ | ✓ | — |
| Review 결정 | assigned reviewer이고 Viewer 이상 | assigned reviewer이고 Viewer 이상 | assigned reviewer | — |
| Publish, move, trash, restore, metadata | — | — | ✓ | — |
| Permission·Publish Policy·Public Link | — | — | ✓ | ✓ |
| 강제 Lease 회수 | — | — | ✓ | ✓ |
| 조기 permanent purge | — | — | ✓ | ✓+Workspace Admin |

`can_manage=true` grant는 access가 EDITOR일 때만 유효하다. action permission은 target에서 계산한
Effective Permission을 쓰며 source Document 권한으로 target content를 우회하지 않는다.

## Subject·inheritance

1. target Document에서 root 방향으로 가장 가까운 explicit grant depth를 찾는다.
2. 그 depth에 User grant가 있으면 그것만 사용한다.
3. User grant가 없고 Group grant 중 `NO_ACCESS`가 있으면 deny한다.
4. 그 외 Group grant 중 가장 높은 access를 사용하고 `can_manage`는 선택된 EDITOR grant들의 OR다.
5. grant가 없으면 `NO_ACCESS, manage=false`다.

가까운 grant가 더 먼 deny/allow를 모두 대체한다. 같은 depth의 User grant는 Group보다 우선한다.
Group deny는 같은 depth의 Group allow보다 우선하지만 User grant를 이기지 않는다. Workspace
Membership이 ACTIVE가 아니면 계산 전에 즉시 deny한다.

## Resource-specific rules

| Resource·operation | 추가 조건 | 존재 비노출 |
|---|---|---|
| Group·Member 설정 | Workspace Admin+ | cross-Workspace ID |
| Discussion·Review | 연결 Document의 현재 permission | 연결 Document 비노출 시 404 |
| Inbox | item.user_id = actor | 다른 사용자 item은 404 |
| AI Job·Proposal | actor owner + target current permission | 다른 사용자 Job은 404 |
| File read | 접근 가능한 현재 owner reference 최소 1개 | asset ID만 안 경우 404 |
| File delete | uploader 또는 Admin, reference 0 | 다른 Workspace asset 404 |
| Search | query 전 permission_scope filter | post-filter 금지 |
| AI Context | source 추가 전 각 source permission | 권한 상실 source 제외+명시 |
| Public Viewer | valid capability+latest Published exact asset set | 모든 실패 동일 404 |
| Audit | Workspace Admin+, content snapshot 별도 permission | target title 비노출 가능 |

## Enforcement points

Browser visibility는 편의 기능일 뿐 보안 경계가 아니다. Axum extractor가 session·CSRF·Workspace
Membership을 검증하고 application command/query handler가 resource permission을 계산한다.
repository는 workspace_id 없는 tenant query interface를 노출하지 않는다. OpenSearch와 AI
retrieval은 결과 생성 전 scope를 적용하고 ObjectStorage download는 application authorization 뒤
짧은 수명의 capability response로만 제공한다.
