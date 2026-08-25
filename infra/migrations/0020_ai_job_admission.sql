ALTER TABLE ai_jobs
  ADD COLUMN context_metadata_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  ADD COLUMN request_key text;

UPDATE ai_jobs
SET request_key = 'legacy:' || id::text
WHERE request_key IS NULL;

ALTER TABLE ai_jobs
  ALTER COLUMN context_metadata_json DROP DEFAULT,
  ALTER COLUMN request_key SET NOT NULL,
  ADD CONSTRAINT ai_jobs_request_key_check
    CHECK (char_length(request_key) BETWEEN 8 AND 200),
  ADD CONSTRAINT ai_jobs_request_key_unique UNIQUE (workspace_id, user_id, request_key);

ALTER TABLE jobs DROP CONSTRAINT jobs_kind_check;
ALTER TABLE jobs ADD CONSTRAINT jobs_kind_check
  CHECK (kind IN ('OUTBOX_TO_STREAM', 'OUTBOX_TO_SEARCH', 'AI_RUNTIME'));
