DO $$
BEGIN
  IF EXISTS (SELECT 1 FROM published_versions) THEN
    RAISE EXCEPTION '0007 requires an empty pre-release published_versions table';
  END IF;
END $$;

ALTER TABLE published_versions
  ADD COLUMN content_fingerprint char(64) NOT NULL,
  ADD COLUMN based_on_version_id uuid,
  ADD COLUMN source_draft_revision bigint NOT NULL;

ALTER TABLE published_versions
  ADD CONSTRAINT published_versions_content_fingerprint_ck
    CHECK (content_fingerprint ~ '^[a-f0-9]{64}$'),
  ADD CONSTRAINT published_versions_source_revision_ck
    CHECK (source_draft_revision >= 0),
  ADD CONSTRAINT published_versions_based_on_fk
    FOREIGN KEY (workspace_id, based_on_version_id)
    REFERENCES published_versions(workspace_id, id),
  DROP CONSTRAINT published_versions_summary_check,
  ADD CONSTRAINT published_versions_summary_ck
    CHECK (char_length(summary) BETWEEN 1 AND 1000 AND summary = btrim(summary));

ALTER TABLE public_links
  ADD CONSTRAINT public_links_expiry_ck CHECK (expires_at IS NULL OR expires_at > created_at),
  ADD CONSTRAINT public_links_revocation_ck CHECK (revoked_at IS NULL OR revoked_at >= created_at);
