# Risk Register

- **문서 ID**: PLAN-04
- **상태**: 동결

| ID | 위험 | 구조적 완화 | 검증 |
|---|---|---|---|
| R-01 | Permission scope와 point resolver 불일치 | 하나의 policy compiler·golden property test | TEST-04 |
| R-02 | Tiptap position에 영속 anchor 결합 | stable Block ID·Region resolver | SPEC-06 |
| R-03 | Atlaskit 외부 지원·peer 호환 | public package만, lock matrix·visual SSR gate | ADR-008 |
| R-04 | OpenSearch 권한 지연 leak | deny-safe scope, sequence tombstone | SPEC-12 |
| R-05 | AI prompt injection·hallucination | no tool/data credential, Source·schema validation | SEC-04 |
| R-06 | CLI credential의 공용 사용 | local only, managed API adapter | ADR-006 |
| R-07 | lease·offline 입력 손실 | revision, idempotency, local recovery | SPEC-07 |
| R-08 | duplicate Job·event side effect | DB state 정본, receipt, expected revision | ARCH-06 |
| R-09 | File GC가 Version을 손상 | all-owner reference graph·grace·recheck | SPEC-15 |
| R-10 | 삭제 데이터가 backup restore로 부활 | deletion ledger 우선 재적용 | OPS-04 |
| R-11 | 전체 범위 통합 지연 | dependency DAG·vertical integration·single final gate | PLAN-02 |
| R-12 | public link를 Workspace access로 오용 | separate principal·route·exact asset set | SEC-03 |

Risk는 우회 허용 사유가 아니다. trigger가 발생하면 별도 Task에서 정본 설계·test와 완화를
같이 변경한다.
