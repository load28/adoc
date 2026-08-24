#!/bin/sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
local_dir="$script_dir/.local"
secret_dir="$local_dir/secrets"
mkdir -p "$secret_dir" "$local_dir/backups"
chmod 700 "$local_dir" "$secret_dir" "$local_dir/backups"
umask 077

write_atomic() {
  target=$1
  value=$2
  temporary="$target.tmp.$$"
  printf '%s\n' "$value" >"$temporary"
  chmod 600 "$temporary"
  mv -f "$temporary" "$target"
}

if [ ! -f "$secret_dir/postgres_password" ]; then
  password=$(openssl rand -hex 24)
  write_atomic "$secret_dir/postgres_password" "$password"
  write_atomic "$secret_dir/database_url" "postgres://postgres:$password@postgres:5432/adoc"
fi

if [ ! -f "$secret_dir/redis_url" ]; then
  redis_password=$(openssl rand -hex 24)
  write_atomic "$secret_dir/redis_url" "redis://default:$redis_password@redis:6379"
  temporary="$secret_dir/redis_acl.tmp.$$"
  printf 'user default on >%s ~* &* +@all\nuser health on nopass -@all +ping\n' \
    "$redis_password" >"$temporary"
  chmod 600 "$temporary"
  mv -f "$temporary" "$secret_dir/redis_acl"
fi

if [ ! -f "$secret_dir/token_hash_pepper" ]; then
  pepper=$(openssl rand -hex 32)
  write_atomic "$secret_dir/token_hash_pepper" "{\"current\":{\"id\":\"local-1\",\"value\":\"$pepper\"}}"
fi

for name in session_hmac csrf_hmac; do
  if [ ! -f "$secret_dir/$name" ]; then
    value=$(openssl rand -hex 32)
    write_atomic "$secret_dir/$name" "{\"current\":{\"id\":\"local-1\",\"value\":\"$value\"}}"
  fi
done

if [ ! -f "$secret_dir/google_client_id" ]; then
  write_atomic "$secret_dir/google_client_id" "local-not-a-google-client"
  write_atomic "$secret_dir/google_client_secret" "local-not-a-google-secret"
fi

printf '%s\n' "local Docker secrets are ready in $secret_dir"
