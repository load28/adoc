# Web API·Screen Contract Completeness

- **문서 ID**: PLAN-39
- **상태**: 구현 기준
- **관련 태스크**: TASK-042

## 목적

OpenAPI operation, Web typed client command, UX-13의 SCR loader/action과 실제 runtime module을 서로
검증 가능한 닫힌 집합으로 만든다. endpoint가 존재하지만 호출할 수 없거나 화면 action이 문서에만
존재하는 상태를 CI에서 거부한다.

## 네 개의 정본 집합

1. `openapi.yaml`의 고유 `operationId`가 HTTP wire operation의 이름과 request/response 타입을 소유한다.
2. client manifest가 operation별 public client method와 사용 surface를 소유한다. 하나의 composite
   command가 여러 operation을 호출하면 각각 별도 행으로 선언한다.
3. screen manifest가 SCR-01~22별 loader, HTTP action, local action과 runtime module을 소유한다.
4. `ApiClient`와 screen module의 실제 public symbol이 실행 가능한 구현 증거를 소유한다.

OpenAPI에 있지만 브라우저가 직접 호출하지 않는 callback, SSE, public binary와 server-only operation도
누락으로 숨기지 않는다. `browser-client`, `browser-navigation`, `server-callback`, `stream`, `binary` 중
하나의 surface를 명시해 전체 합집합이 OpenAPI와 정확히 같게 한다.

## Client command 계약

브라우저의 mutation은 generated schema만 직접 호출하지 않는다. `ApiClient`가 CSRF, idempotency key,
expected revision, lease와 credential policy를 하나의 command로 묶는다. method 이름이 operation ID와
다를 때 manifest가 명시적으로 연결하며 추론하지 않는다. 같은 operation을 둘 이상의 public method가
소유하거나 존재하지 않는 method를 가리키면 실패한다.

브라우저가 URL 이동으로 시작하는 Google login, EventSource가 소유하는 stream, SSR/public file response,
OIDC callback은 일반 JSON client method를 강제하지 않는다. 대신 해당 runtime surface module과 symbol을
manifest에 기록해 orphan이 되지 않게 한다.

## Screen action 계약

각 SCR은 다음 세 action 집합을 가진다.

- `loaderOperations`: 화면이 열릴 때 필요한 HTTP operation. 선택된 resource가 없으면 실행되지 않는
  조건부 loader도 포함한다.
- `actionOperations`: 사용자가 시작해 server state를 바꾸거나 permission-checked resource를 여는
  operation. primary action뿐 아니라 화면이 제공하는 모든 secondary HTTP action을 포함한다.
- `localActions`: panel 전환, editor command, filter 입력, embedded link처럼 HTTP operation이 아닌 action.

모든 screen 행은 route 또는 reusable screen component의 module과 export를 지정한다. OpenAPI operation은
여러 screen에서 사용할 수 있지만 같은 SCR 안에서는 loader와 action에 중복될 수 없다. local action은
operation ID와 이름이 같아 HTTP 호출을 가리는 것을 금지한다.

## 검사와 재현성

`check-web-contract-coverage`는 YAML과 TypeScript source를 읽어 다음을 순서대로 검사한다.

1. OpenAPI operation ID의 누락·중복과 manifest의 missing·orphan·duplicate ownership
2. browser-client 행의 실제 `ApiClient` public method 존재와 declared operation ID marker
3. SCR-01~22의 연속성, action 중복, 미등록 operation, runtime module·export 존재
4. UX-13 표의 SCR ID·route·loader·primary operation이 manifest에서 사라지지 않았는지 확인

검사기는 네 가지 오류 corpus를 `--self-test`에서 실행한다. 생성물은 source manifest의 canonical 정렬과
digest를 사용하며 작업자 환경이나 실행 순서에 의존하지 않는다. root `contracts:check`가 이 검사를
항상 실행하므로 OpenAPI 변경은 client와 screen coverage를 같은 변경에서 갱신해야 한다.

## 완료 조건

- 109개 OpenAPI operation이 정확히 하나의 client/runtime surface에 분류된다.
- SCR-01~22의 loader·HTTP action·local action과 runtime module이 모두 유효하다.
- 누락·orphan·중복·존재하지 않는 symbol negative corpus가 모두 거부된다.
- format, lint, typecheck, unit, build와 실제 Compose integration이 통과한다.
