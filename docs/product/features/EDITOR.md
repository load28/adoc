# Editor 요구사항

- **문서 ID**: PROD-12
- **상태**: 동결

## Content 기능

- paragraph, heading, quote, callout, divider, toggle
- bold, italic, underline, strike, inline code, link, highlight, color, sub·superscript
- bullet, ordered, task와 nested list
- table row·column·header·move·sort, code block language·highlight·copy
- image·file upload, paste, drag, resize와 caption
- undo·redo, find·replace, slash command, Markdown shortcut와 keyboard navigation

## 구조적 계약

Tiptap Core·ProseMirror는 편집 engine이며 저장 정본은 versioned `DocumentContent` schema다.
각 Block은 stable ID를 가진다. Region과 Operation은 ProseMirror position을 영속 식별자로
사용하지 않는다. schema migration은 read-old/write-current 전략을 따른다.

## 입출력

Markdown·Plain Text import와 Markdown·Plain Text·PDF export를 제공한다. 외부 Rich Text
paste는 allowlist schema로 정규화하고 script·unsafe URL·unknown attribute를 제거한다.

## 기기와 접근성

모바일에서도 동일한 content type을 편집할 수 있다. drag, hover와 keyboard shortcut은
menu 기반 대체 조작을 제공한다. selection과 toolbar는 focus를 잃지 않고 screen reader에
상태를 알린다.

## UI 체계

toolbar, menu, dialog, form, token과 icon은 공개 `@atlaskit` component로 구성한다. Editor
canvas의 제품 고유 렌더링도 ADS token만 사용한다.
