ALTER TABLE references_graph ADD COLUMN deleted_at timestamptz;

DROP INDEX references_target_idx;
CREATE INDEX references_target_idx
ON references_graph (workspace_id, target_kind, target_id, created_at DESC, id DESC)
WHERE deleted_at IS NULL;
