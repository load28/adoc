# TASK-053: Editor 저장·미발행 문서 상태 결함 수정

- **상태**: 완료
- **유형**: 결함·설계·구현·품질
- **시작일**: 2026-08-26
- **완료일**: 2026-08-26
- **커밋**: —

## 목적

Draft 첫 변경 저장이 브라우저 런타임 오류로 중단되는 결함과 미발행 문서가 Workspace 부재로
표시되는 상태 의미 결함을 수정한다. Editor 저장과 Published 빈 상태가 정본 설계의 사용자
여정과 일치하도록 회귀 테스트로 고정한다.

## 범위

- 포함: Operation ID 생성 계약, 미발행 문서 전용 빈 상태와 관련 테스트
- 제외: 저장 API·도메인 상태 전이 변경, Publish 정책 변경, 문서 트리 기본 route 변경

## 필수 설계 문서

- [x] 관련 PRD: `docs/product/PRD.md`의 Draft·Publish 범위 유지
- [x] 관련 도메인 문서: `docs/domain/document-system.md`, `docs/domain/workspace-governance.md`의 불변식 유지
- [x] UX 흐름: UX-03, ST-05·06
- [x] 데이터 모델·상태 전이: PLAN-29의 Editor lifecycle·Operation buffer 유지
- [x] API·이벤트 계약: CONTRACT-02·API-02 변경 없음
- [x] 권한·보안: 기존 Workspace access와 권한별 Draft action 유지
- [x] 실패·복구·동시성: PLAN-29 5·6·9절
- [x] 테스트 전략: PLAN-29 10절, PLAN-37 9·10절

## 문서 준비 게이트

- [x] 구현 결과를 바꿀 미해결 질문이 없다.
- [x] 정상·예외·권한·동시성 흐름이 정의되어 있다.
- [x] 경계를 넘는 데이터 계약이 구체적으로 정의되어 있다.
- [x] 구현 단위와 테스트 완료 조건을 문서에서 추적할 수 있다.
- [x] 코드 작성 가능 여부와 근거를 기록했다.

코드 작성 가능. UX-03은 도메인별 empty-state 문구 경계를, PLAN-29는 독립 호출 가능한 ID
factory 계약을, PLAN-37은 미발행 Document 표현과 테스트 조건을 정의한다. API·도메인 계약은
변경하지 않는다.

## 사용자 결정

없음. 사용자가 두 결함의 수정을 요청했다.

## 의사결정

### 결정 1: UUID 생성기를 호출 가능한 함수 계약으로 고정한다

- **상황**: adapter의 기본 ID factory가 Web API method 참조를 분리해 호출하여 첫 변경의 Operation
  생성 단계에서 브라우저 런타임 오류를 만들 수 있다.
- **검토한 대안**: screen에서 factory를 매번 주입하면 호출 지점별 누락 위험이 남는다. adapter 내부의
  기본 factory를 호출 가능한 wrapper로 고정하면 모든 소비자가 같은 계약을 사용한다.
- **선택과 근거**: `() => crypto.randomUUID()`를 기본 factory로 사용하고 기본 factory 경로를 직접
  실행하는 테스트로 브라우저 호출 의미를 고정한다.

### 결정 2: 미발행 상태를 Workspace 빈 상태와 분리한다

- **상황**: PublishedVersion이 없는 문서는 유효한 Document지만 범용 `RouteEmpty`가 Workspace 부재
  문구를 표시한다.
- **검토한 대안**: 범용 empty state에 문구 prop을 추가할 수 있지만 서로 다른 도메인 상태를 계속 한
  component 이름 아래 섞는다. 문서 화면에서 의미가 명확한 전용 상태를 구성하면 오용 경계가 분명하다.
- **선택과 근거**: Published 화면이 문서 제목, 미발행 설명과 권한에 맞는 Draft action을 소유하고
  component test로 Workspace 부재 문구가 노출되지 않음을 검증한다.

## 작업 내역

- 2026-08-26: 코드·실행 서비스·설계 문서를 대조해 저장 오류의 default factory 호출 경계와
  미발행 문서의 범용 Workspace empty-state 재사용을 확인했다.
- 2026-08-26: TASK-053을 등록하고 필수 설계 문서와 결정 범위를 확정했다.
- 2026-08-26: UX-03·PLAN-29·PLAN-37에 도메인별 empty state, ID factory와 회귀 테스트 계약을
  기록하고 문서 준비 게이트를 통과했다.
- 2026-08-26: `editor-schema`에 receiver를 보존하는 공통 UUID factory를 추가하고 Operation 생성과
  문서 import의 기본 factory를 같은 경계로 통합했다.
- 2026-08-26: Published 화면의 범용 Workspace empty state를 문서 제목·미발행 설명·Draft action을
  가진 전용 상태로 교체하고 한국어·영어 catalog와 component test를 추가했다.
- 2026-08-26: 관련 단위 테스트·typecheck와 전체 `bun run check`를 통과한 뒤 Web 컨테이너를 새
  이미지로 교체하고 healthy 상태와 로그인 SSR 응답을 확인했다.

## 이슈 및 해결

- **증상**: Draft 저장 시 `EDITOR_RUNTIME_FAILED`가 표시되고 Operation 저장이 진행되지 않는다.
- **조사**: 오류가 API Problem 이전의 adapter 예외를 범용 코드로 변환하며, 기본 ID factory가
  `crypto.randomUUID` method를 receiver 없이 호출하는 구조임을 확인했다.
- **근본 원인**: Web API method와 호출 가능한 독립 factory를 같은 타입으로 취급했다.
- **구조적 해결**: receiver를 보존하는 factory wrapper를 adapter 경계에 둔다.
- **증상**: 미발행 문서를 클릭하면 Workspace가 없다는 안내가 표시된다.
- **조사**: PublishedVersion 부재 분기가 Workspace 전용 `RouteEmpty`를 재사용함을 확인했다.
- **근본 원인**: 데이터 부재라는 표현 형태만 공유하고 도메인 상태 의미를 분리하지 않았다.
- **구조적 해결**: 미발행 Document 상태를 Published 화면이 직접 소유한다.

## 검증

- [x] 문서 링크와 정본 경계 확인
- [x] PRD·도메인·상세 설계 간 모순 확인
- [x] 관련 자동화 검사
- [x] 관련 테스트

## 결과

Draft 첫 변경은 receiver-safe UUID factory를 통해 Operation을 만들며, 미발행 문서는 Workspace
부재나 준비 중 문구 없이 Document 상태로 표시된다. 기본 factory receiver 회귀 테스트와 미발행
화면 component test를 추가했다. 전체 `bun run check`와 로컬 Web 컨테이너 반영을 완료했다.
