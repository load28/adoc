# TASK-009: Typed configuration·telemetry 기반 구현

- **상태**: 완료
- **유형**: 구현
- **구현 패키지**: IMP-03
- **시작일**: 2026-08-25
- **완료일**: 2026-08-25
- **커밋**: —

## 목적

Web SSR·API·worker가 process environment와 secret 원문을 직접 사용하지 않게 한다. 시작 시
전체 설정을 typed parser로 검증하고, 검증된 설정만 telemetry 초기화에 전달해 구조화
log·trace·metric에서 민감 정보가 유출되지 않는 공통 기반을 만든다.

## 범위

- 포함: `crates/configuration`, API·worker와 Web SSR typed config, unknown·missing·invalid key 거부, duration·
  URL·CIDR·path·conditional constraint, secret file permission·rotation metadata, `--check-config`,
  tracing subscriber, OpenTelemetry endpoint config, metric registry, field redaction, negative corpus,
  app wiring
- 제외: 실제 DB·Redis·OpenSearch 연결 preflight, HTTP server, container secret mount, remote secret
  manager, exporter collector, domain/workspace configuration, production alert/dashboard

## 산출물

- process input을 한 번만 읽고 immutable `ApiConfig`·`WorkerConfig`를 만드는 configuration crate
- secret 값을 노출하지 않는 `SecretValue`·key ID·source metadata
- service/version/correlation 필드가 있는 JSON tracing과 민감 field 차단 layer
- in-process metric registry와 stable metric name·label validation
- 정상 fixture와 unknown·missing·invalid·insecure secret corpus
- API·worker `--check-config`와 startup wiring
- Web SSR runtime config·redacted structured event·bounded metric registry

## 필수 설계 문서

- [x] PROD-06 `product/NON-FUNCTIONAL-REQUIREMENTS.md`
- [x] ARCH-03 `design/architecture/MODULE-ARCHITECTURE.md`
- [x] ARCH-05 `design/architecture/CROSS-CUTTING-CONTRACTS.md`
- [x] PLAN-01 `design/implementation/REPOSITORY-STRUCTURE.md`
- [x] PLAN-03 `design/implementation/DEFINITION-OF-DONE.md`
- [x] PLAN-07 `design/implementation/CONFIGURATION-REFERENCE.md`
- [x] PLAN-08 `design/implementation/WORK-BREAKDOWN.md`
- [x] OPS-01 `design/operations/ENVIRONMENTS-CONFIG.md`
- [x] OPS-03 `design/operations/OBSERVABILITY-SLO.md`
- [x] TEST-04 `design/quality/SECURITY-TESTS.md`
- [x] 도메인·UX·API·이벤트·데이터 상태 전이: N/A — process bootstrap 기반만 구현한다.
- [x] 권한·동시성: N/A — immutable startup config와 thread-safe telemetry handle만 제공한다.

## 문서 준비 게이트

- [x] configuration과 telemetry의 소유권·의존 방향이 정의됐다.
- [x] key별 type·기본값·범위·conditional requirement가 PLAN-07에 정의됐다.
- [x] secret은 `_FILE`만 허용하고 production permission 실패 조건이 정의됐다.
- [x] content·prompt·query·title·token을 telemetry에 넣지 않는 계약이 정의됐다.
- [x] 실제 dependency connectivity는 IMP-04·05 이후 preflight로 분리됐다.
- [x] 구현 결과를 바꿀 미해결 질문이 없다.

## 사용자 결정

사용자는 2026-08-25 회전 secret을 `current` 필수·`previous` 선택의 단일 JSON file로 관리하는
권장안을 승인했다. 환경 분리, typed startup validation, `_FILE` secret, SLO와 redaction 정책은
기존 동결 설계에서 확정됐다.

## 의사결정

### 결정 1: process input과 service config를 분리한다

- **상황**: unit test와 binary가 전역 environment를 직접 변경하면 병렬성·재현성이 깨진다.
- **검토한 대안**: 각 module이 env 직접 조회 / global singleton / `ConfigSource` snapshot을 parser에
  전달.
- **선택과 근거**: binary boundary에서 environment를 한 번 snapshot하고 pure parser가 이를
  `ApiConfig`·`WorkerConfig`로 변환한다. negative corpus가 OS process state 없이 모든 분기를
  검증할 수 있다.

### 결정 2: secret 원문 type을 telemetry 경계 밖으로 내보내지 않는다

- **상황**: 일반 String으로 secret을 보관하면 Debug·error·span field에서 유출될 수 있다.
- **검토한 대안**: naming convention / log filter만 적용 / redacted secret type과 허용 field layer.
- **선택과 근거**: `SecretValue`의 Debug·Display를 항상 redacted하고 telemetry가 secret type을
  받지 않는다. field 이름 기반 deny list는 dynamic instrumentation의 2차 방어로 둔다.

### 결정 3: metric cardinality를 등록 시점에 제한한다

- **상황**: document·user·workspace raw ID label은 비용과 존재 정보 유출을 만든다.
- **검토한 대안**: 호출자 자율 / exporter 후처리 / metric descriptor allowlist.
- **선택과 근거**: stable descriptor와 허용 label key를 registry에 등록하고 그 밖의 label을
  거부한다. workspace는 raw ID가 아니라 사전 계산된 opaque bucket만 허용한다.

### 결정 4: 회전 secret은 원자 교체 가능한 단일 JSON file로 읽는다

- **상황**: PLAN-07은 current·previous key ID를 요구하지만 file serialization을 정의하지 않았다.
- **검토한 대안**: key별 별도 파일 / delimiter line format / versioned JSON object.
- **선택과 근거**: 사용자가 권장한 JSON object를 승인했다. 구조 검증이 가능하고 current·previous를
  한 번에 원자 교체할 수 있으며 Docker secret mount 하나로 전달한다.

## 구현 순서

1. configuration·telemetry crate 경계와 dependency rule을 추가한다.
2. PLAN-07 key catalog를 typed service config와 pure parser로 구현한다.
3. secret file·permission·rotation metadata를 검증한다.
4. JSON tracing·redaction과 metric descriptor registry를 구현한다.
5. API·worker startup과 `--check-config`를 연결한다.
6. Web SSR의 non-secret runtime config와 같은 redaction·cardinality 불변식을 구현한다.
7. positive·negative corpus, redaction·cardinality test와 전체 gate를 실행한다.

## 이슈 및 해결

### 이슈 1: 전체 workspace 검사에서 새 transitive crate 다운로드가 격리 환경에 차단됨

- **증상**: 첫 `bun run check`의 Cargo metadata 단계에서 `valuable` 다운로드가 DNS 제한으로
  실패했다.
- **조사**: 부분 crate compile·test는 통과했고 lockfile에만 존재하던 all-feature dependency가
  전체 Clippy에서 처음 필요함을 확인했다.
- **근본 원인**: 구현 결함이 아니라 sandbox network 제한과 로컬 Cargo cache 미존재였다.
- **구조적 해결**: 승인된 network 경계에서 lockfile 그대로 dependency를 받아 전체 gate와 별도
  clean bootstrap을 다시 실행했다. source·version·lockfile은 변경하지 않았다.

## 검증

- [x] Web SSR·API·worker 최소 정상 config parse
- [x] unknown·missing·invalid enum/duration/URL/range/pattern 거부
- [x] plain secret environment와 production insecure secret permission 거부
- [x] driver별 conditional key와 production fixed retention 검증
- [x] secret Debug·Display·validation error·JSON log 유출 0
- [x] metric unknown name·label·raw cardinality 거부
- [x] `--check-config`가 secret 없이 source·key ID만 출력
- [x] root `bun run check`와 clean bootstrap 통과
- [x] `git diff --check` 통과

## 작업 내역

- 2026-08-25: IMP-03 태스크를 등록하고 PLAN-07·OPS-01·OPS-03·TEST-04 정본을 확인했다.
- 2026-08-25: configuration과 telemetry 책임을 별도 infrastructure crate로 설계 문서에 먼저
  반영했다.
- 2026-08-25: 사용자 승인에 따라 회전 secret JSON file 계약을 PLAN-07과 태스크에 기록했다.
- 2026-08-25: PLAN-07 전체 key catalog를 pure `ConfigSource` parser와 immutable API·worker
  config로 구현하고 Web SSR의 non-secret runtime parser를 추가했다.
- 2026-08-25: secret file permission·rotation·TLS·driver 조건과 production 고정 정책을 검증하고
  `SecretValue`의 Debug·Display·error·preflight redaction을 고정했다.
- 2026-08-25: Rust JSON tracing·causal trace context·bounded metric registry와 Web SSR의 동등한
  redaction·metric 불변식을 구현했다.
- 2026-08-25: API·worker 실제 process `--check-config`, negative corpus, 전체 workspace gate와
  별도 임시 복제본의 frozen Bun clean bootstrap을 통과했다.

## 결과

IMP-03을 완료했다. IMP-04는 이 typed config와 telemetry만 받아 PostgreSQL connection·migration·
transaction 기반을 구현할 수 있다. 실제 dependency connectivity preflight는 IMP-04·05에서
추가한다.
