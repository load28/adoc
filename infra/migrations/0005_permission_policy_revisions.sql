-- Permission and publish-policy local concurrency plus workspace cache stamps.

ALTER TABLE documents
  ADD COLUMN permission_revision bigint NOT NULL DEFAULT 0
    CHECK (permission_revision >= 0),
  ADD COLUMN policy_revision bigint NOT NULL DEFAULT 0
    CHECK (policy_revision >= 0);

CREATE TABLE workspace_access_revisions (
  workspace_id uuid PRIMARY KEY REFERENCES workspaces(id) ON DELETE CASCADE,
  permission_revision bigint NOT NULL DEFAULT 0 CHECK (permission_revision >= 0),
  policy_revision bigint NOT NULL DEFAULT 0 CHECK (policy_revision >= 0),
  updated_at timestamptz NOT NULL DEFAULT now()
);

INSERT INTO workspace_access_revisions(workspace_id)
SELECT id FROM workspaces;

CREATE FUNCTION bump_workspace_permission_revision() RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE
  target_workspace_id uuid;
BEGIN
  IF TG_OP = 'DELETE' THEN
    target_workspace_id := OLD.workspace_id;
  ELSE
    target_workspace_id := NEW.workspace_id;
  END IF;
  INSERT INTO workspace_access_revisions(workspace_id, permission_revision, updated_at)
  SELECT target_workspace_id, 1, now()
  WHERE EXISTS (SELECT 1 FROM workspaces WHERE id = target_workspace_id)
  ON CONFLICT(workspace_id) DO UPDATE
    SET permission_revision = workspace_access_revisions.permission_revision + 1,
        updated_at = EXCLUDED.updated_at;
  IF TG_OP = 'DELETE' THEN RETURN OLD; END IF;
  RETURN NEW;
END;
$$;

CREATE FUNCTION bump_workspace_policy_revision() RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE
  target_workspace_id uuid;
BEGIN
  IF TG_OP = 'DELETE' THEN
    target_workspace_id := OLD.workspace_id;
  ELSE
    target_workspace_id := NEW.workspace_id;
  END IF;
  INSERT INTO workspace_access_revisions(workspace_id, policy_revision, updated_at)
  SELECT target_workspace_id, 1, now()
  WHERE EXISTS (SELECT 1 FROM workspaces WHERE id = target_workspace_id)
  ON CONFLICT(workspace_id) DO UPDATE
    SET policy_revision = workspace_access_revisions.policy_revision + 1,
        updated_at = EXCLUDED.updated_at;
  IF TG_OP = 'DELETE' THEN RETURN OLD; END IF;
  RETURN NEW;
END;
$$;

CREATE TRIGGER membership_permission_revision
  AFTER INSERT OR DELETE OR UPDATE OF status ON memberships
  FOR EACH ROW EXECUTE FUNCTION bump_workspace_permission_revision();
CREATE TRIGGER group_member_permission_revision
  AFTER INSERT OR DELETE OR UPDATE ON group_members
  FOR EACH ROW EXECUTE FUNCTION bump_workspace_permission_revision();
CREATE TRIGGER grant_permission_revision
  AFTER INSERT OR DELETE OR UPDATE ON permission_grants
  FOR EACH ROW EXECUTE FUNCTION bump_workspace_permission_revision();
CREATE TRIGGER document_permission_revision
  AFTER INSERT OR DELETE OR UPDATE OF parent_id, status ON documents
  FOR EACH ROW EXECUTE FUNCTION bump_workspace_permission_revision();
CREATE TRIGGER document_policy_revision
  AFTER INSERT OR DELETE OR UPDATE OF parent_id, status ON documents
  FOR EACH ROW EXECUTE FUNCTION bump_workspace_policy_revision();
CREATE TRIGGER publish_policy_revision
  AFTER INSERT OR DELETE OR UPDATE ON publish_policies
  FOR EACH ROW EXECUTE FUNCTION bump_workspace_policy_revision();
CREATE TRIGGER workspace_default_policy_revision
  AFTER UPDATE OF default_publish_mode ON workspaces
  FOR EACH ROW EXECUTE FUNCTION bump_workspace_policy_revision();
