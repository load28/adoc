# ADR-005: Tiptap Core와 ProseMirror

- **상태**: 승인
- **결정일**: 2026-08-24
- **관련 결정**: DEC-013

## 결정

Tiptap Core·ProseMirror를 client editing engine으로 사용한다. Collaboration·Yjs는 제외한다.
저장 schema, Region과 Operation은 제품 contract가 소유한다.

## 결과

ProseMirror position을 영속 ID로 저장하지 않는다. extension은 schema version·import·export·
operation validator를 함께 제공해야 한다.
