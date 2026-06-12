# Code Review Checklist

Use this checklist for repository changes. It is a review aid, not a replacement for source code or tests.

## Scope

- Is the change the smallest viable slice?
- Does it avoid fallback or compatibility logic?
- Are unrelated files left untouched?
- Are generated files left unedited?

## Backend

- Handlers only parse requests and return responses.
- Services own orchestration, validation, and transactions.
- Repos own SQL and persistence.
- Types live in `types.rs`.
- JSON-facing structs use `camelCase`.
- Schema changes include migrations.

## Frontend

- Pages call domain APIs from `@/api`.
- HTTP URLs stay inside the owning API module.
- Request shaping stays in API modules, not pages.
- Shared state is limited to cross-page state.
- `routeTree.gen.ts` is not edited manually.

## Deployment

- Runtime paths derive from `RUSTZEN_RUNTIME_ROOT`.
- Production config uses `RUSTZEN_SQLITE_PATH` and `RUSTZEN_*`.
- Release output still matches `docs/guides/deployment.md`.

## Docs

- Current facts stay in `architecture.md`, `project-map.md`, or `guides/`.
- Deep explanations, audits, and checklists stay in `reference/`.
- Plans, proposals, fixes, and incidents stay in `history/`.
