ALTER TABLE workspace_sequences
  ADD COLUMN next_projection_sequence bigint NOT NULL DEFAULT 1
    CHECK (next_projection_sequence > 0);

ALTER TABLE outbox_events
  ADD COLUMN projection_sequence bigint;

WITH numbered AS (
  SELECT id, row_number() OVER (PARTITION BY workspace_id ORDER BY occurred_at, id) AS value
  FROM outbox_events
)
UPDATE outbox_events o
SET projection_sequence = numbered.value
FROM numbered
WHERE numbered.id = o.id;

ALTER TABLE outbox_events
  ALTER COLUMN projection_sequence SET NOT NULL,
  ADD CONSTRAINT outbox_events_projection_sequence_check CHECK (projection_sequence > 0),
  ADD CONSTRAINT outbox_events_workspace_projection_sequence_unique
    UNIQUE (workspace_id, projection_sequence);

UPDATE workspace_sequences s
SET next_projection_sequence = COALESCE((
  SELECT max(o.projection_sequence) + 1
  FROM outbox_events o
  WHERE o.workspace_id = s.workspace_id
), 1);

ALTER TABLE jobs DROP CONSTRAINT jobs_kind_check;
ALTER TABLE jobs ADD CONSTRAINT jobs_kind_check
  CHECK (kind IN ('OUTBOX_TO_STREAM', 'OUTBOX_TO_SEARCH'));

CREATE TABLE search_projection_rebuilds (
  id uuid PRIMARY KEY,
  schema_version integer NOT NULL CHECK (schema_version > 0),
  generation bigint NOT NULL CHECK (generation > 0),
  status text NOT NULL CHECK (status IN ('BUILDING', 'CATCHING_UP', 'VALIDATING', 'ACTIVE', 'FAILED')),
  snapshot_watermark_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  replayed_through_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  error_code text,
  started_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now(),
  completed_at timestamptz,
  UNIQUE (schema_version, generation),
  CHECK ((status IN ('ACTIVE', 'FAILED')) = (completed_at IS NOT NULL))
);

CREATE UNIQUE INDEX search_projection_rebuild_active_idx
  ON search_projection_rebuilds ((true))
  WHERE status IN ('BUILDING', 'CATCHING_UP', 'VALIDATING');
