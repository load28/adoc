# Environments와 Configuration

- **문서 ID**: OPS-01
- **상태**: 동결

## 환경

local, test, staging, production을 분리한다. production data·credential을 non-production으로
복사하지 않는다. staging은 topology·migration·provider contract가 production과 같아야 한다.

## Config 분류

- build: artifact version, enabled locale, public asset base
- runtime non-secret: limits, timeout, SLO threshold, feature availability
- secret: OIDC, session signing, DB, provider, backup encryption
- Workspace setting: policy, Writing Rule, AI budget

환경변수는 secret reference와 bootstrap location만 전달하고 typed config가 startup에서 전체
검증한다. unknown·missing key는 fail fast다.

## Feature flag

미완성 domain invariant를 flag로 우회하지 않는다. flag는 rollout·dependency availability에만
쓰며 owner, expiry와 removal task를 가진다.

## Docker Compose

profile은 core, search, ai-local, observability, backup이다. volume ownership, healthcheck,
resource limit과 one-shot migration job을 선언한다.
