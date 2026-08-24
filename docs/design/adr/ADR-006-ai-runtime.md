# ADR-006: 환경별 AI Runtime adapter

- **상태**: 승인
- **결정일**: 2026-08-24
- **관련 결정**: DEC-005, DEC-025

## 결정

로컬·자체 호스팅은 isolated Codex CLI, managed multi-user는 OpenAI Responses API adapter를
사용한다. application은 공통 Runtime port와 structured result만 안다.

## 결과

개인 subscription credential을 공용 server에 저장하지 않는다. provider failure를 다른
model로 자동 fallback하지 않는다.
