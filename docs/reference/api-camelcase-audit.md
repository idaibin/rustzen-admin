# API CamelCase Audit

This audit records the current JSON naming boundary. Source code takes precedence when this file drifts.

## Rule

- Rust and database fields use `snake_case`.
- JSON and frontend-facing fields use `camelCase`.
- Backend response and request structs should use `#[serde(rename_all = "camelCase")]` when they expose multi-word fields to the frontend.
- Frontend API params and types should stay `camelCase`.

## Current Backend Coverage

Current backend files with `#[serde(rename_all = "camelCase")]` include:

- `zen-server/src/features/auth/types.rs`
- `zen-server/src/features/account/types.rs`
- `zen-server/src/features/dashboard/types.rs`
- `zen-server/src/features/system/dict/types.rs`
- `zen-server/src/features/system/log/types.rs`
- `zen-server/src/features/system/menu/types.rs`
- `zen-server/src/features/system/role/types.rs`
- `zen-server/src/features/system/user/types.rs`
- `zen-server/src/infra/system_info.rs`

## Current Frontend Boundary

- API modules live under `zen-web/src/api/`.
- Shared request behavior lives in `zen-web/src/api/request.ts`.
- Domain types live in `types.d.ts` beside their owning API module.

## Review Checklist

- New Rust request and response structs use `rename_all = "camelCase"` when exposed over HTTP.
- Frontend `types.d.ts` names match API JSON names directly.
- Pages do not translate field casing locally.
- SQL aliases are explicit when database names need to map into Rust fields.
- API list methods that feed ProTable return `{ data, total, success }`.
