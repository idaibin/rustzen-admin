# Frontend Guide

Rules for frontend work under `apps/web/`.

## Routes

- Use file-based routes that match the final pathname.
- Keep auth redirects, permission gates, current-user bootstrap, and not-found handling in `apps/web/src/routes/__root.tsx`.
- Keep route paths stable; `apps/web/src/store/useAuthStore.ts` derives permission codes from pathname.
- Do not edit `apps/web/src/routeTree.gen.ts` manually.
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
- Keep layout-only concerns in `apps/web/src/components/base-layout/`.
- Use existing design-system primitives before adding wrappers.
- Prefer plain function components. Use `forwardRef`, `memo`, `useCallback`, or similar React wrappers only when a real caller needs the ref identity or memoized identity.
- Do not add small calculation helpers or generic abstractions for values that are clearer as local constants or fixed props.
- Use root `justfile` as the command source of truth.

## Tables

- Use `ProTable` for admin list pages unless the page has a reason to use lower-level `Table`.
- Keep table column definitions close to the owning route until reuse is real.
- For icon-only action columns, use `TableActionButton` and `TABLE_ACTION_SPACE_SIZE`.
- Set action column `width` to the smallest fixed value that keeps actions on one line. If a page has more actions, calculate the value once while designing the column and write the fixed number directly.
- Do not add a helper such as `tableActionColumnWidth(count)` for simple one-line width math.
- Do not override `.ant-table-cell` padding for a single page. If table density needs to change globally, use Ant Design table size or theme tokens as a product-wide decision.

## Package Manager

- `apps/web` uses pnpm; keep `apps/web/pnpm-lock.yaml` as the frontend lockfile.
- Keep pnpm build-script approvals in `apps/web/pnpm-workspace.yaml`.
- `vite-plus` is the frontend build tool used in package.json scripts (aliases: `vp`).
- Use `pnpm dev`, `pnpm build`, and `pnpm exec ...` for frontend commands.

## Prohibited

- Multiple entrypoints for the same API domain.
- Route-local `-content/` directories before real need.
- Shared API type redeclarations inside pages.
- Route-local state pushed into global stores without cross-page need.
