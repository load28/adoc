# 사용자와 이해관계자

- **문서 ID**: PROD-03
- **상태**: 동결

## 사용자 유형

| 유형 | 책임 | 주요 목표 |
|---|---|---|
| Member | 허용된 지식 열람·협업 | 정확한 문서를 찾고 기여 |
| Editor | Draft 작성·편집 | 생각과 토론을 문서로 수렴 |
| Reviewer | 특정 revision 검토 | 발행 적합성 판단 |
| Workspace Admin | 멤버·Group·정책·운영 설정 | 보안 경계와 조직 정책 유지 |
| Public Viewer | 공유 링크의 단일 Published 문서 열람 | 인증 없이 명시된 결과만 읽기 |

Workspace 역할과 Document access는 별도 축이다. Admin도 Document 내용 권한을 우회하지
않는다. Reviewer는 고정 역할이 아니라 Review 요청 시 지정되는 책임이다.

## 접근 수준

`NO_ACCESS < VIEWER < CONTRIBUTOR < EDITOR` 순으로 누적된다. `Manage`는 별도 capability지만
최소 `EDITOR`가 필요하다. Public Viewer는 Membership과 무관한 제한된 link principal이며
일반 `VIEWER`로 승격되지 않는다.

## 이해관계자

- 조직 지식 책임자: PublishPolicy, Vocabulary와 Writing Rules의 신뢰성
- 보안·운영 담당자: tenant 격리, Audit, 복구와 incident 대응
- 개발·지원 담당자: 설계-코드 추적성, 관측 가능하고 재현 가능한 오류
- 외부 독자: 공개가 명시된 최신 Published Version의 안정적 열람

## 국제화

UI는 한국어와 영어를 제공한다. 사용자별 locale, timezone과 날짜 형식을 적용한다.
문서 본문 언어는 제한하지 않는다.
