# 사용자 여정

- **문서 ID**: PROD-04
- **상태**: 동결

## J-01. Workspace 시작

Google SSO 로그인 → Workspace 생성 또는 초대 수락 → locale·timezone 설정 → Member·Group
구성 → 최상위 Permission과 PublishPolicy 확인.

실패 시 초대 대상 불일치, 만료, 이미 수락됨과 Workspace 삭제 대기를 구분한다.

## J-02. 생각에서 공식 문서까지

문서 생성 → 메모 입력 또는 AI Compose → 자동 저장 → Discussion과 Reference 연결 → AI
Proposal 검토 → Review 요청 → 승인 또는 Changes Requested → 충돌 확인 → Publish → 새
불변 Version 확인.

## J-03. 편집권 인계와 복구

문서 진입 → Edit Lease 요청 → 다른 편집자 존재 시 읽기·Discussion 유지 → 편집권 요청 또는
만료 대기 → revision 기반 저장 → 네트워크 실패 시 Local Recovery Buffer → 재연결 후
서버 revision 비교 → 자동 재적용 또는 충돌 복구.

## J-04. 지식 탐색

검색어 입력 → Permission Scope 내 hybrid retrieval → 문서·Region 결과 이동 → Backlink와
Vocabulary 확인 → AI 질문 → Source별 근거·충돌·부족 상태 확인.

## J-05. 거버넌스 변경

문서·Group 선택 → 현재 Effective Permission 확인 → 변경 영향 미리보기 → Grant 또는
PublishPolicy 변경 → resolver 재평가 → 검색 projection·session cache 무효화 → Audit 확인.

## J-06. 공개 문서 공유

Manage 보유자가 Published 문서에서 link 생성 → link 복사 → 익명 독자가 최신 Published
Version 열람 → 렌더링 파일만 scope-bound URL로 조회 → 소유자가 link 폐기 시 즉시 접근 종료.

공개 화면은 Workspace navigation, child tree, Draft, History, Discussion, Search, AI와
Backlink를 제공하지 않는다.

## J-07. 삭제와 복구

문서 휴지통 이동 → 30일 동안 복구 또는 영향 검토 → 영구 삭제 승인 → 본문 Version,
projection과 참조 제거 → 최소 삭제 Audit 보존. Workspace도 30일 유예 후 같은 원칙으로
tenant 전체를 제거한다.
