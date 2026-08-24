# 도메인 지도

- **문서 ID**: DOM-00
- **상태**: 동결
## 1. 경계

```text
Workspace
├─ Governance
│  ├─ Membership
│  ├─ Permission
│  └─ Publish Policy
├─ Document System
│  ├─ Document Tree
│  ├─ Draft
│  ├─ Published Version
│  ├─ Document Content
│  └─ Document Operation
├─ Collaboration
│  ├─ Discussion
│  ├─ Review
│  └─ Inbox
├─ Knowledge
│  ├─ Reference
│  ├─ Vocabulary
│  └─ Retrieval
├─ Writing Intelligence
│  ├─ AI Task
│  ├─ Context Builder
│  ├─ Writing Policy
│  └─ AI Runtime
└─ Operations
   ├─ File Asset
   └─ Audit
```

| 경계 | 소유하는 의미 | 정본 문서 |
|---|---|---|
| Workspace·Governance | 격리, 멤버십, 접근과 Publish 정책 | [workspace-governance.md](workspace-governance.md) |
| Document System | 문서 identity, 내용, Draft와 공식 Version | [document-system.md](document-system.md) |
| Collaboration | 토론, 검토와 사용자의 작업 항목 | [collaboration.md](collaboration.md) |
| Knowledge | 지식 연결, 공통 용어와 근거 있는 검색 | [knowledge.md](knowledge.md) |
| Writing Intelligence | 제한된 AI 작업, Context와 글쓰기 정책 | [writing-intelligence.md](writing-intelligence.md) |
| Operations | 파일 생명주기와 중요 행동 기록 | [operations.md](operations.md) |

## 2. 핵심 의존 방향

```text
Workspace Scope
    ↓
Permission Scope
    ├─→ Document System
    ├─→ Collaboration
    ├─→ Knowledge Retrieval
    └─→ File Access

Document + Collaboration + Knowledge
    ↓ 제한된 입력 계약
Writing Intelligence
    ↓ 구조화된 결과
Application Domain
```

Permission은 결과를 나중에 거르는 필터가 아니라 모든 조회의 입력 범위다. Writing
Intelligence는 다른 도메인의 DB를 직접 읽거나 쓰지 않고 애플리케이션이 제공한 Context와
Command만 소비한다.

## 3. 공통 원시 개념

- **Identity**: 내용이 바뀌어도 같은 대상을 가리키는 안정적인 식별자
- **Revision**: 변경 가능한 Draft의 낙관적 동시성 단위
- **Version**: Publish된 불변 공식 상태
- **Region**: Document Content의 안정적인 부분 identity
- **Reference**: Source와 Target 사이의 지식 연결
- **Operation**: 문서 상태를 바꾸는 검증 가능한 구조화 명령
- **Scope**: Workspace와 Permission이 허용한 데이터 경계
- **Source**: AI 답변과 변경안의 근거가 된 지식 위치

## 4. 금지 의존성

- AI Runtime → 애플리케이션 DB 직접 읽기·쓰기
- Discussion → 에디터 내부 자료구조 직접 변경
- Retrieval → Permission Scope 우회
- Vocabulary → 특정 AI 모델 또는 Runtime 의존
- Document → Git, Markdown, 특정 DB 포맷 의존
- Editor Content → Review·Publish 정책 의존
- FileAsset → 현재 Draft 하나를 기준으로 생명주기 결정
- Inbox → Audit Event를 그대로 사용자 작업으로 투영

## 5. 상세 설계로 내려갈 때의 계약

도메인 문서는 기술 중립적인 의미와 불변식을 소유한다. 저장 스키마, API, 메시지 전달,
인덱스, 락 구현과 UI 컴포넌트는 `docs/design/`에서 결정한다. 상세 설계가 도메인 불변식을
표현할 수 없다면 도메인을 우회하지 말고 설계를 다시 만든다.
