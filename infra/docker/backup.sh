#!/bin/sh
set -eu

timestamp=$(date -u +%Y%m%dT%H%M%SZ)
staging="/backup/.partial-$timestamp-$$"
final="/backup/$timestamp"
lock=/backup/.backup.lock

if ! mkdir "$lock" 2>/dev/null; then
  printf '%s\n' "another backup owns the destination lock" >&2
  exit 1
fi
cleanup() {
  rm -rf "$staging"
  rmdir "$lock" 2>/dev/null || true
}
trap cleanup EXIT INT TERM
mkdir -p "$staging"
password=$(cat /run/secrets/postgres_password)
export PGPASSWORD="$password"

pg_dump --host postgres --username postgres --dbname adoc --format custom --file "$staging/postgres.dump"
tar -C /objects -cf "$staging/objects.tar" .
migration_version=$(psql --host postgres --username postgres --dbname adoc --tuples-only --no-align \
  --command "SELECT coalesce(max(version), 0) FROM _sqlx_migrations WHERE success")

postgres_size=$(wc -c <"$staging/postgres.dump" | tr -d ' ')
objects_size=$(wc -c <"$staging/objects.tar" | tr -d ' ')
postgres_sha=$(sha256sum "$staging/postgres.dump" | cut -d ' ' -f 1)
objects_sha=$(sha256sum "$staging/objects.tar" | cut -d ' ' -f 1)
printf '%s  %s\n%s  %s\n' "$postgres_sha" postgres.dump "$objects_sha" objects.tar \
  >"$staging/checksums.sha256"
printf '{"createdAt":"%s","releaseSha":"%s","migrationVersion":%s,"artifacts":{"postgres.dump":{"bytes":%s,"sha256":"%s"},"objects.tar":{"bytes":%s,"sha256":"%s"}}}\n' \
  "$timestamp" "${ADOC_RELEASE_SHA:?}" "$migration_version" "$postgres_size" "$postgres_sha" \
  "$objects_size" "$objects_sha" >"$staging/manifest.json"
mv "$staging" "$final"
ln -sfn "$timestamp" /backup/latest
trap - EXIT INT TERM
rmdir "$lock"
printf '%s\n' "$final"
