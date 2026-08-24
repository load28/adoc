# FileAsset

- **문서 ID**: SPEC-15
- **상태**: 동결

## State

`UPLOADING → VALIDATING → READY | FAILED → DELETED`. upload session expiry는 FAILED 후 partial
bytes를 정리한다.

## Upload

CreateUpload(name,mime,size,checksum) → authorized upload target → stream bytes with size cap →
CompleteUpload → checksum·detected MIME·malware scan → READY → preview job. claimed MIME만 믿지
않는다.

## Reference

AttachFile(ownerKind, ownerId, assetId)는 owner write permission과 asset READY를 검사한다.
Detach는 reference만 제거하고 asset을 즉시 삭제하지 않는다.

## Download

일반 actor는 owner Permission을 재검사한다. PublicLinkScope는 exact Published content의 assetId
set에 포함된 경우만 허용한다. response는 Content-Disposition, nosniff, CSP/sandbox와 짧은
cache policy를 사용한다.

## GC

unreferenced detection → 7일 purge_after → 재검사 → ObjectStorage delete → row DELETED. 과거
Version reference가 있으면 대상이 아니다. 실패는 idempotent retry한다.
