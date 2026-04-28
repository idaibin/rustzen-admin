# Frontend Guide

## Scope

- Applies to `zen-web/`.
- Keep this file thin. Formal frontend rules live in `docs/frontend-guide.md`.

## Quick Entry

- `docs/frontend-guide.md`
- `docs/architecture.md`
- `docs/project-map.md`

## Local Rules

- Keep auth routing and permission gates in `zen-web/src/routes/__root.tsx`.
- Keep request code inside `zen-web/src/api/`; pages use domain APIs from `@/api`.
- Do not edit `zen-web/src/routeTree.gen.ts` manually.
- Build the smallest implementation that solves the current need.
- When backend APIs change, update `zen-web/src/api/` first, then update page usage.

## Commands

- `cd zen-web && pnpm dev`
- `cd zen-web && pnpm build`
- `cd zen-web && vp lint`
- `cd zen-web && vp check --fix`
