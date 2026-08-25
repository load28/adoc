# Observability

`catalog.json`이 SLI와 metric의 정본이다. `dashboards/`와 `alerts/`는 catalog의 exact ID와 metric만
참조한다. `node scripts/check-observability.mjs --self-test`가 orphan·누락·고카디널리티 label·runbook
결함과 telemetry registry drift를 거부한다.
