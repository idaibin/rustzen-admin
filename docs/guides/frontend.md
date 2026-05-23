# Frontend Guide

Rules for frontend work under `zen-web/`.

## Routes

- Use file-based routes that match the final pathname.
- Keep auth redirects, permission gates, current-user bootstrap, and not-found handling in `zen-web/src/routes/__root.tsx`.
- Keep route paths stable; `zen-web/src/store/useAuthStore.ts` derives permission codes from pathname.
- Do not edit `zen-web/src/routeTree.gen.ts` manually.
- Do not create future groups such as `rbac`, `audit`, or `runtime` until implementation lands.

## APIs

- Pages import domain APIs from `@/api`.
- API modules import shared request helpers from `@/api/request`.
- Keep HTTP URLs inside the owning API module.
- `apiRequest` unwraps to `data` by default.
- Use `raw: true` only when callers need envelope metadata such as `total`.
- ProTable list methods return `{ data, total, success }`.
- Keep transport shaping and option normalization inside API modules.
- When backend contracts change, update matching frontend API modules and types in the same task.

## State And UI

- React Query owns read-side server state.
- Zustand stays limited to shared auth state and small persisted UI filters.
- Keep page-local tables, forms, and action handlers in the route file until reuse is real.
- Keep layout-only concerns in `zen-web/src/components/base-layout/`.
- Use existing design-system primitives before adding wrappers.
- Use root `justfile` as the command source of truth.

## Package Manager

- `zen-web` uses pnpm; keep `zen-web/pnpm-lock.yaml` as the frontend lockfile.
- Keep pnpm build-script approvals in `zen-web/pnpm-workspace.yaml`.
- Use `pnpm dev`, `pnpm build`, and `pnpm exec ...` for frontend commands.
- Do not introduce `package-lock.json`, `bun.lock`, npm commands, or Bun commands for `zen-web`.

## Prohibited

- Multiple entrypoints for the same API domain.
- Route-local `-content/` directories before real need.
- Shared API type redeclarations inside pages.
- Route-local state pushed into global stores without cross-page need.
