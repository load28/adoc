#!/bin/sh
set -eu

project=adoc-task012
export ADOC_API_PORT=18081
export ADOC_WEB_PORT=18080
export ADOC_RELEASE_SHA=task012

cleanup() {
  docker compose -p "$project" down --volumes --remove-orphans >/dev/null 2>&1 || true
}
trap cleanup EXIT INT TERM
cleanup
sh infra/docker/bootstrap-local.sh
docker compose -p "$project" config --quiet
docker compose -p "$project" up --build --wait
curl --fail --silent http://127.0.0.1:18081/health/ready >/dev/null
curl --fail --silent http://127.0.0.1:18080/health/live >/dev/null
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
