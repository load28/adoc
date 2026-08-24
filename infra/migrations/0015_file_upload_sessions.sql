ALTER TABLE file_assets
  ADD COLUMN detected_mime_type text,
  ADD COLUMN failure_code text,
  ADD COLUMN gc_claimed_at timestamptz,
  ADD COLUMN byte_deleted_at timestamptz,
  DROP CONSTRAINT file_assets_check,
  ADD CONSTRAINT file_assets_ready_state_check
    CHECK ((status IN ('UPLOADING', 'VALIDATING', 'FAILED') AND ready_at IS NULL) OR
           (status IN ('READY', 'DELETED') AND ready_at IS NOT NULL));

CREATE TABLE file_upload_sessions (
  asset_id uuid PRIMARY KEY,
  workspace_id uuid NOT NULL,
  token_hash bytea NOT NULL CHECK (octet_length(token_hash) = 32),
  token_key_id text NOT NULL CHECK (char_length(token_key_id) BETWEEN 1 AND 100),
  expires_at timestamptz NOT NULL,
  uploaded_at timestamptz,
  validation_key text,
  validation_request_hash text,
  completed_at timestamptz,
  FOREIGN KEY (workspace_id, asset_id) REFERENCES file_assets(workspace_id, id) ON DELETE CASCADE,
  CHECK (completed_at IS NULL OR uploaded_at IS NOT NULL),
  CHECK ((validation_key IS NULL) = (validation_request_hash IS NULL))
);
