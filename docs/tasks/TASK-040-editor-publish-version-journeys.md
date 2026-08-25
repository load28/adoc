# TASK-040: Editor·Publish·Version 사용자 여정 완성

- **상태**: 완료
- **유형**: 구현
- **시작일**: 2026-08-25
- **완료일**: 2026-08-25
- **커밋**: —

## 목적

전체 editor command와 Draft → Review/Publish → immutable Version → Diff/restore/conflict 흐름을
Web에서 끊김 없이 제공해 SCR-05·06·09·15와 RQ-05~08·15를 완성한다.

## 범위

- 포함: Published content, toolbar·slash·markdown shortcut·keymap·DnD·multi-select, table/code/media,
  Markdown/plain import, Markdown/plain/PDF export, publish dialog, policy impact, history/diff/restore,
  3-way conflict, File preview/download와 recovery
- 제외: Discussion·Review 상세 composer와 Settings 관리 화면

## 필수 설계 문서

- PROD-11·12·16, DOM-02·06, UX-05·06·10·12~16, SPEC-05~08·15·17~19
- CONTRACT-01·02, DATA-07·08, API-01·02·06~08, SEC-03·04, TEST-01·03·04·07·08
- PLAN-35 및 이 태스크에서 작성할 구현 계약

## 문서 준비 게이트

- [x] command·selection·operation·inverse·import/export 타입 계약 확정
- [x] lease loss·upload failure·stale publish·restore conflict·recovery 정의
- [x] Published immutable read와 Draft mutation 경계 정의
- [x] 구현 단위와 browser 검증 조건 추적

## 사용자 결정

없음. 정본의 전체 첫 구현 범위를 축소하지 않는다.

## 의사결정

- 별도 export service와 브라우저 snapshot export를 검토했다. Content 정본을 복제하지 않고 즉시 사용자에게
  제공할 수 있도록 Markdown·plain은 순수 browser serializer, PDF는 semantic print 경로를 선택했다.
- toolbar·slash·keymap별 command 구현과 단일 registry를 검토했다. availability와 Operation 의미의 분기를
  막기 위해 단일 registry를 선택했다.
- stale 상태에서 자동 덮어쓰기와 명시적 block conflict를 검토했다. immutable base와 local recovery를
  보존하도록 stable block ID 기반 명시적 conflict를 선택했다.

## 작업 내역

- 2026-08-25: TASK-039 완료 뒤 후속 DAG의 두 번째 구현 태스크로 시작했다.
- 2026-08-25: PLAN-37에 route·command·operation·import/export·File·Publish·Version·conflict 계약과
  구현·검증 단위를 고정해 문서 준비 게이트를 통과했다.
- 2026-08-25: 단일 editor command registry와 schema 기반 Markdown·plain interchange를 구현하고
  toolbar·selection·table·code·media·drag handle이 같은 command 조건을 사용하도록 연결했다.
- 2026-08-25: SSR Published snapshot, 정책 기반 Publish, immutable version 비교·복원과 revision·lease
  conflict recovery를 구현했다. File upload·download·preview와 암호화된 local recovery도 같은 화면 경계에 연결했다.
- 2026-08-25: 순수 command/schema/client test와 실제 PostgreSQL·Redis·OpenSearch·ObjectStorage Compose
  통합 검증, 전체 저장소 게이트를 실행했다.

## 이슈 및 해결

- `Headers` instance를 object spread로 합쳐 command의 CSRF·idempotency·revision header가 소실되는 공통
  API client 결함을 발견했다. 모든 JSON·empty request가 `Headers` 정규화 뒤 accept만 보강하도록 경계를
  교체하고 Publish·restore exact header test로 고정했다.

## 검증

- [x] editor command·schema·operation property test
- [x] publish/version/diff/restore/conflict integration test
- [x] import/export round-trip·file lifecycle test
- [x] root gate와 Compose integration

## 결과

PLAN-37의 command·interchange·Published immutable snapshot·Publish·Version·File·conflict 계약을 Web에
구현했다. `bun run check`가 계약 109개, 전체 테스트·빌드·보안·라이선스 검사를 통과했다. Compose
integration은 전체 저장소 통합 테스트와 backup/restore를 통과했으며 API ready p95 2.262ms, Web live
p95 1.159ms, SSR login p95 7.624ms를 기록했다.
