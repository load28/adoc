# Content와 Microcopy

- **문서 ID**: UX-11
- **상태**: 동결

## 용어

`Document`, `Draft`, `Published Version`, `Discussion`, `Review`, `Proposal`의 의미를 섞지
않는다. 한국어 UI는 문서, 초안, 발행 버전, 토론, 검토, 변경 제안을 기본 표시어로 쓴다.

## 상태 문안 구조

오류는 `무슨 일이 발생했는지 → 데이터 영향 → 사용자가 할 수 있는 행동 → correlation ID`
순으로 쓴다. 성공 문안은 실제 server commit 뒤에만 표시한다. AI 결과는 사실처럼 말하지
않고 제안·근거 부족·충돌 상태를 명시한다.

## 위험 작업

delete, permission loss, link publish와 lease takeover confirmation은 대상 이름, 영향 범위와
복구 가능 기한을 포함한다. 일반적인 `확인하시겠습니까?`만 사용하지 않는다.

## 국제화

문장 조각 결합을 금지하고 ICU message로 plural·date·number를 처리한다. 번역 key는 기능이
아닌 의미 중심으로 이름 짓는다. server error는 stable code를 보내고 client가 locale별
message를 렌더링한다.

## 공개 Viewer

Workspace 이름이나 내부 상태를 추론하게 하는 문안을 표시하지 않는다. revoked, expired,
unknown token은 같은 외부 메시지를 사용한다.
