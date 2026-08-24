# ADR-007: local-first ObjectStorage

- **상태**: 승인
- **결정일**: 2026-08-24
- **관련 결정**: DEC-018

## 결정

초기 binary storage는 local filesystem adapter지만 domain은 ObjectStorage port만 사용한다.
AWS S3 adapter가 같은 authorization-independent byte contract를 구현할 수 있어야 한다.

## 결과

storage path를 URL·domain ID로 노출하지 않는다. metadata와 reference lifecycle은 PostgreSQL이
소유한다.
