# ADR-001: TanStack Start와 Rust monorepo

- **상태**: 승인
- **결정일**: 2026-08-24
- **관련 결정**: DEC-012, DEC-015

## 상황과 대안

SSR·CSR UI와 transaction 중심 domain server가 필요하다. full-stack TypeScript, 분리 저장소,
TanStack Start+Rust monorepo를 비교했다.

## 결정

TanStack Start web과 Axum·Tokio·Tower Rust backend를 한 monorepo에 둔다. schema-generated
TypeScript contract만 언어 경계를 넘으며 domain logic을 중복하지 않는다.

## 결과

web server function이 backend domain을 우회하지 않는다. 배포 artifact는 web/API/worker로
분리하고 같은 commit·contract version으로 검증한다.
