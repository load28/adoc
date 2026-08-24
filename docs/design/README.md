# 상세 설계 인덱스

## 현재 결론

전체 제품 범위의 제품·UX·아키텍처·데이터·API·보안·품질·운영 설계를 같은 결정 snapshot으로
작성했다. 최종 구현 가능 판정과 검증 evidence는
[Design Freeze Report](implementation/DESIGN-FREEZE-REPORT.md)가 소유한다.

## 기능별 설계 묶음

| 기능 영역 | 핵심 정본 | 상태 |
|---|---|---|
| Workspace·Permission | UX-04·13~15, SPEC-01~03·18, API-06~08 | 동결 |
| Document·Editor | UX-05~06·16, SPEC-04~08·17·19, CONTRACT-01~02 | 동결 |
| Discussion·Review | UX-07·13, SPEC-09~10·17, API-02·06 | 동결 |
| Knowledge·AI | UX-08, SPEC-11~14·19, CONTRACT-03, DATA-09 | 동결 |
| File·Audit·Retention | SPEC-15~19, DATA-04·07~08, PRIV-01 | 동결 |
| 운영·검증·구현 | OPS-01~07, TEST-07~09, PLAN-01~09 | 동결 |

## 구현 사용 규칙

구현 Task는 [Requirements Traceability](../product/REQUIREMENTS-TRACEABILITY.md)에서 요구사항을
선택하고 연결된 제품·domain·spec·test 문서를 `필수 설계 문서`에 기록한다. 문서에 없는
정책을 코드에서 결정하지 않는다.

## 정본 위치

- UX: `ux/`
- 시스템·ADR: `architecture/`, `adr/`
- 데이터·API: `data/`, `api/`
- domain 상세: `specs/`
- 보안·품질·운영: `security/`, `quality/`, `operations/`
- 구현 계획·동결: `implementation/`
