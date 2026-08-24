# Workspace와 거버넌스 요구사항

- **문서 ID**: PROD-10
- **상태**: 동결

## 인증과 Membership

- 모든 Google 계정은 계정을 만들고 Workspace를 생성할 수 있다.
- 기존 Workspace 가입은 만료 가능한 초대 수락으로만 가능하다.
- Member 제거는 session과 Permission cache를 즉시 무효화한다.
- Group 생성, 이름 변경, member 추가·제거와 삭제를 지원한다.

## Permission

- Document Grant subject는 User 또는 Group이다.
- 현재 Document에서 root 방향으로 가장 가까운 명시적 Grant를 사용한다.
- 같은 위치의 User Grant가 Group보다 우선한다.
- User Grant가 없으면 Group `NO_ACCESS`가 우선하고, deny가 없으면 최고 access를 사용한다.
- `Manage`는 최소 `EDITOR`를 요구한다. Admin도 content access를 우회하지 않는다.
- Document 이동 전 새 조상 아래의 Effective Permission 변화를 미리 계산한다.

## PublishPolicy

- Workspace 기본값은 `DIRECT`다.
- Document tree에서 가장 가까운 policy override를 상속한다.
- `REVIEW_REQUIRED`는 reviewer set과 required approvals를 가진다.
- Review 요청 시 policy snapshot을 보존한다.

## 공개 공유

- `Manage` 보유자만 Published 문서의 Viewer link를 생성·폐기한다.
- link는 단일 Document의 최신 Published Version과 렌더링 File만 허용한다.
- link principal은 Membership이나 일반 Permission Resolver 결과로 변환하지 않는다.

## 인수 조건

권한이 없는 identity는 제목, 자동완성, count, timing과 existence를 통해서도 대상 존재를
알 수 없어야 한다. 모든 변경은 before·after를 포함한 Audit 대상이다.
