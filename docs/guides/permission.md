# Permission Guide

Current capability, module delegation, and menu-reconciliation rules.

## Ownership

- `crates/auth/` owns shared auth types, capability constants, and Admin-native
  route permission checks.
- `crates/ipc/` owns module route access metadata and HMAC delegated context.
- `apps/admin/src/infra/permission/` owns built-in role policy, capability
  projection, and transactional module reconciliation. `mod.rs` orchestrates
  transactions and projection refresh; `capabilities.rs`, `navigation.rs`,
  and `roles.rs` own their respective persistence rules.
- `apps/admin/src/features/modules/` owns fixed module enabled state, Manifest
  synchronization, the immutable runtime registry, and gateway authorization.
- Module Rust route registration is the single source for method, path, public
  or protected access, and required capability.
- Admin route registration and module Manifests are the only supported sources
  of capability definitions. Role management assigns existing capabilities;
  menu management does not create them.
- Menu management displays the runtime navigation inventory. Core Admin entries
  are read-only there; only module-owned title, icon, order, and visibility may
  be overridden.

## Capability rules

Module navigation is persisted in `module_navigation`, keyed by module ID and
Manifest menu code. Multiple pages may reference the same capability without
collapsing their navigation or presentation overrides. The `menus` capability
catalog remains unique by permission code and continues to own role grants.
Both projections reconcile in one transaction before the runtime Manifest changes.

`GET /api/system/menus/inventory` returns navigation IDs;
`PUT /api/system/menus/inventory/{id}` edits only that navigation row's title,
icon, order, and visibility. Capability IDs are a separate namespace. The former
`PUT /api/system/menus/{id}` route is not registered. Hiding or removing a navigation
entry does not revoke a surviving API capability. Removed Manifest entries become
inactive; presentation overrides follow the stable module/menu identity when its
path or required permission changes.

- Admin-native routes use `PermissionsCheck::Require(...)` by default. Use
  `Any(...)` or `All(...)` only for a concrete feature need.
- Capability strings use colon-separated business intent, such as
  `system:user:list`, `monitor:view`, and `reports:manage`.
- `*` is the full grant. Prefix wildcards authorize matching colon-separated
  children.
- `users.is_system`, `roles.is_system`, and `menus.is_system` are record flags,
  not grants.
- User capabilities come from role-menu relations only. Every protected Admin
  and module-gateway request validates its session, enabled user, auth epoch and
  current grants from one SQLite snapshot. Process-local permission snapshots
  may support reconciliation diagnostics and tests, but never authorize a
  request or an access-control write.

## Built-in roles

- `owner` is the only built-in role that receives `*` and the only role that
  may view or manage system modules, system status, scheduled tasks, and
  deployment releases.
- `admin` receives concrete module and ordinary Admin-management capabilities,
  excluding all owner-only capability roots.
- `viewer` receives concrete read-only capabilities, excluding all owner-only
  capability roots.
- Built-in roles cannot be edited or deleted through role management.
- Ordinary role forms cannot assign `*`, an owner-only capability, or a prefix
  wildcard that would cover an owner-only capability.

## Module synchronization

Each module's `module.toml` stores only module identity and default menu
presentation. Rust `ModuleRouter` registration generates the route portion of
the runtime Manifest.

Admin periodically fetches enabled module Manifests outside the request path,
validates the fixed identity and contract, and transactionally reconciles:

- module-owned capability rows;
- default module menus;
- built-in role grants derived from the current capability catalog.

Existing custom-role leaf grants are preserved and newly introduced
capabilities remain unassigned. During the breaking split only, a legacy
`monitor:*`, `insights:*`, or `reports:*` custom-role relation is expanded once
to the exact capabilities in that module's first valid Manifest, then the
wildcard relation is retired so later capabilities are not granted implicitly.

Only after the database transaction commits is the immutable runtime registry
swapped. Invalid or incompatible changes return the module to unavailable state
without partial menu, permission, or route updates. `menus.is_manual = TRUE`
preserves manual presentation overrides. Disabled modules are not polled and
remain unavailable until a fresh valid Manifest is synchronized after
re-enabling.

Runtime navigation uses the last transactionally reconciled active menu rows.
For a user with the required capability, an unavailable or incompatible service
does not hide an otherwise active, visible, non-deleted menu row. Runtime health
and compatibility continue to govern gateway availability independently.
Disabling the module hides its navigation without deleting the stored rows or
presentation overrides. Manual menu-visibility overrides remain effective.

## Request flow

1. Admin matches method and full path in the in-memory registry.
2. For a protected route, Admin decodes the JWT and checks the persisted session,
   enabled user, current auth epoch and required capability in SQLite. Access
   writes repeat the same actor check after acquiring their write transaction.
3. Admin creates an HMAC context containing one user ID and one access value;
   it never forwards roles or a full permission set.
4. The module verifies signature freshness, method, path, module, identity, and
   its exact local route capability before executing the handler.

Public routes skip user authorization but still require signed Admin delegation
at the module boundary. Direct unsigned requests are rejected.

## Prohibited

- super-admin or old-binary fallback logic;
- treating `is_system` as authorization;
- route or permission duplication in TOML or a second registry;
- database, TOML, Manifest, or discovery work in the gateway hot path;
- forwarding complete user roles or capabilities;
- silent or partial Manifest reconciliation.
- manually creating capability definitions through an administration endpoint.
