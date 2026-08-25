# Data Lifecycle과 Retention

- **문서 ID**: DATA-04
- **상태**: 동결

| 데이터 | active 보존 | 삭제·정리 |
|---|---|---|
| Draft | Document 작업 동안 | Publish 뒤 archive context 최소화, 새 Draft와 분리 |
| Published Version | Workspace active 동안 불변 | Document 영구 삭제 또는 Workspace purge 때 제거 |
| Document trash | 30일 | purge job 전 impact·legal hold 검사 |
| Workspace deletion | 30일 유예 | tenant data·index·file·credential 전체 purge |
| FileAsset | 하나라도 reference가 있는 동안 | unreferenced+grace 7일 뒤 GC |
| Public link | revoke/expiry 전 | token hash는 audit minimum 외 제거 |
| Audit | Workspace active 동안 | content purge 뒤 비민감 tombstone으로 축소 |
| Session | 만료+30일 보안 추적 | token hash 제거 |
| AI Context·Result | 30일 | Proposal provenance 최소 hash·Source ID만 유지 |
| Job log | 30일 | metric aggregate만 유지 |
| Workspace Stream Event | 24시간 | client query reset 뒤 만료 row 삭제 |
| Backup | 35일 | 암호화 key lifecycle과 함께 만료 |

## Purge 순서

새 접근 차단 → public link·session revoke → queued job cancel → Search delete → File reference
계산 → domain row purge → ObjectStorage delete → backup expiry marker → minimal Audit tombstone.

## 복구와 삭제 충돌

복구 command는 purge lease 획득 전만 허용한다. purge가 시작되면 status를 `PURGING`으로 바꾸고
부분 복구하지 않는다. 실패한 purge는 step ledger로 재개한다.

## Backup

사용자 삭제가 성공해도 암호화된 backup에는 최대 35일 남을 수 있다. backup restore 후
deletion ledger를 먼저 재적용해 삭제 대상이 운영 상태로 되살아나지 않게 한다.
