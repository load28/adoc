# 프로젝트 문서 지도

- **상태**: 기준선
- **목적**: 전체 제품을 설계·구현·검증·운영하는 데 필요한 문서의 정본 위치와 상태를
  관리한다.
- **상태 값**: `미작성 | 작성 중 | 결정 대기 | 검토 중 | 동결`

이 파일이 구현 정본 문서 목록과 상태의 단일 진실 소스다. TASK-003에서 사용자 결정을 먼저
수집해 전체 문서를 일괄 작성했고 TASK-004에서 구현 계약까지 보강해 교차 검증했다.

## 1. 제품·요구사항 문서

| ID | 경로 | 소유하는 내용 | 상태 |
|---|---|---|---|
| PROD-01 | `product/PRODUCT-BRIEF.md` | 문제, 비전, 가치 제안, 성공 정의 | 동결 |
| PROD-02 | `product/PRODUCT-PRINCIPLES.md` | 사람 통제, 정확성, 공식 지식 등 제품 원칙 | 동결 |
| PROD-03 | `product/USERS-AND-STAKEHOLDERS.md` | 사용자, 역할, 이해관계자, 책임 | 동결 |
| PROD-04 | `product/USER-JOURNEYS.md` | end-to-end 사용자 여정과 use case | 동결 |
| PROD-05 | `product/IMPLEMENTATION-SCOPE.md` | 전체 첫 구현 포함·제외 범위 | 동결 |
| PROD-06 | `product/NON-FUNCTIONAL-REQUIREMENTS.md` | 보안, 성능, 가용성, 접근성, 복구 요구 | 동결 |
| PROD-07 | `product/GLOSSARY.md` | 제품·도메인 공통 용어의 의미 | 동결 |
| PROD-08 | `product/DECISION-REGISTER.md` | 사용자 Decision Gate와 확정된 제품 결정 | 동결 |
| PROD-09 | `product/REQUIREMENTS-TRACEABILITY.md` | 요구사항→설계→테스트 추적 | 동결 |
| PROD-10 | `product/features/WORKSPACE-AND-GOVERNANCE.md` | Workspace, 멤버십, 권한, PublishPolicy 요구 | 동결 |
| PROD-11 | `product/features/DOCUMENT-LIFECYCLE.md` | 트리, Draft, Publish, Version, Conflict 요구 | 동결 |
| PROD-12 | `product/features/EDITOR.md` | 편집 기능, Region, Operation, Import·Export 요구 | 동결 |
| PROD-13 | `product/features/COLLABORATION.md` | Discussion, Review, Inbox 요구 | 동결 |
| PROD-14 | `product/features/KNOWLEDGE.md` | Reference, Vocabulary, Search, Retrieval 요구 | 동결 |
| PROD-15 | `product/features/WRITING-INTELLIGENCE.md` | AI Task, Context, Writing Review와 Runtime 요구 | 동결 |
| PROD-16 | `product/features/FILES-AND-AUDIT.md` | FileAsset, Audit와 보존 요구 | 동결 |
| PROD-17 | `product/PRODUCT-METRICS.md` | 제품 성공 지표, 측정 정의와 guardrail | 동결 |

`product/PRD.md`는 분리된 제품 정본을 연결하는 동결 인덱스다.

## 2. UX·디자인 문서

| ID | 경로 | 소유하는 내용 | 상태 |
|---|---|---|---|
| UX-01 | `design/ux/INFORMATION-ARCHITECTURE.md` | route, navigation, 정보 위계 | 동결 |
| UX-02 | `design/ux/SCREEN-INVENTORY.md` | 전체 화면·패널·dialog 목록과 진입 조건 | 동결 |
| UX-03 | `design/ux/COMMON-STATES.md` | loading, empty, error, denied, retry, optimistic UI | 동결 |
| UX-04 | `design/ux/WORKSPACE-PERMISSION-FLOWS.md` | Workspace·멤버·권한·문서 이동 흐름 | 동결 |
| UX-05 | `design/ux/EDITOR-INTERACTIONS.md` | selection, block, command, table, media, keyboard | 동결 |
| UX-06 | `design/ux/DRAFT-PUBLISH-FLOWS.md` | autosave, lease, review, diff, conflict, history | 동결 |
| UX-07 | `design/ux/COLLABORATION-FLOWS.md` | Discussion, Topic, Mention, Review, Inbox | 동결 |
| UX-08 | `design/ux/KNOWLEDGE-AI-FLOWS.md` | Search, Source, Context, Proposal, AI Job | 동결 |
| UX-09 | `design/ux/DESIGN-SYSTEM.md` | 시각 token과 공통 component 계약 | 동결 |
| UX-10 | `design/ux/ACCESSIBILITY.md` | 키보드, focus, semantic, screen reader, contrast | 동결 |
| UX-11 | `design/ux/CONTENT-AND-MICROCOPY.md` | 용어, 오류·확인 문안, 상태 표현 | 동결 |
| UX-12 | `design/ux/RESPONSIVE-VISUAL-SPECS.md` | 화면별 wireframe, responsive, density와 visual state | 동결 |
| UX-13 | `design/ux/SCREEN-BEHAVIOR-SPECS.md` | 화면별 layout, action, state, API와 완료 조건 | 동결 |
| UX-14 | `design/ux/ATLASKIT-COMPONENT-MATRIX.md` | UI 요소별 공개 Atlaskit package·component 매핑 | 동결 |
| UX-15 | `design/ux/FRONTEND-STATE-ROUTE-CONTRACT.md` | route loader, cache, form·stream state와 ownership | 동결 |
| UX-16 | `design/ux/EDITOR-COMMAND-KEYMAP.md` | Editor command, selection, keymap와 기기별 대체 조작 | 동결 |

## 3. 시스템·아키텍처 문서

| ID | 경로 | 소유하는 내용 | 상태 |
|---|---|---|---|
| ARCH-01 | `design/architecture/SYSTEM-CONTEXT.md` | 사용자·외부 시스템·신뢰 경계 | 동결 |
| ARCH-02 | `design/architecture/CONTAINER-DEPLOYMENT.md` | web, server, worker, DB, index, storage 배치 | 동결 |
| ARCH-03 | `design/architecture/MODULE-ARCHITECTURE.md` | bounded context, module, port, 의존 방향 | 동결 |
| ARCH-04 | `design/architecture/TECHNOLOGY-SELECTION.md` | 기술 후보 평가와 선택 근거 | 동결 |
| ARCH-05 | `design/architecture/CROSS-CUTTING-CONTRACTS.md` | ID, 시간, revision, error, correlation | 동결 |
| ARCH-06 | `design/architecture/TRANSACTION-EVENT-JOB.md` | transaction, outbox, ordering, idempotency, retry | 동결 |
| ARCH-07 | `design/architecture/INTEGRATION-ARCHITECTURE.md` | AI CLI와 외부 Resource 연동 경계 | 동결 |
| ARCH-08 | `design/architecture/SCALABILITY-CAPACITY.md` | 부하 모델, 확장 경계와 용량 기준 | 동결 |
| ADR-00 | `design/adr/TEMPLATE.md` | Architecture Decision Record 템플릿 | 동결 |
| ADR-001 | `design/adr/ADR-001-monorepo-web-rust.md` | TanStack Start·Rust monorepo | 동결 |
| ADR-002 | `design/adr/ADR-002-postgresql-sqlx.md` | PostgreSQL·SQLx | 동결 |
| ADR-003 | `design/adr/ADR-003-search-projection.md` | OpenSearch hybrid projection | 동결 |
| ADR-004 | `design/adr/ADR-004-http-sse.md` | HTTP command·SSE | 동결 |
| ADR-005 | `design/adr/ADR-005-editor-engine.md` | Tiptap Core·ProseMirror | 동결 |
| ADR-006 | `design/adr/ADR-006-ai-runtime.md` | 환경별 AI Runtime adapter | 동결 |
| ADR-007 | `design/adr/ADR-007-object-storage.md` | local-first ObjectStorage | 동결 |
| ADR-008 | `design/adr/ADR-008-atlaskit-ui.md` | 공개 Atlaskit UI 체계 | 동결 |
| ADR-009 | `design/adr/ADR-009-bun-package-manager.md` | Bun package manager·workspace scripts | 동결 |
| ADR-010 | `design/adr/ADR-010-asdf-local-toolchain.md` | asdf local toolchain | 동결 |

중요 기술 선택은 `design/adr/ADR-NNN-<slug>.md`로 각각 기록한다. ARCH-04는 ADR 목록과
선정 결과를 통합해 보여주는 문서다.

## 4. 도메인·데이터·API 문서

| ID | 경로 | 소유하는 내용 | 상태 |
|---|---|---|---|
| DOM-00 | `domain/README.md` | 전체 도메인 지도와 공통 원시 개념 | 동결 |
| DOM-01 | `domain/workspace-governance.md` | Workspace·Permission 상위 불변식 | 동결 |
| DOM-02 | `domain/document-system.md` | Document·Draft·Version·Content 상위 불변식 | 동결 |
| DOM-03 | `domain/collaboration.md` | Discussion·Review·Inbox 상위 불변식 | 동결 |
| DOM-04 | `domain/knowledge.md` | Reference·Vocabulary·Retrieval 상위 불변식 | 동결 |
| DOM-05 | `domain/writing-intelligence.md` | AI Task·Context·Result 상위 불변식 | 동결 |
| DOM-06 | `domain/operations.md` | FileAsset·Audit 상위 불변식 | 동결 |
| DATA-01 | `design/data/CONCEPTUAL-MODEL.md` | aggregate와 관계의 기술 중립 모델 | 동결 |
| DATA-02 | `design/data/LOGICAL-SCHEMA.md` | table·document·constraint·index 논리 설계 | 동결 |
| DATA-03 | `design/data/DATA-DICTIONARY.md` | field, 타입, nullable, validation, ownership | 동결 |
| DATA-04 | `design/data/LIFECYCLE-RETENTION.md` | 생성·보존·삭제·GC와 개인정보 수명주기 | 동결 |
| DATA-05 | `design/data/MIGRATION-STRATEGY.md` | schema·data migration, rollback, compatibility | 동결 |
| DATA-06 | `design/data/ANALYTICS-EVENTS.md` | 제품 지표용 event, property, 개인정보 제한 | 동결 |
| DATA-07 | `design/data/schema.sql` | PostgreSQL table·type·constraint·index 기준 DDL | 동결 |
| DATA-08 | `design/data/DATABASE-INVARIANT-TRANSACTION-MATRIX.md` | invariant별 DB·application 책임과 transaction lock | 동결 |
| DATA-09 | `design/data/OPENSEARCH-PROJECTION-SCHEMA.md` | index mapping, permission filter와 rebuild schema | 동결 |
| API-01 | `design/api/API-CONVENTIONS.md` | command/query, auth, pagination, error, versioning | 동결 |
| API-02 | `design/api/openapi.yaml` | 기계 검증 가능한 HTTP request·response 계약 | 동결 |
| API-03 | `design/api/asyncapi.yaml` | 기계 검증 가능한 비동기 message 계약 | 동결 |
| API-04 | `design/api/EVENT-CATALOG.md` | domain·integration event 의미와 producer·consumer | 동결 |
| API-05 | `design/api/STREAMING-JOBS.md` | AI Job stream, cancellation, resume와 상태 계약 | 동결 |
| API-06 | `design/api/COMMAND-QUERY-CATALOG.md` | 전체 use case의 input·output·precondition catalog | 동결 |
| API-07 | `design/api/ERROR-CATALOG.md` | stable error code, status, retry와 UI action | 동결 |
| API-08 | `design/api/ENDPOINT-COVERAGE.md` | UI action·use case·OpenAPI operation coverage | 동결 |

각 bounded context의 상세 command, 상태 전이와 알고리즘은 별도 문서로 분리한다.

| ID | 경로 | 주제 | 상태 |
|---|---|---|---|
| SPEC-01 | `design/specs/governance/AUTHENTICATION-MEMBERSHIP.md` | 인증·Membership | 동결 |
| SPEC-02 | `design/specs/governance/PERMISSION-RESOLVER.md` | Permission Resolver | 동결 |
| SPEC-03 | `design/specs/governance/PUBLISH-POLICY.md` | PublishPolicy | 동결 |
| SPEC-04 | `design/specs/document/DOCUMENT-TREE.md` | Document Tree | 동결 |
| SPEC-05 | `design/specs/document/CONTENT-SCHEMA.md` | Content Schema | 동결 |
| SPEC-06 | `design/specs/document/REGION-OPERATION-DIFF.md` | Region·Operation·Diff | 동결 |
| SPEC-07 | `design/specs/document/DRAFT-LEASE.md` | Draft·Edit Lease | 동결 |
| SPEC-08 | `design/specs/document/PUBLISH-VERSION.md` | Publish·Version | 동결 |
| SPEC-09 | `design/specs/collaboration/DISCUSSION.md` | Discussion | 동결 |
| SPEC-10 | `design/specs/collaboration/REVIEW-INBOX.md` | Review·Inbox | 동결 |
| SPEC-11 | `design/specs/knowledge/REFERENCE-VOCABULARY.md` | Reference·Vocabulary | 동결 |
| SPEC-12 | `design/specs/knowledge/INDEX-RETRIEVAL-SOURCE.md` | Index·Retrieval·Source | 동결 |
| SPEC-13 | `design/specs/ai/TASK-CONTEXT-RESULT.md` | AI Task·Context·Result | 동결 |
| SPEC-14 | `design/specs/ai/JOB-RUNTIME.md` | AI Job·Runtime | 동결 |
| SPEC-15 | `design/specs/operations/FILE-ASSET.md` | FileAsset | 동결 |
| SPEC-16 | `design/specs/operations/AUDIT.md` | Audit | 동결 |
| SPEC-17 | `design/specs/STATE-TRANSITION-CATALOG.md` | 전체 aggregate·job 상태 전이 | 동결 |
| SPEC-18 | `design/specs/AUTHORIZATION-MATRIX.md` | actor·resource·action별 접근 조건 | 동결 |
| SPEC-19 | `design/specs/ALGORITHM-CATALOG.md` | resolver·merge·anchor·retention 핵심 알고리즘 | 동결 |

기계 검증 가능한 공통 payload는 설명 문서와 분리한다.

| ID | 경로 | 소유하는 계약 | 상태 |
|---|---|---|---|
| CONTRACT-01 | `design/contracts/document-content.schema.json` | versioned Document Content JSON Schema | 동결 |
| CONTRACT-02 | `design/contracts/document-operation.schema.json` | discriminated Document Operation JSON Schema | 동결 |
| CONTRACT-03 | `design/contracts/ai-contracts.schema.json` | AI Task·Context·Result·Proposal JSON Schema | 동결 |
| CONTRACT-04 | `design/contracts/event-payloads.schema.json` | domain·SSE event payload JSON Schema | 동결 |

## 5. 보안·품질·운영 문서

| ID | 경로 | 소유하는 내용 | 상태 |
|---|---|---|---|
| SEC-01 | `design/security/THREAT-MODEL.md` | asset, actor, trust boundary, abuse case와 완화 | 동결 |
| SEC-02 | `design/security/AUTHENTICATION-SESSION.md` | 로그인, session, credential, CSRF·XSS 경계 | 동결 |
| SEC-03 | `design/security/AUTHORIZATION.md` | Permission 적용 지점과 데이터 누출 방지 | 동결 |
| SEC-04 | `design/security/AI-AND-FILE-SECURITY.md` | CLI 격리, prompt injection, upload·download 보안 | 동결 |
| PRIV-01 | `design/security/PRIVACY-RETENTION.md` | 개인정보 inventory, 목적, 보존, export·삭제 | 동결 |
| TEST-01 | `design/quality/TEST-STRATEGY.md` | test level, fixture, environment와 gate | 동결 |
| TEST-02 | `design/quality/ACCEPTANCE-SCENARIOS.md` | 전체 사용자 여정의 인수 시나리오 | 동결 |
| TEST-03 | `design/quality/CONCURRENCY-RECOVERY-TESTS.md` | stale, duplicate, partial failure, recovery | 동결 |
| TEST-04 | `design/quality/SECURITY-TESTS.md` | authorization matrix와 abuse regression | 동결 |
| TEST-05 | `design/quality/AI-WRITING-EVALUATION.md` | 한국어 연구, groundedness, golden set, 회귀 평가 | 동결 |
| TEST-06 | `design/quality/PERFORMANCE-TESTS.md` | workload, target, load·stress·soak test | 동결 |
| TEST-07 | `design/quality/FIXTURE-CATALOG.md` | deterministic Workspace·content·permission fixture | 동결 |
| TEST-08 | `design/quality/CONTRACT-COVERAGE.md` | requirement·command·error·test coverage matrix | 동결 |
| TEST-09 | `design/quality/acceptance.feature` | 실행 가능한 전체 인수 Gherkin | 동결 |
| OPS-01 | `design/operations/ENVIRONMENTS-CONFIG.md` | 환경, 설정, secret와 feature flag | 동결 |
| OPS-02 | `design/operations/CI-CD.md` | build, gate, artifact, deploy, rollback | 동결 |
| OPS-03 | `design/operations/OBSERVABILITY-SLO.md` | metric, log, trace, SLI·SLO와 alert | 동결 |
| OPS-04 | `design/operations/BACKUP-DISASTER-RECOVERY.md` | backup, restore drill, RPO·RTO | 동결 |
| OPS-05 | `design/operations/INCIDENT-RUNBOOK.md` | 장애 분류, 대응, communication, 사후 분석 | 동결 |
| OPS-06 | `design/operations/RELEASE-RUNBOOK.md` | migration, rollout, verification, rollback | 동결 |
| OPS-07 | `design/operations/SUPPORT-RUNBOOK.md` | 사용자 문의, 데이터 조사, escalation과 권한 | 동결 |

## 6. 구현 전 최종 문서

| ID | 경로 | 소유하는 내용 | 상태 |
|---|---|---|---|
| PLAN-00 | `design/IMPLEMENTATION-MASTER-PLAN.md` | 전체 설계 프로그램과 동결 gate | 동결 |
| PLAN-01 | `design/implementation/REPOSITORY-STRUCTURE.md` | 실제 저장소·package·module 구조 | 동결 |
| PLAN-02 | `design/implementation/IMPLEMENTATION-PLAN.md` | 전체 구현 작업 DAG와 통합 순서 | 동결 |
| PLAN-03 | `design/implementation/DEFINITION-OF-DONE.md` | 코드·문서·테스트·운영 완료 조건 | 동결 |
| PLAN-04 | `design/implementation/RISK-REGISTER.md` | 기술·제품·운영 위험, owner와 완화 | 동결 |
| PLAN-05 | `design/implementation/DESIGN-FREEZE-REPORT.md` | 전체 문서 동결과 모순·누락 검토 결과 | 동결 |
| PLAN-06 | `design/implementation/MODULE-INTERFACE-CATALOG.md` | Rust port·service와 TypeScript module interface | 동결 |
| PLAN-07 | `design/implementation/CONFIGURATION-REFERENCE.md` | 모든 runtime config·default·validation·secret | 동결 |
| PLAN-08 | `design/implementation/WORK-BREAKDOWN.md` | 파일·migration·test 단위의 전체 구현 작업 | 동결 |
| PLAN-09 | `design/implementation/DETAIL-GAP-AUDIT.md` | 구현 차단 공백과 보강 결과 | 동결 |
| PLAN-10 | `design/implementation/POSTGRESQL-FOUNDATION.md` | migration·pool·transaction·outbox·멱등성 구현 계약 | 구현 기준 |
| PLAN-11 | `design/implementation/CONTAINER-RUNTIME.md` | image·Compose·health·secret·volume·backup 실행 계약 | 구현 기준 |
| PLAN-12 | `design/implementation/IDENTITY-SESSION.md` | Google OIDC·opaque session·CSRF·preference 구현 계약 | 구현 기준 |
| PLAN-13 | `design/implementation/WORKSPACE-MEMBERSHIP-GROUP.md` | Workspace·Membership·Invitation·Group 구현 계약 | 구현 기준 |
| PLAN-14 | `design/implementation/PERMISSION-PUBLISH-POLICY.md` | Permission Resolver·PublishPolicy 구현 계약 | 구현 기준 |
| PLAN-17 | `design/implementation/PUBLISH-VERSION-PUBLIC-LINK.md` | Publish·Version·Public capability 구현 계약 | 구현 기준 |
| PLAN-18 | `design/implementation/DISCUSSION-MESSAGE-INBOX.md` | Discussion·Message·Inbox 구현 계약 | 구현 기준 |
| PLAN-19 | `design/implementation/REVIEW-APPROVAL.md` | revision-bound Review·Approval·Publish gate 구현 계약 | 구현 기준 |
| PLAN-20 | `design/implementation/REFERENCE-VOCABULARY.md` | Reference graph·Vocabulary 구현 계약 | 구현 기준 |
| PLAN-21 | `design/implementation/FILE-OBJECT-STORAGE.md` | File lifecycle·ObjectStorage·reference·Range·GC 구현 계약 | 구현 기준 |
| PLAN-15 | `design/implementation/CONTENT-OPERATION-REDUCER.md` | Content·Region·Operation reducer 구현 계약 | 구현 기준 |
| PLAN-16 | `design/implementation/DOCUMENT-TREE-DRAFT-LEASE.md` | Document Tree·Draft·Lease 구현 계약 | 구현 기준 |

## 7. 작성 순서

```text
제품 문서 분해(PROD)
→ UX-01 + ARCH-01
→ UX·Architecture 기반 문서
→ Domain Specs + Data + API
→ Security + Quality + Operations
→ Requirements Traceability
→ Implementation Plan + Risk Register
→ Design Freeze Report
→ 전체 제품 구현
```

순서는 문서 의존성일 뿐 제품 기능을 나누는 출시 단계가 아니다.

## 8. 변경 규칙

- 문서를 추가·삭제·통합하려면 TASK 문서에 중복·누락 영향을 기록한다.
- 같은 정책을 둘 이상의 문서가 소유하지 않는다.
- 문서 상태는 검증 근거 없이 `동결`로 바꾸지 않는다.
- 사용자 결정이 필요한 문서는 `결정 대기`로 표시하고 임의 기본값을 채우지 않는다.
