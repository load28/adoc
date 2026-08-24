# adoc — Git 기반 AI 협업 문서

여러 사람이 하나의 팀 문서를 Git으로 함께 발전시키고, AI가 인지부하를 고려해 문서를 구조화해주는 데스크톱 앱.
전체 설계는 [DESIGN.md](./DESIGN.md) 참고.

핵심 원칙:

- **Git Repository = Source of Truth** — 로컬 DB/인덱스는 언제든 삭제하고 재생성 가능한 캐시일 뿐이다.
- **개방 포맷** — 문서는 Markdown + Frontmatter로 저장되어 앱 없이도 읽을 수 있다.
- **AI 변경은 항상 제안** — 사람이 Diff를 검토하고 승인한다. AI는 commit/push하지 않는다.
- **로컬 Agent** — 서버 LLM API 대신 각자의 `claude` / `codex` CLI를 사용한다.

## 구조

```
packages/
├── core/        순수 TS 도메인 (환경 독립)
│                Document Model · Markdown+Frontmatter 파서/직렬화
│                Cognitive Writing Rules (C001–C010) · Document Intent
│                Prompt Compiler (WritingRequest → PromptIR)
│                Writing Engine (Composer/Rewriter/Critic/Merger)
│                AgentAdapter (claude/codex CLI, Mock) · FileSystem/Process 포트
├── git/         문서 중심 Git 레이어 (pull=최신 가져오기, push=팀에 공유,
│                log=History, conflict 3-way 추출)
├── indexer/     Local Index — 검색 · Document Graph · AI Context Retrieval
│                (.teamdoc/cache/, 삭제→재스캔→재생성 가능)
└── node-ports/  Node용 포트 구현 (CLI·테스트 공용)

apps/
├── cli/         adoc CLI — 동일한 도메인 패키지의 헤드리스 셸
└── desktop/     Tauri 2 + React 데스크톱 앱
                 (Rust는 allowlist 프로세스 러너 + 파일시스템만 담당,
                  도메인 로직 전부 webview의 TS에서 실행)
```

도메인 로직은 전부 순수 TypeScript이며 `FileSystemPort` / `ProcessRunnerPort` 두 개의 포트로만
바깥세상과 만난다. CLI는 Node로, 데스크톱은 Tauri(Rust) 커맨드로 같은 포트를 구현한다.
Rust 쪽은 `git` / `claude` / `codex` 세 바이너리만 실행할 수 있다.

## 워크스페이스 레이아웃 (DESIGN.md §4)

```
workspace/
├── workspace.yaml
├── projects/<id>/documents/*.md     # Markdown + Frontmatter
├── decisions/NNN-*.md
└── .teamdoc/
    ├── config.yaml                  # 기본 agent 등
    ├── writing-rules.yaml           # 팀 규칙 (인지 규칙 위에 병합)
    ├── document-types/*.yaml        # design/proposal/decision + 팀 확장
    └── cache/                       # 재생성 가능한 인덱스 (gitignored)
```

## 시작하기

```bash
npm install
npm run build        # 모든 패키지 + 앱 빌드
npm test             # vitest (core/git/indexer)
```

### CLI

```bash
alias adoc="node $(pwd)/apps/cli/dist/index.js"

adoc -C ~/team-docs init --name my-team --remote git@github.com:org/docs.git
adoc -C ~/team-docs new design Match Architecture -p compiler --author minmin

# AI 작성: Intent 분석 → 인지 규칙 적용 → Proposal(diff) → --apply로 수락
echo "지금 인증에서 토큰을 앱에서 관리하고 있는데 서버 세션으로 바꾸려고 한다..." |
  adoc -C ~/team-docs compose design --id auth-v2 -p app --apply

adoc -C ~/team-docs critique projects/app/documents/auth-v2.md   # Critic (분석만)
adoc -C ~/team-docs rewrite  projects/app/documents/auth-v2.md --goal "더 간결하게"
adoc -C ~/team-docs diff                    # 변경사항 보기
adoc -C ~/team-docs share -m "auth v2 초안" # 변경 기록 + 팀에 공유
adoc -C ~/team-docs sync                    # 최신 문서 가져오기
adoc -C ~/team-docs history projects/app/documents/auth-v2.md
adoc -C ~/team-docs merge <path> --ai       # 충돌 시 AI Merge Proposal
adoc -C ~/team-docs search 서버 세션
adoc -C ~/team-docs related auth-v2         # Document Graph
```

`--agent claude`(기본) / `--agent codex` 로 로컬 Agent를 선택한다.

### 데스크톱 앱 (Tauri + React)

사전 요구사항: Rust toolchain, 그리고 Linux는 `libwebkit2gtk-4.1-dev libgtk-3-dev` ([Tauri prerequisites](https://tauri.app/start/prerequisites/)).

```bash
cd apps/desktop
npm run tauri:dev      # 개발 실행
npm run tauri:build    # 배포 빌드
```

UI 구성: 좌측 문서 목록 · 중앙 Markdown 에디터 · 우측 패널

- **AI 탭** — Composer(메모 → 구조화 문서), Rewriter(더 간결하게/근거 강화/…), Critic(규칙 기반 분석)
- **History 탭** — 작성자·메시지 중심 문서 타임라인, 버전 Diff, 복원
- **Sync 탭** — 최신 가져오기 / 변경 기록+공유 / 충돌 해결(Current·Incoming·둘 다·AI 병합)

모든 AI 결과는 Diff 검토 모달을 거쳐 수락해야만 Working Tree에 적용되고,
AI Merge가 해결하지 못한 모순은 사람이 결정하도록 명시적으로 표시된다.

## 검증 상태

- `npm test` — 23개 테스트 통과 (파서 왕복, Prompt Compiler, Engine, Git 3-way 충돌, 인덱스 재생성)
- `cargo check` — Tauri 셸 컴파일 확인
- 실제 `claude` CLI로 compose(Intent 추출 → 초안) / critique(C00x 태그 제안) E2E 동작 확인
