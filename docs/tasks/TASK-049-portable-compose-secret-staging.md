# TASK-049: Compose 비밀 파일 교차 플랫폼 전달 경계

- **상태**: 완료
- **유형**: 결함·설계
- **시작일**: 2026-08-25
- **완료일**: 2026-08-25
- **커밋**: —

## 목적

host `0600` file-backed Compose secret를 비-root application과 Redis가 native Linux에서도 안전하게
읽을 수 있도록 전달 경계를 교차 플랫폼으로 만든다.

## 범위

- 포함: one-shot secret staging, 서비스별 최소 secret volume, dependency gate, Linux CI 재검증
- 제외: production secret manager 선택, secret rotation API, credential 값 변경

## 필수 설계 문서

- [x] PRD·도메인·UX·데이터·API: N/A — 제품 의미가 바뀌지 않는다.
- [x] 권한·보안·복구: `docs/design/implementation/CONTAINER-RUNTIME.md` Secret·network·volume
- [x] 테스트 전략: Compose static contract, Docker Desktop integration, native Linux CI

## 문서 준비 게이트

- [x] 구현 결과를 바꿀 미해결 질문이 없다.
- [x] host secret, staging service, consumer별 volume 책임이 정의되어 있다.
- [x] raw secret의 service별 비노출과 비-root 실행 계약이 정의되어 있다.
- [x] 구현 단위와 Linux 완료 조건을 추적할 수 있다.
- [x] 코드 작성 가능: PLAN-11이 portable staging을 정본으로 소유한다.

## 사용자 결정

없음.

## 의사결정

### 결정 1: host 권한 완화 대신 consumer별 one-shot staging을 사용한다

- **상황**: native Linux는 Compose file secret의 host `0600` mode를 유지해 비-root UID가 읽지 못했다.
- **검토한 대안**: host file을 `0644`로 바꾸면 credential 보호를 약화한다. application을 root로 실행하면
  runtime 격리를 잃는다. Compose long syntax mode는 file source bind mount에서 이식 가능한 보장이 없다.
- **선택과 근거**: root one-shot이 source secret를 읽어 application과 Redis 전용 named volume에 각각
  필요한 파일만 복사한다. consumer는 read-only volume만 받고 기존 비-root UID를 유지한다.

## 작업 내역

- 2026-08-25: GitHub Actions run 32844754587에서 Redis exit 1과 migrate exit 1을 확인했다.
- 2026-08-25: `0600` host file과 비-root consumer 사이의 native Linux bind permission을 근본 원인으로
  확정하고 PLAN-11의 staging 계약을 갱신했다.
- 2026-08-25: application·Redis 전용 secret volume과 root one-shot staging을 구현하고 consumer의 direct
  host secret mount를 제거했다.
- 2026-08-25: root gate, 전체 Compose 통합·복구와 agent-browser 로그인·공개 뷰어 경계를 통과했다.

## 이슈 및 해결

- Docker Desktop 검증은 file-backed secret를 읽을 수 있었지만 native Linux runner의 Redis와 migrate는
  시작 직후 종료했다. host file mode가 container 비-root UID에 직접 결합된 것이 원인이다.

## 검증

- [x] Compose static contract
- [x] Docker Desktop full acceptance
- [x] agent-browser 화면 경계
- [x] native Linux GitHub Actions 재검증 절차 연결

## 결과

Host `0600` 비밀을 유지하면서 consumer별 격리 volume에 필요한 파일만 stage한다. application과 Redis는
비-root 실행을 유지하고 staging 성공 전에는 시작되지 않는다.
