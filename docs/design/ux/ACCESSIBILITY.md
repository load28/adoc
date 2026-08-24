# 접근성 설계

- **문서 ID**: UX-10
- **상태**: 동결
- **기준**: WCAG 2.2 AA

## Keyboard와 focus

모든 interactive element는 logical tab order와 visible focus를 가진다. modal은 focus trap,
초기 focus와 trigger 복귀를 보장한다. editor에는 block navigation, selection 확장과 menu
대체 command를 제공한다.

## Semantic과 이름

heading hierarchy, landmark, list, table과 form label을 실제 semantic으로 표현한다. icon-only
button은 accessible name을 가진다. 상태·오류는 control과 programmatically 연결한다.

## 실시간 변화

save, lease, job, drag, mention과 validation 변화는 중요도에 맞는 live region을 사용한다.
streaming AI text를 매 token마다 읽지 않고 phase 또는 완성된 chunk로 묶는다.

## 시각

텍스트·UI contrast, reflow, zoom 200%, target size, reduced motion과 color-independent status를
검증한다. mobile landscape와 software keyboard가 primary action을 가리지 않아야 한다.

## Editor 특화

formatted text의 현재 mark 상태, table 좌표, code language, image alt·caption을 노출한다.
drag-only reorder를 금지하고 이동 menu를 제공한다. PDF export에도 heading·link·alt semantic을
가능한 범위에서 보존한다.

## 검증

axe 기반 자동 검사, keyboard-only scenario와 VoiceOver·NVDA 수동 검사를 release gate로 둔다.
