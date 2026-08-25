CREATE TYPE event_audience_kind AS ENUM ('INTERNAL', 'WORKSPACE', 'ADMIN', 'USER', 'DOCUMENT');

ALTER TABLE workspace_sequences
  ADD COLUMN next_stream_sequence bigint NOT NULL DEFAULT 1 CHECK (next_stream_sequence > 0);

ALTER TABLE jobs
  ADD COLUMN dedupe_key text,
  ADD COLUMN sequence bigint NOT NULL DEFAULT 1,
  ADD COLUMN cancel_requested_at timestamptz,
  ADD COLUMN correlation_id text,
  ADD COLUMN replay_of_job_id uuid REFERENCES jobs(id) ON DELETE SET NULL,
  ADD COLUMN updated_at timestamptz;

UPDATE jobs
SET dedupe_key = 'legacy:' || id::text,
    correlation_id = id::text,
    updated_at = created_at;

ALTER TABLE jobs
  ALTER COLUMN dedupe_key SET NOT NULL,
  ALTER COLUMN correlation_id SET NOT NULL,
  ALTER COLUMN updated_at SET NOT NULL,
  ADD CONSTRAINT jobs_kind_check CHECK (kind IN ('OUTBOX_TO_STREAM')),
  ADD CONSTRAINT jobs_payload_size_check CHECK (octet_length(payload_json::text) <= 65536),
  ADD CONSTRAINT jobs_dedupe_key_check CHECK (char_length(dedupe_key) BETWEEN 1 AND 200),
  ADD CONSTRAINT jobs_sequence_check CHECK (sequence > 0),
  ADD CONSTRAINT jobs_correlation_id_check CHECK (char_length(correlation_id) BETWEEN 8 AND 128),
  ADD CONSTRAINT jobs_active_lease_check CHECK (status IN ('RUNNING', 'CANCEL_REQUESTED') OR lease_owner IS NULL),
  ADD CONSTRAINT jobs_cancel_request_check CHECK (
    (status = 'CANCEL_REQUESTED') = (cancel_requested_at IS NOT NULL) OR status = 'CANCELLED'
  ),
  ADD CONSTRAINT jobs_kind_dedupe_key_unique UNIQUE (kind, dedupe_key);

CREATE INDEX jobs_expired_lease_idx ON jobs (lease_until, id)
  WHERE status IN ('RUNNING', 'CANCEL_REQUESTED');

ALTER TABLE outbox_events
  ADD COLUMN audience_kind event_audience_kind,
  ADD COLUMN audience_id uuid,
  ADD COLUMN minimum_access document_access,
  ADD COLUMN correlation_id text;

UPDATE outbox_events
SET audience_kind = 'INTERNAL',
    correlation_id = id::text;

ALTER TABLE outbox_events
  ALTER COLUMN audience_kind SET NOT NULL,
  ALTER COLUMN correlation_id SET NOT NULL,
  ADD CONSTRAINT outbox_events_payload_size_check CHECK (octet_length(payload_json::text) <= 65536),
  ADD CONSTRAINT outbox_events_correlation_id_check CHECK (char_length(correlation_id) BETWEEN 8 AND 128),
  ADD CONSTRAINT outbox_events_audience_check CHECK (
    (audience_kind IN ('INTERNAL', 'WORKSPACE', 'ADMIN') AND audience_id IS NULL AND minimum_access IS NULL)
    OR (audience_kind = 'USER' AND audience_id IS NOT NULL AND minimum_access IS NULL)
    OR (audience_kind = 'DOCUMENT' AND audience_id IS NOT NULL AND minimum_access IN ('VIEWER', 'CONTRIBUTOR', 'EDITOR'))
  );

CREATE TABLE workspace_stream_events (
  id uuid PRIMARY KEY,
  workspace_id uuid NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
  sequence bigint NOT NULL CHECK (sequence > 0),
  outbox_event_id uuid NOT NULL REFERENCES outbox_events(id) ON DELETE CASCADE,
  aggregate_id uuid NOT NULL,
  event_type text NOT NULL,
  event_version integer NOT NULL CHECK (event_version > 0),
  payload_json jsonb NOT NULL CHECK (octet_length(payload_json::text) <= 65536),
  audience_kind event_audience_kind NOT NULL CHECK (audience_kind <> 'INTERNAL'),
  audience_id uuid,
  minimum_access document_access,
  correlation_id text NOT NULL CHECK (char_length(correlation_id) BETWEEN 8 AND 128),
  occurred_at timestamptz NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  expires_at timestamptz NOT NULL,
  UNIQUE (workspace_id, sequence),
  UNIQUE (outbox_event_id),
  CHECK (expires_at > created_at),
  CHECK (
    (audience_kind IN ('WORKSPACE', 'ADMIN') AND audience_id IS NULL AND minimum_access IS NULL)
    OR (audience_kind = 'USER' AND audience_id IS NOT NULL AND minimum_access IS NULL)
    OR (audience_kind = 'DOCUMENT' AND audience_id IS NOT NULL AND minimum_access IN ('VIEWER', 'CONTRIBUTOR', 'EDITOR'))
  )
);

CREATE INDEX workspace_stream_replay_idx
  ON workspace_stream_events (workspace_id, sequence, id);
CREATE INDEX workspace_stream_expiry_idx
  ON workspace_stream_events (expires_at, id);

CREATE FUNCTION reject_stream_event_update() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
  RAISE EXCEPTION USING ERRCODE = '55000', MESSAGE = 'workspace stream events are immutable';
END;
$$;

CREATE TRIGGER workspace_stream_events_immutable
  BEFORE UPDATE ON workspace_stream_events
  FOR EACH ROW EXECUTE FUNCTION reject_stream_event_update();
