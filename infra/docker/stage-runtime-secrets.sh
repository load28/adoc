#!/bin/sh
set -eu

application_target=/application-secrets
redis_target=/redis-secrets

mkdir -p "$application_target" "$redis_target"
chown 10001:10001 "$application_target"
chmod 0700 "$application_target"
chmod 0755 "$redis_target"

for name in database_url redis_url google_client_id google_client_secret session_hmac csrf_hmac token_hash_pepper; do
  source_file="/run/secrets/$name"
  staged_file="$application_target/.$name.$$"
  test -s "$source_file"
  cp "$source_file" "$staged_file"
  chown 10001:10001 "$staged_file"
  chmod 0400 "$staged_file"
  mv -f "$staged_file" "$application_target/$name"
done

redis_source=/run/secrets/redis_acl
redis_staged="$redis_target/.redis_acl.$$"
test -s "$redis_source"
cp "$redis_source" "$redis_staged"
chmod 0444 "$redis_staged"
mv -f "$redis_staged" "$redis_target/redis_acl"
