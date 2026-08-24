# PostgreSQL Logical Schema

- **문서 ID**: DATA-02
- **상태**: 동결

모든 ID는 UUIDv7, 시간은 `timestamptz`, mutable row는 `revision bigint`를 사용한다. 아래
표의 tenant row는 모두 `workspace_id not null`과 `(workspace_id, id)` key를 가진다.

## Identity·Governance

| Table | 핵심 column | constraint·index |
|---|---|---|
| users | id, identity_issuer, google_subject, email, display_name, locale, timezone | unique issuer+subject |
| login_flows | state_hash, marker_hash, nonce_hash, pkce_verifier, return_to, expires_at, consumed_at | state unique, expiry index |
| sessions | id_hash, hash_key_id, user_id, idle_expires_at, absolute_expires_at, rotated_from, revoked_at | expiry, active user index |
| user_command_receipts | user_id, operation_id, key, request_hash, response_json, expires_at | unique user+operation+key |
| workspaces | id, slug, name, status, delete_after, revision | unique slug |
| memberships | workspace_id, user_id, role, status, revision | unique pair |
| invitations | id, workspace_id, email, token_hash, expires_at, accepted_at | token unique |
| groups | id, workspace_id, name, revision | unique active name |
| group_members | workspace_id, group_id, user_id | unique triple |
| permission_grants | id, workspace_id, document_id, subject_kind, subject_id, access, manage | unique target+subject |
| publish_policies | document_id, mode, required_approvals, reviewer_rule, revision | one per document |

`manage=true`이면 `access='EDITOR'` check를 둔다. Group subject는 같은 Workspace Group만, User
subject는 active Membership만 허용하도록 deferred constraint trigger로 검증한다.

## Document

| Table | 핵심 column | constraint·index |
|---|---|---|
| documents | id, workspace_id, parent_id, rank, title, status, current_version_id, revision, trashed_at, purge_after | parent/rank index |
| drafts | id, workspace_id, document_id, base_version_id, content_json, schema_version, revision, updated_by | unique active document |
| edit_leases | document_id, holder_user_id, token_hash, expires_at, revision | PK document_id |
| published_versions | id, workspace_id, document_id, number, content_json, schema_version, publisher_id, published_at, summary | unique document+number |
| version_context | version_id, review_snapshot_json, discussion_ids, source_revision | PK version_id |
| public_links | id, workspace_id, document_id, token_hash, expires_at, revoked_at, created_by | token unique |

`documents.parent_id`는 같은 workspace이고 self가 아니어야 한다. cycle은 move transaction의
recursive CTE로 막는다. Published row UPDATE·DELETE는 retention service role 외 trigger로
거부한다. Draft content change는 revision을 정확히 1 증가시킨다.

## Collaboration·Knowledge

| Table | 핵심 column |
|---|---|
| discussions | id, workspace_id, document_id, title, status, revision, closed_at |
| discussion_topics | id, discussion_id, kind, target_id, text, rank |
| messages | id, discussion_id, author_id, body_json, created_at, edited_at, revision |
| message_revisions | message_id, revision, body_json, edited_by, edited_at |
| reviews | id, document_id, draft_id, draft_revision, policy_snapshot_json, status, requested_by |
| review_assignments | review_id, reviewer_id, decision, decided_at, discussion_id |
| inbox_items | id, workspace_id, user_id, kind, source_key, target_json, read_at, resolved_at |
| references | id, workspace_id, source_kind, source_id, target_kind, target_id, snapshot_json |
| vocabulary_concepts | id, workspace_id, canonical_term, definition, status, revision |
| vocabulary_terms | concept_id, normalized_term, kind |

active Vocabulary term은 Workspace 안에서 unique다. Inbox `source_key`는 user별 unique해 event
redelivery가 중복 item을 만들지 않는다.

## AI·File·Operations

| Table | 핵심 column |
|---|---|
| ai_jobs | id, workspace_id, user_id, kind, target_json, expected_revision, status, priority, provider, usage_json, attempt, revision |
| ai_context_sources | job_id, source_kind, source_id, authority, snapshot_hash, included |
| ai_results | job_id, schema_version, result_json, validation_json, completed_at |
| proposals | id, job_id, document_id, base_revision, operations_json, status, applied_revision |
| file_assets | id, workspace_id, storage_key, name, mime, size, checksum, status, uploaded_by, purge_after |
| file_references | asset_id, owner_kind, owner_id, workspace_id |
| audit_events | id, workspace_id, sequence, actor_json, action, target_json, metadata_json, occurred_at |
| jobs | id, workspace_id, kind, payload_json, status, priority, attempt, run_after, lease_until |
| outbox_events | id, workspace_id, aggregate_kind, aggregate_id, sequence, type, version, payload_json, published_at |
| consumer_receipts | consumer, event_id, processed_at |

FileReference는 `(asset_id, owner_kind, owner_id)` unique다. Audit는 workspace sequence로
pagination한다. JSON payload는 schema version과 application validator를 필수로 가진다.

## Index 원칙

모든 tenant query path는 workspace 선두 복합 index를 가진다. partial index로 active Draft,
active Membership, queued Job과 unrevoked public link를 분리한다. JSON 전체 GIN index를
기본값으로 만들지 않고 실제 query field를 generated column으로 승격한다.
