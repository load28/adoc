# TASK-032: Document Editor UX 구현

- **상태**: 완료
- **유형**: 구현
- **구현 패키지**: IMP-23
- **시작일**: 2026-08-25
- **완료일**: 2026-08-25
- **커밋**: —

## 목적

Tiptap 편집 transaction을 제품의 Content·Operation 계약으로 변환하고, 단일 Edit Lease와 순차 저장,
암호화 복구를 결합한 전체 Document Draft Editor를 구현한다.

## 범위

- 포함: Tiptap schema·renderer, toolbar·keymap·slash command, Draft loader, lease acquire·renew·release,
  operation buffer·ack·undo, conflict·offline recovery, paste·drop, File upload, 반응형·접근성 검증
- 제외: Discussion·Review·History panel(IMP-24), AI action 실제 실행(IMP-25), settings·public viewer(IMP-26)

## 필수 설계 문서

- `docs/product/PRD.md`, `docs/product/features/EDITOR.md`
- `docs/domain/document-system.md`
- `docs/design/specs/document/CONTENT-SCHEMA.md`
- `docs/design/specs/document/REGION-OPERATION-DIFF.md`
- `docs/design/ux/EDITOR-INTERACTIONS.md`
- `docs/design/ux/EDITOR-COMMAND-KEYMAP.md`
- `docs/design/ux/FRONTEND-STATE-ROUTE-CONTRACT.md`
- `docs/design/ux/ACCESSIBILITY.md`
- `docs/design/implementation/CONTENT-OPERATION-REDUCER.md`
- `docs/design/implementation/DOCUMENT-TREE-DRAFT-LEASE.md`
- `docs/design/implementation/FILE-OBJECT-STORAGE.md`
- `docs/design/quality/CONCURRENCY-RECOVERY-TESTS.md`
- `docs/design/implementation/DOCUMENT-EDITOR-UX.md`

## 문서 준비 게이트

- [x] Tiptap schema와 제품 Content·Operation 양방향 경계 확정
- [x] lease·operation buffer·ack·undo 상태 전이 확정
- [x] 암호화 복구·conflict·offline·tab 종료 계약 확정
- [x] paste·drop·File placeholder와 publish 차단 계약 확정
- [x] IME·keymap·접근성·multi-session 검증 전략 확정

## 사용자 결정

사용자는 MVP 축소 없이 전체 편집 기능을 구현하고 공개 Atlaskit component와 token만 사용하도록
결정했다. 에이전트는 권장안을 자율 확정해 전체 구현 계획이 끝날 때까지 진행한다.

## 의사결정

- ProseMirror position은 adapter 내부의 순간 값이며 모든 저장·복구 단위는 stable block ID 기반 제품
  Operation이다.
- transaction은 composition 중 buffer에 넣지 않고 composition 종료 뒤 의미 단위 Operation으로 변환한다.
- operation 전송은 단일 in-flight batch로 직렬화하고 server ack 이후에만 recovery record를 삭제한다.
- recovery store는 WebCrypto AES-GCM으로 암호화하며 tab session key가 없는 record는 복호화 실패 상태로
  명시해 평문 fallback하지 않는다.
- File placeholder는 editor-local node state이며 READY asset만 실제 Content image/file block으로 승격한다.
- SPA double-submit header를 구성할 수 있도록 non-HttpOnly CSRF cookie의 path는 `/`로 확장하되 session
  cookie의 HttpOnly·Secure·SameSite와 CSRF MAC 검증은 유지한다.

## 구현 순서

1. PLAN-29와 dependency·adapter 경계를 확정한다.
2. Tiptap Content adapter와 command registry를 구현한다.
3. lease·operation buffer·encrypted recovery controller를 구현한다.
4. Editor screen과 File upload·공통 상태를 연결한다.
5. IME·keymap·multi-session·a11y·root·Compose gate를 검증한다.

## 작업 내역

- 2026-08-25: IMP-23 태스크를 등록하고 관련 제품·도메인·UX·API·동시성 정본을 확인했다.
- 2026-08-25: PLAN-29에서 editor adapter, buffer, lease, recovery, upload와 검증 계약을 확정하고 문서
  준비 게이트를 통과했다.
- 2026-08-25: Tiptap schema adapter, stable block Operation 변환, lease-bound 직렬 buffer와 AES-GCM
  IndexedDB 복구 계층을 구현했다.
- 2026-08-25: Atlaskit toolbar, IME-safe command, Draft lifecycle, File upload·READY 승격과 공통 상태를
  Document route에 연결했다.
- 2026-08-25: schema·buffer·복구·정적 자산 단위 테스트, root gate와 PostgreSQL·Redis·OpenSearch·backup을
  포함한 Docker Compose 통합 gate를 통과했다.

## 이슈 및 해결

- 기존 CSRF cookie가 `/api/v1` path라 Document route의 JavaScript에서 header 값을 읽을 수 없었다. CSRF
  token은 same-origin UI 전체의 command capability이므로 PLAN-12 정본과 cookie helper를 `Path=/`로
  일치시키고 logout 만료 path도 함께 변경했다. session cookie 경계는 변경하지 않았다.
- Draft 단일 recovery key는 두 pending batch 중 뒤 batch가 앞 batch를 덮어썼다. group ID를 AES-GCM
  AAD와 IndexedDB key에 포함해 ack가 정확히 자기 암호문만 삭제하도록 변경했다.
- OpenAPI의 external schema root 참조가 `DocumentOperation` union에 `$defs` wrapper를 섞은 타입을
  생성했다. CONTRACT-02에 명시적인 `operation` definition을 두고 OpenAPI가 그 지점을 참조하게 했다.
- 실제 브라우저에서 theme bootstrap script의 SSR entity escape와 CSR 원문이 달라 hydration이 실패했다.
  inline raw HTML 대신 same-origin 정적 script file을 blocking load해 escape·hydration·XSS 경계를 함께
  해결하고 고정 입력 회귀 검증을 추가했다.
- root public asset은 운영 TanStack handler의 asset manifest에 없어 Docker에서 404였다. bootstrap을
  Vite asset graph에 넣고 SSR·CSR이 동일한 hash URL을 렌더링하도록 변경했다.
- 커스텀 Bun runtime이 SSR과 API proxy만 제공해 Vite client asset 전체가 404였다. `/assets/` hash
  namespace 전용 경로 정규화·MIME·immutable cache·`nosniff` 경계를 만들고 존재하지 않는 asset은
  SSR fallback 없이 404로 닫았다.

## 검증

- [x] schema·transaction·IME·keymap 검증
- [x] buffer·lease·offline·recovery·multi-session 검증
- [x] paste·upload·accessibility·responsive 검증
- [x] root·Compose gate

## 결과

Tiptap과 제품 Content·Operation 사이의 fail-closed adapter, 단일 in-flight lease buffer, 암호화 tab 복구,
File READY 승격을 갖춘 Document Draft Editor를 구현했다. SSR hydration과 운영 client asset 제공 경계까지
Docker 통합 환경에서 검증했다.
