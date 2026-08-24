# Analytics Events

- **문서 ID**: DATA-06
- **상태**: 동결

## 허용 event

| Event | Property |
|---|---|
| draft_created | workspace_bucket, source=empty|restore|ai |
| publish_completed | document_bucket, review_mode, lead_time_bucket |
| discussion_resolved | message_count_bucket, linked_publish 여부 |
| search_outcome | result_count_bucket, opened, followup_action |
| ai_job_outcome | task_kind, provider_kind, status, latency_bucket, usage_bucket |
| proposal_decision | task_kind, accepted_operation_count, rejected_count |
| recovery_outcome | buffer_age_bucket, conflict, success |

## 금지 property

본문·제목·검색어·prompt·Message·email·file name·external URL 원문을 수집하지 않는다.
userId·documentId raw value도 analytics sink에 보내지 않고 rotating keyed bucket을 사용한다.

## 품질

event는 domain transaction과 분리된 outbox consumer가 발행한다. analytics 실패가 product
command를 실패시키지 않는다. schema registry와 sample validation을 CI에서 수행한다.

## 사용자 통제

self-hosted deployment는 analytics default off다. managed 환경은 privacy notice와 목적을
표시하며 operational security log와 product analytics를 분리한다.
