-- document-id: PLAN-12
-- User-scoped idempotency for commands outside a Workspace boundary.

CREATE TABLE user_command_receipts (
  user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  operation_id text NOT NULL CHECK (char_length(operation_id) BETWEEN 1 AND 100),
  key text NOT NULL CHECK (char_length(key) BETWEEN 16 AND 128),
  request_hash text NOT NULL CHECK (request_hash ~ '^[a-f0-9]{64}$'),
  response_json jsonb,
  created_at timestamptz NOT NULL,
  expires_at timestamptz NOT NULL,
  PRIMARY KEY (user_id, operation_id, key),
  CHECK (expires_at > created_at)
);
CREATE INDEX user_command_receipts_expiry_idx ON user_command_receipts (expires_at);
