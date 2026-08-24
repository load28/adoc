DO $$
BEGIN
  IF EXISTS (SELECT 1 FROM messages) OR EXISTS (SELECT 1 FROM message_revisions) THEN
    RAISE EXCEPTION '0008 requires empty pre-release message tables';
  END IF;
END $$;

ALTER TABLE messages
  ADD COLUMN mention_user_ids uuid[] NOT NULL DEFAULT '{}';

ALTER TABLE message_revisions
  ADD COLUMN mention_user_ids uuid[] NOT NULL DEFAULT '{}',
  ADD COLUMN deleted_at timestamptz;

ALTER TABLE inbox_items
  ADD COLUMN revision bigint NOT NULL DEFAULT 0,
  ADD CONSTRAINT inbox_items_revision_ck CHECK (revision >= 0);

ALTER TABLE discussion_topics
  ADD CONSTRAINT discussion_topics_rank_ck
  CHECK (rank ~ '^[0-9]{32}$');

CREATE OR REPLACE FUNCTION reject_message_revision_mutation() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN RAISE EXCEPTION 'message revisions are immutable'; END $$;
CREATE TRIGGER message_revisions_immutable BEFORE UPDATE OR DELETE ON message_revisions
FOR EACH ROW EXECUTE FUNCTION reject_message_revision_mutation();
