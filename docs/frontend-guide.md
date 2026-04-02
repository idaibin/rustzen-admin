# Frontend Guide

## Scope

- Applies to all React frontend implementation under `web/`.
- Covers routing, state, requests, page boundaries, and UI choices in one place. Do not split a second frontend UI spec.

## Structure

- Use `TanStack Router` for the page layer.
- Keep request modules in `web/src/api/<module>/`.
- Put shared components in `web/src/components/`.
- Put shared layouts in `web/src/layouts/`.
- Put shared state in `web/src/stores/`.
- Put query integrations in `web/src/integrations/`.

## UI Stack

- Primary UI stack: `Ant Design` + `@ant-design/pro-components`.
- Use `ProLayout` for the admin shell.
- Use `ProTable` for list views.
- Use `ModalForm` for create and edit flows.
- Use `Tailwind` only as a local supplement, not as a replacement for Antd semantic components.
- Keep global theme, locale, and `ConfigProvider` setup in the root entrypoint.

## Page Rules

- Keep page files aligned with route paths.
- Keep route guards at the root entrypoint such as `web/src/routes/__root.tsx`.
- Build the smallest implementation that solves the current requirement.
- Pages assemble screens only; do not pack all business logic into them.
- Local components may stay near the page only when the scope is local, explicit, and easy to contain.

## State and Requests

- Use `TanStack Query` for data fetching.
- Use `Zustand` for shared state.
- Do not scatter request code inside pages.
- API modules should own `query`, `mutation`, and type definitions together.
- Split query and mutation logic by resource.

## Types

- Use `camelCase` for frontend fields and API params.
- Use module type names like `Item`, `CreateRequest`, `UpdateRequest`, `QueryParams`, and `PageResponse`.
- Do not redeclare the same types inside pages.

## Component Boundaries

- Each component should own one clear responsibility.
- Shared reusable components belong in `web/src/components/`.
- Layout-related components should prefer `web/src/layouts/`.
- Do not create "do-everything" components.
- Do not mix unrelated request or routing logic into components.

## Constraints

- Do not edit `web/src/routeTree.gen.ts` manually.
- Do not keep multiple entrypoints for old APIs.
- Do not stack extra abstraction.
- Do not let page files grow into oversized JSX blocks.
- When APIs change, update `web/src/api/` types and wrappers first, then update page usage.
- Do not rebuild Antd components as pure Tailwind components.
- Do not sacrifice semantics and interaction for visual uniformity.
- Do not add another wrapper UI framework on top.
- Do not push page interaction state into pure presentational components.

## Checks

- Run lint after frontend changes.
- When APIs change, check both types and page call sites together.
