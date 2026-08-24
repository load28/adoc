# Knowledge 요구사항

- **문서 ID**: PROD-14
- **상태**: 동결

## Reference와 Backlink

Reference는 Source와 Target의 stable identity를 저장한다. Target은 Document, Region,
Discussion, Vocabulary Concept 또는 External Resource다. Backlink는 Reference 역조회이며
별도 진실 소스로 저장하지 않는다.

## Vocabulary

Workspace Concept는 canonical term, definition, alias와 deprecated term을 가진다. 이름
충돌과 alias cycle을 금지한다. 사람만 생성·변경·폐기하며 AI는 Proposal만 만든다.

## Search와 Retrieval

PostgreSQL이 정본이고 OpenSearch는 rebuild 가능한 projection이다. lexical·semantic
candidate를 결합하고 Permission Scope 안에서 ranking한다. 사용자 Search, AI Context와
Knowledge Query가 동일한 Retrieval port를 사용한다.

## Source provenance

모든 Knowledge Unit은 workspaceId, documentId, publishedVersion 또는 draftRevision,
regionId, kind와 projection revision을 가진다. AI 결과는 사용한 Unit의 stable identity와
표시 snapshot을 보존한다.

## 외부 지식

기본 Context는 Workspace 지식뿐이다. 사용자가 작업별로 외부 web을 활성화한 경우에만
가져오며 URL, title, retrievedAt과 excerpt hash를 Source로 남긴다. 읽지 못한 내용을
일반 지식으로 보충해 조직 사실처럼 제시하지 않는다.
