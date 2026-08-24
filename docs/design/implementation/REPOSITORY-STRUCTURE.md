# Repository Structure

- **문서 ID**: PLAN-01
- **상태**: 동결

```text
/
├─ apps/
│  ├─ web/                 # TanStack Start React 18.2 + Vite
│  ├─ api/                 # Axum HTTP·SSE binary
│  └─ worker/              # async worker binary
├─ crates/
│  ├─ kernel/
│  ├─ identity/ governance/ document/ collaboration/
│  ├─ knowledge/ writing_intelligence/ operations/
│  ├─ application/ ports/ adapters/
│  └─ telemetry/ test_support/
├─ packages/
│  ├─ contracts/           # generated OpenAPI·event clients
│  ├─ editor-schema/       # Tiptap schema·Operation codec
│  ├─ ui-domain/           # Atlaskit-composed domain UI
│  └─ i18n/                # ko/en ICU resources
├─ infra/
│  ├─ docker/ migrations/ opensearch/ observability/
├─ docs/
└─ Cargo.toml, package.json, workspace config
```

## 경계 규칙

- `crates/*domain*`은 adapters·transport를 import하지 않는다.
- `apps`는 wiring만 하고 domain rule을 소유하지 않는다.
- `packages/contracts`는 생성물이며 수동 편집하지 않는다.
- `ui-domain`은 design system을 재정의하지 않고 public Atlaskit을 직접 조합한다.
- SQL은 owner adapter에, migration은 `infra/migrations`에 둔다.

## Naming

Rust module은 domain noun, command handler는 동사+noun, event는 과거형을 쓴다. TypeScript의
transport type과 domain view model을 구분한다. `utils`, `common`, `misc`에 domain 의미를
숨기지 않는다.

## Dependency gate

Cargo deny와 package dependency rule로 forbidden edge를 CI에서 검사한다. cycle을 feature
flag나 re-export로 숨기지 않는다.
