# Permission Guide

Current capability model and usage rules.

## Ownership

- Shared auth and permission-capability code lives in `crates/auth/`.
- Server capability cache and menu sync live in `apps/server/src/infra/`.
- Route capabilities are registered with `route_with_permission`.
- Startup sync writes registered permission codes into `menus`.

## Rules

- Use `PermissionsCheck::Require("domain:resource:action")` for standard protected routes.
- Use `Any(...)` or `All(...)` only for a concrete feature need.
- Capability strings use `domain:resource:action` today.
- This project keeps the same capability format while describing behavior as boundary checks.
- Stable actions: `list`, `create`, `update`, `delete`, `options`, `status`, `password`, `run`.
- `*` is the full-authorization grant.
- Prefix wildcard grants such as `manage:task:*` authorize matching colon-separated child capabilities such as `manage:task:list` and `manage:task:run:status`.
- `users.is_system`, `roles.is_system`, and `menus.is_system` are built-in record flags, not grants.
- Protect built-in records by checking whether the current user has `*`.
- Missing or expired permission cache is rebuilt from the database on demand to avoid unnecessary re-authentication.

## Capability Naming

- Keep existing admin objects:
  - users
  - roles
  - permissions
  - menus
  - manage tasks
  - manage deploy versions
- New checks should describe capabilities with intent. For example:
  - `system:user:list` (read users)
  - `system:role:create` (manage roles)
  - `dashboard:view` (read dashboard summaries)
- `crates/auth` exports capability constants under `capability::*` so back-end code can avoid inline string duplication.

## Menu Sync

- `permission_code` and `parent_code` define the menu hierarchy.
- `menus.is_manual = TRUE` protects manual rows from sync overwrite.
- Startup sync only updates `is_manual = FALSE` rows.
- `menu_type` is derived from the capability code.

## Prohibited

- Super-admin fallback logic.
- Treating `is_system` as authorization.
- Silent permission-sync failure.
- Promoting reserved permission modes as defaults.
