# Production Readiness Proof

- **문서 ID**: PLAN-42
- **상태**: 구현 기준
- **구현 태스크**: TASK-045

## 1. 증거 경계

release readiness는 문서 체크리스트가 아니라 동일 source SHA에 묶인 machine-readable evidence 집합이다.
저장소 후보 게이트는 외부 credential 없이 실행 가능한 검증을 모두 실행한다. registry, keyless identity,
production traffic, 외부 backup destination처럼 실행 권한 자체가 없는 항목만 `environment_skip`을 허용한다.
실행 시간이 길다는 이유, binary가 없다는 이유와 실패한 검증은 skip이 아니다.

모든 evidence는 `schemaVersion`, `sourceSha`, `sourceDigest`, `generatedAt`, 실행 명령, dependency version,
결과와 artifact digest를 가진다. volatile field를 제외한 `proofIdentity`가 같으면 같은 release unit이다.
content, title, query, prompt, token, file name, credential과 원문 URL은 evidence에 기록하지 않는다.

## 2. Observability 계약

`infra/observability/catalog.json`이 SLI·metric·dashboard panel·alert의 단일 연결 정본이다. 각 SLI는 good·total
event, target, window와 latency threshold를 선언한다. 각 metric은 type, unit과 닫힌 low-cardinality label
집합을 가진다. dashboard panel은 catalog metric만 참조하고 alert는 SLI 또는 운영 invariant 하나를
참조한다. orphan metric·panel·alert와 문서 OPS-03 필수 metric 누락은 gate 실패다.

alert rule은 severity, multi-window 또는 invariant condition, `for`, runbook, owner와 사용자 영향 설명을
가진다. core burn rate, queue age, outbox lag, backup age, purge stuck, permission invariant와 provider credential
failure를 필수 집합으로 둔다. dashboard는 core SLO, async pipeline, dependency, security·lifecycle 네 관점을
제공한다. telemetry registry, redaction, TraceContext와 catalog의 이름·label 집합 차이를 Rust·Node
검사에서 0으로 만든다.

## 3. Performance qualification

`docs/design/quality/performance-profiles.json`은 다음 두 계층을 소유한다.

- `repository`: 모든 commit에서 실제 Compose endpoint로 실행하는 bounded qualification. Document read,
  command acknowledgement, public Viewer, Search와 AI admission/progress manifestation을 포함한다.
- `environment`: load 30분, stress-until-saturation, soak 8시간, spike 10배 5분과 dependency degradation.
  production-equivalent topology와 traffic generator가 필요한 실행은 evidence schema와 command를 저장소에서
  검증하고 실제 결과는 해당 환경에서만 생성한다.

각 profile은 arrival model, concurrency, duration, warm-up, fixture scale, resource budget, latency p95·p99,
error rate, saturation limit, stop condition과 required workload를 선언한다. 평균만으로 통과하지 않는다.
fixture는 권한 밖 title·content를 기록하지 않고 workload label만 출력한다. repository profile은 실제
Compose performance smoke와 browser/contract 결과 digest를 묶으며 mock latency로 통과하지 않는다.

## 4. Supply-chain·security proof

dependency gate는 Cargo와 Bun lockfile의 exact dependency 집합, advisory database timestamp, severity,
allowlist ID·owner·expiry를 evidence로 남긴다. 만료·근거 없는 예외, audit command 실패와 해석 불가능한
출력은 실패다. secret·license·contract gate와 함께 실행한다.

API·worker·web image별 SPDX JSON SBOM을 생성하고 package count·document namespace·image digest를
검증한다. provenance statement는 SLSA provenance predicate와 subject image digest, source SHA,
source digest, builder ID, lockfile·Dockerfile material digest를 포함한다. local candidate는 task 전용
ephemeral Ed25519 key로 statement를 서명하고 즉시 public key로 검증하며 private key는 artifact에 남기지
않는다. 이 서명은 무결성 self-proof이며 production identity가 아니다.

production promotion은 immutable RepoDigest, trusted CI OIDC keyless signature, provenance·SBOM attestation을
모두 요구한다. registry·OIDC가 없으면 각 의존성을 별도 `environment_skip`으로 기록하고 local signature를
production signature로 승격하지 않는다.

## 5. DR·retention proof

Compose DR drill은 backup checksum → 격리 restore → migration range → tenant count → current Version pointer →
Draft revision → Audit sequence → File archive checksum → deletion ledger → outbox replay·queue reconcile →
OpenSearch rebuild 순서를 검증한다. 각 단계의 시작·종료·결과와 RPO·RTO 측정값을 evidence로 남긴다.
rollback compatibility는 현재 migration min/max와 이전 application schema range의 교집합을 검사하고
무조건 down migration을 만들지 않는다.

local backup은 외부 destination이 아니므로 99.9%·RPO 15분·RTO 4시간 production 주장 근거가 아니다.
외부 encrypted destination과 scheduled restore evidence가 없으면 해당 항목만 environment skip이다.
30일 purge와 35일 backup policy는 manifest와 integration assertion이 모두 일치해야 한다.

## 6. Completion·release manifest

completion audit의 RQ-01~20, SCR-01~22와 quality gate는 후속 태스크의 실제 evidence ID로 갱신한다.
`partial`은 0이어야 한다. 외부 조건만 남은 항목은 `environment_skip`과 `reasonCode`, `dependency`,
`verificationCommand`를 가져야 한다. 코드·설정·test가 없는 항목은 environment skip이 될 수 없다.

release candidate 순서는 root → Compose integration → browser → observability → performance repository →
dependency audit → SBOM → provenance sign/verify → DR → completion audit → bundle checksum이다. bundle manifest는
각 evidence path·SHA-256, 3개 image identity와 외부 skip 목록을 포함한다. 실패한 단계 뒤 artifact를
재사용하지 않고 같은 clean main commit에서 전체 순서를 다시 실행한다.

## 7. Gate

1. SLI·metric·panel·alert·runbook exact 집합과 telemetry label 계약이 일치한다.
2. repository performance profile과 profile negative self-test가 통과한다.
3. dependency audit, secret·license, image SBOM과 local provenance 서명 검증이 통과한다.
4. backup·restore·rollback·deletion drill과 RPO/RTO 측정 evidence가 통과한다.
5. completion manifest의 partial이 0이고 environment skip은 외부 의존만 가진다.
6. clean main commit의 release bundle checksum과 proof identity가 재검증된다.
