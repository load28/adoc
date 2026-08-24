# Git 기반 AI 협업 문서 앱 설계

## 1. 제품 목표

여러 사람이 하나의 팀 문서를 지속적으로 함께 작성하고 발전시킬 수 있는 데스크톱 앱을 만든다.

이 앱은 단순한 Markdown 에디터나 사내 위키가 아니다.

핵심 목표는 다음과 같다.

> 사람들이 생각과 정보를 입력하면 AI가 인간의 인지부하를 고려해 읽기 좋은 문서로 구조화하고,
> 여러 사람이 Git을 통해 그 문서를 함께 발전시키며,
> 조직의 결정과 그 이유가 역사로 축적되는 시스템을 만든다.

AI는 문서의 소유자가 아니다.

사람이 작성하고, AI가 작성과 개선을 돕고, 사람이 최종 변경을 승인한다.

또한 중앙 문서 데이터베이스를 별도로 구축하지 않는다.

Git Repository를 문서 데이터의 Source of Truth로 사용한다.

GitHub는 Git Remote, 팀 접근 권한, 저장소 관리와 외부 PR/Issue 관계 표현을 위해 사용한다.

---

## 2. 핵심 원칙

### 2.1 Git이 Source of Truth다

문서의 원본은 데이터베이스가 아니라 Git Repository에 존재한다.

```
Git Repository
      │
      ├── Documents
      ├── Decisions
      ├── Metadata
      ├── Relations
      └── Writing Rules
```

SQLite 등의 로컬 데이터베이스를 사용하더라도 검색 인덱스와 캐시 용도로만 사용한다.

```
Git Repository = Source of Truth
Local DB       = Disposable Cache
```

로컬 DB를 삭제하더라도 Repository만 있으면 전체 상태를 복구할 수 있어야 한다.

### 2.2 문서는 개방된 포맷으로 저장한다

문서를 앱 전용 바이너리나 거대한 JSON으로 저장하지 않는다.

기본 저장 포맷은 Markdown + Frontmatter를 사용한다.

예:

```markdown
---
id: auth-v2
type: design
status: accepted
authors:
  - minmin
relations:
  - type: implements
    target: github:load28/app#124
  - type: supersedes
    target: document:auth-v1
---
# Authentication Architecture v2

## Summary
인증 상태를 서버 세션으로 일원화한다.

## Problem
현재 각 클라이언트가 토큰 갱신 로직을 별도로 관리한다.

## Proposal
...

## Alternatives
...

## Decision
...
```

앱이 없어지더라도 사용자는 Repository를 clone하여 일반 Markdown 도구로 모든 문서를 읽을 수 있어야 한다.

---

## 3. 협업 모델

Google Docs와 같은 실시간 공동 타이핑을 목표로 하지 않는다.

이 앱의 협업 모델은:

> Git 기반 비동기 공동 집필

이다.

```
               GitHub Repository
                       │
             ┌─────────┼─────────┐
             │         │         │
             ▼         ▼         ▼
          Person A  Person B  Person C
             │         │         │
          Local App  Local App  Local App
             │         │         │
          직접 작성    AI 작성    직접 작성
             │         │         │
          commit     commit     commit
             │         │         │
             └──── push/pull ────┘
```

각 사용자는 자신의 로컬 Repository를 가지고 작업한다.

Git의 다음 기능을 협업 기반으로 활용한다.

- clone
- fetch
- pull
- diff
- commit
- push
- branch
- merge
- history
- conflict

Git 기능을 그대로 노출하기보다는 문서 작성자가 이해하기 쉬운 UX로 추상화한다.

---

## 4. Repository 구조

초기 구조는 다음과 같이 한다.

```
workspace/
│
├── workspace.yaml
│
├── projects/
│   └── compiler/
│       ├── project.yaml
│       │
│       └── documents/
│           ├── match-architecture.md
│           ├── type-system.md
│           └── parser-design.md
│
├── decisions/
│   ├── 001-use-semantic-ir.md
│   └── 002-drop-old-matcher.md
│
└── .teamdoc/
    ├── config.yaml
    ├── writing-rules.yaml
    │
    └── document-types/
        ├── design.yaml
        ├── proposal.yaml
        └── decision.yaml
```

팀의 Writing Rule 역시 Repository에 저장한다.

따라서 팀원 모두 동일한 AI 작성 규칙을 공유한다.

---

## 5. Document Model

문서를 단순한 자유형 Markdown으로만 취급하지 않는다.

Markdown 내부에 의미 구조를 유지한다.

예를 들어 Design 문서는 다음 의미를 가진다.

```
Design
  Summary
  Problem
  Context
  Constraints
  Proposal
  Architecture
  Alternatives
  Trade-offs
  Decision
  Consequences
```

화면에서는 자연스러운 하나의 문서처럼 보이지만 Writing Engine에서는 각각의 의미를 이해할 수 있어야 한다.

예:

```typescript
interface Document {
  id: DocumentId
  type: DocumentType
  metadata: DocumentMetadata
  sections: DocumentSection[]
}

interface DocumentSection {
  id: SectionId
  role: SectionRole
  content: EditorContent
}
```

이 구조를 통해 AI에게

> 문서 전체를 다시 작성

하는 것뿐 아니라

> Decision의 rationale만 개선
> Alternatives에 반론 추가
> Proposal을 유지하면서 Architecture 설명 단순화

등의 작업을 요청할 수 있다.

---

## 6. Writing Engine

Writing Engine은 이 제품의 핵심 도메인이다.

LLM 자체가 Writing Engine이 되어서는 안 된다.

```
Raw Input
    ↓
Intent Analysis
    ↓
Information Structure
    ↓
Document Structure
    ↓
Cognitive Rules
    ↓
Team Writing Rules
    ↓
Prompt IR
    ↓
Agent Adapter
    ↓
LLM
```

Writing Engine이 먼저 무엇을 어떤 구조로 전달해야 하는지 결정한다.

LLM은 그 구조에 맞게 실제 표현을 생성한다.

---

## 7. Document Intent

사용자가 입력한 자유로운 생각을 바로 LLM에게 넘기지 않는다.

예:

```
지금 인증에서 토큰을 앱에서 관리하고 있는데
서버 세션으로 바꾸려고 한다.
토큰 갱신 로직이 클라이언트마다 생기고
모바일에서도 문제가 있었다.
서버 의존도가 높아지는 문제는 있다.
PR 124에서 한번 실험했다.
```

Writing Engine은 이를 먼저 의미 구조로 변환한다.

```yaml
Intent:
  DESIGN_PROPOSAL
Problem:
  Client-side token lifecycle management is duplicated.
Motivation:
  - duplicated refresh logic
  - mobile complexity
Proposal:
  Move authentication state to server sessions.
Tradeoff:
  Increased server dependency.
Evidence:
  GitHub PR #124
Unknown:
  - migration strategy
  - expiration policy
```

그 이후 문서 작성 단계로 넘어간다.

---

## 8. Cognitive Writing Rules

AI에게 단순히

> 읽기 쉽게 작성해라.

라고 요청하지 않는다.

사람의 인지부하를 줄이기 위한 규칙을 시스템 자체가 관리한다.

초기 규칙 예시는 다음과 같다.

### C001 — Conclusion First

핵심 결론이나 문서의 목적을 가능한 초반에 제공한다.

### C002 — One Idea Per Paragraph

하나의 문단에 서로 다른 여러 주장을 섞지 않는다.

### C003 — Claim Near Evidence

주장과 그 근거를 가능한 가까이 배치한다.

### C004 — Progressive Disclosure

독자가 아직 필요하지 않은 세부 구현을 상위 개념보다 먼저 설명하지 않는다.

### C005 — Remove Redundancy

같은 의미를 다른 표현으로 반복하지 않는다.

### C006 — Decision Requires Rationale

Decision에는 반드시 결정 이유가 존재해야 한다.

### C007 — Alternatives Require Rejection Reason

검토한 대안에는 선택하지 않은 이유가 있어야 한다.

### C008 — Meaningful Grouping

긴 나열은 의미 단위로 그룹화한다.

### C009 — Contextual Terminology

전문 용어 설명은 독자의 이해에 필요한 경우에만 제공한다.

### C010 — Problem Before Detail

구현 세부사항보다 해결하려는 문제를 먼저 이해시킨다.

이 규칙은 향후 단순 Prompt 목록이 아니라 독립적인 Writing Rule 시스템으로 발전시킨다.

---

## 9. 문서 종류별 구조

문서 종류마다 정보 전달 구조가 다르다.

### Design

```
Summary
   ↓
Problem
   ↓
Constraints
   ↓
Proposal
   ↓
Architecture
   ↓
Alternatives
   ↓
Trade-offs
   ↓
Decision
```

### Proposal

```
Proposal
   ↓
Motivation
   ↓
Expected Benefit
   ↓
Cost / Risk
   ↓
Alternatives
   ↓
Next Step
```

### Decision

```
Decision
   ↓
Rationale
   ↓
Alternatives Considered
   ↓
Accepted Trade-offs
   ↓
Consequences
```

새로운 문서 유형을 추가하더라도 Writing Engine 자체를 수정하지 않는 확장 가능한 구조로 만든다.

---

## 10. AI 역할

AI 작업을 하나의 `generate()` 함수로 추상화하지 않는다.

최소한 다음 역할을 구분한다.

### Composer

새로운 내용을 작성한다.

```
Notes
 ↓
Document Intent
 ↓
Structured Document
```

### Rewriter

기존 내용을 특정 목적에 맞게 수정한다.

예:

- 더 간결하게
- 쉽게 설명
- 논리 구조 개선
- 기술적으로 구체화
- 중복 제거
- 근거 강화
- 반론 추가

### Critic

문서를 직접 변경하지 않고 문제를 분석한다.

예:

- 핵심 결론이 너무 늦게 등장한다.
- 두 문단이 같은 주장을 반복한다.
- Alternative B를 제외한 이유가 없다.
- Decision의 근거가 Proposal 안에 묻혀 있다.

Critic의 결과는 Suggestion이다.

```
Critic
   ↓
Suggestion
   ↓
User Approval
   ↓
Rewrite
```

---

## 11. AI 변경은 항상 제안이다

AI가 공유 문서를 임의로 확정해서는 안 된다.

기본 흐름은:

```
Current Document
       ↓
      AI
       ↓
Proposed Change
       ↓
     Diff
       ↓
 ┌─────┴─────┐
Accept      Reject
   │
   ↓
Working Tree
```

사용자가 최종 변경을 승인한다.

AI가 직접 commit하거나 push하는 것을 기본 동작으로 만들지 않는다.

---

## 12. Local Agent Architecture

서버에서 공통 LLM API를 운영하는 대신 각 사용자가 가지고 있는 로컬 AI Agent 환경을 활용한다.

예:

```
Desktop App
     │
Writing Engine
     │
Agent Runtime
     │
 ┌───┴──────────┐
 │              │
CodexAdapter  ClaudeAdapter
 │              │
Codex          Claude
```

핵심 인터페이스는 특정 Agent에 종속되지 않는다.

```typescript
interface AgentAdapter {
  capabilities(): AgentCapabilities
  execute(
    request: AgentRequest,
    context: AgentContext
  ): AsyncIterable<AgentEvent>
}
```

각 Adapter가 인증, CLI 실행 방식, streaming 방식 등의 차이를 처리한다.

Writing Engine은 어떤 Agent가 실행되는지 몰라야 한다.

---

## 13. Prompt Compiler

Prompt 역시 제품의 중요한 내부 구현이다.

Prompt 문자열을 애플리케이션 곳곳에 직접 작성하지 않는다.

먼저 중간 표현을 생성한다.

```typescript
interface WritingRequest {
  task:
    | "compose"
    | "rewrite"
    | "critique"
    | "merge"
  documentType: DocumentType
  intent: DocumentIntent
  source: ContentBlock[]
  context: Context[]
  rules: WritingRule[]
  target?: SectionId[]
}
```

그리고:

```
WritingRequest
      ↓
Prompt Compiler
      ↓
Agent-specific Prompt
      ↓
Agent Adapter
```

구조로 처리한다.

따라서 Agent가 바뀌더라도 Writing Engine은 유지된다.

---

## 14. Git Integration

Git은 인프라 구현 세부사항이 아니라 협업 모델의 핵심이다.

하지만 일반 사용자가 Git 명령을 직접 다룰 필요는 없다.

예:

```
Git                  App UX
git pull        →    최신 문서 가져오기
git diff        →    변경사항 보기
git commit      →    변경 기록
git push        →    팀에 공유
git log         →    문서 History
git merge       →    다른 사람 변경 합치기
conflict        →    문서 충돌 해결
```

Git 개념을 문서 중심 UX로 변환한다.

---

## 15. Conflict Resolution

동일한 문서를 여러 사람이 변경하면 Git conflict가 발생할 수 있다.

일반 Git conflict marker를 사용자에게 그대로 보여주지 않는다.

앱에서 의미 있는 비교 화면을 제공한다.

```
Architecture

Current
────────────────────
서버 세션으로 인증 상태를 관리한다.

Incoming
────────────────────
서버 세션과 refresh token을 함께 사용한다.

[Current 사용]
[Incoming 사용]
[둘 다 유지]
[직접 수정]
[AI로 병합]
```

---

## 16. AI Merge

Conflict는 AI가 특히 도움을 줄 수 있는 영역이다.

AI에게 다음 세 가지를 제공한다.

```
BASE
CURRENT
INCOMING
```

그리고 단순히 두 문자열을 합치는 것이 아니라:

- 두 작성자의 의미와 의도를 최대한 보존한다.
- 중복되는 설명을 제거한다.
- 새로운 사실을 임의로 추가하지 않는다.
- 서로 모순되는 결정은 임의로 선택하지 않는다.
- 모순이 해결 불가능하면 사용자에게 명확히 표시한다.
- Cognitive Writing Rules를 유지한다.

라는 규칙으로 Merge Proposal을 생성한다.

흐름은:

```
Git Conflict
     ↓
AI Merge
     ↓
Merge Proposal
     ↓
Diff
     ↓
Human Review
     ↓
Resolve
```

AI가 conflict를 임의로 확정하지 않는다.

---

## 17. Document History

Git History를 문서 History로 변환한다.

사용자에게 SHA 중심의 Git UI를 보여주는 것이 목적이 아니다.

예:

```
Authentication Architecture
History

Aug 24 · 민민
Architecture rationale 개선

Aug 24 · 철수
Alternative B 추가

Aug 23 · 민민
Initial design
```

각 History를 선택하면 변경 내용을 Diff로 보여준다.

```
Previous
     ↓
Diff
     ↓
Current
```

필요하면 특정 버전을 복원할 수도 있다.

따라서 별도의 Revision Database를 구축하지 않는다.

---

## 18. Document Graph

문서는 독립적인 파일들의 집합이 아니다.

문서 사이의 의미 관계를 저장한다.

예:

```
                  PR #142
                     ↑
                 implements
                     │
Auth Design ──── Decision #12
     │
     │ supersedes
     ↓
Auth Design v1
```

초기 Relation 종류:

- related_to
- supersedes
- implements
- implemented_by
- motivated_by
- discussed_in
- reverted_by

Document ↔ Document뿐 아니라 외부 리소스도 연결할 수 있다.

---

## 19. GitHub의 역할

GitHub 자체를 제품의 중심으로 만들지 않는다.

GitHub의 역할은 크게 두 가지다.

### Git Remote

팀 문서를 저장하고 공유하기 위한 Private Repository.

### External Relation

개발 문서와 실제 코드 변경의 관계를 표현한다.

예:

```
Design Document
      │
      └── implemented_by
               │
               ▼
           PR #142
```

PR의 코드 리뷰나 CI를 앱 내부에서 다시 구현하지 않는다.

필요한 것은:

> 이 문서의 결정이 어떤 실제 작업과 연결되었는가?

라는 관계다.

---

## 20. Local Index

Repository가 커지면 모든 Markdown을 매번 읽을 수 없다.

따라서 로컬 Index를 둔다.

예:

```
Git Repository
      ↓
Document Scanner
      ↓
Parser
      ↓
Local Index
      ↓
 ┌────┼────────┐
 │    │        │
Search Graph  AI Context
```

SQLite 등을 사용할 수 있다.

하지만 Index는 언제든 재생성 가능해야 한다.

```
delete index
    ↓
scan repository
    ↓
rebuild
```

이 원칙을 반드시 유지한다.

---

## 21. AI Context Retrieval

AI에게 Repository 전체를 무조건 전달하지 않는다.

현재 작업과 관계있는 정보만 수집한다.

예:

```
사용자:
"이 Decision을 다시 정리해줘."
           ↓
Current Decision
           +
Related Proposal
           +
Related Alternatives
           +
Referenced Decisions
           +
Relevant Writing Rules
           ↓
WritingRequest
```

Document Graph가 AI Context Retrieval에도 활용된다.

따라서 Document Graph는 단순한 시각화 기능이 아니다.

---

## 22. 전체 아키텍처

```
                         GitHub
                    Private Repository
                           │
                      fetch/push
                           │
┌─────────────────────────────────────────────────┐
│                  Desktop App                    │
│                                                 │
│  ┌───────────────────────────────────────────┐  │
│  │                Editor                     │  │
│  │                                           │  │
│  │ Direct Editing                            │  │
│  │ AI Suggestions                            │  │
│  │ Diff Review                               │  │
│  │ Conflict Resolution                       │  │
│  └───────────────────┬───────────────────────┘  │
│                      │                          │
│  ┌───────────────────▼───────────────────────┐  │
│  │             Writing Engine                │  │
│  │                                           │  │
│  │ Intent Analysis                           │  │
│  │ Document Structure                        │  │
│  │ Cognitive Writing Rules                   │  │
│  │ Prompt Compiler                           │  │
│  └───────────────────┬───────────────────────┘  │
│                      │                          │
│  ┌───────────────────▼───────────────────────┐  │
│  │             Agent Runtime                 │  │
│  │                                           │  │
│  │ CodexAdapter / ClaudeAdapter / ...        │  │
│  └───────────────────────────────────────────┘  │
│                                                 │
│  ┌───────────────────────────────────────────┐  │
│  │               Git Layer                   │  │
│  │                                           │  │
│  │ commit / push / pull / merge / history    │  │
│  └───────────────────┬───────────────────────┘  │
│                      │                          │
│  ┌───────────────────▼───────────────────────┐  │
│  │              Local Index                  │  │
│  │                                           │  │
│  │ Search / Document Graph / AI Context      │  │
│  │ (Disposable Cache — 언제든 재생성 가능)     │  │
│  └───────────────────────────────────────────┘  │
│                                                 │
└─────────────────────────────────────────────────┘
```
