# TASK-016: Content·Operation reducer 구현

- **상태**: 완료
- **유형**: 구현
- **구현 패키지**: IMP-09
- **시작일**: 2026-08-25
- **완료일**: 2026-08-25
- **커밋**: —

## 목적

Document Content와 Region을 안정적으로 식별하고 schema-valid DocumentOperation batch를 원자 적용하는
순수 reducer를 구현한다. Draft·AI Proposal·Diff·Undo가 동일한 mutation 의미와 precondition을
재사용할 수 있는 기술 독립 경계를 고정한다.

## 범위

- 포함: Content·Operation typed model과 validator, Region resolver, batch dependency·precondition 검증,
  deterministic reducer, inverse Operation, canonical fixture와 property test, Rust·TypeScript 계약 정합성
- 제외: Draft PostgreSQL 저장·Lease·HTTP(IMP-12), Reference 정본(IMP-14), File 생명주기(IMP-15),
  AI Proposal(IMP-21), Tiptap Editor 화면(IMP-23)

## 필수 설계 문서

- [x] 제품·도메인: `product/features/EDITOR.md`, `domain/document-system.md`
- [x] 계약: `design/contracts/document-content.schema.json`,
  `design/contracts/document-operation.schema.json`
- [x] 상세 규칙: `design/specs/document/REGION-OPERATION-DIFF.md`,
  `design/specs/ALGORITHM-CATALOG.md`, `design/specs/STATE-TRANSITION-CATALOG.md`
- [x] 경계·품질: `design/implementation/MODULE-INTERFACE-CATALOG.md`,
  `design/quality/CONTRACT-COVERAGE.md`, `design/quality/TEST-STRATEGY.md`
- [x] 구현 기준: `design/implementation/CONTENT-OPERATION-REDUCER.md`

## 문서 준비 게이트

- [x] 구현 결과를 바꿀 미해결 질문이 없다.
- [x] Content node·Region·Operation·precondition·dependency 불변식이 타입 수준으로 정의됐다.
- [x] batch atomicity, inverse·Undo, limit·오류 분류와 deterministic serialization이 정의됐다.
- [x] IMP-09와 IMP-12·14·15·21·23의 책임 경계를 추적할 수 있다.
- [x] 정상·경계·invalid·property fixture가 설계를 검증할 수 있다.

## 사용자 결정

사용자는 전체 제품 구현과 권장안의 자율 적용을 확정했다.

## 의사결정

### 결정 1: Browser와 server의 text offset을 UTF-16으로 통일한다

- **상황**: Rust Unicode scalar와 ProseMirror JavaScript offset을 섞으면 emoji 뒤 Range가 다른 text를 가리킨다.
- **검토한 대안**: UTF-8 byte / Unicode scalar / UTF-16 code unit.
- **선택과 근거**: Browser selection과 변환 없는 UTF-16을 정본으로 삼고 surrogate 내부 offset을 거부한다.

### 결정 2: Operation은 손실 없는 inverse에 필요한 payload를 스스로 가진다

- **상황**: plain text replacement와 referenceId 없는 add는 mark·Reference inverse를 재구성할 수 없다.
- **검토한 대안**: Undo snapshot만 저장 / adapter별 보완 / rich inline·Reference snapshot 계약.
- **선택과 근거**: REPLACE_TEXT는 inline content를 받고 양쪽 Reference 연산은 ID·source·target을 가진다.
  SET_BLOCK_ATTRS도 SET과 REMOVE를 구분해 null과 부재를 혼동하지 않는다.

### 결정 3: 독립 Operation 순서도 UUID로 결정한다

- **상황**: dependency가 없는 batch를 요청 배열 순서로 적용하면 client·AI 생성 순서에 따라 결과가 달라진다.
- **검토한 대안**: 입력 순서 / kind 우선순위 / dependency DAG와 UUID byte tie-break.
- **선택과 근거**: Kahn sort의 ready 집합을 UUID byte로 정렬해 Rust·TypeScript가 같은 결과와 inverse를 만든다.

## 구현 순서

1. 기존 Content·Operation JSON Schema와 Region·Diff 알고리즘의 공백·충돌을 감사한다.
2. IMP-09 상세 구현 계약과 필요한 정본 문서를 먼저 갱신한다.
3. 순수 domain model·validator·reducer·inverse를 구현한다.
4. generated Rust·TypeScript contract와 canonical fixture를 연결한다.
5. unit·property·negative corpus와 전체 root gate를 실행한다.
6. 완료 기록 후 commit·push하고 다음 구현 패키지로 진행한다.

## 작업 내역

- 2026-08-25: IMP-09 태스크를 등록하고 선행 설계 문서 집합을 식별했다.
- 2026-08-25: PLAN-15에 Content semantic limit, UTF-16 Region, deterministic DAG, 9개 Operation 의미,
  Reference effect, inverse dry-run과 Rust·TypeScript 동일 판정 gate를 고정했다.
- 2026-08-25: DOCUMENT Region, rich inline replacement, reversible Reference·attribute patch를 canonical
  Operation Schema에 반영하고 문서 준비 게이트를 통과했다.
- 2026-08-25: 목록·표의 stable 구조 node를 별도 예외 없이 삽입·이동·교체할 수 있도록 Operation
  payload를 Block·listItem·tableRow·tableCell 공통 contentNode로 일반화했다.
- 2026-08-25: Rust document domain과 Browser용 editor-schema에 Content normalization·semantic validator,
  5개 Region, 9개 Operation, DAG 정렬, 원자 적용, 결정적 inverse와 Reference effect를 구현했다.
- 2026-08-25: exact quote·context 기반 Region 재정착과 UTF-16 경계를 구현하고 동일한 이동 fixture로 검증했다.
- 2026-08-25: Rust·TypeScript 공통 fixture에 Content fingerprint, 적용 순서와 UUIDv5 inverse ID를 고정했다.
- 2026-08-25: 중복 ID·위험 URL·table grid negative test와 전체 저장소 gate를 통과했다.

## 이슈 및 해결

- **증상**: 기존 Operation 계약은 DOCUMENT Region, rich inline replacement, Reference inverse 식별자와
  attribute 부재 표현이 없어 일부 정상 역연산을 표현할 수 없었다.
- **근본 원인**: mutation 입력이 apply에 필요한 값만 담고 손실 없는 inverse 계약을 소유하지 않았다.
- **구조적 해결**: Region 공통 union과 self-contained Operation payload를 정본 Schema에 반영하고 생성 계약,
  Rust·TypeScript reducer와 공통 fixture가 같은 타입을 사용하게 했다.
- **증상**: Block 전용 structural payload로는 listItem·tableRow·tableCell의 일반 삽입과 교체가 불가능했다.
- **근본 원인**: 편집 가능한 tree node와 최상위 Block을 같은 개념으로 제한했다.
- **구조적 해결**: 구조 Operation payload를 허용 parent 계약으로 검증되는 공통 contentNode로 일반화했다.

## 검증

- [x] Content·Operation schema positive·negative corpus
- [x] Region resolve·batch dependency·precondition matrix
- [x] reducer atomicity·determinism·inverse round trip property
- [x] Rust·TypeScript generated contract 정합성
- [x] 전체 root gate

## 결과

Content·Region·Operation의 단일 mutation 의미를 Rust와 TypeScript에 구현했다. 두 구현은 동일한
canonical fixture, fingerprint, Operation 순서와 inverse ID를 생성하며 실패 시 부분 결과를 남기지 않는다.
