# System Hardening 구현 계약

- **문서 ID**: PLAN-33
- **상태**: 구현 기준
- **태스크**: TASK-036 / IMP-27

## 1. Security response boundary

Web runtime은 health, immutable asset, API proxy, SSR와 error response를 반환하기 직전에 하나의
`hardenResponse` 경계를 통과한다. 모든 응답에 `nosniff`, `frame-ancestors`, Referrer Policy,
Permissions Policy와 COOP를 적용한다. asset의 immutable cache는 유지하고 HTML·API·health는 기본
`no-store`다. production HTTPS에서만 HSTS를 추가한다. upstream Set-Cookie와 body stream은 보존한다.

## 2. Performance smoke

Compose 기동 뒤 core ready, Web live, SSR login, unauthenticated session을 warm-up 후 각각 30회 측정한다.
HTTP status가 기대와 다르면 latency와 무관하게 실패다. p95는 nearest-rank로 계산한다. local smoke
threshold는 core/Web 500ms, SSR 1s이며 정본 production 목표보다 느슨해 환경 noise를 허용한다.
결과는 JSON으로 남기며 URL query·body·token은 기록하지 않는다.

## 3. Backup restore evidence

backup manifest와 SHA-256을 검증한 뒤 새 격리 database에 custom dump를 restore한다. source와 restore의
migration max version, tenant row count, Version current pointer 결함, Draft revision 음수, Audit sequence
중복을 비교한다. Object archive checksum도 검증한다. restore database는 gate 종료 시 제거한다.
검증 없는 backup 생성은 RPO/RTO evidence가 아니다.

## 4. Observability와 release evidence

OTel local collector는 OTLP gRPC/HTTP을 받고 traces·metrics·logs를 별도 pipeline으로 export한다.
application content·prompt·query·title·token은 telemetry field 금지 목록을 유지한다. release evidence는
release SHA, 생성 시각, Cargo/Bun lock, migration manifest, contract manifest, Dockerfile의 SHA-256을
canonical JSON으로 생성한다. hash 입력 누락과 dirty generated contract는 gate 실패다.

## 5. Gate

정적 self-test는 header overwrite 방지, percentile boundary, evidence determinism을 검사한다. root gate에
hardening self-test와 evidence check를 추가하고 Compose integration에 live performance와 restore drill을
추가한다. 모든 실패는 non-zero이며 기존 테스트를 건너뛰는 fallback을 두지 않는다.
