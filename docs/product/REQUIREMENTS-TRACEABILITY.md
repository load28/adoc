# Requirements Traceability

- **문서 ID**: PROD-09
- **상태**: 동결

| 요구사항 | 정본 제품 | 핵심 설계 | 검증 |
|---|---|---|---|
| RQ-01 Google SSO·초대 | PROD-10 | SPEC-01·18, API-06 | A-01, TEST-04·09 |
| RQ-02 Group·Permission precedence | PROD-10 | SPEC-02·18·19, DATA-08 | A-01, TEST-08·09 |
| RQ-03 PublishPolicy 상속 | PROD-10·11 | SPEC-03·08·17 | A-02·03, TEST-09 |
| RQ-04 Document tree | PROD-11 | SPEC-04·19, DATA-07 | A-01·08, TEST-09 |
| RQ-05 block editor 전체 기능 | PROD-12 | SPEC-05, CONTRACT-01, UX-16 | A-02, TEST-07·08, a11y |
| RQ-06 Region·Operation·Diff | PROD-12 | SPEC-06·19, CONTRACT-02 | A-03·06, TEST-07·09 |
| RQ-07 단일 Draft·Edit Lease | PROD-11 | SPEC-07·17, DATA-08 | A-02·03, TEST-03·09 |
| RQ-08 immutable Publish·Version | PROD-11 | SPEC-08·17·19, DATA-07 | A-02·03, TEST-09 |
| RQ-09 Discussion·Topic | PROD-13 | SPEC-09·17, API-06 | A-04, TEST-09 |
| RQ-10 Review·Inbox | PROD-13 | SPEC-10·17, DATA-08 | A-02·04, TEST-09 |
| RQ-11 Reference·Vocabulary | PROD-14 | SPEC-11·17, API-02 | A-05, TEST-09 |
| RQ-12 hybrid permission-safe Search | PROD-14 | SPEC-12·18·19, DATA-09 | A-05, TEST-04·06·09 |
| RQ-13 AI task·grounded result | PROD-15 | SPEC-13·19, CONTRACT-03 | A-05·06, TEST-05·07·09 |
| RQ-14 AI queue·dual Runtime | PROD-15 | SPEC-14·17, PLAN-06 | A-06, TEST-03·05·09 |
| RQ-15 File lifecycle | PROD-16 | SPEC-15·17·19, DATA-07 | A-07·08, TEST-09 |
| RQ-16 structured Audit | PROD-16 | SPEC-16, CONTRACT-04, DATA-07 | A-08, TEST-08 |
| RQ-17 공개 단일 문서 Viewer | PROD-10 | SPEC-08·15·18, API-02 | A-07, TEST-04·09 |
| RQ-18 ko/en·responsive | PROD-03·06 | UX-01~19 | A-01~08, TEST-09, visual·a11y |
| RQ-19 99.9%·RPO/RTO | PROD-06 | OPS-03·04, PLAN-07 | TEST-03·06·09, DR drill |
| RQ-20 30일 삭제·privacy | PROD-16 | DATA-04·07·08, PRIV-01 | A-08, TEST-09 |

## 변경 규칙

요구사항 추가·변경은 이 표, 제품 정본, 관련 schema와 최소 하나의 검증을 같은 Task에서
갱신한다. 설계나 test가 없는 행은 구현 준비가 되지 않은 것으로 판정한다.
