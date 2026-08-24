# 비기능 요구사항

- **문서 ID**: PROD-06
- **상태**: 동결

## 보안과 개인정보

- Workspace는 tenant·암호화·조회 경계다.
- Google OIDC session, CSRF 방어, secure cookie와 rotation을 적용한다.
- 모든 파생 조회 전에 Permission Scope를 강제한다.
- AI에는 작업에 필요한 최소 Context만 전달하고 credential을 Job에 포함하지 않는다.
- Public link token은 원문 저장하지 않고 hash로 검증하며 폐기와 만료를 지원한다.

## 가용성과 복구

- 핵심 문서 읽기·쓰기 월간 가용성 SLO: 99.9%.
- RPO: 15분, RTO: 4시간.
- Search·AI·preview 장애는 핵심 Document read/write에서 격리한다.
- Draft 저장은 idempotent command와 Local Recovery Buffer를 제공한다.

## 성능 목표

- 일반 문서·tree query: 서버 p95 300ms 이하, 공개 문서 p95 500ms 이하.
- command acknowledgement: p95 500ms 이하. 비동기 완료는 별도 상태로 표시한다.
- Search 첫 page: p95 1.5초 이하. AI 첫 progress event: p95 2초 이하.
- 기준 부하는 [용량 설계](../design/architecture/SCALABILITY-CAPACITY.md)가 소유한다.

## 접근성과 반응형

- WCAG 2.2 AA를 자동·수동 검증한다.
- Desktop, tablet, mobile에서 같은 핵심 기능을 제공한다.
- Pointer-only 조작에는 keyboard 또는 menu 대체 경로가 있어야 한다.

## 호환성과 품질

- 최신 두 major의 Chrome, Edge, Firefox, Safari를 지원한다.
- 한국어·영어 UI와 locale·timezone을 지원한다.
- schema, event와 API 변경은 호환 window와 migration을 가져야 한다.
