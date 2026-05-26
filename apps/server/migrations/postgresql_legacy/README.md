# PostgreSQL Legacy Migrations

These SQL migrations are preserved only for historical/archival reference.

Current first-phase V2 startup loads migrations from:

```text
apps/server/migrations/sqlite/
```

`crates/storage` relies on that path for runtime migration execution.
