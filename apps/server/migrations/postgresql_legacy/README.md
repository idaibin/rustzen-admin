# PostgreSQL Legacy Migrations

These SQL migrations are preserved only for historical/archival reference.

Current sqlite-first startup loads migrations from:

```text
apps/server/migrations/sqlite/
```

`crates/storage` relies on that path for runtime migration execution.
