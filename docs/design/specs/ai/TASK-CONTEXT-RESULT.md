# AI Task, Context와 Result

- **문서 ID**: SPEC-13
- **상태**: 동결

## Registry

각 Task definition은 kind, input schema, allowed scope, context recipe, output JSON Schema,
timeout class, application policy와 evaluation set version을 가진다. runtime prompt 문자열로
정책을 결정하지 않는다.

## Context Builder

actor permission 확인 → explicit current scope → Discussion·Reference → Vocabulary·Writing Rule
→ Permission-safe Retrieval → optional external web 순으로 구성한다. 각 item은 authority,
provenance, freshness와 include reason을 가진다.

## Result kinds

- Compose/Rewrite/Merge: DocumentOperation[] + explanation + Source mapping
- Review: Finding[] `{ruleId,severity,region,reason,suggestion,sources}`
- Query: claims[] + sources + insufficient/conflict flags
- Discussion Apply: decision groups + Operation proposal

## Validation

JSON schema → source ID membership → Region resolve → Operation scope → content schema → policy
rule 순으로 검증한다. 실패 결과는 Draft에 적용하지 않고 redacted validation summary를 Job에
저장한다.

## Writing Rule precedence

Workspace explicit rule가 같은 ID의 baseline을 재정의한다. 강제 rule 위반 결과는 apply를
차단하고, 권고 rule은 finding만 만든다. rule version을 Result에 고정한다.
