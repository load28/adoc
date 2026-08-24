ALTER TABLE references_graph
  ADD COLUMN target_region_json jsonb,
  ALTER COLUMN source_region_json SET NOT NULL,
  ADD CONSTRAINT references_graph_source_document_fkey
    FOREIGN KEY (workspace_id, source_id) REFERENCES documents(workspace_id, id) ON DELETE CASCADE,
  ADD CONSTRAINT references_graph_source_kind_check CHECK (source_kind = 'DOCUMENT'),
  ADD CONSTRAINT references_graph_target_region_check
    CHECK ((target_kind = 'REGION') = (target_region_json IS NOT NULL));

DO $$
DECLARE constraint_name text;
BEGIN
  SELECT conname INTO constraint_name
  FROM pg_constraint
  WHERE conrelid = 'references_graph'::regclass AND contype = 'u';
  IF constraint_name IS NOT NULL THEN
    EXECUTE format('ALTER TABLE references_graph DROP CONSTRAINT %I', constraint_name);
  END IF;
END $$;

DROP INDEX references_target_idx;
CREATE INDEX references_target_idx
ON references_graph (workspace_id, target_kind, target_id, created_at DESC, id DESC);

ALTER TABLE vocabulary_concepts
  ADD COLUMN replacement_concept_id uuid,
  ADD CONSTRAINT vocabulary_concepts_replacement_fkey
    FOREIGN KEY (workspace_id, replacement_concept_id) REFERENCES vocabulary_concepts(workspace_id, id),
  ADD CONSTRAINT vocabulary_concepts_replacement_status_check
    CHECK (status <> 'ACTIVE' OR replacement_concept_id IS NULL);

ALTER TABLE vocabulary_terms ALTER COLUMN normalized_term DROP EXPRESSION;
ALTER TABLE vocabulary_terms ALTER COLUMN normalized_term SET NOT NULL;

CREATE TABLE vocabulary_concept_revisions (
  workspace_id uuid NOT NULL,
  concept_id uuid NOT NULL,
  revision bigint NOT NULL CHECK (revision > 0),
  canonical_term text NOT NULL,
  definition text NOT NULL,
  status vocabulary_status NOT NULL,
  replacement_concept_id uuid,
  terms_json jsonb NOT NULL,
  changed_by uuid NOT NULL,
  changed_at timestamptz NOT NULL,
  PRIMARY KEY (concept_id, revision),
  FOREIGN KEY (workspace_id, concept_id) REFERENCES vocabulary_concepts(workspace_id, id) ON DELETE CASCADE,
  FOREIGN KEY (workspace_id, changed_by) REFERENCES memberships(workspace_id, user_id)
);

CREATE OR REPLACE FUNCTION reject_vocabulary_revision_mutation() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN RAISE EXCEPTION 'vocabulary concept revisions are immutable'; END $$;
CREATE TRIGGER vocabulary_concept_revisions_immutable BEFORE UPDATE OR DELETE ON vocabulary_concept_revisions
FOR EACH ROW EXECUTE FUNCTION reject_vocabulary_revision_mutation();
