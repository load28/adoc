# CI/CD

- **문서 ID**: OPS-02
- **상태**: 동결

## Pipeline

source checkout → secret/license scan → format/lint → contract generation diff → unit → container
integration → browser/a11y → security → performance smoke → reproducible image·SBOM → sign → staging
migration·deploy → acceptance → production approval → canary → full rollout.

## 실행 정책

로컬 전체 CI의 단일 진입점은 `bun run ci:local`이다. 이 명령은 repository gate, Rust·JavaScript
의존성 감사, provenance·production-readiness, 실제 Compose acceptance와 browser/a11y를 순서대로
실행한다. 일부 단계를 실행한 결과를 전체 CI 성공으로 표현하지 않는다.

GitHub Actions workflow는 `workflow_dispatch`로 운영자가 명시 호출할 때만 실행한다. `push`,
`pull_request`, `schedule`과 다른 자동 event trigger를 두지 않는다. 수동 원격 실행도 로컬 전체
CI와 같은 gate 집합을 검증하며, runner bootstrap 차이만 workflow가 소유한다.

공급망 계약 검사는 workflow trigger와 로컬 명령의 필수 단계 집합을 구조적으로 검사한다.
로컬 또는 원격 실행 중 어느 단계든 실패하면 전체 CI가 실패하며 다음 gate를 성공으로 간주하지
않는다. 재실행은 운영자가 원인을 해결한 뒤 같은 진입점을 다시 명시 호출한다.

## Artifact

web/API/worker image는 immutable digest, git SHA, schema min/max, OpenAPI·event version과 SBOM을
label로 가진다. production에서 source build하지 않는다.

## Migration gate

expand migration을 deploy 전에 수행하고 compatibility query를 실행한다. destructive contract는
별도 release와 backup restore evidence를 요구한다.

## Rollback

application image rollback이 current schema를 읽을 수 있는지 manifest로 검사한다. migration을
무조건 down하지 않는다. canary error budget·invariant alert 초과 시 자동 rollout stop 후
operator가 rollback을 승인한다.

## Branch

이 저장소는 사용자 지시에 따라 main에서 작업한다. commit과 push는 CI를 자동 실행하지 않는다.
release artifact는 해당 main SHA에서 `bun run ci:local`과 수동 GitHub Actions가 모두 통과한
commit에만 tag한다.
