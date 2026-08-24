# 제품 지표

- **문서 ID**: PROD-17
- **상태**: 동결

## 핵심 결과 지표

| 지표 | 정의 | 목표 방향 |
|---|---|---|
| Knowledge convergence | Discussion이 연결된 Publish 중 반영 또는 명시적 미반영 근거가 있는 비율 | 증가 |
| Publish lead time | Draft 생성부터 Publish까지의 중앙값·p90 | 맥락별 감소 |
| Grounded AI acceptance | Source가 유효한 AI Proposal 중 사용자가 수락한 비율 | 증가 |
| Find success | Search 후 5분 내 결과 열람·참조·질문 해결 행동 비율 | 증가 |
| Recovery success | 저장 장애 세션 중 입력 손실 없이 복구한 비율 | 99.9% 이상 |

## Guardrail

- unauthorized exposure count는 0이어야 한다.
- AI 적용 후 10분 이내 Undo·revert 비율을 품질 경보로 추적한다.
- Review 우회, stale approval Publish와 Version mutation은 0이어야 한다.
- Search·AI 장애가 Document read/write SLO를 침해한 횟수를 추적한다.

## 측정 원칙

Analytics Event는 목적에 필요한 최소 identifier만 사용한다. 본문, AI prompt, Message와
검색어 원문을 기본 analytics property로 수집하지 않는다. event schema는
[Analytics Events](../design/data/ANALYTICS-EVENTS.md)가 소유한다.
