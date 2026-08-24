# Support Runbook

- **문서 ID**: OPS-07
- **상태**: 동결

## 접수 정보

사용자 설명, 발생 시각·timezone, Workspace opaque ID, 화면, error code와 correlation ID를
받는다. password, session, public link token, document 원문과 provider credential을 요청하지
않는다.

## 조사 순서

service health → deploy·incident → correlation trace → domain status·revision → permission
explanation → queue·projection lag 순으로 metadata를 본다. content 열람은 기본 조사 단계가 아니다.

## 권한

Support role은 Document content access를 갖지 않는다. 필요한 경우 사용자가 export한 최소
재현자료를 받거나 시간 제한된 audited access 절차를 별도 승인한다.

## 처리

사용자 조작으로 안전하게 복구 가능한 경우 정확한 command를 안내한다. DB 직접 수정, index
강제 문서 삽입과 Audit 삭제를 금지한다. 운영 correction도 application repair command와
idempotency를 사용한다.

## Escalation

tenant leak·data loss는 즉시 incident, 반복되는 product defect는 task+regression test,
provider outage는 dependency status와 Job retry policy로 전달한다.
