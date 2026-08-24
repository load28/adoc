# Incident Runbook

- **문서 ID**: OPS-05
- **상태**: 동결

## 등급

SEV-1: tenant leak, data loss, core 전면 중단. SEV-2: 광범위 core 저하·RPO 위험. SEV-3:
Search·AI·preview 부분 장애. 등급은 원인이 아니라 사용자 영향으로 정한다.

## 초기 15분

incident commander 지정 → change freeze → 영향·시작 시각·tenant scope 확인 → correlation과
metric snapshot 보존 → 사용자 보호를 위한 기능 격리 → status communication.

## 대응 원칙

권한 leak 의심 시 public link·affected route를 deny-safe로 차단한다. data corruption은 write를
중지하고 snapshot을 보존한다. Search·AI 문제는 core를 내리지 않고 dependency circuit을 연다.

## Communication

확인된 사실, 사용자 영향, 우회, 다음 update 시각만 알린다. 추측 원인을 확정처럼 쓰지 않는다.
개별 Workspace 내용이나 identity를 incident channel에 복사하지 않는다.

## 종료

SLI 회복과 invariant test 뒤 종료한다. 5영업일 안에 timeline, root cause, contributing factor,
구조적 corrective task와 검증 test를 남긴다. 사람 실수를 root cause로 끝내지 않는다.
