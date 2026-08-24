# Privacy와 Retention

- **문서 ID**: PRIV-01
- **상태**: 동결

## 개인정보 inventory

Google subject·email·display name, Membership, audit actor, authored content, Discussion, File,
AI Context·Result, session IP security log와 usage record를 처리한다.

## 목적 제한

identity는 인증·협업 표시, content는 문서 서비스, AI Context는 명시적 AI task, security
log는 침해 탐지, analytics는 집계된 제품 품질에만 사용한다. 목적 간 원문 재사용을 금지한다.

## 사용자 권리 기능

사용자는 profile과 자신이 작성한 entity 목록을 조회한다. Workspace Admin은 권한 범위의
Workspace export·deletion을 요청할 수 있다. export는 content, Version, Discussion, Vocabulary,
Reference와 manifest를 중립 archive로 제공하되 다른 사용자의 private security data는 제외한다.

## 삭제

Document·Workspace는 30일 유예 후 purge한다. AI payload 30일, backup 35일 정책은
[Lifecycle](../data/LIFECYCLE-RETENTION.md)을 따른다. Audit는 purge 시 비민감 tombstone으로
축소한다.

## Provider 전달

AI Context inspector에 외부 provider 전달 범위를 표시한다. 외부 web 사용은 task별 opt-in이다.
provider·region·retention 설정은 deployment privacy notice에 기록한다.

## 로그와 지원

지원자가 content를 임의 조회하지 않는다. 사용자 제공 correlation ID와 metadata로 먼저
조사하며 content access가 필요하면 시간 제한·사유·감사 절차를 따른다.
