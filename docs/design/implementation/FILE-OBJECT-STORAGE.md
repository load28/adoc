# File·ObjectStorage 구현 계약

- **문서 ID**: PLAN-21
- **상태**: 구현 기준
- **구현 패키지**: IMP-15

## 1. 책임과 경계

File aggregate는 metadata·state·upload capability·reference·retention을 소유한다. ObjectStorage는 opaque key의
byte I/O만 소유하고 사용자·Workspace·MIME·권한을 알지 못한다. PostgreSQL transaction과 storage write를
분산 transaction으로 묶지 않는다. 먼저 UPLOADING session을 commit하고, byte write는 session capability로
수행하며, complete command가 저장된 byte를 검증해 READY 또는 FAILED로 전이한다.

## 2. UploadSession

CreateUpload은 active Member, idempotency key, sanitized display name, declared MIME, 1 byte 이상 Workspace size
limit 이하, SHA-256을 요구한다. `asset_id`, opaque `storage_key`, 256-bit upload token hash, token key ID,
absolute expiry, expected size/checksum을 저장한다. asset ID·storage key·원문 token은 actor·Workspace·
idempotency key를 HMAC으로 분리 파생한다. 따라서 같은 command replay는 같은 capability를 반환하지만 DB에는
token hash만 남는다. 회전 뒤 replay는 session의 key ID로 current 또는 previous key를 선택한다.

Local adapter의 upload URL은 authenticated `PUT /workspaces/{workspaceId}/files/{assetId}/content`다. 요청은
`X-Upload-Token`, Content-Length, CSRF를 요구하며 session expiry·single-completion을 확인한다. stream 중 hard
limit와 expected size를 적용하고 `<storageRoot>/<two-level key>/<storageKey>.partial`에 쓴 뒤 fsync·atomic
rename한다. storage key는 `[a-f0-9]{64}`만 허용해 traversal을 구조적으로 차단한다. 미래 S3 adapter는 같은
session으로 multipart/presigned target을 반환하되 DB command 계약을 바꾸지 않는다.

## 3. Complete와 검증

Complete는 UPLOADING row를 잠그고 ObjectStorage stat을 확인한다. 실제 size·stream SHA-256, magic-byte MIME,
declared MIME compatibility, malware scan을 순서대로 검증한다. 허용 MIME은 PNG·JPEG·GIF·WebP·PDF·plain text·
Markdown·JSON·ZIP이며 SVG·HTML은 허용하지 않는다. ZIP은 서버에서 preview·entry extraction을 하지 않고
attachment byte로만 제공한다. 미래 extraction 기능은 별도 태스크에서 entry 수·expanded size·compression
ratio 계약을 먼저 추가해야 한다. 성공은 READY·ready_at·revision+1, 실패는 FAILED·failure_code·revision+1과
byte cleanup claim을 만든다.
scan unavailable은 READY로 폴백하지 않고 retryable dependency error로 남긴다.

## 4. Reference projection

File ID는 Content의 image/file block attrs와 Message attachment ID에서 추출한다. owner commit transaction은
old/new set diff 후 추가 asset row를 `FOR SHARE`로 잠그고 same Workspace·READY를 검증한다. 제거는 projection
row만 지운다. 지원 owner는 DRAFT, PUBLISHED_VERSION, MESSAGE, VOCABULARY다. PublishedVersion reference는
immutable version row가 존재하는 동안 제거하지 않는다. Publish는 exact content asset set을 version reference로
복제하고 READY가 아니면 전체 transaction을 거부한다.

## 5. Download와 Range

private metadata/download는 active Membership과 접근 가능한 owner reference 하나를 요구한다. owner permission은
SQL candidate 단계에서 확인하며 권한 없는 reference count를 노출하지 않는다. Public download는 valid public
capability가 고정한 `documentId,currentVersionId`의 PUBLISHED_VERSION reference에 asset ID가 정확히 포함될 때만
허용한다.

단일 byte range만 지원한다. satisfiable range는 206·Content-Range, full은 200, invalid/multiple/unsatisfiable은
416이다. 모든 응답은 `X-Content-Type-Options: nosniff`, restrictive CSP, private no-store 또는 public short cache,
sanitized RFC 5987 Content-Disposition을 사용한다. HTML·SVG·ZIP과 unknown MIME은 attachment다.

## 6. Delete와 GC

사용자 delete는 READY·reference 0 asset만 DELETED로 전이하고 `purge_after=now+7d`를 설정한다. referenced asset은
`FILE_IN_USE`와 안전한 reference count를 반환한다. GC claim은 DELETED·due row를 `FOR UPDATE SKIP LOCKED`로 잡고
reference 0을 재검사한다. reference가 생겼으면 READY로 복구하지 않고 GC claim을 취소하는 별도 lifecycle error로
기록한다. byte delete는 존재하지 않아도 성공이며, 실패 시 row를 유지해 backoff retry한다. 물리 row 제거와
deletion ledger는 IMP-16이 소유한다.

## 7. Port와 오류

`ObjectStorage`는 begin/write/stat/read_range/delete를, `MalwareScanner`는 scan stream 결과만 제공한다. Local과
S3 adapter suite는 동일한 create/overwrite 금지/range/delete idempotency/partial failure corpus를 통과해야 한다.
주요 오류는 `FILE_NOT_FOUND`, `FILE_STATE_INVALID`, `FILE_IN_USE`, `UPLOAD_TOKEN_INVALID`, `UPLOAD_EXPIRED`,
`FILE_SIZE_MISMATCH`, `FILE_CHECKSUM_MISMATCH`, `FILE_MIME_REJECTED`, `FILE_MALWARE_DETECTED`다.

## 8. Event·관측성·검증

상태 mutation은 `FileChanged.v1` outbox와 idempotency response를 같은 DB transaction에 기록한다. event에는 asset
ID·revision·action만 넣고 filename·storage key·token·checksum은 넣지 않는다. metric은 upload bytes·validation
duration·failure code·range bytes·GC retry를 low-cardinality label로 기록한다. 완료 gate는 adapter suite,
PostgreSQL reference/GC race, public scope test, `bun run check`, `bun run compose:integration`이다.
