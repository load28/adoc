# TASK-046: SPDX SBOM 버전 계약 정정

- **상태**: 완료
- **유형**: 결함·설계
- **시작일**: 2026-08-25
- **완료일**: 2026-08-25
- **커밋**: —

## 목적

Docker SBOM provider가 생성하는 유효한 SPDX JSON 2.2를 거부하는 release gate를 provider 독립적인
SPDX 2.x 계약으로 바로잡고 clean main release candidate를 다시 검증한다.

## 범위

- 포함: SPDX version capability, 필수 document field 검증, release manifest version evidence, 전체 재실행
- 제외: SBOM package 생성기 교체, production registry attestation

## 필수 설계 문서

- PLAN-42, OPS-02, OPS-07, SEC-04, TASK-045 release failure evidence

## 문서 준비 게이트

- [x] provider 출력과 product SBOM 계약의 책임 경계 정의
- [x] 허용 SPDX version과 필수 field 정의
- [x] manifest에 실제 version 보존 정의
- [x] 다른 형식·누락 package 거부 조건 정의

## 사용자 결정

없음.

## 의사결정

- Docker Desktop 내장 Syft를 교체하거나 `spdxVersion` 문자열만 2.3으로 바꾸는 방식을 거부했다. 실제
  document 의미를 보존하면서 SPDX JSON 2.2와 2.3을 capability 집합으로 받고 version·namespace·package
  count를 manifest에 기록하는 방식을 선택했다.

## 작업 내역

- 2026-08-25: clean main release에서 Docker SBOM이 `SPDX-2.2`를 생성해 2.3 exact gate가 중단된 뒤
  태스크를 시작했다.
- 2026-08-25: PLAN-42의 SBOM 계약을 provider가 실제 선언한 SPDX JSON 2.2 또는 2.3으로 정정했다.
- 2026-08-25: 공용 SPDX validator와 negative self-test를 추가하고 release manifest가 실제 version,
  namespace와 package count를 보존하게 했다.

## 이슈 및 해결

- release gate가 유효한 `SPDX-2.2` document를 2.3이 아니라는 이유만으로 거부했다. Docker provider
  capability와 제품 계약을 한 문자열로 묶은 것이 원인이었다. 지원 version 집합과 필수 document field를
  공용 validator가 소유하게 해 provider 출력과 release evidence 경계를 분리했다.

## 검증

- [x] SBOM version·namespace·package negative self-test
- [x] clean main 전체 release candidate
- [x] commit·push와 artifact identity 확인

## 결과

SPDX JSON 2.2·2.3만 허용하고 document identity·HTTPS namespace·비어 있지 않은 package inventory를
검증한다. release bundle은 실제 provider version을 변경하지 않고 manifest에 보존하며 다른 version이나
불완전한 document는 구조적으로 거부한다.
