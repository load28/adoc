# TASK-041: Collaboration·Knowledge·Governance 사용자 여정 완성

- **상태**: 완료
- **유형**: 구현
- **시작일**: 2026-08-25
- **완료일**: 2026-08-25
- **커밋**: —

## 목적

현재 일부 action만 연결된 Collaboration·Knowledge·AI·Settings 화면을 정본 primary action과
상태 전이에 맞춰 완성해 SCR-07·08·10~14·16~21과 RQ-02·03·09~16을 충족한다.

## 범위

- 포함: Topic/Message/reference/attachment, Review request·approve·changes request·cancel, Inbox
  navigation, Reference mutation/backlink, Vocabulary lifecycle, AI 6 task/context/source/proposal,
  Group membership, Permission delete/explain·PublishPolicy, Writing Rules, Audit filter/detail
- 제외: editor command 자체와 실제 browser matrix·운영 infrastructure

## 필수 설계 문서

- PROD-10·13~16, DOM-01·03~06, UX-04·07·08·10·12~15, SPEC-02·03·09~19
- CONTRACT-03·04, DATA-07~09, API-01~08, SEC-03·04, TEST-01·03~05·07·08
- PLAN-35 및 이 태스크에서 작성할 구현 계약

## 문서 준비 게이트

- [x] 화면별 primary action과 aggregate transition을 1:1로 고정
- [x] permission scope가 query/search/AI/file보다 먼저 적용됨을 정의
- [x] stale·cancel·retry·invalid proposal·denied source 복구 흐름 정의
- [x] 구현 단위와 exact test ID 추적

## 사용자 결정

없음. 정본의 사람 승인과 AI 비자율 원칙을 따른다.

## 의사결정

- 화면마다 command를 직접 조합하는 방식과 UI-domain의 typed command 경계를 검토했다. exact revision,
  lease, idempotency와 invalidation을 동일하게 보장하기 위해 typed client command 경계를 선택했다.
- 화면별 고립된 선택 상태와 canonical URL state를 검토했다. 공유·Inbox deep link·복구가 가능한 resource
  selection만 URL이 소유하고 composer와 destructive confirmation은 local state가 소유하도록 선택했다.
- Audit client filtering과 server-authorized filtering을 검토했다. 페이지 단위 client filtering은 누락된 결과와
  잘못된 count를 만들므로 server query 계약을 확장하고 permission check 뒤 filter를 적용하도록 선택했다.

## 작업 내역

- 2026-08-25: TASK-040 완료 뒤 후속 DAG의 세 번째 구현 태스크로 시작했다.
- 2026-08-25: 기존 Web surface와 109개 OpenAPI operation을 대조해 끊긴 primary action과 상태 전이를
  식별하고 PLAN-38에 route·command·permission·recovery·test 계약을 고정했다.
- 2026-08-25: Discussion Topic·Message·Attachment, Review, Inbox, Reference, Vocabulary와 AI 6개
  task의 상태 전이를 typed client command와 Atlaskit 화면에 연결했다.
- 2026-08-25: Group membership, 명시 권한 삭제·설명, PublishPolicy와 Audit filter/detail을 exact
  revision 및 서버 권한 경계에 연결했다.
- 2026-08-25: OpenAPI와 생성 계약, PostgreSQL read model, Rust application/domain 경계를 함께
  갱신하고 브라우저 단위 순수 계약 및 실제 인프라 통합 테스트를 추가했다.

## 이슈 및 해결

- Audit를 현재 페이지에서만 거르면 권한 적용 뒤의 전체 결과 집합과 count가 달라졌다. typed
  `AuditFilter`를 HTTP·application·repository에 관통시키고 권한 확인 뒤 parameterized SQL로 적용했다.
- Message read model에 attachment ID가 없어 기존 첨부를 보존한 편집이 불가능했다. 파일 reference
  projection을 Message aggregate 조회에 포함해 읽기와 쓰기 계약을 대칭으로 만들었다.
- AI Job이 target을 돌려주지 않아 Inbox가 대상 문서를 추측해야 했다. admission에 사용한 닫힌
  `AiTarget` union을 저장된 task에서 복원해 job read model과 OpenAPI에 포함했다.

## 검증

- [x] 각 화면 action·state component/integration test
- [x] cross-tenant·permission·stale·idempotency negative test
- [x] event·Inbox·Audit·Outbox 일치 test
- [x] root gate와 Compose integration

## 결과

`bun run check`를 통과했다. `bun run compose:integration`도 PostgreSQL·Redis·OpenSearch를 포함해
통과했다. 성능 smoke의 p95는 API readiness 2.149ms, Web live 1.185ms, SSR login 5.710ms였다.
