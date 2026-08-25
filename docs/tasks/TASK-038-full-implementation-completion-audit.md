# TASK-038: 전체 설계·구현 완료 감사

- **상태**: 완료
- **유형**: 조사·품질
- **시작일**: 2026-08-25
- **완료일**: 2026-08-25
- **커밋**: —

## 목적

IMP-01~28 완료 기록을 선언이나 파일 존재가 아니라 PRD·도메인·상세 설계의 요구사항별 실행 증거로
재검증한다. 누락·부분 구현·간접 증거를 후속 구현 태스크로 전환해 전체 제품 완료 조건을 증명한다.

## 범위

- 포함: PRD 범위, W-01~09, IMP-01~28, HTTP·event·DDL 계약, SCR-01~22, TEST-09,
  보안·접근성·성능·복구·릴리스 gate의 코드 및 실행 증거 전수 대조
- 제외: 기존 완료 기록을 근거로 한 자동 통과, 외부 production 배포·실제 Google credential처럼
  저장소 밖 환경이 필요한 실행

## 필수 설계 문서

- `docs/product/PRD.md`, `docs/domain/README.md`
- `docs/product/REQUIREMENTS-TRACEABILITY.md`
- `docs/design/implementation/IMPLEMENTATION-PLAN.md`
- `docs/design/implementation/WORK-BREAKDOWN.md`
- `docs/design/implementation/DEFINITION-OF-DONE.md`
- `docs/design/quality/TEST-STRATEGY.md`, `docs/design/quality/acceptance.feature`
- `docs/design/operations/CI-CD.md`, `docs/design/operations/RELEASE-RUNBOOK.md`

## 문서 준비 게이트

- [x] 감사 대상과 정본 우선순위 정의
- [x] 요구사항별 직접 증거·반증·누락 판정 기준 정의
- [x] 간접 evidence와 단순 파일 존재를 완료 증거에서 제외
- [x] 환경 의존 항목의 skip 및 의존 후속 skip 기록 방식 정의
- [x] 누락을 별도 구현 태스크로 전환하는 완료 조건 정의

## 사용자 결정

설계 문서대로 마지막 태스크까지 자율 진행한다. 저장소 밖 환경 때문에 실행 불가능한 항목은 이유와
의존성을 기록하고, 저장소 안에서 준비 가능한 구현·자동 검증은 끝까지 수행한다.

## 의사결정

- 감사 단위는 태스크 개수가 아니라 정본 요구사항·화면·operation·상태 전이·운영 gate다.
- test 이름이나 manifest mapping은 assertion과 실제 실행 환경을 확인한 경우에만 증거로 인정한다.
- 누락은 감사 문서에서 완료로 완화하지 않고 dependency DAG에 맞는 별도 구현 태스크로 등록한다.
- 외부 환경 skip은 구현 누락과 구분하며 준비 코드·negative test·runbook까지 완료된 경우에만 허용한다.

## 구현 순서

1. 정본 요구사항과 구현·test inventory를 기계적으로 추출한다.
2. 제품·API·화면·운영 축을 직접 증거와 대조한다.
3. 누락·부분·환경 skip과 의존 관계를 감사 결과에 기록한다.
4. 구현 가능한 누락을 후속 태스크로 등록한다.
5. 감사 검사와 문서 gate를 통과하고 완료한다.

## 작업 내역

- 2026-08-25: IMP-01~28 완료 선언에 대한 독립적인 전체 구현 감사를 등록했다.
- 2026-08-25: OpenAPI 108개 operation, Web API client 61개 method, Web 사용 47개 method,
  SCR-01~22와 RQ-01~20을 실제 route·component·test·운영 artifact와 대조했다.
- 2026-08-25: `PLAN-35`와 기계 검증 manifest를 작성하고 누락을 TASK-039~045로 분해했다.

## 이슈 및 해결

- 기존 TEST-08은 OpenAPI를 105개로 기록했지만 정본 OpenAPI에는 108개 operation이 있다. 숫자를
  수동 복제하지 않고 TASK-043에서 정본 집합 차이를 직접 검사하도록 해결한다.
- TEST-09 manifest는 Rust 통합 test를 연결하지만 웹 UI end-to-end 여정을 실행하지 않는다. 해당
  증거를 전체 인수 완료로 간주하지 않고 TASK-044의 실제 browser gate로 분리한다.

## 검증

- [x] PRD·W-01~09·IMP-01~28 전수 판정
- [x] OpenAPI operation·화면·상태 전이·운영 gate 직접 증거 판정
- [x] 누락·환경 skip·의존 skip 목록과 후속 태스크 추적
- [x] 감사 결과 자체의 자동 일관성 검사

## 결과

`PLAN-35`는 기존 백엔드·계약 기반을 보존하되 전체 제품 완료 선언을 철회하고, 직접 증거가 없는
범위를 TASK-039~045로 연결한다. 감사 manifest 검사를 root gate에 추가했으며 모든 누락이 정확히
하나 이상의 후속 태스크를 가진다.
