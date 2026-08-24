# 제품 요구사항 — 팀 문서 시스템

- **상태**: 기준선
- **버전**: 0.1
- **작성일**: 2026-08-24
- **근거**: 제품 대화 `팀 문서 시스템 만들기`에서 확정된 요구사항

## 1. 제품 정의

### 1.1 문제

팀의 지식은 문서, 대화, 개인의 메모와 외부 자료에 흩어진다. 문서는 한 사람이 쓴
결과물로 남기 쉽고, 토론에서 생긴 관점과 근거는 공식 문서로 수렴하지 않는다. AI
문서 도구는 빠르게 문장을 만들 수 있지만 조직의 확정된 사실, 용어와 권한을 놓치면
신뢰할 수 없는 내용을 더 빠르게 만든다.

### 1.2 제품 비전

여러 사람이 AI의 도움을 받아 생각하고 토론하며, 그 결과를 정확하고 읽기 쉬운 하나의
문서로 지속적으로 발전시키는 웹 기반 협업 문서 시스템을 만든다.

제품의 핵심 순환은 다음과 같다.

```text
정리되지 않은 생각
  → AI와 Draft 작성
  → 사람의 직접 편집
  ↔ Discussion과 Reference
  → AI Writing Review와 토론 반영
  → 사람의 Review와 판단
  → Publish
  → 불변 Published Version
  → Search·Reference·Vocabulary를 통한 조직 지식
  → 다음 Draft와 Discussion의 Context
```

### 1.3 성공의 의미

- 여러 사람의 생각이 별도 산출물로 흩어지지 않고 한 문서의 변화로 수렴한다.
- 독자는 결정된 사실, 미확정 내용과 근거를 구분해 이해할 수 있다.
- AI가 사용한 조직 지식과 출처를 사람이 확인할 수 있다.
- 공식 문서가 어떤 토론과 검토를 거쳐 바뀌었는지 추적할 수 있다.
- AI를 쓰지 않아도 현대적인 업무 문서를 충분히 작성할 수 있다.

## 2. 제품 원칙

### P-01. 사람의 판단이 최종 권한이다

사람은 AI에게 작업을 요청하고 결과를 수락, 거절, 재요청하거나 직접 수정한다. AI는
문서를 직접 Publish하거나 Discussion을 종료하거나 애플리케이션 데이터를 직접
변경하지 않는다.

### P-02. 문서 품질은 정확성과 인지적 가독성으로 판단한다

AI는 자연스러운 문장만 만들지 않는다. 근거, 모순, 확실성, 정보 순서, 문장 구조,
독자가 동시에 기억해야 하는 정보와 팀 용어를 함께 검토한다. 근거 없는 조직 사실을
만들지 않는다.

### P-03. 공식 지식과 작업 상태를 분리한다

Draft는 팀이 다음 공식 문서를 만드는 작업 공간이다. Publish할 때만 새 공식 Version이
생긴다. Published Version은 불변이다.

### P-04. 하나의 개념을 하나의 공통 모델로 해결한다

Permission, Reference, Retrieval, Document Operation처럼 여러 기능이 공유하는 개념은
기능마다 다시 구현하지 않는다. 같은 정책과 Resolver를 모든 소비자가 사용한다.

### P-05. 기술이 제품 요구사항을 결정하지 않는다

Document는 Git, Markdown 또는 특정 에디터 포맷에 종속되지 않는다. 제품·도메인·상세
설계 순으로 결정하고 기술 스택은 계약을 구현하는 수단으로 선택한다.

## 3. 사용자와 핵심 여정

### 3.1 사용자 역할

- **Member**: Workspace에 참여하고 부여받은 문서 권한 안에서 읽고 협업한다.
- **Admin**: 멤버십, Workspace 설정과 최상위 거버넌스를 관리한다.
- **Viewer**: Published 문서를 읽는다.
- **Contributor**: Draft를 읽고 Discussion에 참여한다.
- **Editor**: Draft를 만들고 편집하며 정책이 허용하면 Publish한다.
- **Reviewer**: 특정 Draft revision의 Publish 적합성을 검토한다.

Workspace 역할과 문서 접근 권한은 별개의 축이다. 한 사용자는 여러 Workspace에 속할
수 있으며 Workspace 간 데이터는 섞이지 않는다.

### 3.2 생각에서 공식 문서까지

1. 사용자가 메모, 키워드와 불완전한 문장을 입력한다.
2. AI가 의도와 확실성 수준을 보존해 적절한 정보 구조의 Draft를 만든다.
3. 사용자가 직접 편집하거나 선택 영역과 문서 전체에 AI 작업을 요청한다.
4. 권한 있는 사용자가 Draft를 읽고 문서 단위 Discussion에 참여한다.
5. 사용자가 AI에게 Topics, Messages와 References를 문서에 반영하도록 요청한다.
6. Review 정책을 충족하고 기준 Published Version과의 충돌을 해결한다.
7. Publish 성공 시 불변의 새 Version이 만들어진다.

### 3.3 조직 지식 질문

1. 사용자가 자연어로 질문하거나 의미 기반 검색을 한다.
2. 시스템이 사용자의 Permission Scope를 먼저 계산한다.
3. 허용된 문서, 영역, Discussion과 Vocabulary만 같은 Retrieval 계층에서 찾는다.
4. AI가 근거가 있는 답만 만들고 각 Source를 정확한 위치와 연결한다.
5. 근거가 없거나 충돌하면 조직의 사실로 단정하지 않고 부족·충돌 상태를 알린다.

## 4. 기능 요구사항

### 4.1 Workspace와 문서 트리

- 모든 문서, Discussion, Vocabulary, 파일, Inbox, 권한과 설정은 하나의 Workspace에
  속해야 한다.
- 문서는 자유로운 단일 트리에서 탐색한다. 문서 자체가 자식 문서를 가질 수 있다.
- 트리는 탐색, 정보 위계와 권한 상속만 담당한다.
- 문서 간 의미 관계는 트리가 아니라 Reference Graph가 담당한다.
- 사용자는 문서 생성, 자식 생성, 이름 변경, 이동, 순서 변경, 접기·펼치기,
  휴지통 이동과 복구, 링크 복사를 할 수 있어야 한다.
- 문서 이동 전후에 Effective Permission 변화를 계산하고 영향을 확인해야 한다.

### 4.2 Draft, Publish와 Version

- 하나의 Document는 지속되는 identity이며 내용 자체와 분리된다.
- 한 Document에는 동시에 하나의 공유 Draft만 존재한다.
- Draft는 하나의 Published Version을 기준으로 시작하고 revision을 가진다.
- 권한 있는 사용자는 Draft를 읽고 Discussion할 수 있다.
- 본문 편집은 한 번에 한 사용자만 수행한다. 실시간 공동 타이핑은 제공하지 않는다.
- Draft는 자동 저장되고 네트워크·브라우저 장애에서 최근 입력을 복구할 수 있어야 한다.
- Publish 시 Draft base와 현재 Published Version이 다르면 충돌을 해결해야 한다.
- AI는 3-way merge 제안을 만들 수 있지만 최종 병합은 사람이 승인한다.
- Publish 성공 시에만 새 불변 Published Version을 생성한다.
- 과거 Version 복원은 과거를 수정하지 않고 그 내용으로 새 Draft를 만든다.
- Version에는 publisher, 시간, 변경 요약, 관련 Discussion과 Review 맥락을 연결한다.

### 4.3 문서 에디터

AI 기능을 쓰지 않아도 일반 업무 문서를 작성하는 데 부족함이 없는 확장 가능한 블록
에디터를 제공한다.

- 텍스트: 문단, 다단계 제목, 굵게, 기울임, 밑줄, 취소선, 인라인 코드, 링크,
  하이라이트, 색상, 위·아래 첨자, 서식 제거, 실행 취소·다시 실행
- 블록: 인용, 콜아웃, 글머리·번호·체크·중첩 목록, 코드, 표, 구분선, 토글,
  이미지, 파일
- 조작: 생성, 선택, 다중 선택, 삭제, 복제, 이동, 변환, Drag & Drop
- 입력: Slash Command 검색, Markdown 단축 입력, 키보드 중심 조작
- 선택: 텍스트와 Region 선택에서 서식, AI 작업과 영역 링크 복사
- 표: 행·열 생성·삭제·이동, 헤더, 셀 편집과 정렬
- 코드: 언어 선택, 구문 강조, 복사, 여러 줄 편집
- 붙여넣기: Plain Text, Rich Text, Markdown, URL, 이미지와 코드 구조 보존
- Import: Markdown, Plain Text
- Export: Markdown, Plain Text, PDF
- 문서 내 Find와 선택적인 Replace
- 이미지 업로드·붙여넣기·Drag & Drop, 크기 조절과 Caption
- 새 Block Type을 기존 모델의 예외 분기 없이 추가할 수 있는 확장성

향후 후보인 Video, Audio, Embed, Equation, Diagram, Database-like View와 사용자 정의
Block은 현재 기본 Block 모델에 고정하지 않되 초기 구현 범위로 자동 포함하지 않는다.

### 4.4 Document Region과 Operation

- Block, 연속 Block, Section과 Content Range를 안정적인 Region으로 식별한다.
- 편집으로 주변 내용이 바뀌어도 가능한 한 Region identity를 유지한다.
- Region은 Link, Reference, Discussion Topic, AI Context, Review Finding과 Search Result가
  함께 사용한다.
- 사람과 AI의 변경은 동일한 Document Content 모델에 반영한다.
- AI는 문서 전체 문자열을 임의로 다시 쓰지 않고 범위가 명확한
  `DocumentOperation[]`을 제안한다.
- AI 작업은 요청한 범위 밖의 Table, Code 또는 Block을 이유 없이 바꾸지 않는다.
- 작은 사용자의 명시적 Rewrite는 바로 적용하고 Undo할 수 있다. 광범위한 토론 반영과
  Merge는 Proposal과 Diff 승인을 거친다.

### 4.5 Discussion과 Review

- 모든 Discussion은 하나의 Document에 속한다.
- Discussion은 특정 Region에 종속되지 않는다. 여러 Topic과 Reference로 대상을 나타낸다.
- Topic은 Text, Document, Document Region 또는 External Reference가 될 수 있다.
- Topic은 토론 중 추가·삭제할 수 있고 AI Context의 명시적인 초점이 된다.
- Message는 Mention, 내부 Reference와 Attachment를 포함할 수 있다.
- 내부 문서·영역 링크를 붙이면 식별 가능한 Reference로 표현하고 정확한 위치로 이동한다.
- Discussion은 사람이 Close하고 Reopen한다. 닫힌 토론과 이력은 보존한다.
- AI는 합의, 미합의, 아이디어와 정보 부족을 구분해 변경안을 만든다.
- Review는 별도 댓글 시스템을 만들지 않고 수정 요청에 Discussion을 재사용한다.
- Review는 특정 Draft revision을 대상으로 한다. Draft revision이 바뀌면 이전 승인은
  무효가 된다.
- 기본 Publish는 직접 수행할 수 있다. 중요한 문서는 상속 가능한 Publish Policy로
  Reviewer와 필요 승인 수를 요구할 수 있다.
- AI Writing Review는 기본적으로 요청 기반이며 미확인 제안이 Publish를 자동 차단하지
  않는다.

### 4.6 Permission

- Workspace Membership이 모든 접근의 선행 조건이다.
- Document access는 `NO_ACCESS < VIEWER < CONTRIBUTOR < EDITOR` 계층을 가진다.
- 권한 관리 capability는 access와 분리하되 실제 정책상 최소 access 조건을 상세 설계에서
  확정한다.
- Permission Subject는 User와 Group을 지원할 수 있어야 한다.
- Effective Permission은 현재 Document부터 조상 방향으로 가장 가까운 명시적 설정을
  기준으로 계산한다.
- 개인과 Group 권한, `NO_ACCESS`, 관리 capability가 겹칠 때의 precedence는 구현 전
  보안 상세 설계에서 명시적으로 확정한다.
- Editor, Discussion, Review, Search, Reference, Backlink, AI Retrieval, File과 Version
  History는 하나의 Permission Resolver만 사용한다.
- 접근 불가 대상은 제목, 자동완성과 존재 여부도 노출하지 않는다.
- 권한은 검색 결과나 AI 응답 뒤에 제거하지 않는다. Permission Scope를 먼저 만들고
  그 안에서 Retrieval한다.

### 4.7 Reference, Vocabulary와 Retrieval

- Reference는 Source와 Target을 가진 공통 지식 연결 모델이다.
- Target은 Document, Region, Discussion, Vocabulary Concept와 External Resource를
  지원한다.
- Backlink는 Reference 역조회로 제공하며 별도의 진실 소스로 저장하지 않는다.
- 외부 자료는 원본 서비스가 진실 소스다. 필요한 최소 metadata와 연결만 관리한다.
- AI가 외부 내용을 읽지 못했다면 그 내용을 안다고 가정하지 않는다.
- Vocabulary는 Workspace 수준의 Concept authority다.
- Concept은 canonical term, definition, aliases와 deprecated terms를 가진다.
- AI Writing, Review, Search와 Knowledge Query가 같은 Vocabulary를 사용한다.
- 새 내부 용어 등록과 정의 변경은 사람이 승인한다.
- 제목·본문 검색, 의미 기반 검색, 사용자 검색과 AI Retrieval은 하나의 Knowledge
  Retrieval 계층을 사용한다.
- Knowledge Unit은 documentId, version, region과 knowledge kind를 출처로 유지한다.

### 4.8 AI Writing Intelligence

- 사용자는 메모, 키워드, 문장 조각과 정리되지 않은 생각으로 문서를 시작할 수 있다.
- AI는 고정 템플릿에 끼워 넣지 않고 내용과 독자에 맞는 정보 구조를 만든다.
- 사용자 입력의 확정, 검토 중, 모름과 같은 확실성 수준을 보존한다.
- 중요한 정보가 없으면 질문하고, 사소한 공백은 미확정임을 명시해 Draft를 만들 수 있다.
- Context는 현재 Draft, Discussion, Topics, References, Vocabulary, 관련 지식과 Writing
  Rules로 구성한다.
- 사용자가 이번 요청에서 제공한 정보와 명시적인 조직 지식을 AI 일반 지식보다 우선한다.
- Context 간 충돌을 조용히 선택하지 않고 사용자에게 출처와 함께 제시한다.
- 사용자는 AI에게 전달될 Context를 확인하고 추가·제외할 수 있어야 한다.
- Writing Review는 정확성, 논리, 정보 구조, 인지부하와 용어 일관성을 구체적인 문제,
  이유와 수정안으로 제시한다. 의미가 불명확한 점수 최적화를 유도하지 않는다.
- 한국어 Writing Rules는 연구 근거, 탐지 기준과 수정 계약을 별도 설계로 확정한다.
- AI Runtime은 제품의 Writing Intelligence와 분리한다.
- 현재 개발 방향은 웹서버에서 구독형 AI CLI를 실행하는 `CliRuntime`이다. 브라우저는
  AI를 직접 실행하지 않는다.
- Runtime은 최소 인터페이스로 교체 가능하게 하되 현재 API Runtime은 구현하지 않는다.
- AI 작업은 격리된 Job으로 실행하고 구조화된 결과를 애플리케이션이 검증한 뒤 적용한다.

### 4.9 File, Inbox와 Audit

- File은 Document 하위 바이너리가 아니라 Workspace의 독립 `FileAsset`이다.
- Draft, Published Version, Discussion과 다른 지원 대상은 FileAsset을 참조한다.
- 파일 상태는 업로드 중, 준비, 실패와 삭제를 구분한다.
- Draft에서 Reference를 제거해도 과거 Published Version이 참조하면 Asset을 삭제하지 않는다.
- 모든 참조가 사라진 파일만 보존 정책에 따라 Garbage Collection 대상으로 만든다.
- 파일 URL을 아는 것만으로 접근할 수 없고 Workspace와 참조 대상 권한을 확인한다.
- Inbox는 Mention, Review, Changes Requested, Proposal과 Conflict처럼 사용자가 처리할
  협업 항목을 정확한 위치와 연결한다.
- 읽음과 처리 완료는 서로 다른 상태다.
- Audit은 공식 지식 Version History와 분리해 중요한 시스템·협업 행동을 구조화된
  Event로 기록한다.
- Audit Event는 actor, action, target, metadata와 time을 가지며 UI가 사람이 읽는
  문장으로 변환한다.
- 권한과 정책 변경처럼 의미 있는 Event는 before와 after를 남긴다.

## 5. 품질과 제약

### 5.1 보안

Workspace가 데이터 격리 경계다. 모든 읽기, 검색, 자동완성, AI Context와 파일 접근은
권한이 확인된 범위에서만 시작한다. AI Runtime에는 작업에 필요한 최소 Context만 보낸다.

### 5.2 신뢰성과 복구

Draft 입력은 자동 저장 실패와 네트워크 단절로 쉽게 사라지지 않아야 한다. Published
Version과 Audit은 승인된 보존 정책 없이는 변경·삭제하지 않는다. 비동기 작업은 재시도와
중복 실행에서도 상태 불변식을 지켜야 한다.

### 5.3 추적 가능성

공식 Version의 변경 맥락, AI가 사용한 Source, Review 대상 revision과 적용한
Document Operation을 사람이 추적할 수 있어야 한다.

### 5.4 접근성과 입력 방식

에디터 핵심 작업은 키보드로 수행할 수 있어야 한다. 모든 상태와 제안은 색상 하나에만
의존하지 않고 읽을 수 있어야 한다. 상세 접근성 기준은 UX 설계에서 확정한다.

## 6. 명시적 비목표

- 실시간 다중 사용자 본문 편집과 CRDT
- AI의 자율 Publish, 권한 변경 또는 Discussion 종료
- 문서 트리와 별도의 Project·Category·Space 계층
- 모든 Discussion 메시지에 대한 활동 알림
- 외부 시스템 데이터의 무제한 복제
- 초기에 Spreadsheet, Whiteboard 또는 범용 Project Management 제품 만들기
- 현재 단계의 API 기반 AI Runtime
- 특정 Git·Markdown·DB·에디터 라이브러리를 도메인 계약으로 고정하기

## 7. 구현 착수 조건

이 PRD만으로 코드를 작성하지 않는다. 기능 태스크마다 관련 도메인 문서와 함께 UX,
데이터, API·이벤트, 권한, 실패·복구·동시성, 관측성과 테스트 상세 설계를 모두 준비한다.
현재 준비 상태는 `docs/design/README.md`에서 관리한다.

## 8. 미확정 제품 결정

- 최초 출시 범위와 단계별 제공 순서
- Permission precedence와 Group의 최초 출시 포함 여부
- `Manage` capability의 최소 access 조건
- AI 변경을 즉시 적용할 수 있는 작업 크기 경계
- AI CLI 공급자별 서버 사용 가능 조건과 운영 정책
- 한국어 Writing Rules의 검증된 규칙 집합
- 파일 보존 기간과 Garbage Collection 정책

이 항목은 구현에서 추정하지 않는다. 각 항목을 별도 태스크로 조사·결정하고 PRD와 관련
도메인 문서를 갱신한다.
