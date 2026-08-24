ALTER TABLE reviews DROP CONSTRAINT reviews_workspace_id_draft_id_fkey;

DROP INDEX reviews_active_document_idx;
CREATE UNIQUE INDEX reviews_active_document_idx
ON reviews (document_id)
WHERE status = 'REQUESTED';
