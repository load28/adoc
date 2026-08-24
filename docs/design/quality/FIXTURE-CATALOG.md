# Deterministic Fixture Catalog

- **문서 ID**: TEST-07
- **상태**: 동결

모든 UUID는 fixture namespace에서 고정하고 clock은 `2026-01-15T09:00:00Z`다. password, OAuth
token, 실제 조직 문서와 provider credential은 포함하지 않는다.

## Identity·Workspace

| ID | 이름 | 상태·role | 목적 |
|---|---|---|---|
| U-OWNER | `00000000-0000-7000-8000-000000000001` | Alpha Owner | 설정·삭제·last owner |
| U-ADMIN | `...0002` | Alpha Admin | governance, content 무권한 검증 |
| U-EDITOR | `...0003` | Alpha Member | personal EDITOR grant |
| U-CONTRIB | `...0004` | Alpha Member | Group CONTRIBUTOR |
| U-VIEWER | `...0005` | Alpha Member | inherited VIEWER |
| U-DENIED | `...0006` | Alpha Member | Group NO_ACCESS |
| U-OUTSIDE | `...0007` | Beta Owner | cross-tenant 비노출 |
| W-ALPHA | `10000000-0000-7000-8000-000000000001` | ACTIVE | 전체 기능 fixture |
| W-BETA | `10000000-0000-7000-8000-000000000002` | ACTIVE | tenant isolation |

`...000N` 표기는 실제 fixture 파일에서는 위 UUID prefix의 마지막 12자리를 0 padding한 완전한
UUID를 뜻한다. builder는 축약 문자열을 허용하지 않는다.

## Tree·permission

```text
D-ROOT "Engineering"                 GROUP all=VIEWER
├─ D-SEC "Security"                  GROUP denied=NO_ACCESS
│  └─ D-AUTH "Authentication"        USER editor=EDITOR+manage
├─ D-API "API Guidelines"            GROUP writers=CONTRIBUTOR
└─ D-EMPTY "Unpublished Draft"        USER contrib=CONTRIBUTOR
```

Groups는 `all={VIEWER,DENIED,CONTRIB,EDITOR}`, `writers={CONTRIB}`, `denied={DENIED}`다.
D-AUTH에서 U-EDITOR User grant가 같은 depth의 Group deny보다 우선한다. U-ADMIN은 D-ROOT에
grant가 없어 Document content를 볼 수 없는 별도 variant도 제공한다.

## Content·version

| Fixture | 내용 |
|---|---|
| C-EMPTY-v1 | empty paragraph 한 개, schemaVersion 1 |
| C-FULL-v1 | heading, paragraph marks, quote, callout, lists, code, table, toggle, divider, image, file |
| C-KO-v1 | canonical term `인증`, prohibited synonym `로그인 인증 처리` 포함 |
| C-DEEP-INVALID | limit+1 depth, contract rejection |
| C-DUPLICATE-ID-INVALID | 두 Block의 같은 UUID |
| C-XSS-IMPORT | script/event attr/unsafe URL을 포함한 import input |

D-AUTH는 Published V1(C-FULL), Published V2(C-KO), V2 기반 Draft revision 7을 가진다. Draft
revision 7의 open Review R-AUTH-7과 approvals 1개, READY File F-IMAGE와 F-PDF reference가 있다.

## Concurrency·AI·File

| Fixture | 고정 상태 |
|---|---|
| L-AUTH | U-EDITOR holder, expiry `09:05:00Z`, revision 2, token hash만 DB에 저장 |
| P-AUTH-7 | base revision 7, 독립 op 2개+dependency op 1개, OPEN |
| P-AUTH-6 | stale base revision 6 |
| J-READY | QUEUED AI Job, permitted Source 3개, external web false |
| J-CONFLICT | conflicting internal Source 2개 |
| F-IMAGE | READY png, Published V2와 Draft가 함께 참조 |
| F-ORPHAN | DELETED, reference 0, purge_after가 clock 이전 |
| F-LIVE | DELETED 요청과 동시에 새 Draft reference barrier |

AI provider fixture는 chunk 순서, usage와 schema-valid/invalid result를 고정한다. Search fixture는
같은 lexical/vector 점수와 permission scope를 가져 tie-break가 stable ID로 결정되게 한다.

## Builder contract

`FixtureClock`, `FixtureIds`, `WorkspaceBuilder`, `DocumentBuilder`, `ContentBuilder`,
`EventBarrier`만 test data를 만든다. builder는 schema.sql을 우회하지 않고 public repository
interface 또는 명시적 seed transaction을 사용한다. scenario 종료 시 Workspace ID 기준으로
격리된 row와 object prefix를 제거한다.

## Contract corpus files

`fixtures/*.valid.json`은 해당 이름의 JSON Schema를 통과하고 `*.invalid.json`은 반드시
실패해야 한다. schema compile과 positive/negative validation을 모두 통과해야 fixture 변경을
merge할 수 있다. fixture 기대 결과를 validator 오류에 맞춰 바꾸지 않고 정본 계약 변경으로만
갱신한다.
