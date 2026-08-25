# Enterprise Experience Principles

- **문서 ID**: UX-18
- **상태**: 구현 기준
- **근거**: [조사와 설계 근거](ENTERPRISE-UX-RESEARCH.md), PROD-02, UX-01~17

## EP-01. 문맥이 행동보다 먼저 보인다

모든 authenticated page는 eyebrow 또는 breadcrumb, 하나의 H1, 상태 summary, 설명, action
cluster 순서를 사용한다. 사용자는 action label을 읽기 전에 Workspace·Document·mode와 영향
대상을 알 수 있어야 한다. global header에는 제품·Workspace·검색·계정만 두고 page action을
넣지 않는다.

**검증**: 첫 viewport에 현재 위치, 현재 상태와 primary action이 각각 한 번만 존재한다.

## EP-02. 한 화면은 하나의 primary outcome을 가진다

primary button은 page 또는 열린 dialog 안에 하나만 둔다. 같은 수준의 보조 action은 outline,
secondary 또는 menu로 낮춘다. 위험 action은 destructive section이나 confirmation dialog로
분리한다. 읽기 화면에서는 링크 이동을 button처럼 과장하지 않는다.

**검증**: 동일 container에서 primary emphasis가 둘 이상이면 의사결정 근거가 있어야 한다.

## EP-03. 안정된 frame 안에서 상태가 바뀐다

AppShell, PageHeader, section 위치와 주요 control 폭은 loading·empty·error·ready에서 유지한다.
목록 skeleton은 예상 row 수와 높이를 보존한다. stale content는 제거하지 않고 status banner와
재검증 action을 같은 위치에 둔다.

**검증**: 상태 전환 snapshot에서 heading과 primary action의 큰 layout shift가 없다.

## EP-04. 작업 밀도와 읽기 밀도를 분리한다

문서 본문은 72ch, 16px 이상과 넉넉한 line height를 사용한다. navigation·table·toolbar는
14px, 32~36px control과 40~44px row를 사용한다. 설명 text를 12px로 축소하지 않고 secondary
foreground와 13~14px를 사용한다.

**검증**: reading surface와 control surface가 같은 density token을 무차별 공유하지 않는다.

## EP-05. 노출 수준은 빈도가 아니라 판단 필요성으로 정한다

다음 결정을 내리는 데 필요한 정보는 inline에 둔다. 드물어도 권한 하락·복구 기한·revision
conflict·source 부족처럼 판단을 바꾸는 정보는 숨기지 않는다. raw ID, metadata와 고급 filter는
detail disclosure에 둔다.

**검증**: confirmation 없이 실행할 수 없는 command의 영향이 trigger 전에 요약된다.

## EP-06. 상태는 이름·원인·다음 단계로 표현한다

badge는 상태 이름만 표시한다. banner 또는 inline message가 원인과 데이터 영향을 설명한다.
가능한 recovery action은 바로 옆에 둔다. success는 server commit 뒤 resource surface에 남고,
toast는 보조 announcement로만 사용한다.

**검증**: color를 제거해도 상태와 가능한 행동을 구분할 수 있다.

## EP-07. 위험 작업은 정상 작업과 공간적으로 분리한다

delete, revoke, permission loss, lease takeover와 overwrite는 page의 `Danger zone` 또는 destructive
dialog에서만 실행한다. dialog는 대상, 영향, 복구 가능성, 실행 중 상태와 실패를 자체 소유한다.

**검증**: destructive command가 overflow menu 밖에서 반복 노출되지 않고 focus return이 보장된다.

## EP-08. 모든 기기에서 같은 command에 도달한다

wide rail·context panel은 compact에서 Sheet로 전환한다. drag action에는 Move dialog, hover action에는
overflow menu, side-by-side diff에는 unified diff가 있다. presentation은 달라도 route state,
command ID와 결과는 같다.

**검증**: compact journey에서 desktop 전용 command가 0개다.

## EP-09. 키보드와 보조기술 흐름을 화면 구조로 만든다

skip link → global header → side navigation → page heading → primary task → contextual panel 순서로
landmark와 focus가 이동한다. dialog는 heading에 설명을 연결하고 initial focus와 trigger return을
보장한다. 동적 상태는 묶어서 announce한다.

**검증**: WCAG 2.2 AA, APG pattern, 24px target minimum과 명시된 keyboard scenario를 통과한다.

## EP-10. UI는 제품 불변식을 설명하고 우회하지 않는다

Permission, Draft revision, Published Version, Review invalidation, AI source와 deletion retention을
UI 별도 boolean으로 단순화하지 않는다. server가 제공한 상태를 같은 용어와 구조로 표시한다.

**검증**: 화면 state와 action gate가 domain·API contract에 직접 추적된다.

## EP-11. 장식보다 의미 있는 대비를 쓴다

neutral surface를 기본으로 하고 brand color는 primary action·selection·focus에 제한한다.
shadow는 overlay와 floating toolbar의 계층에만 사용한다. section 구분은 spacing, border와 subtle
surface 순으로 표현한다. gradient, glass effect와 과도한 radius를 사용하지 않는다.

**검증**: 모든 color와 elevation이 semantic token 또는 component variant에 연결된다.

## EP-12. 일관성은 token과 composition으로 강제한다

PageHeader, SectionHeader, DataList, StatusBanner, EmptyState, FilterBar, DetailSheet와 ConfirmDialog를
반복 composition으로 사용한다. 화면별 CSS selector로 component 내부를 수정하지 않는다.

**검증**: 동일 역할의 두 요소가 다른 높이·radius·state vocabulary를 가지지 않는다.
