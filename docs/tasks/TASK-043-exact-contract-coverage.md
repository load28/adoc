# TASK-043: Operation·Event·State exact contract coverage

- **상태**: 완료
- **유형**: 품질
- **시작일**: 2026-08-25
- **완료일**: 2026-08-25
- **커밋**: —

## 목적

TEST-08의 집합 차이 규칙을 실제 parameterized assertion으로 구현해 109개 HTTP operation과 모든
event·state transition의 positive/negative 계약 증거를 만든다.

## 범위

- 포함: operation ID별 query/command matrix, auth·tenant·permission·pagination, validation·idempotency·
  stale·rollback·Audit·Outbox, event producer/consumer와 state transition exact coverage reporter
- 제외: browser interaction과 장시간 성능 workload

## 필수 설계 문서

- PROD-09, SPEC-17~19, API-01~08, CONTRACT-01~04, DATA-07~09
- TEST-01·03·04·07·08, PLAN-03·35 및 이 태스크에서 작성할 test harness 계약

## 문서 준비 게이트

- [x] Query·Command·lease·async operation별 필수 case schema 정의
- [x] exact ID report와 wildcard 금지 규칙 정의
- [x] transaction side effect와 rollback 관찰 경계 정의
- [x] 실제 PostgreSQL·Redis·OpenSearch fixture isolation 정의

## 사용자 결정

없음.

## 의사결정

- test 함수 이름을 정규식으로 추론하는 방식과 exact ID manifest를 검토했다. 하나의 통합 여정이 여러
  operation과 side effect를 검증하는 구조를 보존하면서 wildcard를 금지하기 위해 operation별 profile과
  evidence test를 명시하는 manifest를 선택했다.
- Event producer SQL과 consumer match arm만 정적으로 세는 방식과 schema·producer·consumer evidence의
  삼자 집합을 검토했다. payload schema를 event ID 정본으로 두고 producer·consumer test를 각각 요구하는
  삼자 집합을 선택했다.

## 작업 내역

- 2026-08-25: TASK-042 완료 뒤 후속 DAG의 다섯 번째 품질 태스크로 시작했다.
- 2026-08-25: PLAN-40에 operation profile, Event 삼자 집합, state transition exact ID와 실제 인프라
  evidence 격리 계약을 고정했다.
- 2026-08-25: OpenAPI 109개 operation을 14개 실제 통합 테스트 suite와 1,007개 필수 case ID에
  연결하고 누락·중복·wildcard를 거부하는 exact manifest와 검사기를 추가했다.
- 2026-08-25: schema·catalog·producer·consumer의 23개 Event와 SPEC-17의 정규화된 67개 상태
  전이를 실제 source 및 test evidence에 연결했다.
- 2026-08-25: stream consumer가 23개 wire event와 payload shape를 폐쇄적으로 수용하고 알 수 없는
  event 및 payload drift를 거부하는 단위 테스트를 추가했다.

## 이슈 및 해결

- 기존 문서의 operation·Event 개수 108·19가 정본 OpenAPI·schema의 109·23과 어긋나 있었다.
  문자열 상수를 복제하지 않고 정본에서 집합을 추출하도록 검사기와 문서를 바로잡았다.
- 모든 command가 Audit·Outbox를 각각 한 건 만든다는 규칙은 Audit 도메인의 중요 행위 선별 정책과
  충돌했다. operation profile이 선언한 정확한 개수 또는 0건을 검증하도록 계약을 구조화했다.
- 도메인별 test file 존재만 확인하면 개별 operation 누락을 탐지할 수 없었다. exact evidence manifest와
  집합 차이 검사로 개별 경계를 고정했다.

## 검증

- [x] OpenAPI operation test ID diff 0
- [x] Event producer·consumer test ID diff 0
- [x] State transition test ID diff 0
- [x] negative self-test와 Compose contract gate

## 결과

109개 operation·1,007개 필수 case ID, 23개 Event, 67개 상태 전이의 exact evidence 집합 차이가
0이다. root check와 실제 PostgreSQL·Redis·OpenSearch Compose 통합 테스트가 통과했다. 측정 p95는
API 2.040ms, Web 0.712ms, SSR login 5.073ms다.
