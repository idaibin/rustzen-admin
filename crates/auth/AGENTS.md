# Crate Rules

## Read

- `docs/guides/backend.md`
- `docs/guides/ai-coding-rules.md`

## Boundaries

- Shared authentication and permission-capability helpers for Rust services.
- No duplicate auth logic should be implemented in `apps/server`.

## Rules

- Keep checks based on capability terms; avoid introducing compatibility fallbacks.
- Export minimal stable APIs for callers.
- Keep helpers in tests and public surface aligned with real usages.
