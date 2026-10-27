# Developer quick start

Use this guide to explore the Admin foundation, start local development, and
follow an existing feature from its React page to SQLite. The local starting
point below explicitly selects `core`. Cargo defaults and signed release
commands still select `full`; the Admin-only release remains subject to the
[core/full acceptance gates](../product/features/composable-distribution/tasks.md).

## Choose your entry

| Goal | Start here |
| --- | --- |
| Explore before installing tools | [Demo](https://admin.rustzen.dev), using the demo-only `owner` / `rustzen@123` account |
| Develop the admin template | Run the Admin-only preview below with `just dev-core` |
| Use frontend hot reload or explore extension examples | Follow the optional full-development path below |
| Install a signed full distribution | Follow the [deployment guide](deployment.md) |
| Understand permissions before adapting a feature | Read the [permission guide](permission.md) and the role walkthrough below |

The Admin foundation includes authentication, users, instance-level roles and
permissions, audit logs, and shared application layout. Department data scopes
and tenant isolation are not implemented. Monitor, Insights and Reports are
extension examples with their own processes and SQLite databases.

The public demo is shared. Use it to inspect the interface; perform role and user
changes in a disposable local development instance that you own. The demo account
is not a production credential.

## Prepare local development

Use a fresh clone or a checkout whose existing data you understand. Run commands
from the repository root unless a step changes the directory.

```bash
git clone https://github.com/idaibin/rustzen-admin.git
cd rustzen-admin
```

Install the Rust toolchain selected by `rust-toolchain.toml` (1.95.0). The Web
package declares Node.js 24.15.0 and Bun 1.3.14 in
[`apps/web/package.json`](../../apps/web/package.json). Use those versions;
the pinned Bun entry below also requires pnpm to be available. Install `just`
to use the repository's command entry points.

SQLite is embedded; local development does not require a PostgreSQL server or a
`.env` file.

### 1. Install the Web dependencies

```bash
cd apps/web
pnpm dlx bun@1.3.14 install --frozen-lockfile
cd ../..
```

### 2. Start the Admin-only preview

From the repository root:

```bash
just dev-core
```

The command builds the core Web assets with the pinned Bun version, then runs
`cargo run --locked -p rustzen-admin --no-default-features --features core -- serve`.
Web-build failure stops the recipe before Cargo starts. Open
`http://127.0.0.1:9801`; this one Admin process serves both the UI and its API.
Monitor, Insights, Reports, Chromium and their databases are not required.

The preview binds to loopback and uses development settings. It deliberately
skips `.env` loading and overrides inherited runtime-root and Admin SQLite-path
settings with `target/rz/dev-core` and its `data/db/admin.db`. Logs, uploads and
other Admin runtime files also derive from that dedicated root. The recipe
refuses a symlink at `target`, `target/rz` or the runtime root. Keep this
directory owned by the local preview; do not replace its data paths with links
to another installation. Other settings, such as timezone, JWT configuration
and `RUST_LOG`, still follow their existing environment contracts.

For a fresh preview database, sign in as `owner` / `rustzen@123`. Stopping with
Ctrl-C and rerunning `just dev-core` preserves the preview's data and any
changed password. The default full-development data under `.rustzen-admin`
is separate. Preserve preview data you need before `cargo clean` or removing
`target`, because this local runtime is inside that directory. To use another free port, run
`RUSTZEN_ADMIN_PORT=19801 just dev-core` and open that port instead; the recipe
does not stop an existing listener.

This is a built static preview, without hot module reload. After editing Web
or Rust source, stop and rerun the command to rebuild and restart. It does not
create or install a signed release and does not change the default composition.

### Optional: full development with frontend hot reload

This path keeps the existing full-development behavior. Check inherited
`RUSTZEN_*` values and `.env` before starting: unlike `dev-core`, these commands
use the normal runtime configuration and default `.rustzen-admin` data root.
Stop the core preview first if both would use port 9801.

Build the full Web assets before starting the default Admin process:

```bash
cd apps/web
pnpm dlx bun@1.3.14 run build
cd ../..
cargo run --locked -p rustzen-admin -- serve
```

Admin embeds `apps/web/dist`; a Vite development server alone does not create
the required initial assets. In another terminal, start frontend hot reload:

```bash
cd apps/web
pnpm dlx bun@1.3.14 run dev
```

Open `http://localhost:9800`. The Web configuration proxies `/api` and
`/resources` to Admin on port 9801. Check the actual listener before using the
page; if either port is occupied, identify its owner rather than connecting to a
different project or relying on an automatically selected port.

The default Admin process does not start the three extension services. Start
only the examples you need with their existing `dev-monitor`, `dev-insights`
and `dev-reports` recipes; an unavailable example is not a healthy empty result.

## Follow one complete feature: custom roles

Use the local development owner to inspect **System → Roles** and **System →
Users**. A successful page load is only the first step; the role journey below
also checks persistence and backend access control.

Role management demonstrates a protected list, an editable form, validation,
transactional persistence, and React Query refresh without introducing another
business domain.

| Layer | Source owner | What to follow |
| --- | --- | --- |
| Page | [`system/role.tsx`](../../apps/web/src/routes/system/role.tsx) | List, filters, capability-gated actions and refresh |
| Form | [`-role-dialog.tsx`](../../apps/web/src/routes/system/-role-dialog.tsx) | Inputs, permission loading, submission and failure recovery |
| Frontend API | [`role/api.ts`](../../apps/web/src/api/system/role/api.ts) | List/create/update/delete calls to `/api/system/roles` |
| Route and access policy | [`role/mod.rs`](../../apps/admin/src/features/system/role/mod.rs) | Registered operations and required role capabilities |
| Handler | [`role/handler.rs`](../../apps/admin/src/features/system/role/handler.rs) | HTTP input and response boundary |
| Service | [`role/service.rs`](../../apps/admin/src/features/system/role/service.rs) | Business validation and orchestration |
| Persistence | [`role/repo.rs`](../../apps/admin/src/features/system/role/repo.rs), [`repo_mutation.rs`](../../apps/admin/src/features/system/role/repo_mutation.rs) | Queries and mutations owned by this feature |
| Behavior contract | [Role definition](../product/features/role-definition-management/spec.md), [user assignment](../product/features/user-role-assignment-readiness/spec.md) | Permission rules, editable states and acceptance |

In a disposable instance you own:

1. Open Roles and create a custom role with at least one ordinary assignable
   permission. Give the role a name and code that you can identify for cleanup.
2. Reload the list and reopen the role. Confirm its saved fields and permissions
   are read back from the service.
3. Assign it to a disposable local user through Users. Log in as that user and
   check an allowed action and a denied action, including the backend response.
   A hidden menu alone does not prove denial.
4. Return as the owner, remove the test assignment, then delete the unassigned
   custom role and test user. Confirm their removal. Do not change an existing
   user's roles or delete built-in records to make the example pass.

Built-in `owner`, `admin` and `viewer` roles are immutable through generic role
management. An assigned role cannot be deleted. Custom roles cannot receive `*`
or an owner-only capability. Reuse those rules when adapting a feature; do not
replace backend authorization with a frontend visibility check.

Use the existing [backend](backend.md), [frontend](frontend.md) and
[permission](permission.md) guides when adding your business feature. Preserve the
route → handler → service → repository boundaries and the shared UI owners.

## Startup recovery

- **Missing Web distribution:** `just dev-core` builds `dist-core` before Cargo;
  resolve that build's error first. The full path requires its separate `dist`
  build. Do not create an empty output directory to bypass the asset check.
- **Port occupied or API unavailable:** identify the actual process and target,
  then restore the expected Admin/Web pair. Keep extension-service failures
  separate from the Admin user/role journey.
- **Login rejected:** distinguish a fresh development database, an existing
  password, and a production installation. Do not reset an existing password or
  database as an automatic troubleshooting step.
- **`VersionMismatch`:** preserve the database and verify its owner, runtime root
  and migration basis. This template supports a fresh-install schema. The
  `just reset-db` target defaults to the full-development data root and is not
  the core preview's recovery command. Keep a copy of needed preview data before
  deliberately creating a fresh development database. Existing installation
  update/recovery follows the composition contract and deployment guide.

## Validation boundaries

The root `justfile` defines verification commands. `just check` includes
installation, builds and broader tests; inspect its targets before selecting a
check. Documentation, unit tests, a running page, and an accepted release prove
different things. Record the actual environment, login and completed feature
journey before claiming a clean-install or adoption result.
