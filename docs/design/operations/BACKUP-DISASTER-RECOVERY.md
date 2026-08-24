# Backup과 Disaster Recovery

- **문서 ID**: OPS-04
- **상태**: 동결

## 목표

RPO 15분, RTO 4시간. PostgreSQL continuous WAL+daily full, ObjectStorage incremental snapshot,
config·migration manifest를 35일 암호화 보존한다. Redis·OpenSearch는 backup 정본이 아니다.

## Restore 순서

격리 환경 준비 → key·artifact 검증 → PostgreSQL point-in-time restore → deletion ledger 재적용 →
ObjectStorage restore·checksum → application compatible version → outbox replay → OpenSearch rebuild →
Redis queue reconcile → invariant suite → traffic 전환.

## 검증

매월 automated restore, 분기별 full DR drill을 수행한다. Version pointer, Draft revision, Audit
sequence, File reference와 tenant count checksum을 원본 manifest와 비교한다.

## 지역·단일 host 장애

현재 특정 cloud multi-region을 약속하지 않는다. single-host self-hosted는 operator가 외부
backup destination을 구성해야 SLO를 주장할 수 있다. 설정되지 않으면 health에서 명시한다.

## 실패

backup age 15분 초과, restore test 실패와 encryption key 접근 실패는 alert다. 실패한 backup을
마지막 성공처럼 표시하지 않는다.
