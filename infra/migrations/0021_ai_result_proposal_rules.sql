ALTER TABLE proposals
  ADD COLUMN owner_user_id uuid,
  ADD COLUMN writing_rule_version text,
  ADD COLUMN vocabulary_revision bigint,
  ADD COLUMN validation_json jsonb,
  ADD COLUMN revision bigint NOT NULL DEFAULT 0,
  ADD COLUMN applied_operation_ids uuid[] NOT NULL DEFAULT '{}';

UPDATE proposals p
SET owner_user_id = j.user_id,
    writing_rule_version = COALESCE(j.context_metadata_json->>'writingRuleVersion', 'writing-rules-v1:0'),
    vocabulary_revision = COALESCE((j.context_metadata_json->>'vocabularyRevision')::bigint, 0),
    validation_json = '{"validatorVersion":"legacy","status":"VALIDATED"}'::jsonb
FROM ai_jobs j
WHERE j.id = p.job_id;

UPDATE proposals p
SET applied_operation_ids = ARRAY(
  SELECT (operation->>'opId')::uuid
  FROM jsonb_array_elements(p.operations_json) AS operation
)
WHERE p.status = 'APPLIED';

ALTER TABLE proposals
  ALTER COLUMN owner_user_id SET NOT NULL,
  ALTER COLUMN writing_rule_version SET NOT NULL,
  ALTER COLUMN vocabulary_revision SET NOT NULL,
  ALTER COLUMN validation_json SET NOT NULL,
  ADD CONSTRAINT proposals_owner_membership_fk
    FOREIGN KEY (workspace_id, owner_user_id) REFERENCES memberships(workspace_id, user_id),
  ADD CONSTRAINT proposals_revision_check CHECK (revision >= 0),
  ADD CONSTRAINT proposals_vocabulary_revision_check CHECK (vocabulary_revision >= 0),
  ADD CONSTRAINT proposals_operations_array_check CHECK (jsonb_typeof(operations_json) = 'array'),
  ADD CONSTRAINT proposals_applied_operations_check
    CHECK ((status = 'APPLIED') = (cardinality(applied_operation_ids) > 0));

CREATE INDEX proposals_owner_idx
  ON proposals (workspace_id, owner_user_id, id DESC);

UPDATE writing_configurations
SET baseline_version = 'writing-rules-v1',
    overrides_json = '[]'::jsonb;

ALTER TABLE writing_configurations
  ADD CONSTRAINT writing_configurations_baseline_check
    CHECK (baseline_version = 'writing-rules-v1'),
  ADD CONSTRAINT writing_configurations_overrides_array_check
    CHECK (overrides_json = '[]'::jsonb);
