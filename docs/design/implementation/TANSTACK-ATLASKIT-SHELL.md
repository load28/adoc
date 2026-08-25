# TanStack Shell·Atlaskit 구현 계약

- **문서 ID**: PLAN-28
- **상태**: 구현 기준
- **구현 패키지**: IMP-22
- **정본**: UX-01~03·09~10·12~15, ADR-001·008, SEC-02

## 1. 책임과 경계

IMP-22는 모든 인증 화면과 Workspace 화면이 공유하는 SSR document, provider, route 구조,
navigation, locale·theme, API query 경계와 공통 상태를 소유한다. 기능 화면은 이 기반을 조합한다.

- Editor instance·operation buffer·recovery store는 IMP-23이 소유한다.
- Collaboration·Search·Inbox·Vocabulary 화면은 IMP-24가 소유한다.
- AI Inspector·Proposal Diff는 IMP-25가 소유한다.
- 설정·Trash·Audit·Public Viewer 본문은 IMP-26이 소유한다.
- `packages/ui-domain`은 domain UI와 브라우저 application primitive만 제공하며 Atlaskit component를
  다시 감싼 병행 design system을 만들지 않는다.

## 2. 패키지와 빌드 계약

React와 React DOM은 18.2.0을 유지한다. Atlaskit package는 npm의 공개 Apache-2.0 배포물과 React
18.2 peer 범위만 정확한 버전으로 lock한다. IMP-22가 직접 사용하는 최소 집합은 AppProvider,
CSS reset, tokens, primitives, navigation-system, button, icon, link, drawer, skeleton, spinner,
inline-message와 empty-state다. 후속 화면 package는 각 구현 태스크에서 추가한다.

Vite 8의 공식 `@rolldown/plugin-babel`에는 `@atlaskit/tokens/babel-plugin`만 두고 React transform은
`@vitejs/plugin-react`가 소유한다. application source는
raw color, 자체 token, Atlassian font·logo·brand asset, deprecated navigation package와 Atlaskit
deep import를 사용하지 않는다. 외부 package의 distributed CSS는 Vite가 처리한다.

`@atlaskit/navigation-system` 10.16.1 배포 manifest는 license 필드와 LICENSE 파일을 누락했지만
package가 가리키는 Atlassian 공식 frontend mirror의 최상위 LICENSE는 Apache-2.0이다. 일반 license
검사에서 이를 무조건 허용하지 않는다. `infra/licenses/js-exceptions.json`에 package·정확한 버전·
공식 repository·license 원문 URL을 고정하고 전용 gate가 설치 manifest와 모두 일치할 때만 generic
UNKNOWN 판정을 대체한다. 버전이나 repository가 바뀌면 exception은 자동 실패한다.

## 3. Route topology

route module은 아래 계층을 한 번 등록한다. 기능이 아직 구현되지 않은 leaf는 shell이 소유하지
않으며 후속 태스크가 같은 stable path에 screen을 연결한다.

```text
__root
├─ /                         -> session 결과에 따라 /login 또는 /workspaces로 replace
├─ /login                    -> anonymous auth layout
├─ /invites/$token           -> anonymous invitation layout
├─ /workspaces               -> authenticated collection layout
├─ /w/$workspaceSlug         -> membership-validated WorkspaceShell
│  ├─ /home
│  ├─ /docs/$documentId
│  ├─ /search
│  ├─ /inbox
│  ├─ /vocabulary
│  ├─ /trash
│  └─ /settings/$section
└─ /p/$publicToken           -> application provider만 공유하는 별도 public layout
```

`workspaceSlug`, `documentId`, invitation/public token은 path에서만 받는다. Document search는
`mode`, `panel`, `discussion`, `review`, `job`, `proposal`, `from`, `to`, `region`의 폐쇄형 parser를
쓴다. collection search도 UX-15의 enum·string·cursor만 보존한다. unknown key와 invalid enum은
loader 진입 전에 제거한 canonical search로 replace한다. token, CSRF, lease, idempotency key,
Draft content와 prompt는 URL에 넣지 않는다.

## 4. SSR bootstrap과 권한

SSR query 순서는 `GET /session → GET /workspaces → target Workspace`다. session cookie는 브라우저와
SSR same-origin request에만 전달하고 hydration payload에는 `SessionView`와 현재 Workspace의 표시용
최소 필드만 둔다. 인증 실패는 `/login`, Workspace membership 불일치는 `/workspaces`로 replace한다.
target permission을 확인하기 전 Workspace·Document 제목을 HTML에 출력하지 않는다.

서버 상태는 전역 store에 복제하지 않는다. query key는
`[workspaceId, resourceKind, resourceId, canonicalViewParameters]`이며 Workspace가 바뀌면 이전
Workspace의 restricted cache를 폐기한다. loader와 client reconciliation은 같은 API codec과
Problem parser를 사용한다. hydration 완료 전 command는 실행하지 않는다.

## 5. API client와 오류 경계

브라우저 API client는 generated OpenAPI `paths` 타입을 받아 같은 origin의 `/api/v1`만 호출한다.
credential은 `same-origin`으로 고정하고 redirect·external base URL override를 허용하지 않는다.
unsafe command는 readable CSRF cookie 값을 `X-CSRF-Token` header로 보내며 session token은 읽지
않는다. idempotency key는 command state machine이 생성하고 retry 동안 유지한다.

응답은 content type과 status를 먼저 확인한 뒤 generated response type으로 좁힌다. 오류는
`Problem { code, message, correlationId, fieldErrors?, meta? }`로 정규화한다. provider 원문,
response HTML과 stack은 UI에 표시하지 않는다. 공통 screen은 UX-03 우선순위로
denied/not-found, blocking error, loading, empty, ready 중 하나만 렌더링한다.

## 6. Theme와 locale

`ProductAppProvider`는 공개 `AppProvider`를 최상단에서 직접 사용한다. CSS reset은 root entry에서
한 번 import한다. theme preference는 `LIGHT|DARK|SYSTEM`, locale은 `ko|en`의 폐쇄형 값이다.

SSR은 preference 또는 anonymous default `SYSTEM`을 `data-color-mode`와 초기 theme bootstrap에
반영한다. hydration 전 실행되는 bootstrap은 same-origin 정적 script file이며 고정된 세 값만
해석하고 임의 문자열이나 HTML을 삽입하지 않는다. client는 `setGlobalTheme`로 같은 값을 적용하고 System이면
`prefers-color-scheme` 변경을 구독한다. locale은 `<html lang>`과 translator context에 동시에
적용한다.

번역 catalog는 `packages/i18n`이 소유한다. key는 의미 기반 literal union이고 한국어·영어가
동일 key 집합을 가져야 한다. 누락 key는 개발·test에서 실패하며 production에서 다른 언어의
문장을 조용히 섞지 않는다. 날짜는 UTC 정본 값을 사용자 locale·IANA timezone으로 표시한다.

## 7. Shell과 navigation

WorkspaceShell은 navigation-system layout과 primitives로 다음 landmark를 제공한다.

```text
header[global navigation]
nav[workspace navigation]
main[route content]
aside[context panel, route가 요구할 때만]
```

Wide는 persistent side navigation, Medium은 drawer, Compact는 단일 content와 drawer를 쓴다.
표현 전환은 ADS responsive primitive가 소유하며 route와 server data를 바꾸지 않는다. tree와
context panel은 Compact에서 동시에 열 수 없다. navigation item은 실제 link semantic과 현재
route의 `aria-current`를 유지한다. Jira DOM, Atlassian logo와 제품 전용 raw CSS layout을 복제하지
않는다.

## 8. 공통 상태와 접근성

- `RoutePending`: 최종 landmark 크기를 보존하는 Skeleton이며 300ms 이전 Spinner를 보이지 않는다.
- `RouteEmpty`: 권한이 있는 primary action 하나만 노출하는 EmptyState다.
- `RouteProblem`: stable code·locale message·correlation ID와 retry-safe action만 표시한다.
- `SkipLink`: 첫 focus에서 main landmark로 이동한다.
- route 이동은 main heading으로 focus를 옮기고 document title을 locale별로 갱신한다.
- icon-only action은 accessible name, drawer는 trigger focus 복귀, status 변화는 묶인 live region을
  가진다.

## 9. 실패와 복구

- API unavailable: shell의 이미 검증된 stale navigation을 유지할 수 있지만 새 Workspace 제목이나
  권한 대상은 표시하지 않는다.
- session expired: restricted cache를 폐기하고 현재 canonical internal path만 return target으로
  보존한 뒤 `/login`으로 이동한다.
- hydration mismatch: locale·theme·route bootstrap을 동일 serializer로 생성하며 mismatch를 CI
  failure로 취급한다.
- invalid route/search: 안전한 상위 route와 canonical search로 replace하며 raw 값을 message나 log에
  복제하지 않는다.
- Atlaskit SSR failure: component별 client-only 우회 대신 package upgrade를 차단하고 SSR contract를
  복구한다.

## 10. 검증 계약

1. route parser unit test: unknown·invalid·secret-shaped search 제거와 canonical round trip
2. i18n unit test: catalog key 동등성, locale fallback 거부, timezone formatting
3. API client test: same-origin, cookie credential, Problem normalization, unsafe CSRF header
4. SSR test: ko/en·Light/Dark/System HTML과 hydration 경고 없음
5. component test: landmarks, skip link, keyboard navigation, focus return, axe critical/serious 0
6. responsive test: Compact·Medium·Wide에서 route identity와 action 보존
7. dependency gate: React peer, public entry import, Apache-2.0 license, deprecated package 부재
8. root build와 Docker web image에서 동일 SSR artifact 실행

IMP-22는 위 계약을 자동 검증하고 IMP-23~26이 새 token·provider·route store를 만들지 않고 같은
shell primitive를 사용할 수 있을 때 완료한다.
