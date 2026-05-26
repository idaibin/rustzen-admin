# Frontend Rules

## Read

- `docs/guides/frontend.md`

## Rules

- Keep auth routing and permission gates in `apps/web/src/routes/__root.tsx`.
- Keep request code inside `apps/web/src/api/`; pages use domain APIs from `@/api`.
- Do not edit `apps/web/src/routeTree.gen.ts` manually.
- Build the smallest implementation that solves the current need.
- When backend APIs change, update `apps/web/src/api/` first, then update page usage.

## Command Source

- Use root `justfile` as the command source of truth.
