# 화면별 Enterprise UX Story

- **문서 ID**: UX-19
- **상태**: 구현 기준
- **화면 범위**: UX-02의 SCR-01~22 전체
- **행동 정본**: UX-04~08·13·15~16
- **시각 정본**: UX-09·10·12·17~18

## 1. Story 작성 형식

각 Story는 Figma frame을 대신하는 구현 계약이다.

- **사용자와 의도**: 누가 왜 들어오는지 정의한다.
- **첫 viewport**: route 진입 직후 보여야 하는 위계와 primary action을 정의한다.
- **작업 서사**: 사용자의 인지 순서와 UI 반응을 순서대로 정의한다.
- **상태와 복구**: loading·empty·error·denied·conflict·commit 표현을 정의한다.
- **반응형 변환**: 정보 삭제 없이 구조가 바뀌는 방식을 정의한다.
- **접근성 계약**: landmark, focus, keyboard, name과 announcement를 정의한다.

공통 desktop frame은 56px global header, 264px workspace rail, 남은 폭의 route main과 선택적
360px context panel이다. route main의 일반 page는 최대 1440px와 32px gutter를 사용한다.
문서 canvas는 72ch, 설정 form은 640px, data table은 가용 폭을 사용한다.

## 2. 인증과 Workspace 시작

### ST-01 · SCR-01 Login

- **사용자와 의도**: 미인증 사용자가 제품 가치와 인증 방법을 신뢰하고 Google로 시작한다.
- **첫 viewport**: 왼쪽 brand panel에 `생각을 팀의 공식 지식으로` 가치 문장과 세 단계
  `Draft → Review → Publish`를 둔다. 오른쪽 440px auth card에 logo mark, H1 `Adoc에 로그인`,
  설명, Google button, 개인정보 안내를 둔다. primary action은 Google button 하나다.
- **작업 서사**: 사용자는 가치 확인 → 인증 범위 확인 → Google 선택 → provider 이동 순으로
  진행한다. `returnTo`는 UI에 노출하지 않고 callback 뒤 안전한 목적지로만 사용한다.
- **상태와 복구**: 시작은 안정된 card다. provider error는 button 위 inline alert로 표시하고
  다시 시도 action을 제공한다. 이미 인증됐으면 card를 잠깐 표시하지 않고 redirect한다.
- **반응형 변환**: 900px 미만에서 brand panel은 상단의 짧은 value block으로 축약하고 auth card를
  단일 column으로 둔다. 390px에서 page gutter 20px, card border·shadow를 제거한다.
- **접근성 계약**: `main` 하나, H1 하나, Google icon은 장식, button accessible name은 문장으로
  완결한다. error focus는 alert heading으로 이동하지 않고 button 다음 읽기 순서를 유지한다.

### ST-02 · SCR-02 Invitation

- **사용자와 의도**: 초대받은 사용자가 대상 Workspace와 계정을 확인한 뒤 가입한다.
- **첫 viewport**: centered 560px card에 eyebrow `Workspace 초대`, Workspace 이름, 초대 계정,
  역할 badge, 만료 정보와 primary `초대 수락`을 둔다.
- **작업 서사**: 초대 내용 확인 → 현재 계정 일치 여부 확인 → 수락 → Workspace home 이동이다.
  계정 불일치면 수락 button 대신 `다른 Google 계정으로 로그인`을 제공한다.
- **상태와 복구**: expired·consumed·unknown은 token 세부를 노출하지 않는 neutral unavailable
  state다. command failure는 card 안에서 입력과 token을 유지한다.
- **반응형 변환**: compact에서는 full-width surface와 sticky bottom primary action을 사용한다.
- **접근성 계약**: 초대 상태는 heading과 text로 표현하고 role badge color에 의존하지 않는다.

### ST-03 · SCR-03 Workspace 선택·생성

- **사용자와 의도**: 인증 사용자가 기존 Workspace로 들어가거나 새 Workspace를 만든다.
- **첫 viewport**: minimal global header, H1 `Workspace`, 설명, 우측 primary `새 Workspace`, 검색
  field와 Workspace list를 둔다. list item은 initials avatar, 이름, role, 최근 접근, 상태와
  chevron으로 구성한다.
- **작업 서사**: 기본은 선택이다. 생성은 Dialog에서 이름 입력 → slug preview → 생성으로
  분리한다. rename과 deletion은 row overflow의 관리 Dialog에 둔다.
- **상태와 복구**: empty는 설명과 primary action을 page center에 둔다. deletion scheduled는
  row를 제거하지 않고 countdown badge와 `삭제 예약 취소`를 제공한다.
- **반응형 변환**: wide 2-column list, compact 단일 list다. 관리 dialog는 compact full-height Sheet다.
- **접근성 계약**: Workspace 진입은 link, 관리 action은 button이다. row 전체 click과 내부 button의
  중첩 interactive를 금지한다.

## 3. Workspace와 Document 핵심 흐름

### ST-04 · SCR-04 Workspace Home

- **사용자와 의도**: Member가 최근 작업, 내 할 일과 문서 구조를 빠르게 파악한다.
- **첫 viewport**: PageHeader에 Workspace 이름과 primary `새 문서`, 그 아래 `내 작업` inbox
  summary, `최근 문서`, `시작할 곳` section을 둔다. 데이터가 없는 장식 KPI는 만들지 않는다.
- **작업 서사**: 처리할 항목 확인 → 최근 문서 재진입 → 필요하면 새 문서 생성 순이다.
  새 문서는 parent 위치와 제목을 받는 Dialog에서 생성한다.
- **상태와 복구**: tree는 shell에 이미 있으므로 home loading은 content section skeleton만 쓴다.
  문서가 없으면 새 문서 action 하나와 작성 흐름 설명을 표시한다.
- **반응형 변환**: summary와 recent list는 wide 2-column, compact 단일 stack이다.
- **접근성 계약**: section마다 H2가 있고 카드 자체가 아니라 문서 제목 link만 focusable하다.

### ST-05 · SCR-05 Published Document

- **사용자와 의도**: Viewer 이상 사용자가 최신 공식 내용을 읽고 Draft·협업·history로 이동한다.
- **첫 viewport**: breadcrumb, editable하지 않은 title, `발행됨 vN` badge, 발행자·시각, active Draft
  badge와 action cluster `편집`, `공유`, `더보기`를 둔다. 본문은 중앙 72ch canvas다.
- **작업 서사**: 상태 확인 → 본문 읽기 → 필요 시 right context tab에서 토론·history·reference 확인
  → 권한에 맞는 편집 또는 공유 action 진입이다.
- **상태와 복구**: no version은 빈 문서 설명과 권한별 Draft action을 제공한다. stale version은
  본문 유지 + header banner로 갱신한다. denied면 shell은 유지하고 제목·본문을 제거한다.
- **반응형 변환**: wide context panel 360px, medium overlay Sheet, compact full-height Sheet다. floating
  panel trigger는 bottom bar에 `정보` label과 함께 둔다.
- **접근성 계약**: `article`과 본문 heading hierarchy를 유지한다. version badge와 active Draft는
  text로 읽힌다. panel trigger는 `aria-expanded`·`aria-controls`를 가진다.

### ST-06 · SCR-06 Draft Editor

- **사용자와 의도**: Editor가 lease·저장·revision 상태를 이해하며 내용과 구조를 편집한다.
- **첫 viewport**: compact document header에 breadcrumb, title, `초안` badge, save state, collaborator
  lease와 primary `검토/발행`을 둔다. 그 아래 sticky formatting toolbar, 72ch editor canvas와
  선택적 context panel을 둔다.
- **작업 서사**: lease 획득 → canvas focus → formatting·block command → 250ms operation flush →
  server revision 확인 → 검토 또는 발행이다. find, import/export와 구조 command는 overflow와
  command palette에서 동일 command ID로 제공한다.
- **상태와 복구**: `저장 중`, `저장됨`, `오프라인·로컬 보관`, `충돌`, `읽기 전용`을 header의
  persistent status로 표시한다. 오류가 있어도 canvas를 제거하지 않는다. lease loss는 mutation을
  멈추고 recovery Sheet에 local operation 수와 복사·재시도 action을 둔다.
- **반응형 변환**: wide toolbar는 inline, compact는 핵심 formatting 5개와 `더보기`; block drag는
  move dialog로 대체한다. software keyboard 위에 selection toolbar가 겹치지 않는다.
- **접근성 계약**: toolbar roving focus, pressed mark state, shortcut hint, editor label, live save
  region을 제공한다. color와 icon만으로 save·lease를 구분하지 않는다.

### ST-07 · SCR-07 Discussion Panel

- **사용자와 의도**: Contributor가 문서 맥락 안에서 토론을 찾고 읽고 메시지를 보낸다.
- **첫 viewport**: panel header `토론`, open count, primary `새 토론`, close button을 둔다. list는
  title, status, message count, last activity로 구성한다. detail 선택 전 creation form을 노출하지 않는다.
- **작업 서사**: 목록 scan → detail open → topic·message 확인 → reply composer → committed message
  확인이다. title·topic 편집과 close/reopen은 detail overflow와 별도 Dialog로 둔다.
- **상태와 복구**: 전송 실패 draft는 composer에 남고 inline retry를 제공한다. closed detail은
  composer를 read-only status panel로 바꾼다. restricted reference는 제목을 노출하지 않는다.
- **반응형 변환**: wide panel 안 master-detail, compact는 list → detail의 내부 back navigation이다.
- **접근성 계약**: message는 semantic list·article, composer label, upload status live region을 쓴다.
  새 message announcement는 사용자 위치를 강제로 이동하지 않는다.

### ST-08 · SCR-08 Review Panel

- **사용자와 의도**: 지정 reviewer가 정확한 revision과 변경 내용을 확인하고 결정한다.
- **첫 viewport**: review status, requested revision, requester, policy requirement와 `Diff 보기`를
  먼저 둔다. 하단 decision surface에 `승인` primary와 `수정 요청` secondary를 둔다.
- **작업 서사**: 대상·revision 확인 → diff·discussion 검토 → 결정과 note 작성 → commit → 결과와
  Inbox 해결 확인이다.
- **상태와 복구**: invalidated review는 decision control을 제거하고 변경된 revision 설명과 새 검토
  link를 둔다. 제출 conflict는 작성 note를 유지한다.
- **반응형 변환**: wide diff side-by-side, compact unified diff와 sticky decision bar다.
- **접근성 계약**: diff 삽입·삭제를 color 외 prefix와 accessible label로 표현한다.

### ST-09 · SCR-09 History·Diff

- **사용자와 의도**: Viewer가 version 변화를 비교하고 Editor가 과거 상태로 새 Draft를 만든다.
- **첫 viewport**: version timeline과 compare controls를 둔다. 선택한 두 version summary가 diff보다
  먼저 보인다. restore는 overflow의 위험도 낮은 명시 command다.
- **작업 서사**: version scan → from/to 선택 → compare → change context 확인 → 필요 시 restore
  confirmation → 새 Draft 이동이다.
- **상태와 복구**: active Draft가 있으면 restore impact를 dialog에서 먼저 보여준다. compare 실패는
  선택을 보존한다.
- **반응형 변환**: timeline은 compact vertical list, diff는 unified다.
- **접근성 계약**: version 선택은 checkbox가 아니라 두 개의 명확한 select role 또는 radio group을
  사용한다. timeline 순서와 날짜가 text로 제공된다.

### ST-10 · SCR-10 References

- **사용자와 의도**: Viewer가 현재 문서를 가리키는 지식 연결과 source 상태를 확인한다.
- **첫 viewport**: backlink count, kind filter, reference list를 둔다. row는 source title, kind,
  location, version status와 excerpt를 가진다.
- **작업 서사**: filter → source preview → 권한 있는 target route 이동이다.
- **상태와 복구**: moved·ambiguous·orphaned·restricted를 서로 다른 text label과 설명으로 표시한다.
- **반응형 변환**: row metadata를 두 줄로 wrap하고 preview는 Sheet로 연다.
- **접근성 계약**: reference target은 link, status help는 disclosure이며 tooltip에만 두지 않는다.

## 4. 지식과 개인 작업

### ST-11 · SCR-11 AI Context·Job·Proposal

- **사용자와 의도**: Editor가 AI에 전달할 근거를 통제하고 결과를 검토해 선택 적용한다.
- **첫 viewport**: task type과 scope, included source summary, 사용량·provider health를 표시하고
  primary `실행`을 둔다. prompt field와 source list를 한 form 안에서 구분한다.
- **작업 서사**: task 선택 → context inspector에서 source 포함·제외 → 실행 → phase progress →
  proposal diff와 source 근거 확인 → dependency-closed operation 선택 → 적용 confirmation이다.
- **상태와 복구**: queued·running·validating·ready·failed·cancelled를 step indicator와 text로 표시한다.
  stream disconnect는 job을 유지하고 `상태 다시 확인 중`으로 전환한다. stale proposal은 적용을
  막고 rebase·재실행 action을 제공한다.
- **반응형 변환**: wide diff + source panel, compact unified diff 뒤 source disclosure다.
- **접근성 계약**: progress는 token stream이 아닌 phase 단위 live update다. source checkbox label에
  title·kind·authority가 포함된다.

### ST-12 · SCR-12 Search

- **사용자와 의도**: Member가 권한 범위 안의 공식 지식을 빠르게 찾는다.
- **첫 viewport**: H1 `검색`, large search field, result count·index status, filter chips와 result list를
  둔다. query가 없으면 recent query가 아니라 검색 범위와 예시를 설명한다.
- **작업 서사**: query 입력·submit → 결과 scan → kind/date filter → snippet의 matching region 확인 →
  Document+Region 이동이다.
- **상태와 복구**: no query, no result, index delayed, partial outage를 구분한다. stale index에서도
  permission scope를 완화하지 않는다.
- **반응형 변환**: filter bar는 compact Sheet, active filter는 search field 아래 chip으로 남긴다.
- **접근성 계약**: search landmark, result count live region, highlight에 `mark` semantic을 사용한다.

### ST-13 · SCR-13 Inbox

- **사용자와 의도**: Member가 자신에게 필요한 협업 행동을 우선순위대로 처리한다.
- **첫 viewport**: H1 `받은 작업`, unresolved count, status·kind filter, master-detail list를 둔다.
  row는 kind icon, actionable title, source, age, unread dot와 resolved state를 가진다.
- **작업 서사**: filter → item 선택 → exact target preview → 이동 또는 inline decision → resolved commit →
  다음 unresolved item focus다.
- **상태와 복구**: target unavailable은 item을 숨기지 않고 제한 상태와 resolve action을 제공한다.
  read와 resolved control을 분리한다.
- **반응형 변환**: compact는 list → detail route state, action은 sticky bottom bar다.
- **접근성 계약**: unread는 text alternative를 가진다. 자동 다음 item 이동 전에 결과를 announce한다.

### ST-14 · SCR-14 Vocabulary

- **사용자와 의도**: Member는 조직 용어를 찾고 Admin은 개념을 안전하게 관리한다.
- **첫 viewport**: search, active/deprecated filter, concept list와 detail panel을 둔다. primary `새 개념`은
  권한 있을 때만 보인다.
- **작업 서사**: term search → definition·alias·사용처 확인 → edit → conflict validation → save →
  affected reference summary 확인이다.
- **상태와 복구**: term collision은 field error와 충돌 concept link를 함께 제공한다. deprecated term은
  대체 canonical term을 먼저 표시한다.
- **반응형 변환**: compact detail은 full-height Sheet다.
- **접근성 계약**: term list는 listbox가 아니라 navigation list이며 edit form label과 error 연결을 유지한다.

## 5. 운영과 거버넌스

### ST-15 · SCR-15 Trash

- **사용자와 의도**: Manage 사용자가 복구 기한과 삭제 영향을 확인해 복원하거나 영구 삭제한다.
- **첫 viewport**: retention explanation, search, trashed list를 둔다. row는 title, original path,
  deleted by/time, remaining days와 `복원` action을 가진다.
- **작업 서사**: 대상 선택 → impact detail 확인 → 복원 또는 danger zone의 purge → reason 입력 → commit이다.
- **상태와 복구**: purging은 row를 유지하고 irreversible progress를 표시한다. 실패는 retry ledger 상태를
  설명한다.
- **반응형 변환**: table을 metadata card list로 바꾼다.
- **접근성 계약**: countdown은 날짜도 함께 제공하고 purge dialog initial focus는 heading에 둔다.

### ST-16 · SCR-16 Members

- **사용자와 의도**: Admin이 membership과 invitation 상태를 비교하며 관리한다.
- **첫 viewport**: settings nav, H1 `멤버`, member count, primary `멤버 초대`, search와 table을 둔다.
  column은 사람, email, Workspace role, 상태, 최근 변경, action이다.
- **작업 서사**: search → row detail/role update 또는 invite Dialog → impact 확인 → commit이다.
- **상태와 복구**: pending invitation은 같은 table의 별도 group이다. 마지막 Owner 제거처럼 금지된
  작업은 server reason을 inline에 표시한다.
- **반응형 변환**: compact row는 person summary + detail Sheet다.
- **접근성 계약**: table header·caption, menu button name에 사용자 이름을 포함한다.

### ST-17 · SCR-17 Groups

- **사용자와 의도**: Admin이 group 구성과 권한 영향을 이해하며 membership을 변경한다.
- **첫 viewport**: group list와 selected detail의 split view, primary `그룹 만들기`를 둔다.
- **작업 서사**: group 선택 → member list·grant count 확인 → member 변경 diff → save → revision commit이다.
- **상태와 복구**: stale revision은 local selection을 보존하고 latest diff를 제시한다. delete는 affected
  grant와 권한 하락 수를 dialog에서 보여준다.
- **반응형 변환**: compact list → detail Sheet다.
- **접근성 계약**: member picker는 APG combobox, selected member는 removable text chip이다.

### ST-18 · SCR-18 Permissions·Policy

- **사용자와 의도**: Manage 사용자가 explicit grant와 effective result를 혼동하지 않고 변경한다.
- **첫 viewport**: target Document picker, inheritance breadcrumb, effective permission summary와 explicit
  grant table을 둔다. primary `권한 추가`는 target 선택 뒤 활성화한다.
- **작업 서사**: target 선택 → subject 선택 → explicit/effective/source ancestor 비교 → 변경 입력 →
  descendant·reference·public link impact preview → expected revision commit이다.
- **상태와 복구**: stale policy는 before/local/latest 3단 비교를 보여준다. 자신 또는 subtree의 마지막
  manager 상실은 blocker alert로 표시한다.
- **반응형 변환**: matrix는 subject card와 detail Sheet로 바꾸며 정보는 보존한다.
- **접근성 계약**: access level은 select, capability는 checkbox group, inheritance는 text로 설명한다.

### ST-19 · SCR-19 Writing Settings

- **사용자와 의도**: Admin이 조직 writing rule과 enforcement를 이해하고 version으로 저장한다.
- **첫 viewport**: config version·last editor, rule groups, severity legend와 primary `변경사항 저장`을 둔다.
- **작업 서사**: rule 검색 → enabled·severity·override 편집 → validation summary → save → 새 version 확인이다.
- **상태와 복구**: unsaved change bar를 persistent하게 두고 conflict 시 local changes diff를 보존한다.
- **반응형 변환**: rule table을 accordion list로 바꾼다.
- **접근성 계약**: switch label에 rule 이름과 현재 enforcement를 포함하고 설명을 `aria-describedby`로 연결한다.

### ST-20 · SCR-20 AI Settings

- **사용자와 의도**: Admin이 provider health, concurrency, budget과 usage를 안전하게 운영한다.
- **첫 viewport**: health status, 현재 usage·limit, reset time, config form을 순서대로 둔다. credential 원문은
  절대 표시하지 않는다.
- **작업 서사**: health 확인 → limit 변경 → 영향 summary → save → health·usage 재조회다.
- **상태와 복구**: provider outage는 settings 전체를 막지 않고 health section만 degraded로 표시한다.
  quota violation은 허용 범위와 reset time을 field error에 포함한다.
- **반응형 변환**: summary tiles는 2열에서 단일 column으로 바뀐다.
- **접근성 계약**: usage는 숫자 text가 정본이고 progress bar에는 accessible value를 제공한다.

### ST-21 · SCR-21 Audit

- **사용자와 의도**: Admin이 누가 언제 무엇을 바꿨는지 좁혀 보고 before·after를 검토한다.
- **첫 viewport**: date·actor·action·target filter bar, result count와 event table을 둔다. row는 time,
  actor, action sentence, target과 detail trigger다.
- **작업 서사**: filter → row scan → detail Sheet에서 structured before·after와 correlation 확인 → 다음 page다.
- **상태와 복구**: empty filter result와 no audit history를 구분한다. restricted metadata는 masking reason을
  표시한다.
- **반응형 변환**: compact filter Sheet와 event card list다.
- **접근성 계약**: timestamp는 locale text와 machine-readable `time`, structured JSON은 key-value table로
  먼저 제공하고 raw code disclosure는 선택 사항이다.

## 6. 공개 읽기

### ST-22 · SCR-22 Public Viewer

- **사용자와 의도**: 익명 독자가 공유된 최신 발행 문서만 방해 없이 읽는다.
- **첫 viewport**: minimal brand mark, document title, version date와 72ch article만 둔다. Workspace name,
  user action, tree, search와 application preload는 없다.
- **작업 서사**: title·metadata 확인 → article 읽기 → 허용된 embedded link·asset 열기다.
- **상태와 복구**: invalid·revoked·expired·unpublished는 같은 neutral not-found 화면이다. 내부 reason과
  token 형태를 노출하지 않는다.
- **반응형 변환**: 720px 이상 32px gutter, compact 20px gutter와 같은 typography scale을 사용한다.
- **접근성 계약**: `main > article`, 문서 heading hierarchy, link purpose, image alt와 code scroll을 유지한다.

## 7. Story 완료 판정

각 SCR은 다음 evidence가 모두 있어야 구현 완료다.

1. route fixture의 ready·loading·empty·error·denied 또는 해당 가능한 상태 screenshot
2. wide 1440×1000과 compact 390×844에서 Story의 정보와 command 동등성
3. keyboard-only primary journey, visible focus, dialog focus return
4. axe WCAG 2.2 A·AA violation 0과 heading·landmark 수동 assertion
5. Light·Dark, ko·en, 200% zoom과 long-content overflow 확인
