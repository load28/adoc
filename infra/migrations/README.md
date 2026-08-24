# Migrations

`docs/design/data/schema.sql` is the canonical PostgreSQL bootstrap DDL. Files in this directory are
generated executable SQLx migrations and must not be edited manually.

Run `bun run migrations:generate` after changing the canonical DDL. `bun run migrations:check`
fails when the committed migration differs from the deterministic generator output. The generator
removes the canonical file's outer `BEGIN`/`COMMIT`; SQLx owns the migration transaction.

The initial baseline has no genuine predecessor schema. Its upgrade gate is a second migrator run
against the already-current database and must be a no-op. Starting with the next schema change, the
integration fixture must first apply the previous migration set and then upgrade to latest.
