# Permission Guide

Current permission model and usage rules.

## Ownership

- Shared auth and permission code lives in `crates/auth/`.
- Server permission cache and menu sync live in `apps/server/src/infra/`.
- Route permissions are registered with `route_with_permission`.
- Startup sync writes registered permission codes into `menus`.

## Rules

- Use `PermissionsCheck::Require("domain:resource:action")` for standard protected routes.
- Use `Any(...)` or `All(...)` only for a concrete feature need.
- Permission strings use `domain:resource:action`.
- Stable actions: `list`, `create`, `update`, `delete`, `options`, `status`, `password`.
- `*` is the only full-authorization grant.
- `users.is_system`, `roles.is_system`, and `menus.is_system` are built-in record flags, not grants.
- Protect built-in records by checking whether the current user has `*`.
- Missing or expired permission cache requires re-authentication.

## Menu Sync

- `permission_code` and `parent_code` define the menu hierarchy.
- `menus.is_manual = TRUE` protects manual rows from sync overwrite.
- Startup sync only updates `is_manual = FALSE` rows.
- `menu_type` is derived from the permission code.

## Prohibited

- Super-admin fallback logic.
- Treating `is_system` as authorization.
- Silent permission-sync failure.
- Promoting reserved permission modes as defaults.
