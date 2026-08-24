# ADR-003: OpenSearch hybrid projection

- **상태**: 승인
- **결정일**: 2026-08-24
- **관련 결정**: DEC-017

## 결정

PostgreSQL을 정본으로 유지하고 OpenSearch에 lexical·semantic projection을 구축한다.
Permission Scope를 query 전에 적용하고 projection은 outbox로 갱신·전체 rebuild할 수 있다.

## 결과

Search의 eventual consistency를 UI에 표시한다. index 문서를 애플리케이션 state 복구에
사용하지 않는다.
