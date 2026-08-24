# 제품 Decision Register

- **문서 ID**: PROD-08
- **상태**: 동결
- **목적**: 사용자 확인이 필요한 제품·도메인·보안·아키텍처·핵심 UX 결정을 추적한다.

이 문서는 질문과 결정 상태의 인덱스다. 구체적인 상황, 대안, 근거와 작업 내역은 해당
태스크가 소유하고, 확정된 정책의 정본은 관련 제품·도메인·상세 설계 문서가 소유한다.

## 대기 중

대기 중인 결정이 없다.

## 확정됨

| ID | 결정 | 결정일 | 정본 |
|---|---|---|---|
| DEC-000 | MVP를 두지 않고 PRD 전체를 첫 구현 범위로 삼음 | 2026-08-24 | `IMPLEMENTATION-SCOPE.md` |
| DEC-008 | Workspace를 tenant·보안 경계로 하는 다중 조직 SaaS로 운영 | 2026-08-24 | TASK-003, 향후 ARCH-01~02 |
| DEC-009 | 누구나 계정·Workspace를 생성하고 기존 Workspace는 초대로 가입 | 2026-08-24 | TASK-003, 향후 PROD-03~04 |
| DEC-010 | 사용자 인증은 Google SSO를 사용 | 2026-08-24 | TASK-003, 향후 SPEC-01·SEC-02 |
| DEC-011 | 모든 Google 계정을 허용하고 Google Workspace domain 제한은 두지 않음 | 2026-08-24 | TASK-003, 향후 SPEC-01·SEC-02 |
| DEC-012 | TanStack Start SSR·CSR 프론트엔드와 Rust 백엔드를 단일 모노레포로 구성하고 Docker 배포 지원 | 2026-08-24 | TASK-003, 향후 ARCH-02~04 |
| DEC-013 | Editor는 Tiptap Core·ProseMirror를 사용하고 Tiptap Collaboration·Yjs는 제외 | 2026-08-24 | TASK-003, 향후 PROD-12·SPEC-05 |
| DEC-014 | 애플리케이션 정본 데이터베이스는 PostgreSQL을 사용 | 2026-08-24 | TASK-003, 향후 ARCH-04·DATA-02 |
| DEC-015 | Rust HTTP·실시간 서버는 Axum·Tokio·Tower로 구성 | 2026-08-24 | TASK-003, 향후 ARCH-02~04 |
| DEC-016 | PostgreSQL 접근 계층은 ORM 없이 SQLx를 사용 | 2026-08-24 | TASK-003, 향후 ARCH-04·DATA-02 |
| DEC-017 | 검색은 PostgreSQL 정본과 분리된 OpenSearch projection으로 hybrid search 제공 | 2026-08-24 | TASK-003, 향후 ARCH-02·SPEC-12 |
| DEC-018 | FileAsset은 로컬 저장 구현으로 시작하고 ObjectStorage 경계로 AWS S3 교체 가능하게 설계 | 2026-08-24 | TASK-003, 향후 ARCH-07·SPEC-15 |
| DEC-019 | 비동기 작업 Queue는 Redis를 사용 | 2026-08-24 | TASK-003, 향후 ARCH-06·SPEC-14 |
| DEC-020 | 실시간 전달은 HTTP command와 SSE를 사용하고 WebSocket은 사용하지 않음 | 2026-08-24 | TASK-003, 향후 API-05·ARCH-07 |
| DEC-021 | Group 생성·멤버 관리·Group Permission을 전체 구현에 포함 | 2026-08-24 | TASK-003, 향후 PROD-10·SPEC-02 |
| DEC-022 | 가장 가까운 Grant 우선, 같은 위치의 개인 Grant 우선, Group은 deny 후 최고 access로 병합 | 2026-08-24 | TASK-003, 향후 DOM-01·SPEC-02·SEC-03 |
| DEC-023 | Manage는 EDITOR 이상에서만 허용하고 Admin도 문서 access를 우회하지 않음 | 2026-08-24 | TASK-003, 향후 PROD-10·SPEC-02 |
| DEC-024 | Workspace 기본은 직접 발행이고 문서 트리별 Review Required·승인 인원을 상속·재정의하며 Draft 변경 시 승인을 무효화 | 2026-08-24 | TASK-003, 향후 PROD-11·SPEC-09 |
| DEC-004 | 현재 Region의 제한적 AI 수정만 즉시 적용·Undo하고 다중 Region·문서 전체 변경은 Proposal·Diff 승인 후 적용 | 2026-08-24 | TASK-003, 향후 PROD-15·UX-08·SPEC-13 |
| DEC-007 | Document·Workspace는 30일 삭제 유예 후 영구 제거하고 Audit에는 삭제 사실과 비민감 식별자만 보존 | 2026-08-24 | TASK-003, 향후 DATA-04·PRIV-01·SPEC-15~16 |
| DEC-005 | 로컬·자체 호스팅은 Codex CLI를 지원하고 다중 사용자 운영은 Provider 경계 뒤 OpenAI Responses API를 사용 | 2026-08-24 | TASK-003, 향후 ARCH-07·SPEC-14·SEC-04 |
| DEC-025 | AI Context는 권한 있는 Workspace 지식으로 제한하고 외부 웹 지식은 작업별 명시적 활성화와 출처를 요구 | 2026-08-24 | TASK-003, 향후 PROD-15·SPEC-12~14 |
| DEC-006 | 조직 용어·금칙어·근거 정확성은 강제하고 문체·가독성은 권고하며 versioned baseline과 Workspace 재정의를 지원 | 2026-08-24 | TASK-003, 향후 PROD-15·SPEC-13·TEST-05 |
| DEC-026 | Docker Compose를 완전 지원하고 동일 컨테이너의 수평 확장을 설계하되 특정 Cloud·Kubernetes에는 고정하지 않음 | 2026-08-24 | TASK-003, 향후 ARCH-02·OPS-01~02 |
| DEC-027 | 핵심 문서 읽기·쓰기의 월간 SLO 99.9%, RPO 15분, RTO 4시간을 적용하고 Search·AI 장애를 격리 | 2026-08-24 | TASK-003, 향후 PROD-06·OPS-03~04 |
| DEC-028 | 대화형 AI 작업 우선, Workspace·사용자 동시 실행·예산 한도, 상태·사용량 공개, 자동 모델 전환 금지 | 2026-08-24 | TASK-003, 향후 ARCH-08·SPEC-14·OPS-03 |
| DEC-029 | 모든 기기에서 같은 핵심 기능을 제공하고 입력 장치 의존 기능에는 동등한 대체 조작을 제공 | 2026-08-24 | TASK-003, 향후 UX-01~08·UX-12 |
| DEC-030 | 첫 구현부터 한국어·영어 UI, 사용자별 locale·timezone, 언어 제한 없는 문서 본문을 지원 | 2026-08-24 | TASK-003, 향후 PROD-03·UX-11·PLAN-01 |
| DEC-031 | 공개 Apache-2.0 `@atlaskit` package·token·primitive를 유일한 UI 체계로 사용하고 WCAG 2.2 AA를 준수 | 2026-08-24 | TASK-003, 향후 UX-09~10·UX-12·PROD-06 |
| DEC-032 | 명시적으로 공유한 단일 문서의 최신 Published Version만 익명 Viewer 링크로 제공 | 2026-08-24 | TASK-003, 향후 PROD-05·10·SEC-03 |
| DEC-033 | AI 사용량·예산 통제는 포함하되 결제·요금제·구독 관리는 제품 범위에서 제외 | 2026-08-24 | TASK-003, 향후 PROD-05·ARCH-01 |
| DEC-034 | 고객용 Public API·Webhook은 제외하고 내부 API만 구현 | 2026-08-24 | TASK-003, 향후 PROD-05·API-01·ARCH-07 |
| DEC-035 | JavaScript·TypeScript workspace의 패키지 관리와 script 실행에 Bun을 사용 | 2026-08-25 | TASK-006, ADR-009·ARCH-04·PLAN-08 |
| DEC-036 | 로컬 Rust·Bun·Node toolchain은 asdf와 저장소 `.tool-versions`로 설치·선택 | 2026-08-25 | TASK-007, ADR-010·ARCH-04·PLAN-08 |

## 기록 규칙

- 관련 상세 설계를 시작하기 직전에 영향받지 않는 조사를 먼저 완료한다.
- 한 번에 사용자가 판단할 수 있는 명확한 질문으로 요청한다.
- 답을 받은 날, 선택과 근거, 변경된 정본 문서와 태스크를 기록한다.
- 결정 변경은 기존 행을 지우지 않고 새 결정 ID로 대체 관계를 남긴다.
