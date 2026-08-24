# language: ko
@document-id-TEST-09
기능: 팀의 토론과 초안을 권한이 보장된 공식 지식으로 발행한다

  배경:
    먼저 고정 시각은 "2026-01-15T09:00:00Z"이다
    그리고 "Alpha" 작업공간과 표준 사용자, 그룹, 문서 트리 fixture가 있다

  @governance @permission
  시나리오: 가장 가까운 개인 권한이 같은 위치의 그룹 거부보다 우선한다
    만약 편집자가 "Authentication" 문서를 조회한다
    그러면 유효 권한은 "EDITOR"이고 관리 권한이 있다
    만약 거부 사용자가 같은 문서를 조회한다
    그러면 응답은 존재를 노출하지 않는 404이다

  @governance @isolation
  시나리오: 관리자는 문서 권한을 우회하지 못한다
    만약 문서 grant가 없는 Alpha 관리자가 "Authentication" 문서를 조회한다
    그러면 응답은 404이다
    그리고 검색 결과와 AI Source에도 문서 제목이 없다

  @document @lease @concurrency
  시나리오: 하나의 편집 lease만 Draft를 변경한다
    먼저 편집자가 revision 7 Draft의 lease를 보유한다
    만약 기여자가 같은 문서의 lease를 획득한다
    그러면 오류 코드는 "EDIT_LEASE_HELD"이다
    만약 편집자가 expected revision 7로 유효 Operation을 적용한다
    그러면 Draft revision은 8이다
    그리고 동일 idempotency 요청을 반복해도 revision은 8이다

  @document @review
  시나리오: Draft 변경은 기존 승인을 무효화한다
    먼저 revision 7 Draft에 APPROVED Review가 있다
    만약 유효 Operation으로 Draft revision이 8이 된다
    그러면 Review 상태는 "INVALIDATED"이다
    그리고 revision 7의 승인은 발행에 사용할 수 없다

  @document @publish
  시나리오: 검토 정책을 충족한 exact Draft만 발행한다
    먼저 revision 8 Draft에 필요한 승인 수를 충족한 Review가 있다
    그리고 모든 File은 READY이고 blocking Writing Rule 위반이 없다
    만약 편집자가 revision 8을 발행한다
    그러면 immutable Published Version 3이 생성된다
    그리고 Document의 current Version은 Version 3이다
    그리고 active Draft는 없다

  @document @immutable
  시나리오: 발행 버전은 일반 application role로 변경할 수 없다
    먼저 Published Version 3이 있다
    만약 일반 application role이 Version 3을 갱신한다
    그러면 PostgreSQL 오류 상태는 "55000"이다
    그리고 Version 3의 content hash는 변하지 않는다

  @collaboration
  시나리오: 토론은 닫고 다시 열어도 Message history를 유지한다
    먼저 복수 Topic과 Message가 있는 open Discussion이 있다
    만약 기여자가 이유를 입력해 Discussion을 닫고 다시 연다
    그러면 기존 Message와 Revision은 모두 유지된다
    그리고 close와 reopen Audit Event가 각각 하나 있다

  @knowledge @security
  시나리오: 검색은 권한 밖 후보를 순위 계산 전에 제외한다
    먼저 허용 문서와 거부 문서가 같은 query와 embedding score를 가진다
    만약 거부 사용자가 지식 검색을 수행한다
    그러면 거부 문서는 hit, count, snippet과 Source 어디에도 없다

  @ai @grounding
  시나리오: 조직 사실에 Source가 없으면 AI가 답을 꾸미지 않는다
    먼저 task context에 질문을 지지하는 허용 Source가 없다
    만약 지식 질문 AI Job이 완료된다
    그러면 결과 상태는 "INSUFFICIENT_CONTEXT"이다
    그리고 일반 model 지식으로 조직 사실을 만들지 않는다

  @ai @proposal
  시나리오: 큰 AI 변경은 사람 승인 전 Draft를 바꾸지 않는다
    먼저 revision 8 Draft에 schema-valid Proposal이 생성된다
    그러면 Draft revision은 여전히 8이다
    만약 편집자가 dependency가 닫힌 Operation 집합을 승인해 적용한다
    그러면 Draft revision은 9이다
    그리고 하나의 undo group이 생성된다

  @ai @stale
  시나리오: stale Proposal은 부분적으로도 적용되지 않는다
    먼저 Proposal base revision은 8이고 현재 Draft revision은 9이다
    만약 편집자가 Proposal을 적용한다
    그러면 오류 코드는 "PROPOSAL_STALE"이다
    그리고 Draft content와 revision은 변하지 않는다

  @file @gc
  시나리오: 발행 버전이 참조하는 File은 Draft에서 제거해도 보존한다
    먼저 READY File을 Published Version 2와 Draft가 참조한다
    만약 Draft에서 File reference를 제거하고 GC를 실행한다
    그러면 File 상태는 "READY"이다
    그리고 Published Version 2에서 File을 다운로드할 수 있다

  @public @security
  시나리오: 공개 Viewer는 최신 단일 발행 문서의 embedded File만 본다
    먼저 "Authentication" 최신 Version 3의 공개 link가 있다
    만약 익명 사용자가 공개 link를 연다
    그러면 Version 3 content와 정확한 embedded File ID만 응답한다
    그리고 tree, Draft, history, Discussion, Search와 AI endpoint는 노출되지 않는다

  @retention
  시나리오: 휴지통 30일 전에는 복구하고 이후에는 구조적으로 purge한다
    먼저 문서를 휴지통으로 이동한 지 29일이다
    만약 관리 권한 사용자가 문서를 복구한다
    그러면 문서와 하위 트리는 ACTIVE이다
    먼저 같은 문서를 다시 휴지통으로 이동한 지 30일이 지났다
    만약 retention worker가 purge를 완료한다
    그러면 content, Version, Search projection과 File reference가 제거된다
    그리고 최소 Audit tombstone과 purge ledger만 남는다

  @recovery @outbox
  시나리오: Redis 유실 뒤 PostgreSQL Job과 Outbox로 복구한다
    먼저 queued Job과 unpublished Outbox Event가 있다
    만약 Redis queue를 비우고 reconciler를 실행한다
    그러면 Job wake-up과 Outbox delivery가 복원된다
    그리고 consumer 재전달은 projection을 중복 적용하지 않는다
