#!/bin/sh
set -eu

project=adoc-task017
response_file=$(mktemp)
export ADOC_API_PORT=18081
export ADOC_WEB_PORT=18080
export ADOC_RELEASE_SHA=task012

cleanup() {
  docker compose -p "$project" down --volumes --remove-orphans >/dev/null 2>&1 || true
  rm -f "$response_file"
}
trap cleanup EXIT INT TERM
cleanup
sh infra/docker/bootstrap-local.sh
docker compose -p "$project" config --quiet
docker compose -p "$project" up --build --wait
curl --fail --silent http://127.0.0.1:18081/health/ready >/dev/null
curl --fail --silent http://127.0.0.1:18080/health/live >/dev/null
status=$(curl --silent --output "$response_file" --write-out '%{http_code}' \
  http://127.0.0.1:18080/api/v1/session)
test "$status" = 401
grep -q '"code":"AUTH_REQUIRED"' "$response_file"
status=$(curl --silent --output "$response_file" --write-out '%{http_code}' \
  'http://127.0.0.1:18080/api/v1/auth/google/start?returnTo=https%3A%2F%2Fevil.example')
test "$status" = 422
grep -q '"code":"VALIDATION_FAILED"' "$response_file"
attempt=2
while [ "$attempt" -le 20 ]; do
  status=$(curl --silent --output "$response_file" --write-out '%{http_code}' \
    'http://127.0.0.1:18080/api/v1/auth/google/start?returnTo=https%3A%2F%2Fevil.example')
  test "$status" = 422
  attempt=$((attempt + 1))
done
status=$(curl --silent --output "$response_file" --write-out '%{http_code}' \
  'http://127.0.0.1:18080/api/v1/auth/google/start?returnTo=https%3A%2F%2Fevil.example')
test "$status" = 429
grep -q '"code":"RATE_LIMITED"' "$response_file"
status=$(curl --silent --output "$response_file" --write-out '%{http_code}' \
  'http://127.0.0.1:18080/api/v1/auth/google/callback?code=redacted&state=redacted')
test "$status" = 400
grep -q '"code":"AUTH_CALLBACK_INVALID"' "$response_file"
docker compose -p "$project" exec -T postgres createdb --username postgres adoc_upgrade
docker compose -p "$project" exec -T postgres psql --username postgres --dbname adoc_upgrade \
  <infra/migrations/0001_canonical_baseline.sql >/dev/null
docker compose -p "$project" exec -T postgres psql --username postgres --dbname adoc_upgrade \
  <infra/migrations/0002_identity_session.sql >/dev/null
docker compose -p "$project" exec -T postgres psql --username postgres --dbname adoc_upgrade \
  <infra/migrations/0003_user_command_idempotency.sql >/dev/null
docker compose -p "$project" exec -T postgres psql --username postgres --dbname adoc_upgrade \
  <infra/migrations/0004_invitation_capability_key.sql >/dev/null
docker compose -p "$project" exec -T postgres psql --username postgres --dbname adoc_upgrade \
  <infra/migrations/0005_permission_policy_revisions.sql >/dev/null
docker compose -p "$project" exec -T postgres psql --username postgres --dbname adoc_upgrade \
  <infra/migrations/0006_document_tree_draft_lease.sql >/dev/null
docker compose -p "$project" exec -T postgres psql --username postgres --dbname adoc_upgrade \
  --tuples-only --no-align --command \
  "SELECT count(*) FROM information_schema.columns WHERE table_name='sessions' AND column_name IN ('hash_key_id','idle_expires_at','absolute_expires_at')" \
  | grep -qx 3
docker compose -p "$project" exec -T postgres psql --username postgres --dbname adoc_upgrade \
  --tuples-only --no-align --command \
  "SELECT count(*) FROM information_schema.columns WHERE table_name='invitations' AND column_name='token_key_id'" \
  | grep -qx 1
docker compose -p "$project" exec -T postgres psql --username postgres --dbname adoc_upgrade \
  --tuples-only --no-align --command \
  "SELECT count(*) FROM information_schema.columns WHERE table_name='documents' AND column_name IN ('permission_revision','policy_revision')" \
  | grep -qx 2
docker compose -p "$project" exec -T postgres psql --username postgres --dbname adoc_upgrade \
  --tuples-only --no-align --command \
  "SELECT count(*) FROM information_schema.tables WHERE table_name IN ('workspace_document_revisions','document_move_previews')" \
  | grep -qx 2
docker compose -p "$project" exec -T postgres psql --username postgres --dbname adoc_upgrade \
  --tuples-only --no-align --command \
  "SELECT count(*) FROM information_schema.columns WHERE table_name='edit_leases' AND column_name IN ('client_instance_id','released_at')" \
  | grep -qx 2
docker compose -p "$project" --profile test build test-runner
docker compose -p "$project" --profile test run --rm test-runner \
  cargo test --locked -p adoc-adapters \
  --test identity_session \
  --test workspace_governance \
  --test permission_policy \
  --test document_core \
  -- --ignored --nocapture
docker compose -p "$project" --profile backup run --rm backup >/dev/null
docker compose -p "$project" --profile backup run --rm --entrypoint sh backup -c \
  'test -s /backup/latest/manifest.json && cd /backup/latest && sha256sum -c checksums.sha256'
docker compose -p "$project" --profile search up --wait opensearch
docker compose -p "$project" --profile search stop opensearch >/dev/null
curl --fail --silent http://127.0.0.1:18081/health/ready >/dev/null
cleanup
trap - EXIT INT TERM

if docker ps -a --format '{{.Names}}' | grep -q "^${project}-"; then
  echo "Compose integration leaked containers" >&2
  exit 1
fi
printf '%s\n' "Compose integration passed"
