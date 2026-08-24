# Reference와 Vocabulary

- **문서 ID**: SPEC-11
- **상태**: 동결

## Reference

CreateReference는 source owner permission과 target existence·visibility를 확인한다. target
kind별 stable ID와 표시 snapshot을 저장한다. delete는 source owner command에서 수행한다.

Backlink query는 targetId와 PermissionScope로 references를 역조회한다. source 권한이 없는
row의 count도 반환하지 않는다.

## Vocabulary state

`ACTIVE → DEPRECATED`; deprecated Concept는 replacementConceptId를 가질 수 있다. hard delete는
reference가 없고 audit retention 조건을 만족할 때만 가능하다.

## Term normalization

Unicode NFC, locale-aware case folding과 whitespace collapse를 적용한다. canonical·alias·
deprecated term은 Workspace에서 normalized unique다. display original은 보존한다.

## Commands

CreateConcept, UpdateDefinition, AddAlias, RemoveAlias, DeprecateConcept, ReplaceConcept. AI는
이 command port를 받지 않고 proposal만 반환한다.
