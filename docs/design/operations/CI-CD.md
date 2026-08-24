# CI/CD

- **문서 ID**: OPS-02
- **상태**: 동결

## Pipeline

source checkout → secret/license scan → format/lint → contract generation diff → unit → container
integration → browser/a11y → security → performance smoke → reproducible image·SBOM → sign → staging
migration·deploy → acceptance → production approval → canary → full rollout.

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

이 저장소는 사용자 지시에 따라 main에서 작업한다. CI는 main의 모든 commit에 동일 gate를
적용하며 release artifact는 검증된 commit에만 tag한다.
