# Frontend Guide

## Scope

- Applies to all frontend implementation under `web/`.
- Covers directory boundaries, state, requests, UI composition, and component responsibilities in one place.

## Layout Notes

- See `docs/architecture.md` for the repository tree layout.
- Keep route-local concerns near the route instead of creating a generic directory for a one-off helper.
- Generated files must not be edited manually.

## UI Stack

- Use the existing design system first.
- Prefer established component primitives over custom one-off implementations.
- Reuse shared table, form, confirm, upload, card, and skeleton patterns before introducing new wrappers.
- Keep global theme, locale, and application bootstrap concerns in the root entrypoint.

## Page Rules

- Keep page files aligned with their route entry point.
- Build the smallest implementation that solves the current requirement.
- Pages assemble screens only; do not pack all business logic into them.
- Local components may stay near the page only when the scope is local, explicit, and easy to contain.
- Keep page-specific action entry points near the page when they do not belong in shared components.

## State and Requests

- Use a query library for server state and a lightweight store for shared client state.
- Do not scatter request code inside pages.
- API modules should own query, mutation, and type definitions together.
- Split query and mutation logic by resource or domain boundary.
- Prefer one module folder per API resource or domain boundary instead of mixing unrelated endpoints into one page file.

## API Module Rules

- `web/src/api/` only keeps the shared request layer, barrel exports, request wrappers, API constants, option lists, and module types.
- Use one folder per API module or resource boundary.
- Keep module implementation and module types together in the same folder as `api.ts` and `types.d.ts`.
- Do not call HTTP clients directly from pages or reusable components.
- Keep shared request normalization, auth handling, timeout handling, and download handling in the request layer.
- Keep field-shape normalization inside the API module instead of leaking transport details into pages.
- Use explicit, resource-oriented method names.
- Do not create parallel aliases for the same semantic action.
- Keep multipart payload assembly inside the owning API module.
- Do not duplicate request parsing, success messaging, logout logic, or parameter serialization in resource modules.

### API Subfile Rules

- `web/src/api/index.ts` owns the public barrel exports for module APIs.
- `web/src/api/index.ts` also exports shared app message/modal refs and `MessageContent`.
- `web/src/api/runtime.ts` owns the underlying app message/modal bindings.
- `web/src/api/<domain>/index.ts` owns a grouped domain barrel export when one is needed.
- `web/src/api/request.ts` owns the shared request layer and only exports cross-module request helpers and app-level request plumbing.
- `web/src/api/api.d.ts` owns shared `Api.*` base types only.
- `web/src/api/<module>/api.ts` owns the concrete request functions for that module.
- `web/src/api/<module>/types.d.ts` owns that module's types, enums, and option shapes.
- Keep `api.ts` as the public implementation entrypoint for a module folder.
- Do not add extra wrapper files when a module can be expressed with `api.ts` plus one `types.d.ts` file.
- Keep option list endpoints or option constants inside the owning module folder, not in a separate top-level directory.
- `apiRequest` should unwrap and return `data` by default; use `raw: true` only when the caller needs the full response envelope for pagination or other metadata.
- Shared pages and components should import request helpers from `web/src/api/request.ts`, not from module internals.
- Expose resources through their barrel API, such as `resourceAPI.list()`, `resourceAPI.get()`, `resourceAPI.create()`, and similar resource-action pairs.
- Keep method names short and resource-oriented: `list`, `get`, `create`, `update`, `delete`.
- Use concise names for non-CRUD endpoints, such as `me()` or `stats()`, instead of `get*` wrappers.
- Use short lookup method names like `options()`, `status()`, `password()`, and `byType()` for module-specific endpoints that are not CRUD.
- Keep TanStack Query keys short and segmented, such as `["domain", "resource", "options"]`, instead of slash-delimited strings.

## Types

- Use `camelCase` for frontend fields and API params.
- Use clear module type names such as `Item`, `CreateRequest`, `UpdateRequest`, `QueryParams`, and `PageResult`.
- Do not redeclare the same types inside pages.

## Component Boundaries

- Each component should own one clear responsibility.
- Shared reusable components belong in `web/src/components/`, grouped by feature or responsibility when helpful.
- Do not create "do-everything" components.
- Do not mix unrelated request or routing logic into components.
- Keep menus, headers, and other layout-only concerns inside the layout tree, not page views.

## Constraints

- Do not edit generated files manually.
- Do not keep multiple entrypoints for old APIs.
- Do not stack extra abstraction.
- Do not let page files grow into oversized JSX blocks.
- When APIs change, update API types and wrappers first, then update page usage.
- Do not rebuild existing design-system components as pure utility-class components.
- Do not add another wrapper UI framework on top.
- Do not push page interaction state into pure presentational components.
- Keep route-level state close to the route group, not in global stores unless it is genuinely cross-page.

## Checks

- Run the frontend check after frontend changes.
- Run the frontend build when the change affects production behavior, routing, or page structure.
