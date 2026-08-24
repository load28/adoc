# TASK-008: Rust·TypeScript 계약 생성 기반 구현

- **상태**: 완료
- **유형**: 구현
- **구현 패키지**: IMP-02
- **시작일**: 2026-08-25
- **완료일**: 2026-08-25
- **커밋**: —

## 목적

OpenAPI·AsyncAPI·JSON Schema 정본에서 Rust와 TypeScript transport 타입을 반복 가능하게
생성한다. 두 언어가 같은 fixture를 다르게 승인하는 drift를 CI에서 차단하고, 후속 API·Editor·
AI 구현이 수동 복제 타입을 만들지 않게 한다.

## 범위

- 포함: contract source fingerprint, OpenAPI·AsyncAPI schema normalization, Rust·TypeScript
  generated type, Draft 2020-12 runtime validator, positive·negative fixture 양언어 verdict 비교,
  generated diff와 CI gate
- 제외: API HTTP client·Axum handler, domain value object, reducer, DB model, event producer·consumer,
  OpenAPI·AsyncAPI 정본의 제품 의미 변경

## 산출물

- `packages/contracts`: TypeScript generated type·validator·fixture verdict
- `crates/contracts`: Rust generated type·validator·fixture verdict
- `tools/contract_codegen`: 정본을 읽어 두 언어 generated source와 manifest를 만드는 도구
- root contract generate·check·test 명령과 CI 연결
- 모든 정본 입력의 SHA-256과 generator version을 기록한 generated manifest

## 생성 계약

1. 정본은 `docs/design/api/openapi.yaml`, `asyncapi.yaml`, `docs/design/contracts/*.schema.json`뿐이다.
2. generator는 외부 `$ref`를 로컬 정본 안에서만 해석하고 네트워크 ref를 거부한다.
3. 생성 파일은 수동 편집하지 않으며 source 변경은 generate와 contract test를 함께 요구한다.
4. TypeScript는 OpenAPI operation map과 공통 schema type을 제공한다.
5. Rust는 transport 전용 `adoc-contracts` crate에서 같은 schema와 operation type을 제공한다.
6. 타입 변환이 표현하지 못하는 constraint는 양언어 Draft 2020-12 validator가 강제한다.

## 필수 설계 문서

- [x] PROD-09 `product/REQUIREMENTS-TRACEABILITY.md`
- [x] ARCH-03 `design/architecture/MODULE-ARCHITECTURE.md`
- [x] ARCH-05 `design/architecture/CROSS-CUTTING-CONTRACTS.md`
- [x] PLAN-01 `design/implementation/REPOSITORY-STRUCTURE.md`
- [x] PLAN-06 `design/implementation/MODULE-INTERFACE-CATALOG.md`
- [x] PLAN-08 `design/implementation/WORK-BREAKDOWN.md`
- [x] API-01~05 `design/api/`
- [x] CONTRACT-01~04 `design/contracts/`
- [x] TEST-01 `design/quality/TEST-STRATEGY.md`
- [x] TEST-07 `design/quality/FIXTURE-CATALOG.md`
- [x] TEST-08 `design/quality/CONTRACT-COVERAGE.md`
- [x] 도메인 상태 전이: N/A — serialized contract만 생성하고 domain 의미를 구현하지 않는다.
- [x] UX 흐름: N/A — 화면과 client 호출 구현은 후속 IMP가 소유한다.
- [x] 권한·동시성: N/A — type에 포함된 필드는 보존하지만 resolver·transaction은 구현하지 않는다.

## 문서 준비 게이트

- [x] 계약 정본과 생성물의 단방향 ownership이 정의됐다.
- [x] Rust generated type을 domain과 분리한 `crates/contracts` 경계가 정의됐다.
- [x] OpenAPI operation과 AsyncAPI header·payload를 양언어에서 생성할 범위가 정의됐다.
- [x] type과 runtime constraint의 책임이 분리됐다.
- [x] corpus의 양언어 동일 판정과 generated diff 완료 조건이 정의됐다.
- [x] 구현 결과를 바꿀 미해결 질문이 없다.

## 사용자 결정

없음. Rust·TypeScript 생성, OpenAPI·AsyncAPI·JSON Schema 정본과 Bun·asdf toolchain은 기존
결정에서 확정됐다. 생성 library는 정본을 바꾸지 않는 구현 도구로 선택한다.

## 의사결정

### 결정 1: 언어별 type generator와 공통 runtime validator를 분리한다

- **상황**: JSON Schema의 모든 constraint를 Rust·TypeScript type system으로 동일하게 표현할
  수 없다.
- **검토한 대안**: 수동 타입 / 하나의 언어를 정본으로 역생성 / schema 정본에서 언어별 생성과
  runtime validation 병행.
- **선택과 근거**: JSON Schema 정본에서 언어별 타입을 생성하고 acceptance는 runtime validator
  verdict로 판정한다. 타입 편의 때문에 schema constraint를 약화하지 않는다.

### 결정 2: OpenAPI·AsyncAPI를 공통 JSON Schema bundle로 정규화한다

- **상황**: Rust JSON Schema generator는 API 문서의 operation·message wrapper를 직접 알지 못한다.
- **검토한 대안**: 언어마다 API 문서를 별도 해석 / OpenAPI 3.0 전용 generator / operation·message를
  Draft 2020-12 bundle로 정규화.
- **선택과 근거**: OpenAPI 3.1 schema와 AsyncAPI schema를 이름 있는 `$defs`로 정규화한다. 같은
  bundle을 양언어 generator와 validator가 소비해 해석 drift를 줄인다.

### 결정 3: 생성물과 fingerprint를 저장소에 커밋한다

- **상황**: build 시점에만 생성하면 code review에서 계약 변화와 generator drift를 확인하기 어렵다.
- **검토한 대안**: build-time macro만 사용 / generated source 미커밋 / deterministic source와
  manifest 커밋.
- **선택과 근거**: 생성 결과와 source·tool version fingerprint를 커밋하고 CI에서 재생성 diff를
  검사한다.

## 구현 순서

1. `crates/contracts`, `tools/contract_codegen` 경계와 dependency rule을 추가한다.
2. OpenAPI·AsyncAPI·standalone schema를 공통 bundle로 정규화한다.
3. TypeScript와 Rust generated type·manifest를 생성한다.
4. Ajv 2020과 Rust `jsonschema` validator를 같은 fixture catalog에 연결한다.
5. 두 validator의 fixture verdict JSON을 exact 비교한다.
6. source fingerprint·operation count·event contract와 generated diff를 검사한다.
7. clean bootstrap에서 전체 CI gate를 재실행한다.

## 이슈 및 해결

### 이슈 1: 정적 타입 생성기가 실행 시 조건식을 표현하지 못함

- **증상**: Typify가 Event 계약의 Draft 2020-12 `if/then`에서 생성을 중단했다.
- **조사**: 정적 Rust type이 표현할 수 있는 keyword와 runtime validator 책임을 대조했다.
- **근본 원인**: 조건부 JSON Schema 제약은 Rust type system의 구조 타입으로 직접 변환할 수 없다.
- **구조적 해결**: 정본과 runtime bundle은 조건식을 그대로 보존하고, Rust type 생성용 입력에서만
  `if/then/else`와 `unevaluatedProperties`를 제거한다. 양언어 runtime validator가 원래 제약으로
  fixture를 판정하므로 type 편의를 위해 acceptance를 약화하지 않는다.

### 이슈 2: Cargo 라이선스 식의 표기 차이를 허용 목록이 오판함

- **증상**: 같은 이중 라이선스가 `Apache-2.0/MIT`와 `MIT/Apache-2.0` 순서에 따라 거부됐다.
- **조사**: 새 transitive dependency의 Cargo metadata license 식을 확인했다.
- **근본 원인**: 검사기가 SPDX `OR`만 해석하고 Cargo 생태계의 legacy `/` 이중 라이선스 표기를
  단일 식별자로 비교했다.
- **구조적 해결**: `OR`와 `/`를 동일한 선택 식으로 정규화한다. 새로 유입된 permissive Zlib는
  명시적 허용 항목으로 추가하고 그 밖의 미등록 라이선스는 계속 실패시킨다.

## 검증

- [x] OpenAPI 105 operation이 generated operation map에 모두 존재
- [x] AsyncAPI operation·header·payload type 생성
- [x] CONTRACT-01~04 generated Rust·TypeScript type compile
- [x] positive 4개·negative 4개 fixture 양언어 동일 판정
- [x] 외부 network `$ref` 거부와 local fragment resolution test
- [x] generated source 재실행 diff 0
- [x] source·tool fingerprint 검증
- [x] root `bun run check`와 clean bootstrap 통과
- [x] `git diff --check` 통과

## 작업 내역

- 2026-08-25: IMP-02 구현 태스크를 등록하고 계약·fixture·module 정본을 확인했다.
- 2026-08-25: OpenAPI·AsyncAPI·독립 JSON Schema를 343개 이름 있는 definition으로 정규화하고
  105개 HTTP operation과 2개 event message의 Rust·TypeScript type을 생성했다.
- 2026-08-25: source SHA-256·generator version manifest와 재생성 diff 검사를 연결했다.
- 2026-08-25: Ajv 2020과 Rust jsonschema가 8개 fixture에 내린 판정 JSON을 exact 비교하고,
  network ref 거부·local fragment 정규화 self-test를 연결했다.
- 2026-08-25: 전체 format·lint·typecheck·test·build·secret·license gate와 별도 임시 복제본의
  frozen Bun clean bootstrap을 통과했다.

## 결과

IMP-02 계약 생성 기반을 완료했다. 후속 API·event·Editor 구현은 수동 transport type을 만들지
않고 `@adoc/contracts`와 `adoc-contracts` 생성물을 사용한다.
