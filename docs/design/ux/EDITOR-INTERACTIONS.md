# Editor 상호작용

- **문서 ID**: UX-05
- **상태**: 동결

## 진입과 lease

Editor mode 진입 시 lease를 요청한다. 성공하면 holder·expiry·revision을 표시한다. 실패하면
read-only Draft와 holder 정보를 보여주고 편집권 요청 action을 제공한다. 강제 회수는
Manage 권한과 명시적 확인을 요구한다.

## Selection과 command

text selection, current block, multi-block selection을 구분한다. toolbar와 slash command는
selection kind가 허용하는 action만 제공한다. command 실행 뒤 editor focus와 selection을
복원한다.

## Block 조작

drag handle은 move·duplicate·transform·delete menu와 동일한 command를 호출한다. 모바일은
long press와 move dialog를 사용한다. table은 cell selection과 text selection을 구분하고
row·column action을 context menu로 제공한다.

## 저장 상태

`저장 중 → 저장됨`은 server revision 수신 뒤 전환한다. network failure는 local buffer
시각과 마지막 server save 시각을 표시한다. tab 종료 전 unsynced buffer가 있으면 경고한다.

## AI action

selection menu에서 Rewrite·Review를 시작한다. 좁은 Rewrite는 적용 전 scope를 강조하고
적용 후 Undo toast를 제공한다. 큰 작업은 Context Inspector → job progress → Proposal Diff
순으로 연다.

## 접근성

모든 command에 accessible name과 shortcut hint를 제공한다. drag 결과와 collaborator
lease 변화는 live region으로 알린다. color는 상태의 유일한 신호가 아니다.
