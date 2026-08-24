# 상세 설계 준비 상태

## 1. 현재 결론

현재 저장소에는 제품 요구사항과 도메인 기준선만 있다. 구현 태스크가 요구하는 상세
설계는 아직 작성되지 않았으므로 애플리케이션 코드를 작성할 수 없다.

## 2. 기능별 필수 설계 묶음

각 행은 한 문서가 아니라 구현 전 함께 준비해야 하는 설계 묶음이다. 실제 작업을 시작할
때 먼저 태스크를 만들고, 문서 경계와 파일명을 그 태스크에서 확정한다.

| 기능 영역 | 함께 필요한 상세 설계 | 상태 |
|---|---|---|
| Workspace·Permission | 사용자 여정, 권한 precedence, resolver 계약, 인증·세션, 데이터 격리, 이동 영향, 보안 테스트 | 미작성 |
| Document·Editor | Content schema, Region anchor, Operation, Draft 저장, Edit Lease, Publish·충돌·복구, Import·Export, 에디터 UX·테스트 | 미작성 |
| Discussion·Review | Topic·Message 계약, Mention·Inbox, Review revision, PublishPolicy, 알림 전달, 실패·중복 이벤트 테스트 | 미작성 |
| Knowledge·AI·Operations | Reference·Vocabulary, 인덱스·Retrieval, Source provenance, AI Job·CLI 격리, Writing Rules, File·Audit, 운영·평가 | 미작성 |

기능 영역이 크면 여러 태스크로 나누되, 구현 단위 하나가 소비하는 모든 설계가 준비되기
전에는 해당 코드를 시작하지 않는다.

## 3. 상세 설계 문서의 최소 질문

### UX와 상태

- 사용자는 어디에서 시작하고 어떤 상태를 확인·변경하는가?
- 로딩, 비어 있음, 권한 없음, 충돌, 실패, 재시도와 복구를 어떻게 경험하는가?
- 접근성과 키보드 흐름은 무엇인가?

### 데이터와 경계 계약

- aggregate, identity, revision/version과 불변식은 어떻게 저장되는가?
- API command/query, event와 오류 타입은 무엇인가?
- 원자성, idempotency, ordering과 cache invalidation 계약은 무엇인가?

### 보안과 운영

- 인증·인가가 어느 경계에서 선행되는가?
- 민감한 내용이 로그, 검색 index, AI Context와 파일 URL로 새지 않는가?
- timeout, cancellation, retry, partial failure, backup, migration과 관측성은 무엇인가?

### 검증

- 각 도메인 불변식을 어떤 단위·통합·보안·복구 테스트로 증명하는가?
- 동시 요청과 stale revision을 어떻게 재현하는가?
- AI 품질은 고정 예시가 아니라 어떤 평가 자료와 기준으로 회귀 검증하는가?

## 4. 설계 완료 표시 규칙

문서가 존재한다는 이유로 상태를 완료로 바꾸지 않는다. 관련 태스크 문서에 다음을
기록한 뒤 이 표를 갱신한다.

1. 설계가 참조하는 PRD 요구사항과 도메인 불변식
2. 선택한 대안과 제외한 대안의 근거
3. 미해결 질문과 구현 차단 여부
4. 설계 검토 방법과 결과
5. 설계가 허용하는 구현 단위와 금지하는 우회

## 5. 기술 선택 순서

프레임워크, DB, 에디터 엔진, 검색 엔진, 파일 저장소와 Queue는 위 계약이 구체화된 뒤
선택한다. 익숙한 기술에 맞춰 Domain Operation, Permission Scope 또는 Version 불변식을
약화하지 않는다.
