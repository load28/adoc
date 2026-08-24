# TASK-022: File·ObjectStorage 구현

- **상태**: 완료
- **유형**: 구현
- **구현 패키지**: IMP-15
- **시작일**: 2026-08-25
- **완료일**: 2026-08-25
- **커밋**: 본 태스크 완료 커밋에 포함

## 목적

Workspace FileAsset을 안전하게 업로드·검증·참조·다운로드하고, Local storage를 사용하면서도 S3 adapter로
교체 가능한 byte 경계를 만든다. Draft·PublishedVersion·Message reference와 GC가 경합해도 과거 지식의
byte가 삭제되지 않도록 transaction과 storage lifecycle을 분리한다.

## 범위

- 포함: upload session, local ObjectStorage port/adapter, streamed write·Range read, checksum·MIME·malware
  validation, File reference projection, public exact-version delivery, logical delete·GC race, API·migration·test
- 제외: 일반 Job/SSE runtime(IMP-17), Retention purge orchestration·deletion ledger(IMP-16), File UI(IMP-23·26),
  production S3 credential/configuration(동일 port의 후속 adapter)

## 필수 설계 문서

- [x] 제품·기능: `product/features/FILES-AND-AUDIT.md`, `design/specs/operations/FILE-ASSET.md`
- [x] 상태·알고리즘: `design/specs/STATE-TRANSITION-CATALOG.md`, `design/specs/ALGORITHM-CATALOG.md`
- [x] 데이터·API: `design/data/schema.sql`, `design/api/openapi.yaml`
- [x] 보안·권한: `design/security/AI-AND-FILE-SECURITY.md`, `design/security/AUTHORIZATION.md`
- [x] 품질: `design/quality/TEST-STRATEGY.md`, `design/quality/CONCURRENCY-RECOVERY-TESTS.md`
- [x] 구현 기준: `design/implementation/FILE-OBJECT-STORAGE.md`

## 문서 준비 게이트

- [x] upload session·byte storage·asset state transaction 경계가 정의되어 있다.
- [x] checksum·detected MIME·malware 검증과 실패 cleanup이 정의되어 있다.
- [x] owner reference 권한과 PublishedVersion 보존 불변식이 정의되어 있다.
- [x] 일반·Public download의 exact authorization과 Range 계약이 정의되어 있다.
- [x] GC 재검사·storage delete retry와 S3 교체 경계가 정의되어 있다.
- [x] 구현 결과를 바꿀 미해결 질문이 없다.

## 사용자 결정

사용자는 Local storage로 시작하되 언제든 AWS S3로 교체할 수 있게 설계하고, 구조적 권장안은 별도 승인 없이
적용하도록 확정했다.

## 의사결정

### 결정 1: ObjectStorage는 byte만 소유한다

- DB가 FileAsset state·권한·reference·idempotency를 소유한다.
- Local/S3 adapter는 opaque storage key의 write·stat·range read·delete만 구현한다.

### 결정 2: 업로드 권한은 일회성 session capability다

- upload capability는 actor·Workspace·idempotency key와 회전 key ID로 HMAC 파생해 replay 응답을 재현하고,
  DB에는 token hash·key ID·expiry만 저장한다.
- Local upload route와 미래 S3 presign adapter가 같은 UploadSession aggregate를 사용한다.

### 결정 3: File reference는 owner snapshot에서 재계산한다

- Content·Message commit은 새 owner payload의 asset ID set과 기존 projection을 diff한다.
- READY·same Workspace 검증과 reference 교체를 owner transaction에 포함한다.

### 결정 4: GC는 두 단계다

- reference 0인 READY asset은 DELETED·purge_after로만 전이한다.
- purge worker는 row lock 뒤 reference 0을 다시 확인하고 byte delete 성공 뒤 purge 완료를 기록한다.

## 구현 순서

1. PLAN-21과 canonical DDL·OpenAPI를 고정한다.
2. File domain·ObjectStorage/MalwareScanner port·Local adapter를 구현한다.
3. upload/complete/download/delete application·PostgreSQL transaction·HTTP route를 구현한다.
4. Draft·PublishedVersion·Message reference projection과 public exact scope를 연결한다.
5. adapter suite·GC race·Docker 통합·root gate 후 완료하고 IMP-16으로 진행한다.

## 작업 내역

- 2026-08-25: TASK-022를 등록하고 PLAN-21로 File·ObjectStorage 구현 경계를 고정했다.
- 2026-08-25: ObjectStorage port와 traversal 차단·partial fsync·atomic rename·Range Local adapter를 구현했다.
- 2026-08-25: upload capability·VALIDATING resume·checksum·magic MIME·stream malware 상태 전이를 구현했다.
- 2026-08-25: private·Public current-version file download와 보안 헤더·단일 Range 계약을 연결했다.
- 2026-08-25: Draft·PublishedVersion·Message reference projection과 참조 중 삭제 거부를 연결했다.
- 2026-08-25: FAILED cleanup·DELETED retention을 처리하는 lease 기반 GC worker를 구현했다.
- 2026-08-25: OpenAPI·generated contract·forward migration·Docker 통합 계약을 연결했다.
- 2026-08-25: root gate와 Docker PostgreSQL·Redis·backup·OpenSearch 통합 gate를 통과했다.

## 이슈 및 해결

- futures 계열 잠금 파일이 레지스트리에 없는 macro 조합을 가리켰다. 배포된 dependency family로 잠금
  관계를 다시 해석해 재현 가능한 build를 복구했다.
- PostgreSQL enum 값을 추가한 transaction 안에서 즉시 사용하는 migration은 안전하지 않았다. enum 추가와
  상태 constraint·session table을 0014·0015 forward migration으로 분리했다.
- Docker test-runner가 신규 test-only tempfile dependency를 내부 DNS 제한 때문에 받지 못했다. 표준 임시
  디렉터리와 명시적 cleanup으로 외부 의존성을 제거했다.

## 검증

- [x] Local ObjectStorage traversal·atomic write·Range adapter suite
- [x] upload token·size·checksum·MIME·malware state
- [x] owner permission·same-tenant READY reference projection
- [x] private·public current-version download·security headers
- [x] detach/delete/GC race·idempotency·outbox
- [x] generated contract·root·Compose gate

## 결과

IMP-15를 완료했다. `bun run check`와 `bun run compose:integration`이 모두 통과했다.
