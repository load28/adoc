ALTER TABLE ai_jobs
  ADD COLUMN context_fingerprint text,
  ADD COLUMN runtime_job_id uuid;

UPDATE ai_jobs
SET context_fingerprint = repeat('0', 64)
WHERE context_fingerprint IS NULL;

ALTER TABLE ai_jobs
  ALTER COLUMN context_fingerprint SET NOT NULL,
  ADD CONSTRAINT ai_jobs_context_fingerprint_check
    CHECK (context_fingerprint ~ '^[a-f0-9]{64}$'),
  ADD CONSTRAINT ai_jobs_runtime_job_unique UNIQUE (runtime_job_id),
  ADD CONSTRAINT ai_jobs_runtime_job_fk
    FOREIGN KEY (runtime_job_id) REFERENCES jobs(id) ON DELETE SET NULL;

ALTER TABLE ai_context_sources
  ADD COLUMN stable_id text,
  ADD COLUMN include_reason text,
  ADD COLUMN snapshot_text text NOT NULL DEFAULT '',
  ADD COLUMN source_revision bigint NOT NULL DEFAULT 0,
  ADD COLUMN permission_key text;

UPDATE ai_context_sources
SET stable_id = source_id,
    include_reason = 'EXPLICIT_REFERENCE'
WHERE stable_id IS NULL OR include_reason IS NULL;

ALTER TABLE ai_context_sources
  ALTER COLUMN stable_id SET NOT NULL,
  ALTER COLUMN include_reason SET NOT NULL,
  ALTER COLUMN snapshot_text DROP DEFAULT,
  ALTER COLUMN source_revision DROP DEFAULT,
  ADD CONSTRAINT ai_context_sources_include_reason_check
    CHECK (include_reason IN ('CURRENT_TARGET', 'EXPLICIT_REFERENCE', 'DISCUSSION_CONTEXT', 'VOCABULARY_POLICY', 'RETRIEVED_RELATED', 'USER_PROVIDED')),
  ADD CONSTRAINT ai_context_sources_snapshot_text_size_check
    CHECK (octet_length(snapshot_text) <= 65536),
  ADD CONSTRAINT ai_context_sources_source_revision_check
    CHECK (source_revision >= 0),
  ADD CONSTRAINT ai_context_sources_permission_key_check
    CHECK (permission_key IS NULL OR permission_key ~ '^[a-f0-9]{64}$');
