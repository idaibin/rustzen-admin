# Frontend Guidelines

## Source of Truth

- `docs/architecture.md`: layout summary and command entry

## Project Layout

- `web/src/routes`: route pages and layouts
- `web/src/api`: module-level request adapters and type declarations
- `web/src/components`: shared UI components
- `web/src/stores`: shared client state

## Working Rules

- Keep request logic inside `web/src/api/<module>/`.
- Do not define ad hoc request code directly inside page components.
- Do not hand-edit generated files such as `web/src/routeTree.gen.ts`.
- Keep route and component filenames aligned with the feature they implement.
- Prefer `camelCase` for JSON and TypeScript fields.
- Update frontend types when backend request or response shapes change.

## Commands

- `cd web && pnpm dev`: start the frontend directly
- `cd web && pnpm build`: build the frontend bundle
- `just check`: run the frontend lint step together with backend validation
