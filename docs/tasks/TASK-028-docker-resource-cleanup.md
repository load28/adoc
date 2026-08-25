# TASK-028: Docker 개발 리소스 정리

- **상태**: 완료
- **유형**: 운영
- **시작일**: 2026-08-25
- **완료일**: 2026-08-25
- **커밋**: —

## 목적

반복된 Compose 검증으로 누적된 Docker 리소스를 정리해 로컬 디스크를 회수한다. 개발에
필요한 실행 환경과 영속 데이터는 유지하고, 재생성 가능한 임시 리소스만 제거한다.

## 범위

- 포함: Docker 사용량·소유 관계 조사, 종료된 임시 컨테이너, dangling image, TASK-012~015
  구형 빌드·명시적 debug image, 사용하지 않는 build cache 정리, 정리 전후 사용량 검증
- 제외: 실행 중인 개발 컨테이너, 사용 중인 image, 프로젝트의 named volume과 영속 데이터,
  source tree·의존성·도구 체인 삭제

## 필수 설계 문서

- [x] `design/architecture/CONTAINER-DEPLOYMENT.md`
- [x] `design/implementation/CONTAINER-RUNTIME.md`
- [x] `design/operations/ENVIRONMENTS-CONFIG.md`
- [x] 제품·도메인·UX·API·이벤트·권한 문서: N/A — 제품 동작을 변경하지 않는 로컬 운영 작업이다.
- [x] 데이터·복구: PostgreSQL·ObjectStorage named volume은 삭제하지 않는다.

## 문서 준비 게이트

- [x] 실행 중인 리소스와 named volume을 보존 대상으로 고정했다.
- [x] 삭제 대상을 재생성 가능한 종료 컨테이너·dangling image·구형 task/debug image·build
  cache로 제한했다.
- [x] volume prune와 전체 image prune를 금지했다.
- [x] 정리 전후 Docker 사용량을 비교한다.
- [x] 코드 작성은 필요하지 않다.

## 사용자 결정

사용자는 Docker에 누적된 미사용 컨테이너와 데이터를 정리하되 개발에 필요한 리소스는
유지하도록 결정했다.

## 의사결정

### 결정 1: 소유 관계가 명확한 재생성 가능 리소스만 정리한다

- **상황**: 전체 prune은 공간을 크게 회수하지만 개발 데이터와 다음 실행에 필요한 cache까지
  지울 수 있다.
- **검토한 대안**: Docker 전체 초기화 / 전역 prune / 상태를 조사한 뒤 제한 정리.
- **선택과 근거**: 실행 중인 컨테이너, 모든 volume, 현재 `adoc-task017-*` image와 기반
  image를 유지한다. 종료 컨테이너, dangling image, build cache, TASK-012~015의 구형 빌드와
  명시적 debug image는 이름을 정확히 확인한 뒤 제거한다.

## 작업 내역

- 2026-08-25: TASK-028을 등록하고 보존·정리 경계를 문서로 고정했다.
- 2026-08-25: 정리 전 Docker가 image 30.92GB, build cache 19.78GB, named volume
  284.1MB를 사용한다고 확인했다.
- 2026-08-25: 종료 컨테이너와 dangling image를 정리하고, 미사용 build cache를 모두
  제거했다.
- 2026-08-25: TASK-012~015 구형 image와 두 debug image를 명시적 이름으로 제거했다.
- 2026-08-25: 현재 `adoc-task017-*` image 6개, 기반 image 5개, named volume 7개가 모두
  보존됐음을 확인했다.

## 이슈 및 해결

- Docker의 prune 결과와 `system df`는 공유 layer를 서로 다른 기준으로 집계했다. 회수량을
  합산하지 않고 정리 전후 `system df`의 최종 사용량으로 검증했다.

## 검증

- [x] 정리 전 Docker 사용량과 리소스 목록 기록
- [x] 실행 중인 개발 컨테이너 보존
- [x] named volume과 영속 데이터 보존
- [x] 정리 후 사용량과 Git 작업 트리 확인

## 결과

build cache를 0B로 정리하고 image 사용량을 30.92GB에서 5.089GB로 줄였다. 현재 Compose와
기반 image 11개, 다른 프로젝트를 포함한 named volume 7개와 284.1MB의 영속 데이터는
삭제하지 않았다.
