# TASK-012: Docker Compose 실행 기반 구현

- **상태**: 완료
- **유형**: 구현
- **구현 패키지**: IMP-05
- **시작일**: 2026-08-25
- **완료일**: 2026-08-25
- **커밋**: 이 태스크 완료 커밋

## 목적

동일 artifact와 typed configuration을 사용해 local·single-host 환경을 재현하는 Docker Compose
reference를 만든다. migration 선행, secret file, volume ownership, core/degraded health와 backup을
선언에 그치지 않고 clean `up/down` integration gate로 검증한다.

## 범위

- 포함: multi-stage image, core/search/ai-local/observability/backup profile, PostgreSQL·Redis·
  OpenSearch·local object volume, migration one-shot, secret bootstrap, healthcheck, resource limit,
  backup artifact·manifest, Compose 정적·실행 검증
- 제외: domain HTTP route, Google credential 실연결, S3, production registry·TLS termination,
  Kubernetes·Cloud manifest, continuous WAL archive와 외부 backup destination

## 필수 설계 문서

- [x] `design/architecture/CONTAINER-DEPLOYMENT.md`
- [x] `design/architecture/SYSTEM-CONTEXT.md`
- [x] `design/implementation/CONFIGURATION-REFERENCE.md`
- [x] `design/implementation/POSTGRESQL-FOUNDATION.md`
- [x] `design/implementation/CONTAINER-RUNTIME.md`
- [x] `design/implementation/REPOSITORY-STRUCTURE.md`
- [x] `design/implementation/WORK-BREAKDOWN.md`
- [x] `design/operations/ENVIRONMENTS-CONFIG.md`
- [x] `design/operations/BACKUP-DISASTER-RECOVERY.md`
- [x] `design/operations/OBSERVABILITY-SLO.md`
- [x] UX·domain·HTTP API·사용자 권한: N/A — runtime topology 기반이며 제품 command를 추가하지 않는다.

## 문서 준비 게이트

- [x] service topology와 persistent·rebuildable state 경계가 ARCH-02에 정의됐다.
- [x] secret은 `_FILE`만 사용하고 named volume·profile·health 요구가 OPS-01에 정의됐다.
- [x] migration checksum·pending 0과 PostgreSQL 16 preflight가 PLAN-10에 정의됐다.
- [x] PostgreSQL·ObjectStorage만 복구 정본이며 backup 외부 destination 한계가 OPS-04에 정의됐다.
- [x] 구현 결과를 바꿀 미해결 질문이 없다.

## 사용자 결정

사용자는 Docker 배포, PostgreSQL·Redis·OpenSearch, 초기 local object storage와 향후 S3 확장을
확정했다. 권장안은 별도 승인 없이 선택한다.

## 의사결정

### 결정 1: 하나의 Compose project에 profile로 선택적 dependency를 분리한다

- **상황**: 여러 Compose file을 복제하면 service·volume·secret 계약이 환경마다 어긋난다.
- **검토한 대안**: 환경별 파일 복제 / 모든 dependency 상시 실행 / 단일 file과 profile.
- **선택과 근거**: core를 기본으로 하고 search·ai-local·observability·backup만 profile로 활성화한다.
  모든 profile은 같은 network·image label·secret·volume 정본을 재사용한다.

### 결정 2: migration 성공을 application 시작의 필수 dependency로 둔다

- **상황**: app이 구 schema에서 기동하면 부분 장애와 data corruption 가능성이 있다.
- **검토한 대안**: app startup 자동 migration / operator 수동 / one-shot migration service.
- **선택과 근거**: 동일 image의 one-shot migrator가 checksum과 preflight를 통과해야 app을 시작한다.
  여러 replica가 schema ownership을 경쟁하지 않는다.

### 결정 3: backup profile은 복호화 가능한 원본과 검증 manifest를 함께 만든다

- **상황**: 파일 존재만으로는 backup 성공이나 restore 가능성을 증명하지 못한다.
- **검토한 대안**: `pg_dump`만 저장 / database volume snapshot / dump·object archive·checksum manifest.
- **선택과 근거**: application-consistent PostgreSQL dump, object archive, migration version·SHA-256 manifest를
  staging volume에 원자 publish한다. 외부 destination·암호화가 없으면 production SLO를 주장하지 않는다.

## 구현 순서

1. image·service·profile·secret·volume·health·backup 계약을 상세 설계에 고정한다.
2. migration/preflight mode와 process-level health 기반을 구현한다.
3. Dockerfile, Compose, local secret bootstrap과 backup script를 구현한다.
4. config 정적 검사와 clean build/up/health/backup/down/volume cleanup을 실행한다.
5. 전체 gate 후 완료 처리하고 commit·push한다.

## 이슈 및 해결

- Bun `1.3.13-bookworm` tag가 registry에 존재하지 않았다. floating tag로 낮추지 않고 공식
  `1.3.13-debian` exact patch tag와 digest를 확인해 builder·runtime을 함께 교체했다.
- 격리 Docker build에서 Web tsconfig의 상위 `tsconfig.base.json`이 누락됐다. repository-root build
  contract를 Dockerfile의 명시적 입력으로 추가해 local·container build의 module boundary를 맞췄다.

## 검증

- [x] Compose config와 image target 정적 검사
- [x] clean build·migration·core up와 health 통과
- [x] search profile up와 core/degraded health 분리
- [x] backup profile artifact·checksum manifest 생성
- [x] down 후 container·network 제거와 명시적 test volume cleanup
- [x] root `bun run check`와 `git diff --check`

## 작업 내역

- 2026-08-25: IMP-05 태스크를 등록하고 container·환경·backup 정본을 확인했다.
- 2026-08-25: PLAN-11에 artifact, service profile, migration, secret, network, volume, health·종료와
  backup manifest의 실행 계약을 코드보다 먼저 고정했다.
- 2026-08-25: Rust·Bun multi-stage image, migration·volume-init one-shot, core와 선택 profile,
  file secret·internal network·named volume·healthcheck를 구현했다.
- 2026-08-25: PostgreSQL dump와 object archive를 checksum manifest와 함께 원자 publish하는 backup을
  구현하고 destination lock으로 중복 실행을 차단했다.
- 2026-08-25: 고유 Compose project에서 clean build, PostgreSQL 16 migration, Redis, API·Worker·Web
  health, OpenSearch profile, backup checksum, container·network·volume cleanup을 실제 검증했다.
- 2026-08-25: root `asdf exec bun run check`와 `git diff --check`를 통과했다.

## 결과

동일 artifact의 local·single-host 실행 reference와 clean integration gate가 준비됐다. PostgreSQL과
object_data는 정본으로, Redis·OpenSearch는 재구축 가능한 dependency로 분리했으며 선택 dependency
장애가 core lifecycle을 오염시키지 않는 profile 경계를 고정했다.
