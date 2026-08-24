# 제품 용어집

- **문서 ID**: PROD-07
- **상태**: 동결

| 용어 | 정의 |
|---|---|
| Workspace | tenant, Membership와 보안의 최상위 경계 |
| Document | 내용과 분리된 지속 identity와 tree node |
| Draft | 하나의 Document에 최대 하나인 변경 가능한 공유 작업 내용 |
| Draft revision | Draft 내용 변경마다 증가하는 낙관적 동시성 번호 |
| Published Version | Publish 성공 시 생성되는 불변 내용 snapshot |
| Edit Lease | 한 시점의 본문 편집자 한 명을 보장하는 만료 가능한 권리 |
| Region | Block·Section·range를 안정적으로 식별하는 위치 |
| Document Operation | 사람·AI 변경을 표현하는 검증 가능한 구조화 명령 |
| Discussion | 하나의 Document에 속하고 여러 Topic을 갖는 토론 |
| Review | 정확한 Draft revision의 발행 적합성 판단 과정 |
| Reference | 지식 Source와 Target 사이의 명시적 연결 |
| Vocabulary Concept | canonical term, definition, alias와 deprecated term의 authority |
| Permission Scope | 파생 조회 전에 계산된 접근 가능한 identity 집합 |
| AITask | 허용된 Context와 결과 schema를 가진 제한된 AI 작업 |
| Proposal | 적용 전 Diff와 승인을 요구하는 Operation 묶음 |
| FileAsset | Document와 독립된 Workspace binary asset |
| Audit Event | 중요 시스템·협업 행위의 구조화된 불변 기록 |
| Public Viewer Link | 단일 최신 Published Version만 허용하는 익명 capability token |

영문 도메인 용어는 코드와 계약에서 그대로 사용한다. UI 번역은 의미를 바꾸지 않으며
[콘텐츠 문안](../design/ux/CONTENT-AND-MICROCOPY.md)이 표시 용어를 소유한다.
