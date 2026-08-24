# 반응형 Visual Spec

- **문서 ID**: UX-12
- **상태**: 동결

## Breakpoint 의미

고정 기기명이 아니라 available inline size로 layout을 바꾼다. ADS responsive primitive와
token을 사용하며 임의 breakpoint 체계를 새로 만들지 않는다.

| 폭 상태 | 구조 | 주요 조작 |
|---|---|---|
| Compact | 단일 content stack | bottom action, full-screen panel |
| Medium | content + overlay navigation/panel | top app bar, drawer |
| Wide | tree + content + optional panel | resizable region, persistent context |

## Document

본문은 읽기 가능한 최대 line length를 유지하고 넓은 table·code는 자체 horizontal scroll을
가진다. selection toolbar는 viewport와 software keyboard를 피해 배치한다.

## Tree와 panel

compact에서 tree와 contextual panel은 동시에 열리지 않는다. deep link로 panel 진입 시
back action이 document로 복귀한다. wide의 panel width preference는 사용자별로 저장한다.

## Diff와 settings

wide는 side-by-side, compact는 unified Diff를 기본으로 한다. permission matrix와 table은
column priority, sticky label과 detail drawer로 동일 정보를 보존한다.

## Visual regression

한국어·영어, Light·Dark, compact·medium·wide, 200% zoom과 long content fixture를 snapshot
matrix로 검증한다.
