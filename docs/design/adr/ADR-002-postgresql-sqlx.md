# ADR-002: PostgreSQL과 SQLx

- **상태**: 승인
- **결정일**: 2026-08-24
- **관련 결정**: DEC-014, DEC-016

## 결정

PostgreSQL을 domain·Audit·Job 정본으로 사용한다. Rust persistence는 ORM 없이 SQLx의
명시적 SQL과 compile-time query 검증을 사용한다.

## 결과

invariant를 unique, foreign key, check와 transaction으로 함께 고정한다. repository가
aggregate 경계를 소유하며 handler가 SQL을 직접 실행하지 않는다.
