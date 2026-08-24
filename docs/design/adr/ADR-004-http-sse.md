# ADR-004: HTTP command와 SSE

- **상태**: 승인
- **결정일**: 2026-08-24
- **관련 결정**: DEC-020

## 결정

상태 변경은 idempotent HTTP command, server-to-client 전달은 SSE를 사용한다. 실시간 공동
타이핑이 없으므로 WebSocket과 CRDT를 사용하지 않는다.

## 결과

SSE는 cursor·reconnect·gap recovery를 가져야 하며 command 성공의 정본이 아니다.
