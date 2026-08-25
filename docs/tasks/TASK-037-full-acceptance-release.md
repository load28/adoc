# TASK-037: Full Acceptance Release

- **상태**: 완료
- **유형**: 구현·운영
- **구현 패키지**: IMP-28
- **시작일**: 2026-08-25
- **완료일**: 2026-08-25
- **커밋**: —

## 목적

W-01~09와 TEST-09 전체를 동일 artifact에서 검증하고, 배포자가 내용과 출처를 재검증할 수 있는
하나의 versioned release bundle을 만든다.

## 범위

- 포함: acceptance manifest·실행 gate, 누락된 acceptance regression, `1.0.0` version 정합성,
  image metadata·SBOM·release manifest·checksum bundle, CI release 준비와 runbook evidence
- 제외: 외부 production traffic 전환, cloud credential 생성, 사용자 승인 없는 registry publish·tag

## 필수 설계 문서

- `docs/product/PRD.md`, `docs/domain/README.md`
- `docs/design/quality/TEST-STRATEGY.md`, `docs/design/quality/ACCEPTANCE-SCENARIOS.md`
- `docs/design/quality/acceptance.feature`, `docs/design/quality/CONTRACT-COVERAGE.md`
- `docs/design/operations/CI-CD.md`, `docs/design/operations/RELEASE-RUNBOOK.md`
- `docs/design/operations/BACKUP-DISASTER-RECOVERY.md`
- `docs/design/implementation/IMPLEMENTATION-PLAN.md`, `docs/design/implementation/WORK-BREAKDOWN.md`
- `docs/design/implementation/FULL-ACCEPTANCE-RELEASE.md`

## 문서 준비 게이트

- [x] W-01~09와 TEST-09의 완료 의미·실행 evidence 계약 확정
- [x] version·source digest·image·SBOM·bundle 계약 확정
- [x] 누락·중복·skip을 거부하는 acceptance manifest 경계 확정
- [x] 실패·재실행·production promotion 경계 확정
- [x] 외부 배포 없이 검증 가능한 local candidate 범위 확정

## 사용자 결정

전체 범위를 한 번에 완성하고 권장안을 자율 확정한다. production 외부 시스템 변경에는 기존 안전
승인 경계를 유지한다.

## 의사결정

- 제품 첫 전체 범위 version은 MVP 표기가 없는 SemVer `1.0.0`으로 통일한다.
- Gherkin은 설명 파일로만 두지 않고 scenario와 실제 test evidence가 1:1인 manifest를 정본으로 둔다.
- 하나의 release는 세 runtime image와 contract·migration·SBOM·acceptance evidence의 digest 집합이다.
- 외부 credential이 없는 local 산출물은 명시적인 local candidate이며 production release로 오인하지 않는다.

## 구현 순서

1. PLAN-34와 acceptance manifest schema·negative self-test를 구현한다.
2. TEST-09에서 실제 회귀 evidence가 부족한 invariant를 보강한다.
3. version·image·SBOM·bundle 생성기와 CI gate를 구현한다.
4. root·Compose acceptance와 bundle checksum을 검증한다.
5. 완료 기록과 최종 구현 상태를 확정한다.

## 작업 내역

- 2026-08-25: IMP-28을 등록하고 TEST-01·02·09, OPS-02·04·06, PLAN-02·08의 전체 완료
  계약을 확인했다.
- 2026-08-25: PLAN-34에서 TEST-09 evidence, versioned release unit, local·production promotion
  경계와 실패 복구를 확정하고 문서 준비 게이트를 통과했다.
- 2026-08-25: TEST-09의 15개 scenario를 실제 test evidence와 1:1로 묶는 manifest와 누락·중복·
  skip negative 검사기를 구현했다.
- 2026-08-25: 승인 무효화, Discussion reopen history, 근거 없는 AI claim 거부, stale Proposal atomic
  거부 회귀를 보강했다.
- 2026-08-25: Cargo·JavaScript·OCI image version을 `1.0.0`으로 통일하고 세 image·SPDX SBOM·
  contract·migration·evidence·checksum을 하나로 묶는 candidate 생성기를 구현했다.
- 2026-08-25: source commit `7d11460ff8b22f723d1fd4ebd7833dd879e8bb55`에서 root·Compose gate를
  다시 실행하고 `adoc-1.0.0-7d11460ff8b2.tar.gz`를 생성했다.
- 2026-08-25: artifact 133,571,851 bytes, SHA-256
  `6916f5a0aaab777d2ec10d99c337a40586adb0251eaa9bcaf523338153c87356`, release identity
  `ab41e51812af1ba386c588d70f594f1e975d380e2abbc307b0e4707ab4f85e5d`를 재검증했다.

## 이슈 및 해결

- Operation이 없는 `INSUFFICIENT_CONTEXT` AI 결과도 dependency 검사에서 거부됐다. 빈 결과를 우회
  상태로 처리하지 않고 dependency 집합의 항등원으로 정의해 정상 결과는 허용하고 가짜 선택은 거부했다.
- release version을 runtime 환경 변수로도 전달해 typed configuration이 알 수 없는 키로 거부했다.
  build metadata와 runtime configuration 경계를 분리해 image build argument에만 유지했다.
- migration manifest의 마지막 entry에 `version` field가 있다고 가정해 첫 후보가 잘못된 metadata를
  기록했다. 파일명의 4자리 sequence를 schema로 검증하고 1부터 연속인 마지막 version만 기록하도록
  수정한 뒤 전체 후보를 폐기하고 clean commit에서 다시 생성했다.

## 검증

- [x] acceptance manifest negative self-test·15 scenario mapping
- [x] root·Compose 전체 acceptance
- [x] image metadata·SPDX SBOM·versioned bundle·checksum
- [x] clean tree·main·remote push 상태: source commit과 `origin/main` 일치 확인

## 결과

W-01~09와 TEST-09 15개 scenario가 동일 source digest에서 통과했다. API·worker·web image,
SPDX SBOM, 21개 migration, canonical contract, acceptance evidence와 checksum을 포함한 `1.0.0`
local candidate를 생성해 IMP-01~28 전체 구현을 완료했다.
