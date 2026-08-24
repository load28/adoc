# File과 Audit 요구사항

- **문서 ID**: PROD-16
- **상태**: 동결

## FileAsset

FileAsset은 Workspace의 독립 asset이고 content에는 reference만 둔다. upload session,
checksum, MIME allowlist, size limit, malware scan과 preview 상태를 가진다. Draft,
Published Version과 Discussion의 reference를 지원한다.

초기 binary 저장은 local ObjectStorage adapter다. storage key를 domain ID로 노출하지 않고
향후 AWS S3 adapter가 같은 put/get/delete/signed-access 계약을 구현한다.

과거 Published Version이 참조하는 asset은 현재 Draft에서 제거돼도 삭제하지 않는다. 모든
owner reference가 사라지고 retention이 끝난 뒤 GC한다.

## 접근

일반 file 요청은 Membership과 owner permission을 확인한다. Public Viewer는 공유된 단일
Published Version의 렌더링 reference에만 짧은 수명의 scope-bound access를 받는다.

## Audit

Audit Event는 actor, action, target, structured metadata, before·after와 time을 저장한다.
Version History와 Inbox를 대체하지 않는다. key stroke·heartbeat 같은 고빈도 event는
영구 Audit 대상이 아니다.

영구 삭제 뒤에는 삭제 사실, tenant-scoped opaque ID, actor와 time만 남기며 제목·본문·파일명
등 민감한 snapshot을 제거한다.
