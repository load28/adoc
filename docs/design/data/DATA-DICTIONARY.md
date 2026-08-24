# Data Dictionary

- **문서 ID**: DATA-03
- **상태**: 동결

## 공통 field

| Field | Type | 규칙 |
|---|---|---|
| id | UUIDv7 | client 의미 없음, opaque |
| workspace_id | UUIDv7 | tenant row 필수, 변경 불가 |
| revision | bigint | 0 이상, 성공 mutation마다 +1 |
| created_at/updated_at | timestamptz | server UTC |
| status | enum | 정의된 transition만 허용 |
| rank | text | sibling lexical ordering, rebalance 가능 |
| schema_version | integer | serialized payload reader 선택 |
| token_hash | bytea | 원 token 미저장, constant-time compare |

## Content

`content_json`은 `{schemaVersion, root:{type:'doc', children}, blockIndex}` 구조다. Block ID는
UUIDv7 string이고 document 안에서 unique다. text mark는 allowlisted kind와 validated attr만
가진다. 최대 크기와 node depth는 shared ContentLimits로 검증한다.

## Permission

`subject_kind`: USER|GROUP. `access`: NO_ACCESS|VIEWER|CONTRIBUTOR|EDITOR. `manage`는 boolean.
Effective result는 저장하지 않고 resolver output cache만 둘 수 있다.

## Snapshot

snapshot JSON은 표시 재현에 필요한 최소 title·version·region excerpt hash를 가진다. 현재
entity의 권한을 우회하는 content cache로 사용하지 않는다.

## 민감도

- Restricted: content, Message, AI Context·Result, file bytes, email
- Confidential metadata: title, Reference graph, audit before·after
- Operational: opaque ID, status, timing, size, error code

Restricted와 Confidential은 application log·analytics에서 원문을 금지한다.

## Nullability

상태에 따라 필수인 값은 nullable+암묵 규칙으로 두지 않고 check constraint를 둔다. 예:
`status='PUBLISHED'` 같은 중복 상태는 없고 current_version_id 존재로 표현한다.
