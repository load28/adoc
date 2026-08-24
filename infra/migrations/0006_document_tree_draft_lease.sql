-- Document tree watermark, move preview capability and client-bound edit lease.

ALTER TABLE documents DROP CONSTRAINT documents_rank_check;
DROP INDEX documents_sibling_rank_idx;

WITH ranked AS (
  SELECT id,
         row_number() OVER (
           PARTITION BY workspace_id, parent_id
           ORDER BY rank COLLATE "C", id
         ) AS position
  FROM documents
)
UPDATE documents AS document
SET rank = lpad(ranked.position::text, 32, '0')
FROM ranked
WHERE ranked.id = document.id;

ALTER TABLE documents ALTER COLUMN rank TYPE text COLLATE "C";
ALTER TABLE documents
  ADD CONSTRAINT documents_rank_check CHECK (rank ~ '^[0-9A-Za-z]{32}$');
CREATE UNIQUE INDEX documents_sibling_rank_idx
  ON documents (workspace_id, parent_id, rank) WHERE status <> 'PURGING';

CREATE TABLE workspace_document_revisions (
  workspace_id uuid PRIMARY KEY REFERENCES workspaces(id) ON DELETE CASCADE,
  tree_revision bigint NOT NULL DEFAULT 0 CHECK (tree_revision >= 0),
  updated_at timestamptz NOT NULL DEFAULT now()
);

INSERT INTO workspace_document_revisions(workspace_id)
SELECT id FROM workspaces;

CREATE FUNCTION initialize_workspace_document_revision() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
  INSERT INTO workspace_document_revisions(workspace_id) VALUES(NEW.id);
  RETURN NEW;
END;
$$;
CREATE TRIGGER workspace_document_revision_initialize
  AFTER INSERT ON workspaces
  FOR EACH ROW EXECUTE FUNCTION initialize_workspace_document_revision();

CREATE TABLE document_move_previews (
  token_hash bytea PRIMARY KEY CHECK (octet_length(token_hash) = 32),
  workspace_id uuid NOT NULL,
  actor_user_id uuid NOT NULL,
  document_id uuid NOT NULL,
  claims_json jsonb NOT NULL,
  expires_at timestamptz NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  FOREIGN KEY (workspace_id, actor_user_id)
    REFERENCES memberships(workspace_id, user_id) ON DELETE CASCADE,
  FOREIGN KEY (workspace_id, document_id)
    REFERENCES documents(workspace_id, id) ON DELETE CASCADE,
  CHECK (jsonb_typeof(claims_json) = 'object'),
  CHECK (expires_at > created_at)
);
CREATE INDEX document_move_previews_expiry_idx ON document_move_previews (expires_at);

ALTER TABLE edit_leases ADD COLUMN client_instance_id uuid;
UPDATE edit_leases SET client_instance_id = gen_random_uuid();
ALTER TABLE edit_leases ALTER COLUMN client_instance_id SET NOT NULL;
ALTER TABLE edit_leases ADD COLUMN released_at timestamptz;
ALTER TABLE edit_leases
  ADD CONSTRAINT edit_leases_release_check CHECK (released_at IS NULL OR released_at >= acquired_at);
