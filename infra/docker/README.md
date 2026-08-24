# Docker Compose 실행 환경

로컬 secret을 만든 뒤 core profile을 실행합니다.

```sh
asdf exec bun run compose:bootstrap
docker compose up --build --wait
```

선택 dependency는 `search`, `ai-local`, `observability`, `backup` profile로 실행합니다. 로컬 backup은
`backup_data` volume에 PostgreSQL dump, object archive와 checksum manifest를 만듭니다. 암호화된 외부
destination이 없으므로 production backup으로 간주하지 않습니다.

```sh
docker compose --profile search up --wait opensearch
docker compose --profile backup run --rm backup
asdf exec bun run compose:integration
```

secret 원문은 gitignore된 `infra/docker/.local/secrets`에만 둡니다. 일반 `docker compose down`은 named
volume을 보존합니다. 통합 검증만 고유 project의 volume을 명시적으로 제거합니다.
