# Document Content Schema

- **문서 ID**: SPEC-05
- **상태**: 동결

## Root

```json
{"schemaVersion":1,"root":{"type":"doc","children":[]}}
```

모든 block node는 `id`, `type`, `attrs`, `children?`를 가진다. inline text는 text와 marks를
가진다. ID는 문서 안에서 unique하고 clone 시 새 ID를 받는다.

## Block kinds

paragraph, heading(level 1..6), quote, callout, bulletList, orderedList, taskList, listItem,
codeBlock(language), table, tableRow, tableCell, toggle, divider, image(assetId, alt, caption,
width), file(assetId, caption).

## Marks

bold, italic, underline, strike, code, link(href), highlight(token), textColor(token), subscript,
superscript. URL은 http/https/mailto allowlist와 normalized form을 사용한다.

## Validation

root depth, node count, text byte, table dimension과 attrs를 shared validator로 제한한다. File
node는 같은 Workspace READY asset만 Draft에 둘 수 있고 Publish 시 모든 asset READY를
재검증한다.

## Evolution

Tiptap extension별 schema fragment와 migration을 등록한다. unknown node를 조용히 drop하지
않고 unsupported-content error와 raw recovery export를 제공한다.
