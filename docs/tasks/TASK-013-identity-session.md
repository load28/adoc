# TASK-013: Identity·Session 구현

- **상태**: 완료
- **유형**: 구현
- **구현 패키지**: IMP-06
- **시작일**: 2026-08-25
- **완료일**: 2026-08-25
- **커밋**: 이 태스크 완료 커밋

## 목적

Google OIDC Authorization Code+PKCE를 사용해 사용자를 식별하고, 원문을 저장하지 않는 opaque
session과 session-bound CSRF, preference 동시성 계약을 구현한다. 인증 경계를 이후 모든 Workspace·
Document API가 재사용할 수 있는 extractor와 application port로 고정한다.

## 범위

- 포함: Google discovery·authorization·token·JWKS adapter, login flow 1회성 저장, user upsert,
  session 발급·검증·rotation·logout, CSRF·Origin 검증, session/preferences HTTP API, cookie 정책,
  PostgreSQL repository, OIDC fake·PostgreSQL·HTTP security contract test
- 제외: Workspace 생성·Membership·Invitation(IMP-07), 세부 Permission(IMP-08), login 화면과 SSR
  loader(IMP-22), production Google credential을 사용하는 외부 E2E

## 필수 설계 문서

- [x] `product/PRD.md`, `product/features/WORKSPACE-AND-GOVERNANCE.md`, `product/NON-FUNCTIONAL-REQUIREMENTS.md`
- [x] `domain/workspace-governance.md`
- [x] UX: `design/ux/SCREEN-BEHAVIOR-SPECS.md`, `design/ux/FRONTEND-STATE-ROUTE-CONTRACT.md`
- [x] 데이터·상태: `design/data/LOGICAL-SCHEMA.md`, `design/data/schema.sql`,
  `design/data/LIFECYCLE-RETENTION.md`
- [x] API: `design/api/openapi.yaml`, `design/api/API-CONVENTIONS.md`,
  `design/api/COMMAND-QUERY-CATALOG.md`, `design/api/ERROR-CATALOG.md`
- [x] 권한·보안: `design/security/AUTHENTICATION-SESSION.md`, `design/security/THREAT-MODEL.md`,
  `design/security/PRIVACY-RETENTION.md`
- [x] 실패·복구·동시성: `design/architecture/INTEGRATION-ARCHITECTURE.md`,
  `design/data/DATABASE-INVARIANT-TRANSACTION-MATRIX.md`
- [x] 테스트: `design/quality/SECURITY-TESTS.md`, `design/quality/FIXTURE-CATALOG.md`
- [x] 구현 기준: `design/implementation/IDENTITY-SESSION.md`,
  `design/implementation/MODULE-INTERFACE-CATALOG.md`

## 문서 준비 게이트

- [x] 구현 결과를 바꿀 미해결 질문이 없다.
- [x] 정상·provider 오류·replay·만료·rotation·CSRF·revision conflict 흐름이 정의됐다.
- [x] OIDC port, repository transaction, cookie, HTTP·DDL 계약이 타입 수준으로 정의됐다.
- [x] IMP-06과 IMP-07·08·22의 책임 경계를 추적할 수 있다.
- [x] PLAN-12를 구현 기준으로 사용해 코드를 작성할 수 있다.

## 사용자 결정

사용자는 Google SSO, 모든 Google domain 허용, PostgreSQL session, 권장안의 자율 적용을 확정했다.

## 의사결정

### 결정 1: OIDC protocol은 port 뒤의 Google adapter로 격리한다

- **상황**: provider HTTP·JWT detail이 domain이나 handler에 들어가면 검증 누락과 provider 종속이 생긴다.
- **검토한 대안**: handler 직접 HTTP / Google SDK 종속 / 좁은 OIDC port와 adapter.
- **선택과 근거**: discovery·PKCE·token·JWKS 검증을 adapter가 수행하고 application에는 검증된 subject만
  반환한다. fake adapter도 같은 port contract를 통과한다.

### 결정 2: login flow는 hash-keyed PostgreSQL one-shot으로 관리한다

- **상황**: client cookie만으로 state를 관리하면 callback replay를 원자적으로 막을 수 없다.
- **검토한 대안**: signed state cookie / process memory / 만료·소비 상태가 있는 DB flow.
- **선택과 근거**: state hash를 key로 nonce·PKCE verifier·return path·expiry·consumed state를 저장하고
  callback transaction에서 한 번만 소비한다. 여러 API replica에서도 같은 결과를 보장한다.

### 결정 3: session token과 CSRF는 서로 다른 rotating key를 사용한다

- **상황**: bearer token 원문 저장과 key 재사용은 DB 유출·cross-protocol 공격 영향을 키운다.
- **검토한 대안**: signed JWT / 무키 SHA-256 / opaque token+keyed hash와 별도 CSRF MAC.
- **선택과 근거**: 256-bit opaque token은 session 전용 HMAC key로 hash해서 DB에 저장한다. CSRF token은 별도
  key로 session hash에 bind한다. 현재·이전 key ID를 제한적으로 읽고 신규 발급은 current만 쓴다.

## 구현 순서

1. PLAN-12에 protocol·domain·storage·HTTP·security·failure·test 계약을 고정한다.
2. canonical DDL과 migration에 login flow·session lifetime 불변식을 추가한다.
3. domain model, application service·port와 PostgreSQL·Google adapter를 구현한다.
4. Axum route·extractor·cookie·CSRF·problem response를 OpenAPI와 연결한다.
5. deterministic unit·contract·PostgreSQL·HTTP security test와 전체 gate를 실행한다.
6. 완료 기록 후 commit·push하고 IMP-07로 진행한다.

## 작업 내역

- 2026-08-25: IMP-06 태스크를 등록하고 제품·도메인·OIDC·session·API·DDL·보안 정본을 확인했다.
- 2026-08-25: PLAN-12에 provider 격리, one-shot flow, opaque session, CSRF와 test gate를 고정했다.
- 2026-08-25: canonical schema의 identity 변경을 forward migration과 SHA-256 manifest로 봉인했다.
- 2026-08-25: domain·application·Google OIDC·PostgreSQL·Redis adapter와 Axum·web proxy 경계를 구현했다.
- 2026-08-25: session 전용 HMAC, session-bound CSRF, 응답 크기 제한, 로그인 rate limit을 보안 계약으로 고정했다.
- 2026-08-25: JWT 실패 corpus, PostgreSQL 동시성, Redis rate limit과 전체 저장소 gate를 검증했다.

## 이슈 및 해결

- 기존 migration generator는 최신 canonical schema에서 이미 적용 가능한 `0001`만 다시 만드는 구조라
  forward-only 변경을 표현할 수 없었다. baseline을 수정하지 않고 filename·SHA-256 manifest로 모든
  migration을 봉인하는 일반 workflow로 PLAN-10과 도구를 확장한다.
- Google OIDC TLS adapter가 사용하는 `webpki-roots`는 root certificate data를
  `CDLA-Permissive-2.0`으로 배포해 기존 license gate가 거부했다. 잠긴 crate의 라이선스 원문에서
  사용·수정·공유 허용, 공유 시 라이선스 문구 제공, 결과물 무제한 조항을 확인하고 허용 정책과 회귀
  테스트에 해당 SPDX identifier를 명시한다.
- 최종 보안 정합성 검토에서 범용 token pepper가 login flow와 session에 함께 쓰이고 CSRF envelope가
  내부 session hash를 노출하는 구현 불일치를 발견했다. login flow는 token pepper, session은 전용
  session HMAC key로 분리하고 CSRF cookie는 key ID·nonce·MAC만 포함하도록 PLAN-12를 먼저 바로잡는다.
- Redis 1.6의 내부 hash dependency `xxhash-rust`가 `BSL-1.0`이라 license gate가 거부했다. 잠긴
  crate의 원문에서 사용·재배포·파생 저작물 허용과 machine-executable object code의 notice 예외를
  확인하고 허용 정책과 회귀 테스트에 SPDX identifier를 명시한다.

## 검증

- [x] domain·application unit와 negative corpus
- [x] OIDC fake/provider contract와 replay·nonce·issuer·audience·signature 실패
- [x] PostgreSQL user/session/preference concurrency와 expiry·revoke·rotation
- [x] HTTP cookie·Origin·CSRF·open redirect·problem response
- [x] 실제 PostgreSQL 16 identity integration
- [x] root `bun run check`, `git diff --check`, secret scan

## 결과

Google OIDC를 검증된 identity로 격리하고 opaque session·CSRF·preference 계약을 PostgreSQL과 Redis,
Axum HTTP 경계에 구현했다. 실제 컨테이너 통합 검증과 전체 저장소 gate가 통과했다.
