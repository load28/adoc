#!/bin/sh
set -eu

project=adoc-task044
run_id=${ADOC_BROWSER_RUN_ID:-local}
fixture_file=artifacts/browser/fixture.json
export ADOC_API_PORT=18181
export ADOC_WEB_PORT=18180
export ADOC_RELEASE_SHA=task044
export ADOC_API_PUBLIC_ORIGIN=http://localhost:8080
export ADOC_HOST_UID
export ADOC_HOST_GID
ADOC_HOST_UID=$(id -u)
ADOC_HOST_GID=$(id -g)

cleanup() {
  status=$?
  if [ "$status" -ne 0 ]; then
    docker compose -p "$project" logs --no-color web api worker >artifacts/browser/service.log 2>&1 || true
    sed -n '1,400p' artifacts/browser/service.log >&2 || true
  fi
  docker compose -p "$project" down --volumes --remove-orphans >/dev/null 2>&1 || true
  rm -f "$fixture_file"
  return "$status"
}
trap cleanup EXIT INT TERM
mkdir -p artifacts/browser
cleanup
sh infra/docker/bootstrap-local.sh
docker compose -p "$project" config --quiet
docker compose -p "$project" up --build --wait
docker compose -p "$project" --profile test build test-runner
fixture_json=$(docker compose -p "$project" --profile test run --rm \
  -e "ADOC_BROWSER_RUN_ID=$run_id" test-runner \
  target/release/adoc-browser-fixtures)
printf '%s\n' "$fixture_json" >"$fixture_file"
node -e 'const fs=require("node:fs");const value=JSON.parse(fs.readFileSync(process.argv[1],"utf8"));if(value.schemaVersion!==1)process.exit(1)' "$fixture_file"

snapshot_flag=
if [ "${ADOC_UPDATE_BROWSER_SNAPSHOTS:-0}" = 1 ]; then
  snapshot_flag=--update-snapshots
fi
docker compose -p "$project" --profile browser run --rm browser-runner \
  node node_modules/@playwright/test/cli.js test $snapshot_flag

cleanup
trap - EXIT INT TERM
if docker ps -a --format '{{.Names}}' | grep -q "^${project}-"; then
  echo "Browser gate leaked containers" >&2
  exit 1
fi
printf '%s\n' "Browser quality gate passed"
