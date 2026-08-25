# TASK-045: 운영 hardening·최종 release 재검증

- **상태**: 완료
- **유형**: 구현·운영
- **시작일**: 2026-08-25
- **완료일**: 2026-08-25
- **커밋**: —

## 목적

관측성·성능·공급망·DR의 저장소 내 gate를 완성하고 TASK-039~044 결과로 전체 완료 manifest와
release candidate를 다시 생성한다.

## 범위

- 포함: dashboard·alert rules, metric/trace/log contract tests, Document/command/Search/AI workload,
  load·stress·soak·spike/degradation profile, dependency vulnerability, SBOM·provenance·signature verify,
  backup restore/rollback/deletion drill, final manifest·Compose·browser·release bundle
- 제외: credential·registry·production traffic이 없으면 실제 외부 실행만 environment_skip

## 필수 설계 문서

- PROD-06·09, ARCH-02·06·08, SEC-01~04, PRIV-01
- TEST-01·03·04·06·08·09, OPS-01~07, PLAN-03·07·33~35
- 이 태스크에서 작성할 observability·load·supply-chain·promotion 구현 계약

## 문서 준비 게이트

- [x] SLI metric과 alert rule의 machine-readable 연결 정의
- [x] workload profile·target·resource budget·실행 시간 경계 정의
- [x] vulnerability·provenance·signature와 외부 skip 증거 schema 정의
- [x] DR drill·release promotion·rollback의 불변 artifact 정의

## 사용자 결정

없음. 외부 환경만 필요한 실행은 이유와 의존 skip을 기록한다.

## 의사결정

- 운영 증거를 한 스크립트의 자유 형식 출력으로 합치는 방식과 SLI·workload·supply-chain·DR별 정본을
  검토했다. 실패 경계와 owner를 독립 검증할 수 있도록 영역별 machine-readable 정본을 두고 최종 proof
  manifest가 digest로 조합하는 방식을 선택했다.
- 외부 gate를 모두 skip하거나 local 결과를 production 증거로 간주하는 방식을 거부했다. 저장소에서 실행
  가능한 검증은 모두 실제 실행하고 registry identity·production traffic·외부 backup 권한만 구조화된
  environment skip으로 분리한다.
- local candidate를 unsigned로 두는 방식과 production key를 요구하는 방식을 검토했다. ephemeral Ed25519로
  provenance 무결성을 실제 sign/verify하되 trusted keyless production identity와 명시적으로 구분한다.
- RustSec `RUSTSEC-2023-0071`을 무시하거나 테스트 의존성을 런타임 위험으로 취급하는 방식을 검토했다.
  `rsa 0.9.10`이 배포 artifact가 아닌 RS256 unit fixture 생성에만 사용되고 수정 버전이 없으므로
  owner·90일 만료를 가진 예외로 등록하고 registry·Cargo audit·CI의 exact ID 일치를 gate로 둔다.

## 작업 내역

- 2026-08-25: TASK-044 완료·push 뒤 후속 DAG의 최종 운영·release 태스크로 시작했다.
- 2026-08-25: PLAN-42에 observability exact catalog, 두 계층 workload, dependency·SBOM·provenance,
  DR·rollback, completion·release proof 계약을 코드보다 먼저 고정하고 문서 준비 게이트를 통과했다.
- 2026-08-25: 실제 Bun audit은 취약점 0건을 확인했다. Cargo audit에서 테스트 전용 RSA advisory 1건을
  확인해 배포 경계를 검증하고 만료되는 정본 예외와 CI exact-match gate로 고정했다.
- 2026-08-25: 5개 SLI·21개 metric·4개 dashboard·7개 alert를 telemetry registry와 exact 연결하고
  repository·load·stress·soak·spike·degradation 성능 profile을 machine-readable 정본으로 만들었다.
- 2026-08-25: 3개 image SBOM, SLSA statement, ephemeral Ed25519 sign/verify, 외부 환경 skip과 completion
  audit를 하나의 production-readiness proof와 release manifest로 연결했다.
- 2026-08-25: Compose backup checksum과 격리 restore를 실행해 migration 21 일치와 5초 local RTO,
  10단계 DR proof를 생성했다. root gate와 54개 cross-engine browser gate가 통과했다.
- 2026-08-25: agent-browser로 실제 Compose UI의 Google SSO 진입점과 미인가 public viewer 실패 경계를
  접근성 snapshot·화면 캡처로 확인하고 전용 세션·컨테이너·볼륨을 정리했다.

## 이슈 및 해결

- RustSec 감사가 테스트 전용 `rsa 0.9.10`의 `RUSTSEC-2023-0071`을 보고했다. 배포 artifact 의존성이
  아니고 수정 버전도 없음을 확인해 `platform-security` owner와 2026-11-23 만료를 가진 예외로 등록했다.
  정본 registry·로컬 audit runner·CI ignore ID가 다르면 gate가 실패하게 했다.
- sandbox에서 Docker socket 접근이 거부됐다. 동일 명령을 승인된 Docker 경계에서 재실행했으며 제품
  데이터와 무관한 전용 project·volume만 만들고 종료 시 제거했다.

## 검증

- [x] completion manifest partial 0
- [x] root·Compose·browser·performance·DR·security gate
- [x] versioned immutable release candidate와 checksum
- [x] main commit·push 뒤 artifact identity 재검증

## 결과

RQ-01~20, SCR-01~22와 7개 quality gate의 `partial`을 0으로 만들었다. 저장소에서 실행 가능한 검증은
실제로 통과시켰고 production traffic·registry OIDC·외부 backup·실 credential만 6개 구조화된
`environment_skip`으로 남겼다. 같은 clean main SHA에서 생성하는 release candidate는 API·worker·web
image, SPDX JSON SBOM, provenance, DR·acceptance evidence와 전체 checksum을 하나의 identity로 고정한다.
