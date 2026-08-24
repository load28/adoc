# TASK-003: 전체 제품 설계 문서 일괄 작성

- **상태**: 완료
- **유형**: 제품·설계
- **시작일**: 2026-08-24
- **완료일**: 2026-08-24
- **커밋**: —

## 목적

중요한 제품 방향성과 기술 결정을 사용자에게 먼저 확인한다. 모든 결정이 끝나면 참조
대화와 확정된 결정을 근거로 `docs/DOCUMENT-MAP.md`의 전체 설계 문서를 한 번에 작성하고
교차 검증한다.

## 범위

- 포함: 제품, 요구사항, UX, 아키텍처, ADR, 도메인 상세, 데이터, API, 보안, 개인정보,
  품질, 테스트, 운영, 구현 계획과 설계 동결 보고서
- 제외: 애플리케이션 코드, 인프라 생성, 외부 서비스 계정 변경, 배포

## 필수 설계 문서

- [x] 초기 통합 PRD와 도메인 기준선
- [x] 전체 구현 범위
- [x] 프로젝트 문서 지도
- [x] 중요한 제품 방향 결정 완료
- [x] 중요한 기술 결정 완료
- [x] 문서 지도의 전체 설계 문서 일괄 작성
- [x] 요구사항·설계·테스트 추적성 검증
- [x] 전체 설계 동결 보고서

## 문서 준비 게이트

- [x] 사용자 Decision Register의 차단 결정이 모두 확정됐다.
- [x] 참조 대화와 기존 기준선의 요구사항 누락이 없다.
- [x] 전체 문서를 동일한 결정 snapshot을 기준으로 작성했다.
- [x] 문서 간 정본 경계, 링크, 용어, 상태와 계약이 일치한다.
- [x] 코드 작성 가능 여부를 Design Freeze Report가 판정했다.

## 사용자 결정

방향·기술 결정은 여러 회차로 나누어 질문할 수 있지만, 설계 본문은 모든 답이 끝난 뒤
한 번에 작성한다. 질문은 상황·영향·권장안을 포함하고 사용자의 답을 이 절과
`product/DECISION-REGISTER.md`에 기록한다.

### 결정 묶음 A: 제품 운영 경계

- **상태**: 완료
- **대상**: 배포·tenant 모델, 최초 사용자 범위, 계정·인증 정책
- **사용자 결정**:
  - 2026-08-24: Workspace를 tenant·보안 경계로 하는 다중 조직 SaaS로 운영한다.
  - 2026-08-24: 누구나 계정과 Workspace를 만들 수 있고 기존 Workspace는 초대로 가입한다.
  - 2026-08-24: 사용자 인증은 Google SSO를 사용한다.
  - 2026-08-24: 모든 Google 계정을 허용하고 Google Workspace domain 제한은 두지 않는다.

### 결정 묶음 B: 핵심 기술 스택

- **상태**: 완료
- **대상**: web framework, language/runtime, database, editor engine, search·vector,
  object storage, queue·worker
- **사용자 결정**:
  - 2026-08-24: 프론트엔드는 TanStack Start로 구성하고 SSR과 CSR을 혼합한다.
  - 2026-08-24: 백엔드는 Rust로 구성한다.
  - 2026-08-24: 프론트엔드와 백엔드는 하나의 monorepo에서 관리한다.
  - 2026-08-24: Docker 배포를 지원한다.
  - 2026-08-24: 실시간 통신 기능은 Rust 백엔드가 제공한다.
  - 2026-08-24: Editor는 Tiptap Core·ProseMirror를 사용한다.
  - 2026-08-24: 실시간 공동 편집이 비목표이므로 Tiptap Collaboration·Yjs는 사용하지 않는다.
  - 2026-08-24: 애플리케이션 정본 데이터베이스는 PostgreSQL을 사용한다.
  - 2026-08-24: Rust HTTP·실시간 서버는 Axum·Tokio·Tower로 구성한다.
  - 2026-08-24: PostgreSQL 접근 계층은 ORM 없이 SQLx를 사용한다.
  - 2026-08-24: 검색은 PostgreSQL 정본과 분리된 OpenSearch projection으로 구성하고
    keyword·semantic hybrid search를 제공한다.
  - 2026-08-24: FileAsset은 로컬 저장 구현으로 시작한다. ObjectStorage 경계를 두어
    AWS S3 구현으로 교체할 수 있게 한다.
  - 2026-08-24: 비동기 작업 Queue는 Redis를 사용한다.
  - 2026-08-24: 실시간 전달은 HTTP command와 SSE를 사용하며 WebSocket은 사용하지 않는다.

### 결정 묶음 C: 제품 정책

- **상태**: 완료
- **대상**: Group·Permission precedence, Manage, Review·Publish, AI 적용 승인 경계,
  파일·Audit 보존
- **사용자 결정**:
  - 2026-08-24: Group 생성·멤버 관리·Group Permission을 전체 구현에 포함한다.
  - 2026-08-24: Permission은 가장 가까운 Document의 명시적 Grant를 먼저 사용한다.
    같은 위치에서는 개인 Grant가 Group Grant보다 우선한다. 개인 Grant가 없으면 Group의
    `NO_ACCESS`가 우선하고, deny가 없으면 가장 높은 access를 사용한다.
  - 2026-08-24: `Manage`는 최소 `EDITOR` access를 요구한다.
  - 2026-08-24: Workspace Admin도 Document 내용 access를 우회하지 않는다.
  - 2026-08-24: Workspace 기본 PublishPolicy는 직접 발행으로 한다. 문서 트리별로
    `REVIEW_REQUIRED`와 필수 승인 인원을 상속·재정의하며 Draft 내용 변경은 기존 승인을
    모두 무효화한다.
  - 2026-08-24: 현재 Region의 제한적 AI 수정은 즉시 적용하고 Undo를 제공한다. 여러
    Region이나 문서 전체를 바꾸는 작업은 Proposal과 Diff를 제시한 뒤 사용자 승인을 받아
    적용한다.
  - 2026-08-24: Document는 휴지통에서 30일간 복구할 수 있다. 영구 삭제 시 본문 Version,
    검색 projection과 FileAsset reference를 제거하고 Audit에는 삭제 사실과 비민감
    식별자만 보존한다. Workspace도 30일 유예 후 전체 데이터를 제거한다.

### 결정 묶음 D: AI·운영 정책

- **상태**: 완료
- **대상**: CLI provider와 실행 환경, 동시성·비용, AI Context·외부 지식,
  한국어 Writing Rule, SLO·백업·배포
- **사용자 결정**:
  - 2026-08-24: 로컬·자체 호스팅에서는 Codex CLI 구독 인증을 지원한다. 다중 사용자
    운영 환경에서는 Provider 인터페이스 뒤에서 OpenAI Responses API를 사용하며 개인
    구독 인증을 공용 서버에 저장하지 않는다.
  - 2026-08-24: AI Context는 기본적으로 권한이 확인된 Workspace 지식으로 제한한다.
    외부 웹 검색은 사용자가 작업별로 명시적으로 활성화해야 하며 외부 자료와 모델 일반
    지식을 출처 없이 문서 사실로 적용하지 않는다.
  - 2026-08-24: 조직 용어·금칙어·근거 정확성은 강제 Writing Rule로 적용한다. 문체·
    가독성·표현 선호는 이유를 표시하는 권고 Rule로 적용한다. 기본 Rule은 versioned
    baseline으로 관리하고 Workspace가 추가·재정의할 수 있게 한다.
  - 2026-08-24: 로컬·단일 서버용 Docker Compose를 완전 지원한다. 운영 환경은 동일한
    container를 수평 확장할 수 있게 설계하되 특정 Cloud나 Kubernetes 배포 파일에는
    고정하지 않는다.
  - 2026-08-24: 핵심 문서 읽기·쓰기의 월간 가용성 SLO는 99.9%, RPO는 15분, RTO는
    4시간으로 한다. Search와 AI 장애는 핵심 문서 기능에서 격리한다.
  - 2026-08-24: 사용자 대화형 AI 작업을 background 작업보다 우선한다. Workspace·
    사용자별 동시 실행 한도와 관리자 예산을 두고 대기·취소 상태와 사용량을 공개한다.
    Provider 실패 시 다른 모델로 자동 전환하지 않는다.

### 결정 묶음 E: UX 방향

- **상태**: 완료
- **대상**: 기기별 기능 범위, UI 지원 언어, 시각 디자인·theme·접근성
- **사용자 결정**:
  - 2026-08-24: Desktop·tablet·mobile에서 동일한 핵심 기능을 제공한다. 화면 밀도와
    조작 방식만 기기에 맞추고 입력 장치 의존 기능에는 동등한 대체 수단을 제공한다.
  - 2026-08-24: 첫 구현부터 한국어와 영어 UI를 지원한다. 사용자별 locale·timezone을
    적용하고 UI 문구는 국제화하며 문서 본문 언어는 제한하지 않는다.
  - 2026-08-24: 별도 디자인 시스템을 만들지 않는다. Apache-2.0으로 공개 배포된
    `@atlaskit` React package, design token과 primitive를 유일한 UI 체계로 사용한다.
    제품 고유 UI는 이를 조합하되 자체 token이나 병행 component library를 만들지 않는다.
    Atlassian 상표·logo·전용 font와 비공개 asset은 사용하지 않는다. Light·Dark·System
    theme과 WCAG 2.2 AA를 지원한다.

### 결정 묶음 F: 제품 외부 경계

- **상태**: 완료
- **대상**: 익명·Guest 공유, 결제·요금제, 고객용 Public API·Webhook
- **사용자 결정**:
  - 2026-08-24: 사용자가 명시적으로 공유한 단일 Document의 최신 Published Version만
    익명 Viewer link로 제공한다. 해당 문서 렌더링에 필요한 File만 link scope 안에서
    제공한다. Document tree, Draft, History, Discussion, Search, AI, Backlink와 Workspace
    정보는 공개하지 않는다.
  - 2026-08-24: AI 사용량과 관리자 예산 통제는 포함하되 결제·요금제·구독 관리는 제품
    범위에서 제외한다.
  - 2026-08-24: 고객용 Public API와 Webhook은 제외하고 웹 애플리케이션 내부 API만
    구현한다. 향후 외부 계약은 기존 Domain Command·Event 의미를 재사용한다.

## 의사결정

### 결정 1: 전체 문서를 결정 완료 후 한 번에 작성한다

- **상황**: 파일별로 문서를 순차 작성하면 뒤의 기술 결정 때문에 앞 문서를 반복 수정하고
  서로 다른 결정 snapshot이 섞일 수 있다.
- **검토한 대안**: 문서별 점진 작성 / 결정 선행 후 전체 일괄 작성.
- **선택과 근거**: 사용자의 요청에 따라 중요한 결정을 먼저 모두 수집한다. 그 뒤 전체
  문서를 한 번에 작성하고 통합 검증한다.

### 결정 2: 중요한 결정만 사용자에게 묻고 나머지는 기존 대화와 원칙으로 완성한다

- **상황**: 모든 세부사항을 질문하면 결정 비용이 커지고 전체 작성이 지연된다.
- **검토한 대안**: 모든 항목 확인 / 에이전트 임의 확정 / 중요한 방향·기술만 사용자 확인.
- **선택과 근거**: 제품 범위, 사용자 경험의 핵심 정책, 보안·데이터 수명주기와 기술
  선택은 사용자에게 확인한다. 그 외 세부 계약은 참조 대화와 구조적 원칙으로 작성한다.

## 작업 내역

- 2026-08-24: 전체 설계 문서 일괄 작성 태스크를 등록했다.
- 2026-08-24: 사용자 결정 영역을 제품 운영, 기술 스택, 제품 정책, AI·운영으로 분류했다.
- 2026-08-24: DEC-008~010의 사용자 결정을 기록하고 Google 계정 범위를 DEC-011로 분리했다.
- 2026-08-24: DEC-011~013의 사용자 결정을 기록했다.
- 2026-08-24: PostgreSQL 사용 결정을 DEC-014로 기록했다.
- 2026-08-24: Rust server, SQLx, OpenSearch 결정을 DEC-015~017로 기록했다.
- 2026-08-24: File storage, Redis Queue, SSE 결정을 DEC-018~020으로 기록했다.
- 2026-08-24: Group·Permission·Manage 정책을 DEC-021~023으로 기록했다.
- 2026-08-24: Review·Publish, AI 적용, 삭제·보존 정책을 DEC-024·004·007로 기록했다.
- 2026-08-24: AI Runtime, 외부 지식, 한국어 Writing Rule 정책을 DEC-005·025·006으로 기록했다.
- 2026-08-24: 배포, SLO·복구, AI 용량·비용 정책을 DEC-026~028로 기록했다.
- 2026-08-24: 기기별 UX, 국제화, Atlaskit UI 정책을 DEC-029~031로 기록했다.
- 2026-08-24: 공개 Viewer link, 결제 제외, Public API 제외를 DEC-032~034로 기록했다.
- 2026-08-24: 제품·UX·아키텍처·ADR·도메인 상세·데이터·API·보안·품질·운영·구현
  문서 94개 ID를 같은 결정 snapshot으로 일괄 작성했다.
- 2026-08-24: PRD를 분리 정본 인덱스로 전환하고 RQ-01~20 추적 표를 작성했다.
- 2026-08-24: Design Freeze Report에서 전체 문서를 구현 가능한 상태로 판정했다.

## 이슈 및 해결

없음.

## 검증

- [x] 문서 지도 94개 ID가 각각 한 문서에 선언되고 모두 `동결`인지 확인
- [x] Markdown 내부 link와 문서 지도 file reference 존재 확인
- [x] RQ-01~20의 제품→설계→test 추적 확인
- [x] Permission, PublicLinkScope, revision, event·job·retention 계약 교차 확인
- [x] OpenAPI·AsyncAPI YAML parse 확인
- [x] OpenAPI 내부 `$ref` 181개 해석과 operationId 30개 중복 없음 확인
- [x] `git diff --check`

## 결과

전체 제품 설계와 검증을 완료했다. 애플리케이션 코드는 변경하지 않았다. 다음 작업은
TASK-004 구현 태스크를 등록하고 PLAN-02의 전체 구현 DAG를 수행하는 것이다.
