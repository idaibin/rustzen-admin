# Frontend Guide

## Scope

- Applies to `web/`.
- Keep only quick local rules for work inside `web/`; do not duplicate the full specification documents.

## Quick Entry

- See `docs/frontend-guide.md` for the full frontend specification, including API module layout, `apiRequest`/`raw` behavior, and list-return conventions.
- See `docs/architecture.md` for repository-wide rules.
- See `docs/project-map.md` for project path lookup.

## Directory Highlights

- `web/src/routes/`: pages and route entrypoints
- `web/src/api/index.ts`: API barrel exports and shared app message/modal refs
- `web/src/api/runtime.ts`: shared app message/modal bindings
- `web/src/api/request.ts`: shared request layer
- `web/src/api/`: request wrappers, API types, and option lists
- `web/src/components/`: shared components, grouped by feature or responsibility when useful
- `web/src/store/`: shared state
- `web/src/routeTree.gen.ts`: generated file, do not edit manually

## Rules

- Use stable, domain-scoped API objects; do not invent alternate export names for the same domain.
- Use `apiRequest` with default unwrapping for most calls; use `raw: true` only when the call needs the full `Api.ApiResponse` (for example pagination `total`). Import `apiRequest` / `apiDownload` from `web/src/api/request.ts`.
- Implement `list` (and other module methods) so the return shape matches the consumer (for example ProTable `request`); details and current per-resource patterns are in `docs/frontend-guide.md` (**API Module Rules**).
- Keep request logic only in `web/src/api/<module>/`.
- Do not write request code directly inside pages.
- Keep root app wiring in `web/src/main.tsx` and route guards in `web/src/routes/__root.tsx`.
- Do not edit `web/src/routeTree.gen.ts` manually.
- Pages only assemble screens; components should keep a single responsibility.
- Build the smallest implementation that solves the current need.
- When backend APIs change, update `web/src/api/` first, then update page usage.

## Commands

- `cd web && pnpm dev` runs the frontend dev server
- `cd web && pnpm build` builds the frontend production bundle
- `cd web && vp lint` runs frontend lint
- `cd web && vp check --fix` runs frontend checks and fixes supported issues

## Maintenance Rules

- Command descriptions in `web/AGENTS.md`, `docs/*`, and `justfile` must stay aligned. Update them together when command entrypoints change.
