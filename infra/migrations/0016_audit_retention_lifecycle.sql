ALTER TABLE audit_events
  ADD COLUMN before_json jsonb,
  ADD COLUMN after_json jsonb,
  ADD COLUMN correlation_id text,
  ADD COLUMN redacted_at timestamptz;

UPDATE audit_events SET correlation_id = id::text WHERE correlation_id IS NULL;

ALTER TABLE audit_events
  ALTER COLUMN correlation_id SET NOT NULL,
  ADD CONSTRAINT audit_events_correlation_id_check
    CHECK (char_length(correlation_id) BETWEEN 8 AND 128);

ALTER TABLE purge_ledger
  ADD COLUMN status text,
  ADD COLUMN step text,
  ADD COLUMN attempt integer,
  ADD COLUMN run_after timestamptz,
  ADD COLUMN lease_owner text,
  ADD COLUMN lease_until timestamptz,
  ADD COLUMN last_error_code text,
  ADD COLUMN updated_at timestamptz;

UPDATE purge_ledger
SET status = CASE WHEN completed_at IS NULL THEN 'PENDING' ELSE 'COMPLETED' END,
    step = CASE WHEN completed_at IS NULL THEN 'PENDING' ELSE 'COMPLETED' END,
    attempt = 0,
    run_after = started_at,
    updated_at = COALESCE(completed_at, started_at);

ALTER TABLE purge_ledger
  ALTER COLUMN status SET NOT NULL,
  ALTER COLUMN status SET DEFAULT 'PENDING',
  ALTER COLUMN step SET NOT NULL,
  ALTER COLUMN step SET DEFAULT 'PENDING',
  ALTER COLUMN attempt SET NOT NULL,
  ALTER COLUMN attempt SET DEFAULT 0,
  ALTER COLUMN run_after SET NOT NULL,
  ALTER COLUMN updated_at SET NOT NULL,
  ADD CONSTRAINT purge_ledger_target_kind_check CHECK (target_kind IN ('DOCUMENT', 'WORKSPACE')),
  ADD CONSTRAINT purge_ledger_status_check CHECK (status IN ('PENDING', 'RUNNING', 'RETRY', 'COMPLETED')),
  ADD CONSTRAINT purge_ledger_step_check CHECK (step IN ('PENDING', 'ACCESS_REVOKED', 'OBJECTS_CAPTURED', 'DOMAIN_PURGED', 'OBJECTS_PURGED', 'AUDIT_REDACTED', 'COMPLETED')),
  ADD CONSTRAINT purge_ledger_attempt_check CHECK (attempt >= 0),
  ADD CONSTRAINT purge_ledger_lease_check CHECK ((lease_owner IS NULL) = (lease_until IS NULL)),
  ADD CONSTRAINT purge_ledger_completion_check CHECK ((status = 'COMPLETED') = (completed_at IS NOT NULL)),
  ADD CONSTRAINT purge_ledger_result_hash_check CHECK (result_hash IS NULL OR result_hash ~ '^[a-f0-9]{64}$');

CREATE INDEX purge_ledger_claim_idx ON purge_ledger (run_after, started_at, id)
  WHERE status IN ('PENDING', 'RETRY', 'RUNNING');

CREATE TABLE purge_object_deletions (
  ledger_id uuid NOT NULL REFERENCES purge_ledger(id) ON DELETE CASCADE,
  storage_key text NOT NULL CHECK (storage_key ~ '^[a-f0-9]{64}$'),
  attempt integer NOT NULL DEFAULT 0 CHECK (attempt >= 0),
  last_error_code text,
  deleted_at timestamptz,
  PRIMARY KEY (ledger_id, storage_key)
);

DROP TRIGGER audit_events_immutable ON audit_events;

CREATE FUNCTION retention_mutation_allowed() RETURNS boolean LANGUAGE sql STABLE AS $$
  SELECT current_user = 'adoc_retention'
    OR (current_user = 'postgres' AND current_setting('adoc.retention_context', true) = 'on')
$$;

CREATE FUNCTION reject_audit_event_mutation() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
  IF TG_OP = 'UPDATE' AND retention_mutation_allowed() AND
     NEW.id = OLD.id AND NEW.workspace_id = OLD.workspace_id AND NEW.sequence = OLD.sequence AND
     NEW.actor_json = OLD.actor_json AND NEW.action = OLD.action AND NEW.target_json = OLD.target_json AND
     NEW.correlation_id = OLD.correlation_id AND NEW.occurred_at = OLD.occurred_at AND
     NEW.redacted_at IS NOT NULL THEN
    RETURN NEW;
  END IF;
  RAISE EXCEPTION USING ERRCODE = '55000', MESSAGE = 'audit_events is append-only';
END;
$$;

CREATE TRIGGER audit_events_immutable
  BEFORE UPDATE OR DELETE ON audit_events
  FOR EACH ROW EXECUTE FUNCTION reject_audit_event_mutation();

CREATE OR REPLACE FUNCTION reject_message_revision_mutation() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
  IF retention_mutation_allowed() AND TG_OP = 'DELETE' THEN RETURN OLD; END IF;
  RAISE EXCEPTION 'message revisions are immutable';
END $$;

CREATE OR REPLACE FUNCTION reject_review_decision_revision_mutation() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
  IF retention_mutation_allowed() AND TG_OP = 'DELETE' THEN RETURN OLD; END IF;
  RAISE EXCEPTION 'review decision revisions are immutable';
END $$;

CREATE OR REPLACE FUNCTION reject_vocabulary_revision_mutation() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
  IF retention_mutation_allowed() AND TG_OP = 'DELETE' THEN RETURN OLD; END IF;
  RAISE EXCEPTION 'vocabulary concept revisions are immutable';
END $$;

CREATE OR REPLACE FUNCTION reject_immutable_row() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
  IF retention_mutation_allowed() AND TG_OP = 'DELETE' THEN RETURN OLD; END IF;
  RAISE EXCEPTION USING ERRCODE = '55000', MESSAGE = 'immutable history cannot be changed';
END;
$$;
