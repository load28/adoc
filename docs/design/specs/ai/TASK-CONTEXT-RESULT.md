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

Context Inspector는 같은 Builder로 preview를 만들고 canonical artifact fingerprint를 반환한다.
Job admission은 동일한 Task·Source 선택으로 current Context를 다시 구성해 fingerprint가 같을
때만 snapshot을 저장한다. preview session을 별도 정본으로 저장하지 않는다.

Source 본문 snapshot은 generic Job payload가 아니라 AI Context 전용 저장소에 보존한다. preview
응답은 본문을 반환하지 않고 identity, authority, 표시 snapshot, include reason과 omission만
반환한다. external web fetch adapter가 없으면 opt-in 요청도 명시적으로 거부한다.

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
