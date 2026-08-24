CREATE TABLE review_decision_revisions (
  workspace_id uuid NOT NULL,
  review_id uuid NOT NULL,
  reviewer_id uuid NOT NULL,
  revision bigint NOT NULL CHECK (revision > 0),
  decision review_decision NOT NULL,
  discussion_id uuid,
  decided_at timestamptz NOT NULL,
  PRIMARY KEY (review_id, reviewer_id, revision),
  FOREIGN KEY (workspace_id, review_id) REFERENCES reviews(workspace_id, id) ON DELETE CASCADE,
  FOREIGN KEY (workspace_id, reviewer_id) REFERENCES memberships(workspace_id, user_id),
  FOREIGN KEY (workspace_id, discussion_id) REFERENCES discussions(workspace_id, id)
);

CREATE OR REPLACE FUNCTION reject_review_decision_revision_mutation()
RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN RAISE EXCEPTION 'review decision revisions are immutable'; END $$;

CREATE TRIGGER review_decision_revisions_immutable
BEFORE UPDATE OR DELETE ON review_decision_revisions
FOR EACH ROW EXECUTE FUNCTION reject_review_decision_revision_mutation();
