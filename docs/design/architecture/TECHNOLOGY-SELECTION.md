# Technology Selection

- **문서 ID**: ARCH-04
- **상태**: 동결

| 영역 | 선택 | 구조적 이유 |
|---|---|---|
| Web | TanStack Start, React 18.2, TypeScript, Vite | route별 SSR·CSR, typed client |
| JS Toolchain | Bun package manager·workspace scripts | 단일 lockfile, frozen install, 빠른 workspace 실행 |
| Local Toolchain | asdf + root `.tool-versions` | Rust·Bun·Node 설치와 버전 선택 재현 |
| UI | public Apache-2.0 `@atlaskit` | 단일 token·component 체계 |
| Editor | Tiptap Core·ProseMirror | schema·transaction extension, Yjs 불필요 |
| API | Rust, Axum·Tokio·Tower | explicit async·middleware·SSE |
| DB | PostgreSQL + SQLx | transaction·constraint·typed SQL |
| Search | OpenSearch hybrid | lexical·vector projection 분리 |
| Queue | Redis | priority queue·ephemeral coordination |
| File | local ObjectStorage adapter | local first, S3 contract 유지 |
| Realtime | HTTP command + SSE | server→client state delivery에 충분 |
| AI | Codex CLI / OpenAI Responses API adapters | environment별 안전한 credential |

## 제외

ORM, WebSocket, CRDT·Yjs, Tiptap Collaboration, customer-facing API, silent model fallback,
병행 UI library와 custom design tokens를 제외한다.

Bun은 JavaScript·TypeScript package manager와 workspace script runner 경계에 사용한다. Rust
backend를 대체하지 않으며 Web production runtime은 배포 adapter의 Node-compatible output
계약을 유지한다. 정확한 Bun 버전과 dependency version은 manifest·toolchain file·lockfile에
고정한다.

로컬 toolchain은 root `.tool-versions`와 `asdf install`로 준비한다. 도구별 global version이나
별도 installer를 bootstrap 절차로 사용하지 않는다. Cargo·Bun manifest와 CI는 `.tool-versions`의
정확한 버전과 일치해야 한다.

## Build compatibility

TanStack Start의 Vite build는 Atlaskit distributed CSS를 처리하고 Vite 8의 공식
`@rolldown/plugin-babel`에서 `@atlaskit/tokens/babel-plugin`을 실행한다. ADS Compiled source를 자체 작성하지 않고 public
prebuilt component·primitive를 사용한다. React 18.2 peer dependency와 SSR matrix를 dependency
upgrade gate에서 검증한다.

## ADR

[ADR 목록](../adr/README.md)이 각 선택의 대안·결과를 소유한다. library version은 설계 의미가
아니며 lockfile과 dependency policy가 소유한다.
