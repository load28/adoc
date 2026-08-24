-- Invitation capability rotation metadata and lifetime invariant.
-- Existing invitations predate reconstructable capabilities and are revoked structurally.

ALTER TABLE invitations ADD COLUMN token_key_id text;

UPDATE invitations
SET token_key_id = 'legacy-revoked',
    revoked_at = COALESCE(revoked_at, now()),
    expires_at = GREATEST(expires_at, created_at + interval '1 second'),
    revision = revision + 1
WHERE token_key_id IS NULL;

ALTER TABLE invitations ALTER COLUMN token_key_id SET NOT NULL;
ALTER TABLE invitations
  ADD CONSTRAINT invitations_token_key_id_check
  CHECK (char_length(token_key_id) BETWEEN 1 AND 64);
ALTER TABLE invitations
  ADD CONSTRAINT invitations_expiry_order_check
  CHECK (expires_at > created_at);
