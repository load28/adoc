# Editor Command·Keymap

- **문서 ID**: UX-16
- **상태**: 동결
- **Operation 정본**: [Document Operation Schema](../contracts/document-operation.schema.json)

## 공통 규칙

shortcut은 macOS에서 `Mod=⌘`, Windows/Linux에서 `Mod=Ctrl`이다. Browser·OS 기본 shortcut을
가로채지 않는다. 모든 command는 toolbar, menu 또는 dialog 경로도 제공한다. 조합 입력 중에는
text command를 실행하지 않으며 한국어 IME composition 종료 뒤 operation으로 변환한다.

| Command ID | Key | 허용 selection | 생성 Operation·동작 |
|---|---|---|---|
| editor.undo | `Mod+Z` | all | ack된 inverse operation batch |
| editor.redo | `Mod+Shift+Z` | all | undo batch 재적용 |
| text.bold | `Mod+B` | text range | `SET_MARKS ADD/REMOVE bold` |
| text.italic | `Mod+I` | text range | `SET_MARKS ADD/REMOVE italic` |
| text.underline | `Mod+U` | text range | `SET_MARKS ADD/REMOVE underline` |
| text.link | `Mod+K` | text range | validated link mark dialog |
| block.palette | `/` at empty prefix | caret | command palette, operation 없음 |
| block.insertAfter | `Mod+Enter` | block | `INSERT_BLOCK` paragraph |
| block.delete | menu | block/multi-block | ordered `DELETE_BLOCK` |
| block.duplicate | menu | block/multi-block | new IDs의 `INSERT_BLOCK` |
| block.moveUp | `Alt+Shift+↑` | block | `MOVE_BLOCK` |
| block.moveDown | `Alt+Shift+↓` | block | `MOVE_BLOCK` |
| block.transform | palette/menu | block | compatible block replace·attrs |
| list.indent | `Tab` | list item | parent·index `MOVE_BLOCK` |
| list.outdent | `Shift+Tab` | list item | parent·index `MOVE_BLOCK` |
| table.nextCell | `Tab` | table cell | selection 이동; 마지막 cell이면 row insert |
| table.previousCell | `Shift+Tab` | table cell | selection 이동 |
| table.rowAction | menu | table cell | row block operations |
| region.comment | `Mod+Alt+M` | text/block range | Discussion create dialog |
| ai.rewrite | `Mod+Alt+R` | non-empty text range | narrow AI task; result preview |
| ai.review | menu | text/block/document | Context Inspector → AI Job |
| reference.insert | `@`/paste/menu | caret/range | `ADD_REFERENCE`+display node |
| editor.saveNow | `Mod+S` | all | pending batch flush; browser save 억제 |
| editor.exit | `Escape` | no popup | pending state 확인 후 Published mode |

## Command gate

command availability는 `content schema capability ∩ selection capability ∩ effective permission ∩
lease state ∩ document state`다. UI에서 숨기는 것과 별개로 server가 Operation Schema, expected
revision, lease와 permission을 다시 검사한다. read-only에서 copy, selection, Source 열기와
Discussion 보기만 유지한다.

## Paste·drop

paste는 plain text, allowlisted rich text, internal URL, image/file 순으로 parser를 선택한다.
HTML은 script, style, event attr과 unknown node를 제거한 뒤 Content Schema로 변환한다. internal
Document URL은 권한 확인 후 Reference가 되고 실패하면 plain link로 남는다. file drop은 upload
placeholder를 만들지만 READY 전 publish/review를 차단한다.

## Undo 경계

typing·formatting·block operation은 500ms와 explicit command boundary로 undo group을 나눈다.
Publish, Review, Permission, Discussion과 File lifecycle은 editor undo 대상이 아니다. narrow AI
rewrite는 적용 operation의 inverse를 하나의 group으로 제공한다. Proposal 일부 적용은 선택한
dependency-closed operation 집합을 하나의 group으로 만든다.
