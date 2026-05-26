# V2 AI-First Local-First Roadmap

This historical feature record turns `docs/history/plans/v2-design.md` into executable and verifiable work items.

## Direction

V2 moves `main` toward an AI-first, local-first, SQLite-first Rust admin/runtime framework.

This does not mean adding AI product features. It means making the repository easier for AI coding tools and human maintainers to understand, run, modify, and review.

## Guardrails

- Do not add PostgreSQL compatibility layers during the V2 first phase.
- Do not introduce provider traits such as `DatabaseProvider`, `QueryExecutor`, or `StorageBackend`.
- Do not add Redis, Kafka, Docker Compose, microservices, plugin markets, desktop runtime, or multi-agent orchestration.
- Preserve the existing admin capabilities only as simple system capabilities, not as a broader enterprise RBAC narrative.
- Keep every task small enough to review and verify independently.
- Update code, docs, and command entry points together when a task changes structure or runtime behavior.

## Task 1: Preserve the PostgreSQL-first baseline

**Priority:** P0

**Goal:** Keep the current PostgreSQL-first implementation available before `main` starts accepting breaking V2 changes.

**Files:** None required.

**Steps:**

- [ ] Check the working tree before branch operations.

```bash
git status --short
```

Expected: either no output, or unrelated local changes are explicitly noted before continuing.

- [ ] Create and publish the legacy PostgreSQL branch from the current baseline.

```bash
git checkout main
git pull --ff-only
git checkout -b legacy/pg-admin
git push origin legacy/pg-admin
git checkout main
```

- [ ] Verify the branch exists remotely.

```bash
git ls-remote --heads origin legacy/pg-admin
```

Expected: one `refs/heads/legacy/pg-admin` line.

**Acceptance:**

- `legacy/pg-admin` exists on the remote.
- `main` remains available for V2 breaking changes.
- No source files are changed by this task.

## Task 2: Normalize legacy router branch naming

**Priority:** P0

**Goal:** Replace ambiguous React Router branch names with one archive branch name.

**Files:** None required.

**Steps:**

- [ ] Inspect local and remote branches that still use old router names.

```bash
git branch --all --list '*react-router*'
```

Expected: any old `dev-react-router` or `feat-react-router` branches are visible before renaming.

- [ ] Create or update the archive branch name.

```bash
git checkout dev-react-router
git branch -m legacy/react-router
git push origin legacy/react-router
```

If the active source branch is `feat-react-router`, use it in place of `dev-react-router`.

- [ ] Verify the archive branch exists remotely.

```bash
git ls-remote --heads origin legacy/react-router
```

Expected: one `refs/heads/legacy/react-router` line.

**Acceptance:**

- `legacy/react-router` is the only router archive branch expected by documentation.
- Old router branch names are no longer used for new work.
- No source files are changed by this task.

## Task 3: Reposition project documentation for V2

**Priority:** P0

**Goal:** Make repository entry documents describe the V2 direction clearly.

**Files:**

- Modify: `README.md`
- Modify: `AGENTS.md`
- Modify: `docs/README.md`
- Modify: `docs/architecture.md`
- Modify: `docs/project-map.md`
- Modify: `docs/guides/backend.md`
- Modify: `docs/guides/deployment.md`
- Modify: `docs/guides/permission.md`

**Steps:**

- [ ] Replace the old positioning with the V2 positioning.

Use this project description in `README.md`:

```text
rustzen-admin is an AI-first and local-first Rust admin/runtime framework, designed for simple local development, SQLite-first storage, and long-term AI-assisted maintenance.
```

- [ ] Update architecture and guides so PostgreSQL is not documented as the default runtime database.

Required wording:

```text
SQLite is the default V2 storage backend.
PostgreSQL-first behavior is archived under legacy/pg-admin.
```

- [ ] Keep the historical boundary explicit.

Required wording in `docs/README.md` or `docs/history/README.md`:

```text
V2 design records under docs/history/ are historical inputs. Current implementation truth remains source code, docs/architecture.md, and docs/guides/.
```

- [ ] Verify there are no stale first-class PostgreSQL claims in current documentation.

```bash
rg -n "PostgreSQL is the only|PostgreSQL-first|PG architecture|traditional admin" README.md AGENTS.md docs/README.md docs/architecture.md docs/project-map.md docs/guides
```

Expected: no matches, except historical or explicit legacy references.

- [ ] Verify Markdown formatting and whitespace.

```bash
git diff --check
```

Expected: no output.

**Acceptance:**

- Entry documentation presents V2 as AI-first, local-first, and SQLite-first.
- Legacy PostgreSQL wording is limited to archive context.
- Current-truth documents are still separate from historical records.

## Task 4: Make SQLite the default backend storage

**Priority:** P0

**Goal:** Run the backend with SQLite by default and remove PostgreSQL as a required local dependency.

**Files:**

- Modify: `Cargo.toml`
- Modify: `zen-server/Cargo.toml`
- Modify: `zen-server/src/infra/config.rs`
- Modify: `zen-server/src/infra/db.rs`
- Modify: `zen-server/src/infra/app.rs`
- Create: `zen-server/migrations/sqlite/0001_init.sql`
- Move or replace: `zen-server/migrations/0101_system_table.sql`
- Move or replace: `zen-server/migrations/0102_system_relation.sql`
- Move or replace: `zen-server/migrations/0103_system_view.sql`
- Move or replace: `zen-server/migrations/0104_system_func.sql`
- Move or replace: `zen-server/migrations/0105_system_seed.sql`

**Steps:**

- [ ] Replace PostgreSQL connection configuration with SQLite defaults.

Required local defaults:

```text
RUSTZEN_STORAGE=sqlite
RUSTZEN_SQLITE_PATH=./data/rustzen.db
```

- [ ] Add SQLite dependencies only.

Expected dependency shape in `zen-server/Cargo.toml`:

```toml
sqlx = { version = "...", features = ["runtime-tokio-rustls", "sqlite", "migrate", "chrono", "uuid"] }
```

Do not include PostgreSQL features in V2 first-phase storage dependencies.

- [ ] Create SQLite migrations under `zen-server/migrations/sqlite/`.

Expected first-phase layout:

```text
zen-server/migrations/sqlite/0001_init.sql
zen-server/migrations/sqlite/0002_auth.sql
zen-server/migrations/sqlite/0003_workspace.sql
```

- [ ] Update SQL syntax in repos only where SQLite requires it.

Repository SQL must remain explicit and must not use `SELECT *`.

- [ ] Verify backend compile.

```bash
cargo check -p server
```

Expected: completes successfully.

- [ ] Verify repository-wide check target.

```bash
just check
```

Expected: `cargo check -p server` and `vp lint` both complete successfully.

**Acceptance:**

- Local backend startup no longer requires PostgreSQL.
- SQLite database files are created under the local runtime data path.
- There is no multi-database provider abstraction.
- Existing admin capabilities still compile against SQLite-backed repos.

## Task 5: Simplify local startup

**Priority:** P0

**Goal:** Make the core service run locally with the smallest command surface.

**Files:**

- Modify: `justfile`
- Modify: `README.md`
- Modify: `docs/guides/deployment.md`
- Modify: `zen-server/src/infra/config.rs`
- Modify: `zen-server/src/main.rs`

**Steps:**

- [ ] Add or preserve a single backend startup command.

Required command:

```bash
cargo run -p server
```

- [ ] Keep `just dev-server` as a convenience wrapper only if it remains thin.

Allowed wrapper:

```make
dev-server:
    cargo watch -x 'run -p server'
```

- [ ] Document local startup without PostgreSQL, Redis, Kafka, Docker Compose, or microservices.

Required README snippet:

```bash
cargo run -p server
```

- [ ] Verify backend startup reaches config loading.

```bash
cargo run -p server
```

Expected: the server starts or fails only on an explicit port conflict or invalid local environment value.

**Acceptance:**

- A developer can start the backend without provisioning external services.
- README startup instructions match executable commands.
- `justfile` remains the command source of truth.

## Task 6: Introduce the V2 apps and crates layout

**Priority:** P1

**Goal:** Move toward a clear runnable-apps and shared-crates structure without over-splitting modules.

**Files:**

- Modify: `Cargo.toml`
- Move: `zen-server/` to `apps/server/`
- Move: `zen-web/` to `apps/web/`
- Move: `zen-core/` to `crates/auth/` or split into `crates/auth/` and `crates/capability/`
- Create: `crates/storage/`
- Create: `crates/config/`
- Create: `crates/runtime/`
- Modify: `README.md`
- Modify: `AGENTS.md`
- Modify: `docs/README.md`
- Modify: `docs/architecture.md`
- Modify: `docs/project-map.md`
- Modify: `docs/guides/backend.md`
- Modify: `docs/guides/frontend.md`
- Modify: `docs/guides/deployment.md`
- Modify: `justfile`
- Modify: `deploy/binary.Dockerfile`
- Modify: `deploy/release.Dockerfile`
- Modify: `deploy/runtime.Dockerfile`

**Steps:**

- [ ] Move runnable apps under `apps/`.

Expected layout:

```text
apps/server/
apps/web/
```

- [ ] Move shared Rust capabilities under `crates/`.

Expected first-phase layout:

```text
crates/auth/
crates/capability/
crates/config/
crates/runtime/
crates/storage/
```

- [ ] Keep the layout intentionally shallow.

Do not create these modules in the first phase:

```text
workspace-core
workspace-service
workspace-repo
workspace-api
```

- [ ] Update workspace members.

Expected `Cargo.toml` member shape:

```toml
[workspace]
members = [
    "apps/server",
    "crates/auth",
    "crates/capability",
    "crates/config",
    "crates/runtime",
    "crates/storage",
]
resolver = "2"
```

- [ ] Update frontend commands to use the new path.

Expected command shape:

```make
build-web:
    cd apps/web && pnpm build
```

- [ ] Verify command paths.

```bash
just check
just build
```

Expected: both commands complete successfully.

**Acceptance:**

- Runnable applications live under `apps/`.
- Shared capabilities live under `crates/`.
- Documentation and command entry points all describe the same layout.
- The move does not add compatibility directories or duplicate entry points.

## Task 7: Extract storage without provider over-abstraction

**Priority:** P1

**Goal:** Put SQLite connection, migrations, transactions, and repository helpers behind a small storage crate.

**Files:**

- Create: `crates/storage/Cargo.toml`
- Create: `crates/storage/src/lib.rs`
- Create: `crates/storage/src/sqlite.rs`
- Create: `crates/storage/src/migration.rs`
- Modify: `apps/server/Cargo.toml`
- Modify: `apps/server/src/infra/db.rs`
- Modify: backend repo files under `apps/server/src/features/`

**Steps:**

- [ ] Add a storage crate that exposes SQLite-specific functions.

Allowed public API shape:

```rust
pub mod migration;
pub mod sqlite;

pub use sqlite::{connect_sqlite, SqlitePool};
```

- [ ] Keep the connection API concrete.

Allowed function shape:

```rust
pub async fn connect_sqlite(database_url: &str) -> Result<SqlitePool, sqlx::Error>
```

- [ ] Do not add backend-neutral provider traits.

Forbidden names:

```text
DatabaseProvider
QueryExecutor
StorageBackend
```

- [ ] Verify forbidden names are absent.

```bash
rg -n "DatabaseProvider|QueryExecutor|StorageBackend" apps crates
```

Expected: no output.

- [ ] Verify backend compile.

```bash
cargo check -p server
```

Expected: completes successfully.

**Acceptance:**

- Storage code has a clear owner under `crates/storage`.
- Server features use SQLite-backed persistence through concrete helpers.
- No multi-database abstraction appears in the first-phase implementation.

## Task 8: Reframe RBAC as system capability boundaries

**Priority:** P1

**Goal:** Keep users, roles, permissions, and menus working while reducing traditional RBAC-first language.

**Files:**

- Modify: `crates/capability/`
- Modify: `crates/auth/`
- Modify: `apps/server/src/infra/permission.rs`
- Modify: backend system feature files under `apps/server/src/features/`
- Modify: `docs/guides/permission.md`
- Modify: `docs/architecture.md`
- Modify: `docs/project-map.md`

**Steps:**

- [x] Keep existing admin objects.

Required retained concepts:

```text
users
roles
permissions
menus
```

- [x] Introduce capability naming only where it reduces ambiguity.

Allowed examples:

```text
user.read
user.write
role.manage
workspace.manage
system.config
```

- [x] Do not introduce desktop-only capabilities in the first phase.

Forbidden first-phase examples:

```text
filesystem.read
shell.exec
```

- [x] Verify current route protection still uses explicit permission checks.

```bash
rg -n "PermissionsCheck::Require|route_with_permission" apps/server crates
```

Expected: protected routes still register explicit requirements.

- [x] Verify permission checks compile.

```bash
cargo check -p server
```

Expected: completes successfully.

**Acceptance:**

- Existing auth and permission behavior remains present.
- Documentation describes permissions as system capability boundaries.
- No complex capability framework is introduced.

## Task 9: Add AI coding rules and module ownership notes

**Priority:** P2

**Goal:** Make the repository easier for AI tools to navigate without turning historical notes into implementation truth.

**Files:**

- Create: `docs/guides/ai-coding-rules.md`
- Modify: `docs/README.md`
- Modify: `docs/project-map.md`
- Modify: `AGENTS.md`
- Create or update: `apps/server/AGENTS.md`
- Create or update: `apps/web/AGENTS.md`
- Create or update: `crates/storage/AGENTS.md`
- Create or update: `crates/auth/AGENTS.md`
- Create or update: `crates/capability/AGENTS.md`

**Steps:**

- [ ] Add AI coding rules as a current guide, not a historical plan.

Required guide topics:

```text
module boundaries
command source
documentation source of truth
no compatibility fallback logic
no speculative abstraction
how to update code and docs together
```

- [ ] Keep subdirectory `AGENTS.md` files thin.

Allowed shape:

```text
# Rules

- Read the relevant guide in docs/guides/.
- Keep this directory's boundary narrow.
- Do not add compatibility fallbacks or speculative abstractions.
```

- [ ] Verify no current docs point to `docs/history/` as implementation truth.

```bash
rg -n "history/.*truth|implementation truth.*history" docs README.md AGENTS.md
```

Expected: no matches that promote historical files as current truth.

- [ ] Verify documentation formatting.

```bash
git diff --check
```

Expected: no output.

**Acceptance:**

- AI coding rules are discoverable from `docs/README.md`.
- Core directories have short ownership notes.
- Historical records remain explicitly non-current.

## Task 10: Keep P3 scope deferred

**Priority:** P3

**Goal:** Prevent V2 first-phase work from expanding into unrelated platform features.

**Files:**

- Modify only if needed: `docs/project-map.md`
- Modify only if needed: `docs/architecture.md`

**Steps:**

- [ ] Confirm these features are absent from first-phase task branches.

```bash
rg -n "desktop|plugin|sync|PostgreSQL Provider|enterprise deployment|microservice|Kafka|Redis|Docker Compose" README.md AGENTS.md docs apps crates deploy
```

Expected: no matches that describe these as first-phase implementation work.

- [ ] If a future-direction note is needed, keep it explicit.

Allowed wording:

```text
Desktop, plugins, sync, PostgreSQL provider support, and enterprise deployment are outside the V2 first-phase scope.
```

**Acceptance:**

- P3 items do not appear in P0, P1, or P2 implementation branches.
- V2 first-phase work remains focused on local-first SQLite runtime, structure, and documentation.

## First-phase completion checks

Run these after P0 and P1 tasks land:

```bash
cargo check -p server
just check
just build
git diff --check
```

Expected:

- Backend compiles.
- Frontend lint and build commands complete through the root command source.
- Documentation and command paths match the final layout.
- No PostgreSQL service is required for local backend startup.
- No provider abstraction exists for storage.
