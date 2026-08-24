# Migrations

`docs/design/data/schema.sql` is the latest canonical PostgreSQL bootstrap DDL. Files in this directory
are forward-only SQLx migrations. A committed migration must never be edited or removed.

Add only the next contiguous `NNNN_<slug>.sql`, update the canonical DDL to the same final state, then run
`bun run migrations:seal`. The command verifies every previously sealed checksum before appending the new
file to `manifest.json`. `bun run migrations:check` rejects mutation, deletion, insertion and checksum drift.
SQLx independently owns each migration transaction and records its checksum.

The initial baseline has no genuine predecessor schema. Every later change must test both clean latest
apply and previous sealed set to latest upgrade.
