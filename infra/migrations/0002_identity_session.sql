-- document-id: PLAN-12
-- Forward-only Identity and Session storage contract.

ALTER TABLE users
  ADD COLUMN identity_issuer text NOT NULL DEFAULT 'https://accounts.google.com';
ALTER TABLE users DROP CONSTRAINT users_google_subject_key;
ALTER TABLE users ADD CONSTRAINT users_identity_issuer_subject_key
  UNIQUE (identity_issuer, google_subject);

CREATE TABLE login_flows (
  state_hash bytea PRIMARY KEY CHECK (octet_length(state_hash) = 32),
  marker_hash bytea NOT NULL CHECK (octet_length(marker_hash) = 32),
  hash_key_id text NOT NULL CHECK (hash_key_id ~ '^[A-Za-z0-9._-]{1,64}$'),
  nonce_hash bytea NOT NULL CHECK (octet_length(nonce_hash) = 32),
  pkce_verifier text NOT NULL CHECK (char_length(pkce_verifier) BETWEEN 43 AND 128),
  return_to text NOT NULL CHECK (char_length(return_to) BETWEEN 1 AND 2048),
  created_at timestamptz NOT NULL,
  expires_at timestamptz NOT NULL,
  consumed_at timestamptz,
  CHECK (expires_at > created_at),
  CHECK (consumed_at IS NULL OR consumed_at >= created_at)
);
CREATE INDEX login_flows_expiry_idx ON login_flows (expires_at);

ALTER TABLE sessions RENAME COLUMN expires_at TO idle_expires_at;
ALTER TABLE sessions
  ADD COLUMN hash_key_id text NOT NULL DEFAULT 'legacy',
  ADD COLUMN last_seen_at timestamptz,
  ADD COLUMN absolute_expires_at timestamptz;
UPDATE sessions
SET last_seen_at = created_at,
    absolute_expires_at = idle_expires_at;
ALTER TABLE sessions
  ALTER COLUMN last_seen_at SET NOT NULL,
  ALTER COLUMN absolute_expires_at SET NOT NULL,
  ADD CONSTRAINT sessions_hash_key_id_check
    CHECK (hash_key_id ~ '^[A-Za-z0-9._-]{1,64}$'),
  ADD CONSTRAINT sessions_last_seen_check CHECK (last_seen_at >= created_at),
  ADD CONSTRAINT sessions_idle_expiry_check CHECK (idle_expires_at >= last_seen_at),
  ADD CONSTRAINT sessions_absolute_expiry_check CHECK (absolute_expires_at >= idle_expires_at);
DROP INDEX sessions_active_user_idx;
CREATE INDEX sessions_active_user_idx
  ON sessions (user_id, idle_expires_at, absolute_expires_at)
  WHERE revoked_at IS NULL;
