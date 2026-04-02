# Permission Guide

> This document explains only the current permission model and usage constraints. It does not expand on reserved future capability.

## Current Model

- Route permissions are mounted through `RouterExt::route_with_permission`.
- The current project mainly uses `PermissionsCheck::Require("domain:resource:action")` by default.
- `Any(...)` and `All(...)` are reserved in code and should not be promoted as standard practice.
- Permission checks rely on in-memory cache written at login. Missing or expired cache is treated as a re-login requirement.

## Code Locations

- Route extension: `server/src/common/router_ext.rs`
- Permission model: `server/src/infra/permission.rs`
- Current user extractor: `server/src/infra/extractor.rs`

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
- Do not treat "super-admin fallback" or "any-of-many permissions" as the default mode for new endpoints.
- Only evaluate `Any(...)` or `All(...)` when a feature explicitly needs combined permission logic.

## Cache Behavior

- Cache the user's permission set after successful login.
- Clear the permission cache on logout.
- When cache expires, return an invalid login state and require re-authentication.
- The current implementation does not auto-refresh from the source and does not apply implicit fallbacks.

## Error Rules

- No current user context: treat as an authentication error.
- Missing or expired permission cache: treat as an invalid login state.
- Insufficient permission: return a permission denied error.

## Logging Rules

- Permission middleware should log the permission target, user information, and allow or deny result.
- Log messages must describe the real check mode. Do not present reserved modes as current default behavior.
