# Frontend Guide

## Scope

- Applies to `web/`.
- Keep only quick local rules for work inside `web/`; do not duplicate the full specification documents.

## Quick Entry

- See `docs/frontend-guide.md` for the full frontend specification.
- See `docs/architecture.md` for repository-wide rules.
- See `docs/project-map.md` for project path lookup.

## Directory Highlights

- `web/src/routes/`: pages and route entrypoints
- `web/src/api/`: request wrappers and API types
- `web/src/components/`: shared components
- `web/src/layouts/`: shared layouts
- `web/src/stores/`: shared state
- `web/src/integrations/`: query and framework integrations
- `web/src/routeTree.gen.ts`: generated file, do not edit manually

## Rules

- Keep request logic only in `web/src/api/<module>/`.
- Do not write request code directly inside pages.
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
