# Frontend Guide

## Scope

- Applies to `zen-web/`.
- Keep only quick local rules for work inside `zen-web/`.

## Quick Entry

- `docs/frontend-guide.md`
- `docs/architecture.md`
- `docs/project-map.md`

## Directory Highlights

- `zen-web/src/routes/`: pages, route entrypoints, and route-local `-content/` components
- `zen-web/src/api/index.ts`: API barrel exports and shared app message/modal refs
- `zen-web/src/api/runtime.ts`: shared app message/modal bindings
- `zen-web/src/api/request.ts`: shared request layer
- `zen-web/src/api/`: request wrappers, API types, and option lists
- `zen-web/src/components/`: shared components, grouped by feature or responsibility when useful
- `zen-web/src/store/`: shared state
- `zen-web/src/routeTree.gen.ts`: generated file, do not edit manually

## Local Rules

- Use stable, domain-scoped API objects; do not invent alternate export names for the same domain.
- Use `apiRequest` with default unwrapping for most calls; use `raw: true` only when the call needs the full `Api.ApiResponse` (for example pagination `total`). Import `apiRequest` / `apiDownload` from `zen-web/src/api/request.ts`.
- Implement `list` (and other module methods) so the return shape matches the consumer (for example ProTable `request`); details and current per-resource patterns are in `docs/frontend-guide.md` (**API Module Rules**).
- Keep request logic only in `zen-web/src/api/<module>/`.
- Do not write request code directly inside pages.
- Keep root app wiring in `zen-web/src/main.tsx` and route guards in `zen-web/src/routes/__root.tsx`.
- Do not edit `zen-web/src/routeTree.gen.ts` manually.
- Pages only assemble screens; components should keep a single responsibility.
- When a route starts accumulating modal forms, table actions, or local interaction state, split those blocks into a route-local `-content/` component beside the page.
- Build the smallest implementation that solves the current need.
- When backend APIs change, update `zen-web/src/api/` first, then update page usage.

## Commands

- `cd zen-web && pnpm dev` runs the frontend dev server
- `cd zen-web && pnpm build` builds the frontend production bundle
- `cd zen-web && vp lint` runs frontend lint
- `cd zen-web && vp check --fix` runs frontend checks and fixes supported issues
