# 제품 요구사항 인덱스 — 팀 문서 시스템

- **상태**: 동결
- **버전**: 1.0
- **작성일**: 2026-08-24
- **근거**: 제품 대화, DEC-000~034와 TASK-003

## 제품 정본

| 문서 | 소유하는 내용 |
|---|---|
| [제품 개요](PRODUCT-BRIEF.md) | 문제, 비전, 가치와 성공 정의 |
| [제품 원칙](PRODUCT-PRINCIPLES.md) | 사람 통제, 공식 지식, 권한·복구 원칙 |
| [사용자](USERS-AND-STAKEHOLDERS.md) | 사용자, 역할, 이해관계자와 국제화 |
| [사용자 여정](USER-JOURNEYS.md) | end-to-end 흐름과 실패·복구 |
| [구현 범위](IMPLEMENTATION-SCOPE.md) | 첫 전체 구현의 포함·제외 경계 |
| [비기능 요구사항](NON-FUNCTIONAL-REQUIREMENTS.md) | 보안, SLO, 성능, 접근성, 호환성 |
| [용어집](GLOSSARY.md) | 공통 제품·도메인 용어 |
| [Decision Register](DECISION-REGISTER.md) | 사용자 결정과 정본 연결 |
| [추적성](REQUIREMENTS-TRACEABILITY.md) | 요구사항→설계→검증 |
| [제품 지표](PRODUCT-METRICS.md) | 결과 지표와 guardrail |

## 기능 정본

- [Workspace와 거버넌스](features/WORKSPACE-AND-GOVERNANCE.md)
- [Document 생명주기](features/DOCUMENT-LIFECYCLE.md)
- [Editor](features/EDITOR.md)
- [Collaboration](features/COLLABORATION.md)
- [Knowledge](features/KNOWLEDGE.md)
- [Writing Intelligence](features/WRITING-INTELLIGENCE.md)
- [File과 Audit](features/FILES-AND-AUDIT.md)

## 핵심 순환

```text
생각·근거 입력 → Draft → 사람·AI 편집 → Discussion → Review
→ Publish → 불변 Published Version → Search·Reference·AI Context → 다음 Draft
```

AI는 직접 Publish·Permission 변경·Discussion 종료를 수행하지 않는다. Search와 AI는
Permission Scope 뒤에서만 동작한다. 익명 공개는 명시된 단일 최신 Published 문서의 Viewer
link로만 제공한다.

## 구현 착수 조건

이 인덱스 한 건으로 코드를 작성하지 않는다. [문서 지도](../DOCUMENT-MAP.md)의 전체 설계와
[Design Freeze Report](../design/implementation/DESIGN-FREEZE-REPORT.md)가 동결이고, 구현
Task가 필요한 정본 문서를 명시적으로 읽은 뒤에만 코드를 작성한다.
