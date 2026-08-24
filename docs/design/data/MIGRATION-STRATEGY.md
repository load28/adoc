# Data Migration Strategy

- **문서 ID**: DATA-05
- **상태**: 동결

## 원칙

schema migration은 forward-only이고 application rollback이 가능한 expand→migrate→contract
순서를 따른다. destructive DDL과 application deploy를 같은 단계에서 수행하지 않는다.

## 단계

1. nullable/new table·index를 online 방식으로 추가한다.
2. 새 code가 old·new를 읽고 new를 쓴다.
3. idempotent batch job이 workspace checkpoint로 backfill한다.
4. count, checksum과 invariant query로 검증한다.
5. reader를 new로 전환한다.
6. 최소 두 release 뒤 old field를 contract한다.

## Content schema

reader는 지원 window의 과거 schema를 현재 in-memory schema로 migrate한다. save/publish는
current schema만 쓴다. Published Version 원본 payload는 바꾸지 않고 renderer adapter로
읽는다.

## Event·API

event version별 consumer를 유지하고 producer 전환 뒤 lag 0과 retention window가 지난 후
old consumer를 제거한다. API schema의 additive 변경을 우선하고 required field 추가는 새
version을 사용한다.

## Rollback

DB rollback script보다 application rollback 호환을 우선한다. 잘못된 backfill은 source
column과 migration ledger로 보정 migration을 실행한다.
