# Permission Guide

> This document explains only the current permission model and usage constraints. It does not expand on reserved future capability.

## Current Model

- Route permissions are mounted through `rustzen_core::permission::RouterExt::route_with_permission`.
- The current project mainly uses `PermissionsCheck::Require("domain:resource:action")` by default.
- `Any(...)` and `All(...)` are reserved in code, currently unused, and should not be promoted as standard practice.
- JWT decode, auth context extraction, permission rule evaluation, and route permission registration live in `core/`.
- Permission checks in `server/` rely on in-memory cache written at login. Missing or expired cache is treated as a re-login requirement.
- Route permissions are also collected at registration time and synchronized into `menus` on startup through `sync_permissions(pool)`.
- Route permission registration is fail-fast: lock poisoning in `register_permission_codes(...)` or `take_registered_permission_codes()` aborts startup instead of silently skipping sync data.
- Menu hierarchy is derived from `permission_code` and `parent_code`.
- `menus.is_manual = TRUE` means the menu has been manually maintained and is no longer overwritten by sync.
- Startup sync only updates rows with `is_manual = FALSE`.
- System built-in menus are backfilled with `is_manual = FALSE`.
- `*` is the wildcard permission for system administrators. It is displayed as `All Permissions` and grants access to every permission check.
- `users.is_system`, `roles.is_system`, and `menus.is_system` are built-in record flags only. They do not grant authorization by themselves.
- When code needs to protect a built-in record from normal admins, check whether the current user has `*`.

## Code Locations

- Route extension and permission registry: `core/src/permission/`
- Auth claims, current user, extractor, and JWT codec: `core/src/auth/`
- Server permission cache and menu sync: `server/src/infra/permission.rs`
- Startup sync: `server/src/infra/app.rs`
- Server auth runtime wiring: `server/src/infra/auth_runtime.rs`

## Recommended Pattern

```rust
Router::new().route_with_permission(
    "/users",
    get(list_users),
    PermissionsCheck::Require("system:user:list"),
)
```

## Usage Rules

- Use `Require(...)` for standard CRUD routes.
- Permission strings follow `domain:resource:action`.
- For system pages, use `create`, `update`, `delete`, `list`, `options`, `status`, and `password` as the stable action names.
- Do not treat "super-admin fallback" or "any-of-many permissions" as the default mode for new endpoints.
- Do not use `is_system` as a permission grant. Only `*` represents full authorization.
- Only evaluate `Any(...)` or `All(...)` when a feature explicitly needs combined permission logic.

## Cache Behavior

- Cache the user's permission set after successful login.
- Clear the permission cache on logout.
- When cache expires, return an invalid login state and require re-authentication.
- The current implementation does not auto-refresh from the source and does not apply implicit fallbacks.

## Menu Sync Behavior

- Route permission codes are expanded by `:` into parent codes before synchronization.
- `menus.parent_code` is the source field for building tree relationships during sync.
- `parent_id` is rebuilt from `parent_code` after upsert, and root nodes keep `parent_id = 0`.
- Leaf permissions are humanized from every segment, and `:*` nodes append `Management` to the humanized base title.
- `menu_type` is derived by a fixed rule: permissions ending in `:*` are `directory`, permissions ending in `:list` or `:view` are `menu`, and all other leaf permissions are `button`.
- Manual menu edits keep `is_manual = TRUE`; sync-generated menus use `FALSE`.

## Error Rules

- No current user context: treat as an authentication error.
- Missing or expired permission cache: treat as an invalid login state.
- Insufficient permission: return a permission denied error.

## Logging Rules

- Permission middleware should log the permission target, user information, and allow or deny result.
- Log messages must describe the real check mode. Do not present reserved modes as current default behavior.
