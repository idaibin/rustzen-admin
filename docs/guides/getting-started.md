# Developer quick start

Use this guide to explore the Admin foundation, start local development, and
follow an existing feature from its React page to SQLite. The current default
composition is `full`; the Admin-only release remains subject to the
[core/full acceptance gates](../product/features/composable-distribution/tasks.md).

## Choose your entry

| Goal | Start here |
| --- | --- |
| Explore before installing tools | [Demo](https://admin.rustzen.dev), using the demo-only `owner` / `rustzen@123` account |
| Develop the admin template | Follow the local-development steps below |
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
the pinned Bun entry below also requires pnpm to be available.

SQLite is embedded; local development does not require a PostgreSQL server or a
`.env` file. Explicit environment overrides still apply. Review them before using
an existing machine or checkout, especially database paths and runtime roots.

### 1. Prepare the Web assets

```bash
cd apps/web
pnpm dlx bun@1.3.14 install --frozen-lockfile
pnpm dlx bun@1.3.14 run build
cd ../..
```

The default Admin build embeds `apps/web/dist`. Its Cargo build script checks
that this distribution exists and is non-empty, so complete this step **before**
running Cargo. A Vite development server alone does not create these assets.
This is a local Web build, not a signed release or an installation.

### 2. Start Admin in one terminal

From the repository root:

```bash
cargo run -p rustzen-admin -- serve
```

The development default listens on port 9801 and creates its fresh SQLite data
under the configured runtime root (default `.rustzen-admin`). For a fresh
non-production database, the development owner is `owner` / `rustzen@123`.
Existing owner credentials are preserved. Production installation generates and
hands off a separate initial password as described in the deployment guide.

This command uses the default `full` feature set and starts the Admin process.
It does not start the three extension services or create an Admin-only package.
If an extension service is not running, its pages must not be interpreted as a
healthy, populated example.

### 3. Start the Web development server in another terminal

```bash
cd apps/web
pnpm dlx bun@1.3.14 run dev
```

Open `http://localhost:9800`. The Web configuration proxies `/api` and
`/resources` to Admin on port 9801. Check the actual listener before using the
page; if either port is occupied, identify its owner rather than connecting to a
different project or relying on an automatically selected port.

Use the local development owner to inspect **System → Roles** and **System →
Users**. A successful page load is only the first step; the role journey below
also checks persistence and backend access control.

## Follow one complete feature: custom roles

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

- **Missing Web distribution:** complete step 1 before Cargo. Do not create an
  empty `dist` directory to bypass the embedded-asset check.
- **Port occupied or API unavailable:** identify the actual process and target,
  then restore the expected Admin/Web pair. Keep extension-service failures
  separate from the Admin user/role journey.
- **Login rejected:** distinguish a fresh development database, an existing
  password, and a production installation. Do not reset an existing password or
  database as an automatic troubleshooting step.
- **`VersionMismatch`:** preserve the database and verify its owner, runtime root
  and migration basis. This template supports a fresh-install schema. The
  `just reset-db` target deletes data; use it only for an explicitly disposable
  database after checking its exact scope. Existing installation update/recovery
  follows the composition contract and deployment guide.

## Validation boundaries

The root `justfile` defines verification commands. `just check` includes
installation, builds and broader tests; inspect its targets before selecting a
check. Documentation, unit tests, a running page, and an accepted release prove
different things. Record the actual environment, login and completed feature
journey before claiming a clean-install or adoption result.
