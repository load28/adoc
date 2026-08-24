# TASK-017: Document Tree·Draft·Lease 구현

- **상태**: 완료
- **유형**: 구현
- **구현 패키지**: IMP-10
- **시작일**: 2026-08-25
- **완료일**: 2026-08-25
- **커밋**: —

## 목적

문서 identity와 계층을 안전하게 생성·변경하고, 단일 active Draft와 server-time Edit Lease 아래에서
IMP-09 Operation batch를 원자 저장하는 Document core를 구현한다. 권한 판정부터 PostgreSQL lock,
idempotency, Review 무효화와 outbox까지 하나의 command 경계로 고정한다.

## 범위

- 포함: tree 조회·생성·rename·move preview/commit·reorder·trash·restore, fractional rank와 rebalance,
  Draft 생성·조회·Operation 저장, Lease acquire·renew·release·강제 takeover, PostgreSQL adapter,
  application service·HTTP 경계, idempotency·outbox·race integration test
- 제외: permanent purge 실행(IMP-16), Publish·Version·public link(IMP-11), Review 생성·승인(IMP-13),
  Reference 정본 반영(IMP-14), SSE 전송 runtime(IMP-17), Web editor와 local recovery UI(IMP-23)

## 필수 설계 문서

- [x] 제품·도메인: `product/features/DOCUMENT-LIFECYCLE.md`, `domain/document-system.md`
- [x] 상태·알고리즘: `design/specs/document/DOCUMENT-TREE.md`, `design/specs/document/DRAFT-LEASE.md`,
  `design/specs/STATE-TRANSITION-CATALOG.md`, `design/specs/ALGORITHM-CATALOG.md`
- [x] 데이터: `design/data/schema.sql`, `design/data/DATABASE-INVARIANT-TRANSACTION-MATRIX.md`,
  `design/data/DATA-DICTIONARY.md`, `design/data/LIFECYCLE-RETENTION.md`
- [x] API·이벤트: `design/api/openapi.yaml`, `design/api/asyncapi.yaml`,
  `design/contracts/event-payloads.schema.json`, `design/contracts/document-operation.schema.json`
- [x] 권한·보안: `design/security/AUTHORIZATION.md`,
  `design/security/THREAT-MODEL.md`, `design/implementation/PERMISSION-PUBLISH-POLICY.md`
- [x] 품질: `design/quality/TEST-STRATEGY.md`, `design/quality/CONTRACT-COVERAGE.md`
- [x] 구현 기준: `design/implementation/DOCUMENT-TREE-DRAFT-LEASE.md`

## 문서 준비 게이트

- [x] 구현 결과를 바꿀 미해결 질문이 없다.
- [x] tree·Draft·Lease 정상·예외·권한·동시성 흐름이 정의되어 있다.
- [x] API·DB·event·idempotency 계약이 타입과 lock 순서 수준으로 정의되어 있다.
- [x] IMP-08·09와 IMP-11·13·14·16·17·23의 책임 경계를 추적할 수 있다.
- [x] barrier race와 transaction rollback을 포함한 테스트 완료 조건이 정의되어 있다.

## 사용자 결정

사용자는 전체 제품 구현과 권장안의 자율 적용을 확정했다.

## 의사결정

### 결정 1: Tree 순서는 고정 폭 base-62 rank로 표현한다

- **상황**: 가변 길이 문자열 rank는 DB collation과 언어별 구현 차이가 있으면 순서가 달라질 수 있다.
- **검토한 대안**: 연속 integer 재번호 / 가변 LexoRank / C collation 고정 폭 base-62.
- **선택과 근거**: 32자리 base-62와 `COLLATE "C"`를 사용한다. 두 anchor 중간값이 없을 때만 해당
  sibling 집합을 균등 재배치해 reorder write와 lock 범위를 제한한다.

### 결정 2: Move preview는 PostgreSQL의 일회용 capability로 저장한다

- **상황**: 서명만 한 preview token은 권한·정책과 sibling anchor 변화 뒤에도 별도 서버 상태 없이 재사용될 수 있다.
- **검토한 대안**: stateless HMAC / Redis TTL / PostgreSQL token hash row.
- **선택과 근거**: 원문 32-byte token은 한 번만 반환하고 SHA-256만 DB에 5분 보존한다. commit이 row를
  lock·소비하고 모든 fingerprint를 다시 검사해 PostgreSQL transaction과 동일한 신뢰 경계를 유지한다.

### 결정 3: Lease를 사용자와 Browser client instance에 함께 결합한다

- **상황**: 사용자 ID만 저장하면 같은 사용자의 다른 탭이 기존 holder인지 구분할 수 없다.
- **검토한 대안**: user 단위 공유 / token만 사용 / user+client instance+token 결합.
- **선택과 근거**: `holder_user_id`, `client_instance_id`, token hash, lease revision을 모두 일치시킨다.
  acquire·force takeover는 token을 회전하고 renew만 같은 token의 TTL을 연장한다.

### 결정 4: 명시적 trash root만 상태를 가지며 descendant는 ancestry로 숨긴다

- **상황**: subtree 전체 row의 상태를 바꾸면 큰 tree에서 lock·write amplification이 발생하고 중첩 trash 의미가 흐려진다.
- **검토한 대안**: descendant 상태 일괄 변경 / 별도 closure table / root tombstone+recursive ancestry.
- **선택과 근거**: command target만 TRASHED로 바꾸고 descendant의 effective state는 trashed ancestor CTE로 계산한다.
  restore와 purge도 명시적 root를 기준으로 하며 descendant lease·active Review만 같은 transaction에서 종료한다.

## 구현 순서

1. 기존 tree·Draft·Lease API, DDL, 상태 전이와 permission 계약의 공백·충돌을 감사한다.
2. IMP-10 상세 구현 계약과 필요한 정본 계약을 먼저 갱신한다.
3. domain model·application port·PostgreSQL transaction을 구현한다.
4. HTTP adapter와 generated contract를 연결한다.
5. unit·PostgreSQL barrier race·idempotency·root gate를 실행한다.
6. 완료 기록 후 commit·push하고 IMP-11로 진행한다.

## 작업 내역

- 2026-08-25: IMP-10 태스크를 등록하고 선행 설계 문서 집합을 식별했다.
- 2026-08-25: PLAN-16에 tree rank·watermark, one-time move preview, Draft transaction, client-bound Lease,
  trash ancestry, 권한·idempotency·outbox·barrier race 계약을 고정하고 문서 준비 게이트를 통과했다.
- 2026-08-25: Document domain model과 Application port를 구현하고 PostgreSQL adapter·HTTP route를 연결했다.
- 2026-08-25: migration 0006에 고정 폭 rank, tree revision, move preview capability, client-bound Lease tombstone을 반영했다.
- 2026-08-25: Draft Operation 원자 저장, Review 무효화, outbox와 복합 trash cursor를 구현했다.
- 2026-08-25: 기존 통합 테스트를 새 rank 계약으로 이전하고 Lease 동시 획득 barrier race를 추가했다.
- 2026-08-25: root gate와 격리 Compose 통합 게이트를 통과했다.

## 이슈 및 해결

- 기존 권한 테스트가 단문 rank를 직접 삽입해 새 DB 제약을 위반했다. 공용 seed를 32자리 canonical rank로 바꿨다.
- 새 통합 테스트가 환경 변수만 읽어 Docker secret 파일을 찾지 못했다. 기존 secret loader 계약으로 통일했다.
- Adapter가 Document domain crate를 직접 참조해 계층 게이트를 위반했다. Application이 경계 타입을 재노출하도록 의존 방향을 복구했다.
- 테스트 workspace를 직접 삭제하면 감사 외래키가 membership 보존을 강제했다. 격리 볼륨 자체를 테스트 수명주기로 사용했다.

## 검증

- [x] tree cycle·rank·trash·restore transaction
- [x] Draft revision·Operation atomicity·Review invalidation
- [x] Lease acquire·renew·release·takeover barrier race
- [x] idempotency·outbox·권한·tenant negative corpus
- [x] generated contract와 전체 root gate

## 결과

문서 트리·Draft·Edit Lease의 domain/application/PostgreSQL/HTTP 경계를 구현했다. 고정 폭 rank와 tree
watermark, 일회용 move preview, server-time Lease, Operation 원자 저장을 idempotency·outbox와 같은
transaction에 결합했다. 전체 root gate와 격리 PostgreSQL·Redis·Docker Compose 통합 게이트가 통과했다.
