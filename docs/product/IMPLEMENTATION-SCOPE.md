# 전체 제품 구현 범위

- **문서 ID**: PROD-05
- **상태**: 동결
- **작성일**: 2026-08-24
- **정본 관계**: 제품 요구사항의 의미는 [PRD](PRD.md)가 소유하고, 이 문서는 첫 구현의
  포함·제외 범위를 소유한다.

## 1. 구현 전략

MVP나 기능 축소 버전을 별도로 만들지 않는다. PRD에 정의된 필수 제품 요구사항 전체를
첫 구현 범위로 삼는다. 모든 상세 설계를 먼저 완성하고 전체 설계 동결 게이트를 통과한
뒤 하나의 개발 단계에서 구현한다.

구현 내부에는 기술적 의존 순서가 있지만 이는 출시 범위를 나누는 단계가 아니다. 최종
완료 판정은 전체 기능, 통합 흐름과 운영 검증이 모두 충족됐을 때만 내린다.

## 2. 첫 구현 포함 범위

### 2.1 Workspace와 Governance

- 사용자 인증, Workspace 생성·전환과 Member·Admin 관리
- User·Group Permission Subject와 Group 관리
- 문서 트리 권한 상속, override와 단일 Permission Resolver
- PublishPolicy 상속, Reviewer와 필요 승인 수
- 모든 조회·검색·AI·파일 접근 이전의 Permission Scope 적용

### 2.2 Document와 Editor

- Document identity, 자유로운 트리, 이동·정렬·휴지통·복구
- 단일 공유 Draft, 자동 저장, Edit Lease와 입력 복구
- 불변 Published Version, History, Diff와 과거 Version 기반 Draft 복원
- Publish conflict 감지, 3-way 비교와 사람·AI 병합 제안
- PRD 4.3의 전체 기본 Rich Text·Block·Table·Code·Image·File 편집 기능
- Slash Command, Markdown shortcut, 키보드 조작, Drag & Drop과 다중 선택
- Markdown·Plain Text Import, Markdown·Plain Text·PDF Export
- 안정적인 Region, Reference와 구조화 Document Operation

### 2.3 Collaboration

- 문서 단위 Discussion, 복수 Topic과 Message
- Mention, 내부 Document·Region Reference와 Attachment
- Discussion Close·Reopen과 과거 맥락 보존
- Discussion 분석과 Document Operation Proposal·Diff·승인 흐름
- 특정 Draft revision에 대한 Review, 승인 무효화와 Changes Requested
- Mention, Review, Proposal과 Conflict를 처리하는 Inbox

### 2.4 Knowledge

- Document·Region·Discussion·Vocabulary·External Resource Reference
- Backlink와 안정적인 대상 이동
- Workspace Vocabulary Concept, canonical term, alias와 deprecated term
- 제목·본문 lexical search와 semantic retrieval
- 사용자 Search, AI Context와 Knowledge Query가 공유하는 Retrieval 계층
- Permission-safe index와 Source provenance

### 2.5 Writing Intelligence

- Raw Thoughts Compose, 범위 Rewrite와 요청 기반 Writing Review
- Discussion Apply, Conflict Merge와 근거 기반 Knowledge Query
- Context 확인·추가·제외, authority와 충돌 표시
- Vocabulary와 Workspace Writing Rules 적용
- 검증된 한국어 인지부하 Writing Rules
- 구조화 AIResult, Operation validation, Proposal·Diff·Undo 정책
- 서버 Job Queue, 격리된 구독형 CLI Runtime, streaming·timeout·cancellation
- 운영 환경용 OpenAI Responses API Runtime과 provider-neutral Runtime port

### 2.6 Files와 Operations

- FileAsset upload·validation·preview·download와 Reference 수명주기
- Published Version을 보존하는 Garbage Collection 정책
- 구조화 Audit Event, before·after와 조회
- 단일 최신 Published 문서만 제공하는 폐기 가능한 익명 Viewer link
- 보안 로그, 관측성, 백업·복구와 데이터 migration 기반

## 3. 전체 완료 사용자 여정

아래 여정은 하나라도 끊기면 전체 구현이 완료된 것으로 보지 않는다.

```text
Workspace 생성과 멤버 초대
→ 문서 트리와 권한 구성
→ 정리되지 않은 생각에서 Draft 생성
→ 사람·AI 편집과 자동 저장
→ Draft를 보며 Discussion·Reference·Mention
→ 토론 반영 Proposal 검토
→ 정책에 따른 Review
→ 동시 Publish 충돌 감지·해결
→ 불변 Version Publish
→ Search·Backlink·Vocabulary·AI 질문에서 근거 확인
→ History·Audit·Inbox에서 변화와 할 일 추적
```

## 4. 첫 구현 제외 범위

PRD의 명시적 비목표와 다음 향후 후보는 첫 구현에 포함하지 않는다.

- 실시간 다중 사용자 본문 편집과 CRDT
- Spreadsheet, Whiteboard와 범용 Project Management
- Video, Audio, 범용 Embed, Equation, Diagram, Database-like View와 Custom Block
- AI의 자율 Publish, 권한 변경 또는 Discussion 종료
- 문서 트리와 별도의 Project·Category·Space 계층
- 외부 시스템 전체 데이터 복제와 외부 서비스 편집
- 다중 Provider 선택 UI와 조용한 Provider fallback
- 결제·요금제·구독 관리
- 고객용 Public API와 Webhook
- 공개 문서의 Draft·History·Discussion·Search·AI·Backlink·tree 노출

제외 기능을 대비한 추상화를 미리 과도하게 만들지 않는다. 다만 현재 도메인 계약을 깨지
않고 이름 있는 구현을 추가할 수 있는 경계는 유지한다.

## 5. 완료 기준

- Master Plan의 모든 필수 상세 설계가 구현과 일치한다.
- 포함 범위의 모든 사용자 여정이 웹 UI에서 끝까지 동작한다.
- 도메인 불변식이 단위·통합·동시성·보안·복구 테스트로 검증된다.
- 접근 불가 데이터가 Search, Reference, AI Context와 File 경로에서 노출되지 않는다.
- Published Version, Review revision과 Source provenance를 재구성할 수 있다.
- AI Job 실패·취소·재시도와 잘못된 결과가 Draft를 손상시키지 않는다.
- 백업·복구, migration, 관측성과 운영 Runbook이 준비된다.
- 자동화된 품질 게이트와 전체 인수 시나리오가 통과한다.

## 6. 범위 변경 규칙

AI를 통한 구현 변경 비용이 낮더라도 범위를 코드에서 먼저 바꾸지 않는다. 새 기능이나
정책 변경은 태스크를 만들고 PRD, 이 문서, 관련 도메인·상세 설계와 테스트 계약을 함께
갱신한다. 변경된 전체 설계가 다시 일관된 상태일 때 구현을 수정한다.
