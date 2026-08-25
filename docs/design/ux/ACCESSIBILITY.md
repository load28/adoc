# 접근성 설계

- **문서 ID**: UX-10
- **상태**: 동결
- **기준**: WCAG 2.2 AA

접근성은 UX-19 화면 Story의 필수 축이며 component library가 자동 보장한다고 가정하지 않는다.
shadcn/ui primitive의 semantic과 Radix interaction을 유지하고 제품 composition에서 accessible
name, description, focus order와 live announcement를 완성한다.

## Keyboard와 focus

모든 interactive element는 logical tab order와 visible focus를 가진다. modal은 focus trap,
초기 focus와 trigger 복귀를 보장한다. editor에는 block navigation, selection 확장과 menu
대체 command를 제공한다.

focus indicator는 최소 2 CSS px perimeter 면적과 인접·변화 대비 3:1을 목표로 `ring` token을
사용한다. `outline: none`은 같은 element에 동등한 ring이 있을 때만 허용한다. pointer target은
최소 24×24px, 제품 기본 interactive target은 32×32px, compact primary control은 44px 높이다.

## Semantic과 이름

heading hierarchy, landmark, list, table과 form label을 실제 semantic으로 표현한다. icon-only
button은 accessible name을 가진다. 상태·오류는 control과 programmatically 연결한다.

route마다 H1은 하나다. Workspace shell은 `header`, `nav`, `main`, 선택적 `aside`를 제공한다.
route 이동은 main 시작으로 focus를 관리하되 사용자가 입력 중인 UI를 임의로 이동시키지 않는다.
table은 실제 비교축이 있을 때만 사용하고 caption·column header를 제공한다.

## 실시간 변화

save, lease, job, drag, mention과 validation 변화는 중요도에 맞는 live region을 사용한다.
streaming AI text를 매 token마다 읽지 않고 phase 또는 완성된 chunk로 묶는다.

toast는 status surface를 대체하지 않는다. command commit은 polite, destructive failure와 session
loss는 assertive로 announce한다. list가 갱신돼도 focus를 body로 잃지 않는다.

## 시각

텍스트·UI contrast, reflow, zoom 200%, target size, reduced motion과 color-independent status를
검증한다. mobile landscape와 software keyboard가 primary action을 가리지 않아야 한다.

Light·Dark 각각에서 일반 text 4.5:1, large text 3:1, control boundary·state 3:1을 검증한다.
400% zoom에서도 320 CSS px viewport에 2차원 page scroll이 생기지 않는다. table·code·diff처럼
내용 의미상 필요한 region만 자체 horizontal scroll을 가진다.

## Composite keyboard contract

| Component | Keyboard 핵심 | Focus 계약 |
|---|---|---|
| Dialog·Sheet | Tab cycle, Escape close | initial focus, trigger return |
| Menu | Arrow navigation, Home/End, Escape | trigger 유지·복귀 |
| Tabs | Arrow tab 이동, Enter/Space activation | active tab과 panel 연결 |
| Document tree | Arrow expand·level 이동, Enter open | focus와 selection 구분 |
| Combobox | text edit, Arrow option, Escape close | input DOM focus 유지 |
| Editor toolbar | Arrow roving, Enter command | command 뒤 selection 복원 |

## Editor 특화

formatted text의 현재 mark 상태, table 좌표, code language, image alt·caption을 노출한다.
drag-only reorder를 금지하고 이동 menu를 제공한다. PDF export에도 heading·link·alt semantic을
가능한 범위에서 보존한다.

## 검증

axe 기반 자동 검사, keyboard-only scenario와 VoiceOver·NVDA 수동 검사를 release gate로 둔다.
자동화 evidence는 screen reader 읽기 품질을 대신하지 않는다. SCR-01~22마다 heading·landmark,
primary journey focus, error association과 responsive reflow 결과를 기록한다.
