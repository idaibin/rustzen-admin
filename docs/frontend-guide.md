# Frontend Guide

## Scope

- Applies to frontend implementation under `zen-web/`.
- Defines current rules only. Do not document future frontend groups as current structure.

## Related Docs

- `docs/README.md`
- `docs/architecture.md`
- `docs/project-map.md`
- `docs/agents/operating-rules.md`

## UI Rules

- Use the existing design system first.
- Prefer established primitives over one-off wrappers.
- Keep global theme, locale, and bootstrap concerns in the root entrypoint.

## Route Rules

- Use file-based routes that match the final pathname.
- Keep auth redirects, permission gates, current-user bootstrap, and not-found handling in `zen-web/src/routes/__root.tsx`.
- Keep route paths stable because `zen-web/src/store/useAuthStore.ts` derives permission codes from pathname.
- Permission codes follow current store behavior: normal pages map to `<path>:list`, `:create` is kept, and numeric `edit` / `detail` segments are ignored.
- Do not create future frontend groups such as `identity`, `access`, `audit`, or `runtime` until implementation actually lands.

## API Rules

- Pages import domain APIs from `@/api`.
- API modules import shared request helpers from `@/api/request`.
- Keep HTTP URLs inside the owning API module. Do not add parallel endpoint constant files.
- `apiRequest` unwraps to `data` by default.
- Use `raw: true` only when callers need envelope metadata such as `total`.
- ProTable-facing `list` methods return `{ data, total, success }`.
- Keep transport shaping, menu tree assembly, and option normalization inside API modules.
- Keep method names short and resource-oriented.
- Keep static frontend-only options in `zen-web/src/constant/options.ts`.

## State And Pages

- React Query owns read-side server state.
- Zustand stores stay limited to shared auth state and small persisted UI filters.
- Mutations stay in submit, confirm, and page action handlers, followed by local reload or navigation.
- Keep page-local table columns, modal forms, and action handlers inside the route file until reuse or size makes extraction clear.
- Do not pre-create route-local `-content/` directories.
- Keep menus, headers, and layout-only concerns inside `zen-web/src/components/base-layout/`.

## Type Rules

- Use `camelCase` for frontend fields and API params.
- Use clear module type names such as `Item`, `CreateRequest`, `UpdateRequest`, `QueryParams`, and `PageResult`.
- Do not redeclare shared API types inside pages.

## Constraints

- Do not edit generated files manually.
- Do not keep multiple entrypoints for the same API domain.
- Do not stack extra abstraction.
- Do not rebuild design-system components as utility-only components.
- Do not push route-local state into global stores unless it is genuinely cross-page.

## Checks

- `cd zen-web && vp check --fix`
- `cd zen-web && pnpm build` when routing, page structure, or production behavior changes.
