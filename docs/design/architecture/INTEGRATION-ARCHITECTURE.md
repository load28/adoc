# Integration Architecture

- **문서 ID**: ARCH-07
- **상태**: 동결

## AI Runtime port

```text
execute(AIRequest, EventSink, Cancellation) -> RuntimeResult
health() -> ProviderHealth
capabilities() -> ModelCapabilities
```

AIRequest는 rendered prompt가 아니라 task kind, sanitized Context artifact, output JSON Schema,
timeout과 provider profile을 가진다. Runtime은 application DB·ObjectStorage credential을 받지
않는다.

## Codex CLI adapter

local/self-hosted trusted host에서 non-interactive child process로 실행한다. per-job temp dir,
read-only input, no application repository, disabled network 기본, stdout size limit와 process
group termination을 적용한다. subscription credential은 host operator가 관리한다.

## OpenAI adapter

managed multi-user 환경에서 service credential을 secret store에서 주입한다. structured output,
streaming, timeout과 usage를 adapter가 표준 Runtime event로 변환한다. provider request ID는
암호화하지 않은 prompt와 분리해 저장한다.

## External web

task opt-in이 있을 때 allowlisted fetch service를 통해 가져온다. redirect, private IP, size,
MIME와 timeout을 제한해 SSRF를 막는다. retrieved content는 untrusted Source로 표시하며
instruction으로 실행하지 않는다.

## ObjectStorage

`put`, `complete`, `openAuthorized`, `delete`, `stat` port를 제공한다. local path와 S3 key는
domain 밖 adapter detail이다.

## Google OIDC

OIDC discovery, authorization code+PKCE, state·nonce와 exact redirect URI를 사용한다. Google
access token은 로그인 완료 뒤 보존하지 않는다.
