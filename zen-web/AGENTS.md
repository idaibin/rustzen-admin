# Frontend Rules

## Read

- `docs/guides/frontend.md`

## Rules

- Keep auth routing and permission gates in `zen-web/src/routes/__root.tsx`.
- Keep request code inside `zen-web/src/api/`; pages use domain APIs from `@/api`.
- Do not edit `zen-web/src/routeTree.gen.ts` manually.
- Build the smallest implementation that solves the current need.
- When backend APIs change, update `zen-web/src/api/` first, then update page usage.

## Command Source

- Use root `justfile` as the command source of truth.
