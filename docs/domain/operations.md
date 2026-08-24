# Operations 도메인

- **문서 ID**: DOM-06
- **상태**: 동결
## 1. 책임

여러 도메인이 참조하는 File Asset의 저장 생명주기와, 공식 Version History와 구분되는
중요 시스템 행동의 Audit 기록을 소유한다.

## 2. File Asset

```text
FileAsset
├─ id
├─ workspaceId
├─ storageKey
├─ originalName
├─ mediaType
├─ size
├─ checksum
├─ status: UPLOADING | READY | FAILED | DELETED
├─ uploadedBy
└─ timestamps

FileReference
├─ fileAssetId
├─ ownerKind
├─ ownerId
└─ role
```

### 불변식

- FileAsset은 하나의 Workspace에 속한다.
- Content에는 바이너리가 아니라 fileId Reference를 저장한다.
- `READY`가 아닌 Asset을 Published Version에 포함할 수 없다.
- 같은 파일 이름의 재업로드는 기존 Asset을 덮지 않고 새 Asset identity를 만든다.
- File access는 Workspace와 참조 owner의 Effective Permission을 모두 확인한다.
- 영구 공개 storage URL을 권한 확인 대신 사용하지 않는다.
- 현재 Draft에서 Reference가 없어도 과거 Version이 참조하면 Asset을 삭제하지 않는다.
- 모든 지원 owner의 Reference와 보존 기간을 확인한 뒤에만 Garbage Collection한다.

## 3. File 수명주기

```text
Create metadata(UPLOADING)
→ upload and integrity validation
   ├─ success: READY
   └─ failure: FAILED → retry or delete
→ references removed
→ unreferenced candidate
→ 7-day unreferenced grace
→ physical delete + DELETED tombstone/audit
```

부분 업로드는 upload session 만료 뒤 정리한다. checksum, detected MIME와 악성 파일 검사를
통과해야 READY가 된다. 모든 Reference가 사라진 Asset은 7일 grace 뒤 다시 검사하고
삭제한다. 상세 계약은 SPEC-15와 DATA-04가 소유한다.

## 4. Audit Event

```text
AuditEvent
├─ id
├─ workspaceId
├─ actor: USER | SYSTEM
├─ action
├─ target
├─ metadata
└─ occurredAt
```

사람이 읽는 완성 문장을 저장하지 않는다. 구조화 Event를 보존하고 표시 계층에서 지역화된
문장으로 변환한다.

## 5. Audit 대상

- Document 생성, 이동, 휴지통, 복구와 영구 삭제
- Draft 생성과 Publish 완료
- Discussion 생성, 종료와 재개
- Review 요청, 승인과 수정 요청
- Permission과 PublishPolicy 변경
- Vocabulary 생성, 변경과 폐기
- Workspace Member 추가·제거와 역할 변경

키 입력, 자동 저장 heartbeat, 문단 이동과 AI Review 실행 같은 고빈도 작업은 기본 영구
Audit 대상이 아니다. 보안·운영 요구가 생기면 목적과 보존 기간을 먼저 설계한다.

## 6. Audit 불변식

- 권한·정책·역할 변경은 before와 after를 남긴다.
- actor가 System이면 발생시킨 정책·Job과 원인을 metadata로 추적할 수 있어야 한다.
- AI를 독립 actor로 두지 않는다. AI Proposal 적용은 적용한 User의 행동이며 AI provenance는
  metadata에 둔다.
- Audit은 Inbox를 대신하지 않고 Version History를 복제하지 않는다.
- Event는 Workspace가 active인 동안 보존한다. Document·Workspace 영구 삭제 시 삭제 사실,
  opaque ID, actor와 time만 남기고 민감한 before·after와 표시 snapshot을 제거한다.
