# 반응형 Visual Spec

- **문서 ID**: UX-12
- **상태**: 동결

## Breakpoint 의미

고정 기기명이 아니라 available inline size로 layout을 바꾼다. Tailwind breakpoint와 container
query를 같은 의미 구간에 매핑하며 화면별 임의 breakpoint를 만들지 않는다.

| 폭 상태 | 구조 | 주요 조작 |
|---|---|---|
| Compact `<640px` | 단일 content stack | 20px gutter, bottom action, full-height Sheet |
| Medium `640~1023px` | content + overlay navigation/panel | 24px gutter, top app bar, Sheet |
| Wide `≥1024px` | 264px rail + content + optional 360px panel | 32px gutter, persistent context |

breakpoint 직전·직후에 command와 route state가 동일해야 한다. 200% zoom으로 실질 available width가
줄면 Compact 또는 Medium 구조로 자연스럽게 전환한다.

## Document

본문은 읽기 가능한 최대 line length를 유지하고 넓은 table·code는 자체 horizontal scroll을
가진다. selection toolbar는 viewport와 software keyboard를 피해 배치한다.

article은 최대 72ch, 일반 form은 640px, collection·table은 최대 1440px다. empty·error도 ready와
같은 width container를 사용한다. PageHeader action은 Medium에서 wrap하고 Compact에서 content
아래 또는 sticky bottom action으로 이동한다.

## Tree와 panel

compact에서 tree와 contextual panel은 동시에 열리지 않는다. deep link로 panel 진입 시
back action이 document로 복귀한다. wide의 panel width preference는 사용자별로 저장한다.

navigation rail은 Wide에서 264px다. collapse하면 icon-only rail을 유지하지 않고 content 확보가
필요한 편집 화면에서 완전히 닫으며 명시적 reopen button을 둔다. Compact Sheet는 100vw 이하,
Medium Sheet는 최대 360px다.

## Diff와 settings

wide는 side-by-side, compact는 unified Diff를 기본으로 한다. permission matrix와 table은
column priority, sticky label과 detail drawer로 동일 정보를 보존한다.

Table은 Compact에서 제목·status·가장 중요한 metadata를 card row에 남기고 나머지는 DetailSheet로
연다. horizontal scroll만 제공해 label과 row context를 잃게 하지 않는다. Filter는 Sheet로 옮겨도
active filter chip과 result count를 page에 남긴다.

## Auth·public·overlay

- Login은 900px 이상 split layout, 그 미만 single column이다.
- Public Viewer는 navigation breakpoint와 무관하게 72ch reading layout을 유지한다.
- Dialog 폭은 simple 420px, form 560px, impact·diff 760px이며 viewport에서 40px 여백을 보장한다.
- Compact의 복잡한 Dialog는 full-height Sheet로 바뀌고 primary action은 safe-area 위에 둔다.

## Overflow contract

한국어·영어 30% label expansion, 200% zoom, 320px reflow와 긴 unbroken ID를 검증한다. 사용자
content는 wrap 또는 region scroll을 사용하고 page 전체를 넓히지 않는다. action label을 임의로
ellipsis 처리하지 않으며 overflow menu로 command 전체를 보존한다.

## Visual regression

한국어·영어, Light·Dark, compact·medium·wide, 200% zoom과 long content fixture를 snapshot
matrix로 검증한다.
