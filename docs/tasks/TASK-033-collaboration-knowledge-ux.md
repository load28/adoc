# TASK-033: Collaboration·Knowledge UX 구현

- **상태**: 완료
- **유형**: 구현
- **구현 패키지**: IMP-24
- **시작일**: 2026-08-25
- **완료일**: 2026-08-25
- **커밋**: —

## 목적

Document 맥락의 Discussion·Review·History·Reference panel과 사용자 Inbox, 권한 안전 Search,
Vocabulary 관리를 Atlaskit 화면과 정본 API 계약으로 연결한다.

## 범위

- 포함: Discussion·Message·Topic, Review request·decision·Diff, Inbox read·resolve, Search·Source,
  Reference·Backlink, Vocabulary CRUD·deprecate, route deep link, 공통 상태·접근성 검증
- 제외: AI 실행·Proposal 화면(IMP-25), Governance settings·Audit·Trash·Public Viewer(IMP-26)

## 필수 설계 문서

- `docs/product/features/COLLABORATION.md`, `docs/product/features/KNOWLEDGE.md`
- `docs/domain/collaboration.md`, `docs/domain/knowledge.md`
- `docs/design/ux/COLLABORATION-FLOWS.md`, `docs/design/ux/SCREEN-INVENTORY.md`
- `docs/design/ux/FRONTEND-STATE-ROUTE-CONTRACT.md`, `docs/design/ux/ACCESSIBILITY.md`
- `docs/design/specs/collaboration/DISCUSSION-MESSAGE.md`
- `docs/design/specs/collaboration/REVIEW-APPROVAL.md`
- `docs/design/specs/collaboration/INBOX-NOTIFICATION.md`
- `docs/design/specs/knowledge/REFERENCE-GRAPH.md`
- `docs/design/specs/knowledge/VOCABULARY.md`
- `docs/design/specs/knowledge/INDEX-RETRIEVAL-SOURCE.md`
- `docs/design/api/openapi.yaml`, `docs/design/adr/ADR-004-http-sse.md`
- `docs/design/implementation/COLLABORATION-KNOWLEDGE-UX.md`

## 문서 준비 게이트

- [x] route·panel·query key와 deep-link 소유권 확정
- [x] Discussion·Review·Inbox command와 revision·멱등성 계약 확정
- [x] Search·Reference·Vocabulary 권한·비공개·cursor 계약 확정
- [x] SSE invalidation·offline·오류·복구 계약 확정
- [x] keyboard·responsive·접근성·screen behavior 검증 전략 확정

## 사용자 결정

사용자는 MVP 축소 없이 전체 기능을 구현하고 공개 Atlaskit component와 token만 사용하도록 결정했다.
권장 설계는 별도 승인 없이 자율 확정한다.

## 의사결정

- URL의 panel과 target ID가 공유 가능한 선택 상태의 정본이며 composer 초안만 browser session에 둔다.
- query cache는 Workspace·resource·permission fingerprint를 포함하고 command 성공은 영향받는 key만 무효화한다.
- restricted Reference·Search Source는 제목·count·snippet을 추측하지 않고 단일 제한 상태로 렌더링한다.
- Review decision과 Discussion 상태 변경은 exact revision과 새 idempotency key를 요구하며 낙관 확정하지 않는다.
- SSE는 정본 payload를 대체하지 않고 cursor 순서에 따라 query invalidation 신호로만 사용한다.

## 구현 순서

1. PLAN-30과 UI domain API·state contract를 확정한다.
2. Document collaboration panel과 Review·History·Reference surface를 구현한다.
3. Inbox·Search·Vocabulary route를 구현한다.
4. deep link·SSE invalidation·restricted state·responsive·a11y를 검증한다.
5. root·Compose gate를 통과하고 완료 기록한다.

## 작업 내역

- 2026-08-25: IMP-24 태스크를 등록하고 제품·도메인·UX·API 정본을 확인했다.
- 2026-08-25: PLAN-30에서 route, cache, command, permission, realtime, 접근성 계약을 확정하고 문서
  준비 게이트를 통과했다.
- 2026-08-25: Discussion·Review·History·Reference panel과 Inbox·Search·Vocabulary 화면을
  Atlaskit 기반으로 구현하고 정본 API client에 연결했다.
- 2026-08-25: SSE event를 query invalidation으로만 변환하는 UI domain 경계를 구현하고 단위 테스트로
  대문자 wire event 계약을 고정했다.
- 2026-08-25: root check와 Docker Compose 통합 검증을 모두 통과했다.

## 이슈 및 해결

- 외부 JSON Schema의 root fragment를 OpenAPI에서 참조하면 생성기가 wrapper object로 해석했다.
  `DocumentContent`를 명시적인 `$defs/content` 정본으로 만들고 모든 참조를 그 fragment로 통일했다.
- SSE event 이름을 화면 표기식 CamelCase로 가정한 불일치가 있었다. Rust 전송 경계의 대문자 event
  normalization을 확인하고 UI mapping과 테스트를 wire contract에 맞췄다.

## 검증

- [x] Discussion·Review·Inbox 상태·명령 검증
- [x] Search·Reference·Vocabulary 권한·cursor 검증
- [x] deep link·SSE·offline·restricted 상태 검증
- [x] keyboard·responsive·a11y·root·Compose gate

## 결과

문서 협업과 지식 탐색의 주요 surface가 권한 안전 API, exact revision command, SSE cache
invalidation 계약을 통해 연결됐다. root 정적·단위 검증과 전체 Docker Compose 통합 검증을 통과했다.
