# Release Runbook

- **문서 ID**: OPS-06
- **상태**: 동결

## 사전 조건

main CI green, signed image·SBOM, schema compatibility, backup/restore freshness, error budget,
known issue와 rollback image를 확인한다.

## 순서

expand migration → worker compatibility deploy → API canary → web canary → smoke·synthetic
Publish → 10%/50%/100% rollout → outbox·index lag 안정화 → release record.

## 검증

login, document read/save, lease, Publish, Search, AI job admission, File read와 public viewer
synthetic을 tenant-isolated fixture에서 실행한다. Audit·event가 정확히 생성되는지도 확인한다.

## 중단 기준

core burn rate, permission invariant, migration error, queue age와 data checksum threshold를
넘으면 자동 중단한다. 기능 flag로 schema mismatch를 숨기지 않는다.

## Rollback

traffic을 이전 image로 전환하고 worker producer version을 함께 맞춘다. backward incompatible
write가 시작됐다면 forward fix 또는 read-only recovery plan을 따른다.
