# TASK-036: System Hardening

- **상태**: 완료
- **유형**: 구현
- **구현 패키지**: IMP-27
- **시작일**: 2026-08-25
- **완료일**: 2026-08-25
- **커밋**: —

## 목적

전체 제품의 보안 응답 경계, 성능 smoke, 백업 복원, 관측성 구성과 공급망 증거를 자동화해
NFR SLO·RPO·RTO를 검증 가능한 release gate로 만든다.

## 범위

- 포함: Web security headers, live performance smoke, backup restore invariant, observability pipeline,
  release evidence/SBOM 입력 검증, CI·Compose gate
- 제외: 실제 production 배포·traffic 전환과 release tag(IMP-28), cloud multi-region

## 필수 설계 문서

- `docs/design/security/THREAT-MODEL.md`, `docs/design/security/AUTHENTICATION-SESSION.md`
- `docs/design/quality/SECURITY-TESTS.md`, `docs/design/quality/PERFORMANCE-TESTS.md`
- `docs/design/operations/OBSERVABILITY-SLO.md`, `docs/design/operations/BACKUP-DISASTER-RECOVERY.md`
- `docs/design/operations/CI-CD.md`, `docs/design/operations/RELEASE-RUNBOOK.md`
- `docs/design/implementation/SYSTEM-HARDENING.md`

## 문서 준비 게이트

- [x] browser·API security header와 capability cache 경계 확정
- [x] SLO performance smoke workload·percentile·failure 기준 확정
- [x] backup restore·checksum·migration·invariant 비교 계약 확정
- [x] trace·metric·log collector와 secret/cardinality 경계 확정
- [x] supply-chain evidence·CI·실패 복구 계약 확정

## 사용자 결정

권장안을 자율 확정한다. 현재 배포는 Docker를 지원하며 local object storage를 정본으로 두되 S3 adapter
확장 경계를 유지한다.

## 의사결정

- security header는 SSR·asset·proxy·error를 포함한 Web의 모든 response에 한 경계에서 적용한다.
- performance gate는 live Compose endpoint의 p95와 error rate를 함께 측정하고 평균으로 숨기지 않는다.
- backup 성공은 dump 생성만이 아니라 격리 DB restore·migration version·핵심 invariant 비교까지 요구한다.
- observability collector는 traces·metrics·logs pipeline을 분리하고 debug exporter는 local profile에만 둔다.
- release evidence는 lockfile·migration manifest·contract manifest·image label source의 hash를 묶어 생성한다.

## 구현 순서

1. PLAN-33을 확정하고 security response boundary를 구현한다.
2. performance·backup restore·release evidence 검사기를 구현한다.
3. Compose observability와 통합 gate를 연결한다.
4. negative/self test와 root·Compose gate를 통과한다.
5. 완료 기록 후 IMP-28로 인계한다.

## 작업 내역

- 2026-08-25: IMP-27 태스크를 등록하고 보안·성능·DR·관측성·CI 정본을 확인했다.
- 2026-08-25: PLAN-33에서 response, percentile, restore, telemetry, evidence 계약을 확정하고 문서
  준비 게이트를 통과했다.
- 2026-08-25: 모든 Web response에 중앙 보안 정책을 적용하고 cache·cookie 보존을 단위 테스트로
  고정했다.
- 2026-08-25: live endpoint 30회 p95 smoke와 lockfile·migration·contract·image 입력 digest 증거를
  root release gate에 연결했다.
- 2026-08-25: OTLP trace·metric·log pipeline을 분리하고 Compose backup을 격리 DB에 실제 복원해
  migration version과 핵심 데이터 invariant를 비교하도록 강화했다.

## 이슈 및 해결

- 기존 백업 검증은 파일 checksum까지만 확인해 논리 복원 가능성을 보장하지 못했다. 임시 파일 검사로
  보완하지 않고 격리 데이터베이스 복원과 schema·data invariant 비교를 동일한 Compose gate에 넣었다.

## 검증

- [x] security header·cache·negative corpus: Web unit test와 live header probe 통과
- [x] live performance percentile·error rate: API 2.265ms, Web 0.713ms, SSR login 6.35ms p95·오류 0
- [x] backup restore·checksum·migration·invariant: checksum·격리 restore·migration·revision·audit 통과
- [x] observability·evidence·root·Compose gate: `bun run check`, `bun run compose:integration` 통과

## 결과

PLAN-33의 보안·성능·복구·관측성·공급망 계약을 자동 release gate로 구현했다. IMP-27을 완료하고
전체 인수 및 release 묶음을 수행하는 IMP-28로 인계한다.
